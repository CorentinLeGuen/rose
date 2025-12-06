use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_endpoint: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    pub db_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            s3_region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_bucket: std::env::var("S3_BUCKET").expect("S3_BUCKET must be set"),
            s3_access_key_id: std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID must be set"),
            s3_secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY").expect("S3_SECRET_ACCESS_KEY must be set"),
            s3_endpoint: std::env::var("S3_ENDPOINT").ok(),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: std::env::var("SERVER_PORT").unwrap_or_else(|_| "12055".to_string()).parse().expect("SERVER_PORT must be a valid port number"),
            db_url: std::env::var("DB_URL").expect("DB_URL must be set"),
        })
    }
}