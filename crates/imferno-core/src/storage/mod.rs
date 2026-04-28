//! Storage abstraction for IMF package I/O.
//!
//! Provides the [`Storage`] trait and built-in implementations for
//! local filesystem ([`fs::FsStorage`]) and S3 ([`s3::S3Storage`],
//! behind the `aws-s3` feature flag).

pub mod fs;

#[cfg(feature = "aws-s3")]
pub mod s3;

/// Parsed storage URI.
///
/// Recognized schemes: `file`, `s3`. Bare paths (no scheme) are normalized
/// to `file://` URIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageUri {
    pub scheme: Scheme,
    /// For `file`: an absolute or relative filesystem path.
    /// For `s3`: the key prefix (no leading `/`).
    pub path: String,
    /// For `s3` only: the bucket name.
    pub bucket: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    File,
    S3,
}

impl StorageUri {
    /// Parse a URI string. Bare paths become `file://` URIs.
    pub fn parse(input: &str) -> Result<Self, StorageError> {
        // Bare paths (no scheme://) — treat as filesystem
        if !input.contains("://") {
            return Ok(StorageUri {
                scheme: Scheme::File,
                path: input.to_string(),
                bucket: None,
            });
        }

        let parsed = url::Url::parse(input)
            .map_err(|e| StorageError::InvalidUri(format!("{input}: {e}")))?;

        match parsed.scheme() {
            "file" => Ok(StorageUri {
                scheme: Scheme::File,
                path: parsed.path().to_string(),
                bucket: None,
            }),
            "s3" => {
                let bucket = parsed
                    .host_str()
                    .ok_or_else(|| {
                        StorageError::InvalidUri(format!("s3 URI missing bucket: {input}"))
                    })?
                    .to_string();
                // url crate prefixes path with /, drop it for S3 key prefix semantics
                let path = parsed.path().trim_start_matches('/').to_string();
                Ok(StorageUri {
                    scheme: Scheme::S3,
                    path,
                    bucket: Some(bucket),
                })
            }
            other => Err(StorageError::UnsupportedScheme(other.to_string())),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("invalid URI: {0}")]
    InvalidUri(String),
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("backend error: {0}")]
    Backend(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_uri() {
        let uri = StorageUri::parse("file:///tmp/imp").unwrap();
        assert_eq!(uri.scheme, Scheme::File);
        assert_eq!(uri.path, "/tmp/imp");
        assert_eq!(uri.bucket, None);
    }

    #[test]
    fn parse_s3_uri() {
        let uri = StorageUri::parse("s3://my-bucket/path/to/imp/").unwrap();
        assert_eq!(uri.scheme, Scheme::S3);
        assert_eq!(uri.bucket, Some("my-bucket".to_string()));
        assert_eq!(uri.path, "path/to/imp/");
    }

    #[test]
    fn parse_bare_path_becomes_file_uri() {
        let uri = StorageUri::parse("/tmp/imp").unwrap();
        assert_eq!(uri.scheme, Scheme::File);
        assert_eq!(uri.path, "/tmp/imp");
    }

    #[test]
    fn parse_unsupported_scheme_errors() {
        let err = StorageUri::parse("ftp://example.com/imp").unwrap_err();
        assert!(matches!(err, StorageError::UnsupportedScheme(_)));
    }
}
