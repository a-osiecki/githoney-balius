use pallas_codec::minicbor::{self, Encode};
use pallas_crypto::hash::Hasher;
use pallas_primitives::{
    conway::{self, Redeemers},
    CostModel, PlutusData,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::tx_builders::ogmios;

pub type PlutusVersion = u8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageViews(pub BTreeMap<PlutusVersion, CostModel>);

impl<C> Encode<C> for LanguageViews {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let entries = build_language_view_entries(&self.0)
            .map_err(minicbor::encode::Error::message)?;

        e.map(entries.len() as u64)?;

        for entry in entries {
            match entry.kind {
                LanguageViewKind::PlutusV1 => {
                    e.bytes(entry.key_bytes.as_ref().expect("v1 key bytes"))?;
                    e.bytes(&entry.value_cbor)?;
                }
                LanguageViewKind::PlutusV2OrV3 => {
                    e.encode(entry.version)?;
                    e.encode(&entry.cost_model)?;
                }
            }
        }

        Ok(())
    }
}

pub fn compute_script_data_hash(
    redeemers: &Redeemers,
    datums: Option<&[PlutusData]>,
    language_views: &LanguageViews,
) -> Result<pallas_crypto::hash::Hash<32>, String> {
    let mut buf = Vec::<u8>::new();

    minicbor::encode(redeemers, &mut buf)
        .map_err(|e| format!("failed to encode redeemers: {e}"))?;
    if let Some(d) = datums {
        minicbor::encode(d, &mut buf).map_err(|e| format!("failed to encode datums: {e}"))?;
    }
    minicbor::encode(language_views, &mut buf)
        .map_err(|e| format!("failed to encode language views: {e}"))?;

    Ok(Hasher::<256>::hash(&buf))
}

pub async fn build_language_views(tx: &conway::Tx) -> Result<LanguageViews, String> {
    let mut versions = plutus_versions_in_witness_set(&tx.transaction_witness_set);
    if versions.is_empty() {
        // Githoney ref script is Plutus v3.
        versions.insert(2);
    }

    let pp = ogmios::protocol_params().await?;
    let mut views = BTreeMap::new();

    for version in versions {
        let cost_model = cost_model_for_version(&pp, version)?;
        views.insert(version, cost_model);
    }

    Ok(LanguageViews(views))
}

pub fn parse_redeemer_tag(purpose: &str) -> Result<conway::RedeemerTag, String> {
    match purpose {
        "spend" => Ok(conway::RedeemerTag::Spend),
        "mint" => Ok(conway::RedeemerTag::Mint),
        "cert" => Ok(conway::RedeemerTag::Cert),
        "reward" => Ok(conway::RedeemerTag::Reward),
        "vote" => Ok(conway::RedeemerTag::Vote),
        "propose" | "proposal" => Ok(conway::RedeemerTag::Propose),
        other => Err(format!("unknown redeemer purpose: {other}")),
    }
}

fn plutus_versions_in_witness_set(wits: &conway::WitnessSet) -> BTreeSet<PlutusVersion> {
    let mut versions = BTreeSet::new();
    if wits.plutus_v1_script.is_some() {
        versions.insert(0);
    }
    if wits.plutus_v2_script.is_some() {
        versions.insert(1);
    }
    if wits.plutus_v3_script.is_some() {
        versions.insert(2);
    }
    versions
}

fn cost_model_for_version(
    pp: &ogmios::ProtocolParams,
    version: PlutusVersion,
) -> Result<Vec<i64>, String> {
    let key = match version {
        0 => "plutus:v1",
        1 => "plutus:v2",
        2 => "plutus:v3",
        _ => return Err(format!("unsupported plutus version: {version}")),
    };
    pp.plutus_cost_models
        .get(key)
        .cloned()
        .ok_or_else(|| format!("missing {key} in plutusCostModels"))
}

enum LanguageViewKind {
    PlutusV1,
    PlutusV2OrV3,
}

struct LanguageViewEntry {
    version: PlutusVersion,
    cost_model: CostModel,
    key_sort: Vec<u8>,
    key_bytes: Option<Vec<u8>>,
    value_cbor: Vec<u8>,
    kind: LanguageViewKind,
}

fn build_language_view_entries(
    views: &BTreeMap<PlutusVersion, CostModel>,
) -> Result<Vec<LanguageViewEntry>, String> {
    let mut entries = Vec::with_capacity(views.len());

    for (version, cost_model) in views {
        match *version {
            0 => {
                let mut inner = vec![];
                let mut sub = minicbor::Encoder::new(&mut inner);

                sub.begin_array()
                    .map_err(|e| format!("encode v1 cost model array start: {e}"))?;
                for v in cost_model.iter() {
                    sub.encode_with(v, &mut ())
                        .map_err(|e| format!("encode v1 cost model value: {e}"))?;
                }
                sub.end()
                    .map_err(|e| format!("encode v1 cost model array end: {e}"))?;

                let key_bytes =
                    minicbor::to_vec(0).map_err(|e| format!("encode v1 key: {e}"))?;
                let key_sort =
                    minicbor::to_vec(&key_bytes).map_err(|e| format!("encode v1 key: {e}"))?;

                entries.push(LanguageViewEntry {
                    version: *version,
                    cost_model: cost_model.clone(),
                    key_sort,
                    key_bytes: Some(key_bytes),
                    value_cbor: inner,
                    kind: LanguageViewKind::PlutusV1,
                });
            }
            1 | 2 => {
                let key_sort =
                    minicbor::to_vec(*version).map_err(|e| format!("encode key: {e}"))?;
                entries.push(LanguageViewEntry {
                    version: *version,
                    cost_model: cost_model.clone(),
                    key_sort,
                    key_bytes: None,
                    value_cbor: Vec::new(),
                    kind: LanguageViewKind::PlutusV2OrV3,
                });
            }
            _ => return Err(format!("unsupported plutus version: {version}")),
        }
    }

    entries.sort_by(|a, b| {
        let a_len = a.key_sort.len();
        let b_len = b.key_sort.len();
        a_len
            .cmp(&b_len)
            .then_with(|| a.key_sort.cmp(&b.key_sort))
    });

    Ok(entries)
}
