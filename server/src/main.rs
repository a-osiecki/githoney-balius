use axum::Router;
use tokio::net::TcpListener;

mod routes;

#[tokio::main]
async fn main() {
    let app: Router = routes::router();

    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Server listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await.unwrap();
}