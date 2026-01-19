use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub s3_bucket: String,
    pub server_host: String,
    pub server_port: u16,
    pub db_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            s3_bucket: std::env::var("S3_BUCKET").expect("S3_BUCKET must be set"),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: std::env::var("SERVER_PORT").unwrap_or_else(|_| "12055".to_string()).parse().expect("SERVER_PORT must be a valid port number"),
            db_url: std::env::var("DB_URL").expect("DB_URL must be set"),
        })
    }
}