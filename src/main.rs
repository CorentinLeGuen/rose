use std::net::SocketAddr;
use axum::{
    routing::get,
    Json,
    Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(health));

    let addr: SocketAddr = "0.0.0.0:8003".parse().unwrap();
    println!("Service running on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app
    ).await.unwrap();
}
