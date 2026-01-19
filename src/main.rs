mod config;
mod error;
mod handlers;
mod storage;
mod entities;

use axum::{
    routing::{get, head, put, delete},
    Router,
};
use storage::S3Client;
use sea_orm::{Database, DatabaseConnection};
use tower::{ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub store_client: S3Client,
    pub db: DatabaseConnection,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // loading config
    let config = Config::from_env()?;
    tracing::info!("Config of bucket '{}' loaded", config.s3_bucket);
    
    // create Object Storage client
    let store_client = S3Client::new(&config).await;
    tracing::info!("Object Store initialized");

    let db = Database::connect(config.db_url).await?;
    tracing::info!("Database connected");

    let state = AppState {
        store_client,
        db,
        config,
    };

    let app = Router::new()
        .route("/objects/{*key}", get(handlers::get_object))
        .route("/objects/{*key}", head(handlers::head_object))
        .route("/objects/{*key}", put(handlers::put_object))
        .route("/objects/{*key}", delete(handlers::delete_object))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}