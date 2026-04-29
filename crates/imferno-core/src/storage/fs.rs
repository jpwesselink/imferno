//! Local-filesystem implementation of the [`Storage`](super::Storage) trait.

use super::{Entry, Scheme, Storage, StorageError, StorageUri};
use std::path::Path;

/// Storage backend reading from the local filesystem.
///
/// `Entry::uri` produced by this backend is a bare absolute path (no
/// `file://` prefix), which preserves backward compatibility with the
/// keys returned by [`crate::package::read_dir`]. `StorageUri::parse`
/// recognizes bare paths and treats them as `file://` URIs.
#[derive(Debug, Default, Clone)]
pub struct FsStorage;

impl FsStorage {
    pub fn new() -> Self {
        Self
    }
}

impl Storage for FsStorage {
    fn list(&self, uri: &StorageUri) -> Result<Vec<Entry>, StorageError> {
        if uri.scheme != Scheme::File {
            return Err(StorageError::UnsupportedScheme(format!("{:?}", uri.scheme)));
        }
        let path = Path::new(&uri.path);
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&canonical)? {
            let entry = entry?;
            let p = entry.path();
            let metadata = entry.metadata()?;
            entries.push(Entry {
                uri: p.display().to_string(),
                size: metadata.len(),
                is_file: metadata.is_file(),
            });
        }
        Ok(entries)
    }

    fn read_to_string(&self, uri: &StorageUri) -> Result<String, StorageError> {
        if uri.scheme != Scheme::File {
            return Err(StorageError::UnsupportedScheme(format!("{:?}", uri.scheme)));
        }
        std::fs::read_to_string(&uri.path).map_err(|e| match e.kind() {
            std::io::ErrorKind::InvalidData => {
                StorageError::Backend(format!("non-UTF-8 contents: {}", uri.path))
            }
            _ => StorageError::Io(e),
        })
    }
}
