use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub os_region: String,
    pub os_bucket: String,
    pub os_access_key_id: String,
    pub os_secret_access_key: String,
    pub os_endpoint: Option<String>,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            os_region: std::env::var("OS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            os_bucket: std::env::var("OS_BUCKET").expect("OS_BUCKET must be set"),
            os_access_key_id: std::env::var("OS_ACCESS_KEY_ID").expect("OS_ACCESS_KEY_ID must be set"),
            os_secret_access_key: std::env::var("OS_SECRET_ACCESS_KEY").expect("OS_SECRET_ACCESS_KEY must be set"),
            os_endpoint: std::env::var("OS_ENDPOINT").ok(),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: std::env::var("SERVER_PORT").unwrap_or_else(|_| "12055".to_string()).parse().expect("SERVER_PORT must be a valid port number"),
        })
    }
}