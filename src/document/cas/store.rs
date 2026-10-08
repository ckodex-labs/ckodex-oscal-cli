use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

use crate::error::{AppError, Result, io_error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CasStats {
    pub root_dir: PathBuf,
    pub total_objects: usize,
    pub total_bytes: u64,
}

#[derive(Clone, Debug)]
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
        if clean_hash.len() != 64 || !clean_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::Configuration(format!(
                "Invalid CAS digest format (must be 64-hex SHA-256): {digest}"
            )));
        }
        let lower = clean_hash.to_ascii_lowercase();
        let prefix = &lower[0..2];
        let filename = &lower[2..];
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
        let actual_hash = actual_digest
            .strip_prefix("sha256:")
            .unwrap_or(&actual_digest);
        let expected_hash = digest.strip_prefix("sha256:").unwrap_or(digest);
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
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

    #[test]
    fn test_cas_store_rejects_directory_traversal() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-cas-traversal-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let malicious_digests = vec![
            "sha256:../../etc/passwd",
            "../foo",
            "../../etc/shadow",
            "/etc/passwd",
            "....",
            "sha256:..",
            "..",
            "sha256:../etc/passwd",
            "sha256:../../../../../../../../../../../../../../../../",
            "sha256:..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\..\\",
            "../../../../../../../../../../../../../../../../",
        ];

        for malicious in malicious_digests {
            assert!(
                cas.blob_path(malicious).is_err(),
                "blob_path must reject malicious path traversal: {malicious}"
            );
            assert!(
                cas.get_bytes(malicious).is_err(),
                "get_bytes must reject malicious path traversal: {malicious}"
            );
            assert!(
                cas.get_str(malicious).is_err(),
                "get_str must reject malicious path traversal: {malicious}"
            );
            assert!(
                !cas.has_blob(malicious),
                "has_blob must return false for malicious path traversal: {malicious}"
            );
        }

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cas_store_rejects_malformed_and_non_hex_digests() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-cas-malformed-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let malformed_digests = vec![
            "",
            "sha256:",
            "1234",
            "sha256:1234",
            // 63 hex digits (too short)
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
            // 65 hex digits (too long)
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0",
            // 64 non-hex characters ('g')
            "sha256:gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            // 64 non-hex characters ('z')
            "sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
            // 64 whitespace characters
            "sha256:                                                                ",
            // 64 chars containing special symbols / control chars
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd\n",
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd\0",
        ];

        for malformed in malformed_digests {
            let err = cas
                .blob_path(malformed)
                .expect_err("must reject malformed digest");
            match err {
                AppError::Configuration(msg) => {
                    assert!(msg.contains("Invalid CAS digest format"));
                }
                other => panic!("expected AppError::Configuration, got {other:?}"),
            }
            assert!(cas.get_bytes(malformed).is_err());
            assert!(cas.get_str(malformed).is_err());
            assert!(!cas.has_blob(malformed));
        }

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cas_store_case_insensitivity_and_prefix_handling() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-cas-case-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let payload = "Content addressed data";
        let canonical_digest = cas.put_str(payload).expect("put should succeed");

        let raw_hash = canonical_digest.strip_prefix("sha256:").unwrap();
        let upper_hash = raw_hash.to_ascii_uppercase();
        let upper_digest = format!("sha256:{upper_hash}");

        // Retrieve using lowercase with prefix
        assert_eq!(cas.get_str(&canonical_digest).unwrap(), payload);
        // Retrieve using lowercase raw hash (no prefix)
        assert_eq!(cas.get_str(raw_hash).unwrap(), payload);
        // Retrieve using uppercase with prefix
        assert_eq!(cas.get_str(&upper_digest).unwrap(), payload);
        // Retrieve using uppercase raw hash (no prefix)
        assert_eq!(cas.get_str(&upper_hash).unwrap(), payload);

        // Verify has_blob for all variants
        assert!(cas.has_blob(&canonical_digest));
        assert!(cas.has_blob(raw_hash));
        assert!(cas.has_blob(&upper_digest));
        assert!(cas.has_blob(&upper_hash));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cas_store_integrity_check_failure() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-cas-tamper-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let payload = "Original untampered content";
        let digest = cas.put_str(payload).expect("put should succeed");

        // Tamper with the blob file directly on disk
        let path = cas.blob_path(&digest).expect("blob path should be valid");
        fs::write(&path, b"Tampered malicious content").expect("tamper write should succeed");

        // Attempt get_bytes and get_str
        let err = cas
            .get_bytes(&digest)
            .expect_err("get_bytes must fail integrity check");
        match err {
            AppError::Configuration(msg) => {
                assert!(msg.contains("CAS integrity check failed for object"));
            }
            other => panic!("expected AppError::Configuration, got {other:?}"),
        }

        let err_str = cas
            .get_str(&digest)
            .expect_err("get_str must fail integrity check");
        match err_str {
            AppError::Configuration(msg) => {
                assert!(msg.contains("CAS integrity check failed for object"));
            }
            other => panic!("expected AppError::Configuration, got {other:?}"),
        }

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cas_store_not_found() {
        let temp_dir =
            std::env::temp_dir().join(format!("mizan-cas-notfound-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let nonexistent_digest =
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(!cas.has_blob(nonexistent_digest));

        let err = cas
            .get_bytes(nonexistent_digest)
            .expect_err("must return not found");
        match err {
            AppError::Configuration(msg) => {
                assert!(msg.contains("CAS object not found"));
            }
            other => panic!("expected AppError::Configuration, got {other:?}"),
        }

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cas_store_prune_and_stats() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-cas-prune-{}", std::process::id()));
        let cas = CasStore::new(temp_dir.clone());

        let _d1 = cas.put_str("blob 1").expect("put 1");
        let _d2 = cas.put_str("blob 2").expect("put 2");

        let stats = cas.stats().expect("stats");
        assert_eq!(stats.total_objects, 2);

        let pruned = cas.prune().expect("prune");
        assert_eq!(pruned, 2);

        let stats_after = cas.stats().expect("stats after");
        assert_eq!(stats_after.total_objects, 0);
        assert_eq!(stats_after.total_bytes, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
