use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

use crate::error::{io_error, AppError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CasStats {
    pub root_dir: PathBuf,
    pub total_objects: usize,
    pub total_bytes: u64,
}

pub struct CasStore {
    root_dir: PathBuf,
}

impl Default for CasStore {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("mizan-cas-store"))
    }
}

impl CasStore {
    pub fn new(root_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&root_dir);
        Self { root_dir }
    }

    pub fn compute_digest(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    fn blob_path(&self, digest: &str) -> Result<PathBuf> {
        let clean_hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        if clean_hash.len() < 4 {
            return Err(AppError::Configuration(format!(
                "Invalid CAS digest format: {digest}"
            )));
        }
        let prefix = &clean_hash[0..2];
        let filename = &clean_hash[2..];
        Ok(self.root_dir.join(prefix).join(filename))
    }

    pub fn put_bytes(&self, data: &[u8]) -> Result<String> {
        let digest = Self::compute_digest(data);
        let path = self.blob_path(&digest)?;

        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(&path, data).map_err(|e| io_error(&path, e))?;
        }

        Ok(digest)
    }

    pub fn put_str(&self, data: &str) -> Result<String> {
        self.put_bytes(data.as_bytes())
    }

    pub fn get_bytes(&self, digest: &str) -> Result<Vec<u8>> {
        let path = self.blob_path(digest)?;
        if !path.exists() {
            return Err(AppError::Configuration(format!(
                "CAS object not found: {digest}"
            )));
        }
        let bytes = fs::read(&path).map_err(|e| io_error(&path, e))?;

        // Verify cryptographic integrity on read
        let actual_digest = Self::compute_digest(&bytes);
        if actual_digest != digest
            && !digest.ends_with(actual_digest.strip_prefix("sha256:").unwrap())
        {
            return Err(AppError::Configuration(format!(
                "CAS integrity check failed for object: expected {digest}, got {actual_digest}"
            )));
        }

        Ok(bytes)
    }

    pub fn get_str(&self, digest: &str) -> Result<String> {
        let bytes = self.get_bytes(digest)?;
        String::from_utf8(bytes).map_err(|e| AppError::Configuration(e.to_string()))
    }

    pub fn has_blob(&self, digest: &str) -> bool {
        if let Ok(path) = self.blob_path(digest) {
            path.exists()
        } else {
            false
        }
    }

    pub fn stats(&self) -> Result<CasStats> {
        let mut total_objects = 0;
        let mut total_bytes = 0;

        if self.root_dir.exists() {
            for entry in fs::read_dir(&self.root_dir)
                .map_err(|e| io_error(&self.root_dir, e))?
                .flatten()
            {
                if entry.path().is_dir() {
                    for blob in fs::read_dir(entry.path())
                        .map_err(|e| io_error(entry.path(), e))?
                        .flatten()
                    {
                        if blob.path().is_file() {
                            total_objects += 1;
                            if let Ok(meta) = blob.metadata() {
                                total_bytes += meta.len();
                            }
                        }
                    }
                }
            }
        }

        Ok(CasStats {
            root_dir: self.root_dir.clone(),
            total_objects,
            total_bytes,
        })
    }

    pub fn prune(&self) -> Result<usize> {
        let mut count = 0;
        if self.root_dir.exists() {
            let stats = self.stats()?;
            count = stats.total_objects;
            let _ = fs::remove_dir_all(&self.root_dir);
            let _ = fs::create_dir_all(&self.root_dir);
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_store_put_get_integrity() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-cas-test-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let payload = "NIST SP 800-53 r5 AC-2 Implementation Policy";
        let digest = cas.put_str(payload).expect("put should succeed");

        assert!(digest.starts_with("sha256:"));
        assert!(cas.has_blob(&digest));

        let retrieved = cas.get_str(&digest).expect("get should succeed");
        assert_eq!(retrieved, payload);

        let stats = cas.stats().expect("stats should succeed");
        assert_eq!(stats.total_objects, 1);
        assert_eq!(stats.total_bytes, payload.len() as u64);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
