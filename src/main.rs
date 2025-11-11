mod config;
mod error;
mod handlers;
mod storage;

use axum::{
    routing::{get, head, put, delete},
    Router,
};
use config::Config;
use storage::{OSClient, OSConfig};
use tower::{ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    tracing::info!("Config loaded, bucket {} and region {}", config.os_bucket, config.os_region);
    
    // create Object Storage client
    let store_config = OSConfig {
        bucket: config.os_bucket.clone(),
        region: config.os_region.clone(),
        access_key_id: config.os_access_key_id.clone(),
        secret_access_key: config.os_secret_access_key.clone(),
        endpoint: config.os_endpoint.clone(),
    };
    let store_client = OSClient::new(store_config)?;
    tracing::info!("Object Store initialized");

    let app = Router::new()
        .route("/objects/{*key}", get(handlers::get_object))
        .route("/objects/{*key}", head(handlers::head_object))
        .route("/objects/{*key}", put(handlers::put_object))
        .route("/objects/{*key}", delete(handlers::delete_object))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(store_client);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}