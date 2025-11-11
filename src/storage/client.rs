use anyhow::Result;
use bytes::Bytes;
use object_store::{
    aws::AmazonS3Builder, 
    path::Path as ObjectPath, 
    ObjectMeta, 
    ObjectStore,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct OSClient {
    store: Arc<dyn ObjectStore>,
}

pub struct OSConfig {
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: Option<String>,
}

impl OSClient {
    pub fn new(config: OSConfig) -> Result<Self> {
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&config.bucket)
            .with_region(&config.region)
            .with_access_key_id(&config.access_key_id)
            .with_secret_access_key(&config.secret_access_key);
        // add endpoint if provided
        if let Some(endpoint) = config.endpoint {
            builder = builder.with_endpoint(&endpoint);
        }

        let store = builder.build()?;

        Ok(Self {
            store: Arc::new(store),
        })
    }

    pub async fn get(&self, path: &str) -> Result<Bytes, object_store::Error> {
        let location = ObjectPath::from(path);
        let result = self.store.get(&location).await?;
        let bytes = result.bytes().await?;
        Ok(bytes)
    }

    pub async fn head(&self, path: &str) -> Result<ObjectMeta, object_store::Error> {
        let location = ObjectPath::from(path);
        self.store.head(&location).await
    }

    pub async fn put(&self, path: &str, data: Bytes) -> Result<(), object_store::Error> {
        let location = ObjectPath::from(path);
        self.store.put(&location, data.into()).await?;
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<(), object_store::Error> {
        let location = ObjectPath::from(path);
        self.store.delete(&location).await?;
        Ok(())
    }
}