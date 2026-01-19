use aws_config::{self, BehaviorVersion};
use aws_sdk_s3::{
    Client, 
    error::SdkError,
    config::Builder as S3ConfigBuilder,
    operation::{
        delete_object::{DeleteObjectError, DeleteObjectOutput}, 
        get_object::{GetObjectError, GetObjectOutput}, 
        head_object::{HeadObjectError, HeadObjectOutput}, 
        put_object::{PutObjectError, PutObjectOutput}
    },
    primitives::ByteStream,
};
use crate::config::Config;
use bytes::Bytes;


#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket_name: String,
}

impl S3Client {
    pub async fn new(config: &Config) -> Self {
        let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let s3_config_builder = S3ConfigBuilder::from(&aws_config);

        let client = Client::from_conf(s3_config_builder.build());
        
        Self {
            client,
            bucket_name: config.s3_bucket.clone(),
        }
    }

    pub async fn get(
        &self, 
        path: &str, 
        version_id: Option<&str>
    ) -> Result<GetObjectOutput, SdkError<GetObjectError>> 
    {
        let mut request = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(path);

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
        let mut request = self.client
            .head_object()
            .bucket(&self.bucket_name)
            .key(path);

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
            .bucket(&self.bucket_name)
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
        let mut request = self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path);

        if let Some(vid) = version_id {
            request = request.version_id(vid);
        }

        request.send().await
    } 

}
