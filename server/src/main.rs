use axum::Router;
use dotenvy;
use tokio::net::TcpListener;

mod evaluate_tx;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let app: Router = routes::router();

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Server listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await.unwrap();
}
