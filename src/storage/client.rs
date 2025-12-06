use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    Client, config::{Credentials, Region}, error::SdkError, 
    operation::{
        delete_object::{DeleteObjectError, DeleteObjectOutput}, 
        get_object::{GetObjectError, GetObjectOutput}, 
        head_object::{HeadObjectError, HeadObjectOutput}, 
        put_object::{PutObjectError, PutObjectOutput}
    },
    primitives::ByteStream,
};
use bytes::Bytes;


#[derive(Clone, Debug)]
pub struct S3Config {
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket: String,
}

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new(config: S3Config) -> Self {
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "custom",
        );
        let mut aws_config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(config.region));
        if let Some(endpoint) = config.endpoint {
            aws_config = aws_config.endpoint_url(endpoint);
        }
        let sdk_config = aws_config.load().await;
        let client = Client::new(&sdk_config);
        
        Self {
            client,
            bucket: config.bucket,
        }
    }

    pub async fn get(
        &self, 
        path: &str, 
        version_id: Option<&str>
    ) -> Result<GetObjectOutput, SdkError<GetObjectError>> 
    {
        let mut request = self.client.get_object().bucket(&self.bucket).key(path);

        if let Some(vid) = version_id {
            request = request.version_id(vid);
        }

        request.send().await
    }

    pub async fn head(
        &self,
        path: &str,
        version_id: Option<&str>,
    ) -> Result<HeadObjectOutput, SdkError<HeadObjectError>>
    {
        let mut request = self.client.head_object().bucket(&self.bucket).key(path);

        if let Some(vid) = version_id {
            request = request.version_id(vid);
        }

        request.send().await
    }

    pub async fn put(
        &self,
        path: &str,
        data: Bytes,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>>
    {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(ByteStream::from(data))
            .send()
            .await
    }

    pub async fn delete(
        &self,
        path: &str,
        version_id: Option<&str>
    ) -> Result<DeleteObjectOutput, SdkError<DeleteObjectError>>
    {
        let mut request = self.client.delete_object().bucket(&self.bucket).key(path);

        if let Some(vid) = version_id {
            request = request.version_id(vid);
        }

        request.send().await
    } 

}
