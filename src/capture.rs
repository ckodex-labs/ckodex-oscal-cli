use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    config::AppConfig,
    error::{io_error, redact_endpoint, Result},
};

static CAPTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaptureManifest {
    pub id: String,
    pub method: String,
    pub endpoint: String,
    pub captured_at_unix_ms: u128,
    pub request_sha256: String,
    pub response_sha256: String,
    pub request_bytes: usize,
    pub response_bytes: usize,
    pub proto_lock_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CapturePruneResult {
    pub older_than_days: u64,
    pub cutoff_unix_ms: u128,
    pub dry_run: bool,
    pub candidates: Vec<CaptureManifest>,
    pub deleted_ids: Vec<String>,
}

pub fn write<M1: Message, M2: Message>(
    config: &AppConfig,
    method: &str,
    request: &M1,
    response: &M2,
) -> Result<CaptureManifest> {
    let request_bytes = request.encode_to_vec();
    let response_bytes = response.encode_to_vec();
    let id = capture_id();
    let directory = config.capture_root.join(&id);
    let pending_directory = config.capture_root.join(format!(".{id}.pending"));
    fs::create_dir_all(&config.capture_root)
        .map_err(|error| io_error(&config.capture_root, error))?;
    ensure_directory(&config.capture_root)?;
    restrict_permissions(&config.capture_root, 0o700)
        .map_err(|error| io_error(&config.capture_root, error))?;

    let result = (|| {
        fs::create_dir(&pending_directory).map_err(|error| io_error(&pending_directory, error))?;
        restrict_permissions(&pending_directory, 0o700)
            .map_err(|error| io_error(&pending_directory, error))?;

        let request_path = pending_directory.join("request.pb");
        let response_path = pending_directory.join("response.pb");
        fs::write(&request_path, &request_bytes).map_err(|error| io_error(&request_path, error))?;
        fs::write(&response_path, &response_bytes)
            .map_err(|error| io_error(&response_path, error))?;
        restrict_permissions(&request_path, 0o600)
            .map_err(|error| io_error(&request_path, error))?;
        restrict_permissions(&response_path, 0o600)
            .map_err(|error| io_error(&response_path, error))?;

        let manifest = CaptureManifest {
            id,
            method: method.to_owned(),
            endpoint: redact_endpoint(&config.endpoint),
            captured_at_unix_ms: unix_ms(),
            request_sha256: sha256(&request_bytes),
            response_sha256: sha256(&response_bytes),
            request_bytes: request_bytes.len(),
            response_bytes: response_bytes.len(),
            proto_lock_sha256: sha256(include_bytes!("../proto.lock")),
        };
        let manifest_path = pending_directory.join("manifest.json");
        let json = serde_json::to_vec_pretty(&manifest)?;
        fs::write(&manifest_path, json).map_err(|error| io_error(&manifest_path, error))?;
        restrict_permissions(&manifest_path, 0o600)
            .map_err(|error| io_error(&manifest_path, error))?;
        fs::rename(&pending_directory, &directory).map_err(|error| io_error(&directory, error))?;
        Ok(manifest)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&pending_directory);
    }
    result
}

pub fn list(root: &Path) -> Result<Vec<CaptureManifest>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    ensure_directory(root)?;
    let mut manifests: Vec<CaptureManifest> = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| io_error(root, error))? {
        let entry = entry.map_err(|error| io_error(root, error))?;
        let entry_type = entry
            .file_type()
            .map_err(|error| io_error(entry.path(), error))?;
        if !entry_type.is_dir() || entry_type.is_symlink() {
            continue;
        }
        let path = entry.path().join("manifest.json");
        let file_type = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata.file_type(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error(&path, error)),
        };
        if file_type.is_file() && !file_type.is_symlink() {
            let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
            let manifest: CaptureManifest = serde_json::from_slice(&bytes)?;
            validate_capture_id(&manifest.id)?;
            manifests.push(manifest);
        }
    }
    manifests.sort_by_key(|manifest| std::cmp::Reverse(manifest.captured_at_unix_ms));
    Ok(manifests)
}

pub fn load(root: &Path, id: &str) -> Result<(CaptureManifest, Vec<u8>)> {
    validate_capture_id(id)?;
    ensure_directory(root)?;
    let directory = root.join(id);
    ensure_directory(&directory)?;
    let manifest_path = directory.join("manifest.json");
    let request_path = directory.join("request.pb");
    let response_path = directory.join("response.pb");
    ensure_regular_file(&manifest_path)?;
    ensure_regular_file(&request_path)?;
    ensure_regular_file(&response_path)?;
    let manifest: CaptureManifest = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|error| io_error(&manifest_path, error))?,
    )?;
    if manifest.id != id {
        return Err(crate::error::AppError::CaptureIntegrity(
            "manifest ID does not match the requested capture".to_owned(),
        ));
    }
    let request = fs::read(&request_path).map_err(|error| io_error(&request_path, error))?;
    let response = fs::read(&response_path).map_err(|error| io_error(&response_path, error))?;
    if request.len() != manifest.request_bytes || response.len() != manifest.response_bytes {
        return Err(crate::error::AppError::CaptureIntegrity(
            "capture byte count does not match the manifest".to_owned(),
        ));
    }
    verify_digest("request", &manifest.request_sha256, &request)?;
    verify_digest("response", &manifest.response_sha256, &response)?;
    Ok((manifest, response))
}

pub fn verify(root: &Path, id: Option<&str>) -> Result<Vec<CaptureManifest>> {
    let ids = match id {
        Some(id) => vec![id.to_owned()],
        None => list(root)?
            .into_iter()
            .map(|manifest| manifest.id)
            .collect(),
    };
    ids.into_iter()
        .map(|capture_id| load(root, &capture_id).map(|(manifest, _)| manifest))
        .collect()
}

pub fn prune(root: &Path, older_than_days: u64, confirm: bool) -> Result<CapturePruneResult> {
    if older_than_days == 0 {
        return Err(crate::error::AppError::InvalidArgument(
            "older-than-days must be greater than zero".to_owned(),
        ));
    }
    let age_ms = u128::from(older_than_days)
        .checked_mul(86_400_000)
        .ok_or_else(|| {
            crate::error::AppError::InvalidArgument("older-than-days is too large".to_owned())
        })?;
    let cutoff_unix_ms = unix_ms().saturating_sub(age_ms);
    let candidates = list(root)?
        .into_iter()
        .filter(|manifest| manifest.captured_at_unix_ms < cutoff_unix_ms)
        .map(|manifest| load(root, &manifest.id).map(|_| manifest))
        .collect::<Result<Vec<_>>>()?;
    let mut deleted_ids = Vec::new();
    if confirm {
        for manifest in &candidates {
            remove_capture(root, &manifest.id)?;
            deleted_ids.push(manifest.id.clone());
        }
    }
    Ok(CapturePruneResult {
        older_than_days,
        cutoff_unix_ms,
        dry_run: !confirm,
        candidates,
        deleted_ids,
    })
}

fn ensure_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error(path, error))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(crate::error::AppError::CaptureIntegrity(format!(
            "capture directory is not a real directory: {}",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error(path, error))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(crate::error::AppError::CaptureIntegrity(format!(
            "capture file is not a regular file: {}",
            path.display()
        )));
    }
    Ok(())
}

fn remove_capture(root: &Path, id: &str) -> Result<()> {
    validate_capture_id(id)?;
    ensure_directory(root)?;
    let directory = root.join(id);
    ensure_directory(&directory)?;
    for name in ["manifest.json", "request.pb", "response.pb"] {
        let path = directory.join(name);
        ensure_regular_file(&path)?;
        fs::remove_file(&path).map_err(|error| io_error(&path, error))?;
    }
    fs::remove_dir(&directory).map_err(|error| io_error(&directory, error))?;
    Ok(())
}

fn verify_digest(kind: &str, expected: &str, bytes: &[u8]) -> Result<()> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(crate::error::AppError::CaptureIntegrity(format!(
            "{kind} digest mismatch: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn capture_id() -> String {
    format!(
        "capture-{}-{}-{}",
        unix_ms(),
        std::process::id(),
        CAPTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn validate_capture_id(id: &str) -> Result<()> {
    let suffix = id.strip_prefix("capture-").ok_or_else(|| {
        crate::error::AppError::InvalidArgument("capture ID must start with capture-".to_owned())
    })?;
    if suffix.is_empty()
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
        || id.contains('/')
        || id.contains('\\')
        || id == "."
        || id == ".."
    {
        return Err(crate::error::AppError::InvalidArgument(
            "capture ID contains an invalid path segment".to_owned(),
        ));
    }
    Ok(())
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

#[cfg(unix)]
fn restrict_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, OutputFormat, Theme};
    use crate::proto::oscal::services::v1::{SearchRequest, SearchResponse};
    use std::time::Duration;

    fn test_root(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("oscal-cli-{name}-{}", std::process::id()))
    }

    fn write_fixture(root: &Path, response: &[u8], response_sha256: String) -> String {
        let id = "capture-123-45-0".to_owned();
        let directory = root.join(&id);
        fs::create_dir_all(&directory).expect("fixture directory should be created");
        let request = b"request";
        fs::write(directory.join("request.pb"), request).expect("request should be written");
        fs::write(directory.join("response.pb"), response).expect("response should be written");
        let manifest = CaptureManifest {
            id: id.clone(),
            method: "test.method".to_owned(),
            endpoint: "http://127.0.0.1:50051".to_owned(),
            captured_at_unix_ms: 123,
            request_sha256: sha256(request),
            response_sha256,
            request_bytes: request.len(),
            response_bytes: response.len(),
            proto_lock_sha256: "sha256:fixture".to_owned(),
        };
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should be written");
        id
    }

    #[test]
    fn capture_ids_cannot_escape_root() {
        assert!(validate_capture_id("../manifest").is_err());
        assert!(validate_capture_id("capture-123-45-0").is_ok());
    }

    #[test]
    fn load_rejects_tampered_response() {
        let root = test_root("tampered-response");
        let _ = fs::remove_dir_all(&root);
        let id = write_fixture(&root, b"tampered", sha256(b"original"));

        let result = load(&root, &id);

        assert!(matches!(
            result,
            Err(crate::error::AppError::CaptureIntegrity(message))
                if message.contains("response digest mismatch")
        ));
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn load_accepts_matching_request_and_response() {
        let root = test_root("matching-capture");
        let _ = fs::remove_dir_all(&root);
        let response = b"response";
        let id = write_fixture(&root, response, sha256(response));

        let (_, loaded) = load(&root, &id).expect("matching capture should load");

        assert_eq!(loaded, response);
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn verify_and_prune_require_integrity_and_explicit_confirmation() {
        let root = test_root("verify-prune");
        let _ = fs::remove_dir_all(&root);
        let id = write_fixture(&root, b"response", sha256(b"response"));

        let verified = verify(&root, None).expect("matching captures should verify");
        assert_eq!(verified.len(), 1);

        let dry_run = prune(&root, 1, false).expect("prune should support dry runs");
        assert!(dry_run.dry_run);
        assert_eq!(dry_run.candidates.len(), 1);
        assert!(root.join(&id).is_dir());

        let confirmed = prune(&root, 1, true).expect("confirmed prune should succeed");
        assert!(!confirmed.dry_run);
        assert_eq!(confirmed.deleted_ids, vec![id.clone()]);
        assert!(!root.join(id).exists());

        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[test]
    fn prune_rejects_zero_days() {
        let root = test_root("prune-zero");
        let result = prune(&root, 0, false);
        assert!(matches!(
            result,
            Err(crate::error::AppError::InvalidArgument(message))
                if message.contains("older-than-days")
        ));
    }

    #[test]
    fn write_is_atomic_and_redacts_endpoint_metadata() {
        let root = test_root("atomic-redacted-capture");
        let _ = fs::remove_dir_all(&root);
        let config = AppConfig {
            endpoint: "http://user:secret@observer.example:50051?token=hidden".to_owned(),
            token: None,
            timeout: Duration::from_secs(10),
            tls_domain: None,
            ca_cert: None,
            client_cert: None,
            client_key: None,
            output: OutputFormat::Json,
            theme: Theme::Ledger,
            capture_root: root.clone(),
            capture_enabled: true,
            read_only: false,
            valence: false,
        };

        let manifest = write(
            &config,
            "test.method",
            &SearchRequest::default(),
            &SearchResponse::default(),
        )
        .expect("capture should be written");

        assert_eq!(manifest.endpoint, "http://observer.example:50051");
        assert!(root.join(&manifest.id).is_dir());
        assert!(!root.join(format!(".{}.pending", manifest.id)).exists());
        assert_eq!(list(&root).expect("capture should list").len(), 1);
        fs::remove_dir_all(root).expect("fixture should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn load_rejects_symlinked_capture_files() {
        use std::os::unix::fs::symlink;

        let root = test_root("symlinked-response");
        let outside = test_root("symlinked-response-outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        let id = write_fixture(&root, b"response", sha256(b"response"));
        fs::write(&outside, b"response").expect("outside fixture should be written");
        fs::remove_file(root.join(&id).join("response.pb"))
            .expect("response fixture should be removed");
        symlink(&outside, root.join(&id).join("response.pb"))
            .expect("response symlink should be created");

        let result = load(&root, &id);

        assert!(matches!(
            result,
            Err(crate::error::AppError::CaptureIntegrity(message))
                if message.contains("not a regular file")
        ));
        fs::remove_dir_all(root).expect("fixture root should be removed");
        fs::remove_file(outside).expect("outside fixture should be removed");
    }
}
