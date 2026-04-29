//! S3 implementation of the [`Storage`](super::Storage) trait.
//!
//! Wraps the async AWS SDK in a synchronous interface using a private
//! tokio runtime. Construction is async; the resulting handle is sync.

use super::{Entry, Scheme, Storage, StorageError, StorageUri};
use aws_sdk_s3::Client;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// S3-backed storage. Wraps an `aws_sdk_s3::Client` and a private tokio runtime.
#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    runtime: Arc<Runtime>,
}

impl S3Storage {
    /// Construct from an existing client. Caller controls region / credentials / endpoint.
    pub fn from_client(client: Client) -> Result<Self, StorageError> {
        let runtime = Runtime::new()
            .map_err(|e| StorageError::Backend(format!("failed to start tokio runtime: {e}")))?;
        Ok(Self {
            client,
            runtime: Arc::new(runtime),
        })
    }

    /// Construct using the AWS default credential chain (env / profile / IMDS).
    pub fn from_default() -> Result<Self, StorageError> {
        let runtime = Runtime::new()
            .map_err(|e| StorageError::Backend(format!("failed to start tokio runtime: {e}")))?;
        let client = runtime.block_on(async {
            let cfg = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            Client::new(&cfg)
        });
        Ok(Self {
            client,
            runtime: Arc::new(runtime),
        })
    }
}

impl Storage for S3Storage {
    fn list(&self, uri: &StorageUri) -> Result<Vec<Entry>, StorageError> {
        if uri.scheme != Scheme::S3 {
            return Err(StorageError::UnsupportedScheme(format!("{:?}", uri.scheme)));
        }
        let bucket = uri
            .bucket
            .as_ref()
            .ok_or_else(|| StorageError::InvalidUri("s3 URI missing bucket".into()))?
            .clone();
        let prefix = uri.path.clone();
        let client = self.client.clone();

        self.runtime.block_on(async move {
            let mut entries = Vec::new();
            let mut continuation: Option<String> = None;
            loop {
                let mut req = client.list_objects_v2().bucket(&bucket).prefix(&prefix);
                if let Some(t) = continuation.take() {
                    req = req.continuation_token(t);
                }
                let resp = req
                    .send()
                    .await
                    .map_err(|e| StorageError::Backend(format!("S3 ListObjectsV2: {e}")))?;
                for obj in resp.contents() {
                    let Some(key) = obj.key() else { continue };
                    entries.push(Entry {
                        uri: format!("s3://{bucket}/{key}"),
                        size: obj.size().unwrap_or(0) as u64,
                        is_file: true,
                    });
                }
                if resp.is_truncated() == Some(true) {
                    continuation = resp.next_continuation_token().map(|s| s.to_string());
                } else {
                    return Ok(entries);
                }
            }
        })
    }

    fn read_to_string(&self, uri: &StorageUri) -> Result<String, StorageError> {
        if uri.scheme != Scheme::S3 {
            return Err(StorageError::UnsupportedScheme(format!("{:?}", uri.scheme)));
        }
        let bucket = uri
            .bucket
            .as_ref()
            .ok_or_else(|| StorageError::InvalidUri("s3 URI missing bucket".into()))?
            .clone();
        let key = uri.path.clone();
        let client = self.client.clone();

        self.runtime.block_on(async move {
            let resp = client
                .get_object()
                .bucket(&bucket)
                .key(&key)
                .send()
                .await
                .map_err(|e| StorageError::Backend(format!("S3 GetObject {key}: {e}")))?;
            let body = resp
                .body
                .collect()
                .await
                .map_err(|e| StorageError::Backend(format!("S3 read body {key}: {e}")))?;
            String::from_utf8(body.into_bytes().to_vec())
                .map_err(|_| StorageError::Backend(format!("S3 object {key}: not valid UTF-8")))
        })
    }
}
