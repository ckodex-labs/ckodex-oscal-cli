use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    document::io_safe::{MAX_OSCAL_FILE_BYTES, atomic_write_bytes, read_bounded_utf8_file},
    error::{AppError, Result},
};

/// Maximum permitted waiver lease TTL (10 years / 3,650 days).
pub const MAX_WAIVER_TTL_DAYS: i64 = 3_650;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaiverStatus {
    Active,
    Expired,
    Revoked,
}

impl std::fmt::Display for WaiverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Expired => write!(f, "EXPIRED"),
            Self::Revoked => write!(f, "REVOKED"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerogationLease {
    pub id: String,
    pub rule_id: String,
    pub reason: String,
    pub scope: String,
    pub author: String,
    pub created_at: String,
    pub expires_at: String,
    pub status: WaiverStatus,
    pub fingerprint: String,
}

impl DerogationLease {
    pub fn is_valid(&self) -> bool {
        if self.status != WaiverStatus::Active {
            return false;
        }
        if let Ok(exp) = DateTime::parse_from_rfc3339(&self.expires_at) {
            Utc::now() < exp.with_timezone(&Utc)
        } else {
            false
        }
    }

    pub fn time_remaining_display(&self) -> String {
        if let Ok(exp) = DateTime::parse_from_rfc3339(&self.expires_at) {
            let now = Utc::now();
            let exp_utc = exp.with_timezone(&Utc);
            if now >= exp_utc {
                "EXPIRED".to_string()
            } else {
                let diff = exp_utc - now;
                if diff.num_days() > 0 {
                    format!("{}d {}h", diff.num_days(), diff.num_hours() % 24)
                } else if diff.num_hours() > 0 {
                    format!("{}h {}m", diff.num_hours(), diff.num_minutes() % 60)
                } else {
                    format!("{}m", diff.num_minutes().max(1))
                }
            }
        } else {
            "UNKNOWN".to_string()
        }
    }

    pub fn compute_fingerprint(
        rule_id: &str,
        reason: &str,
        scope: &str,
        author: &str,
        created_at: &str,
        expires_at: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(b"|");
        hasher.update(reason.as_bytes());
        hasher.update(b"|");
        hasher.update(scope.as_bytes());
        hasher.update(b"|");
        hasher.update(author.as_bytes());
        hasher.update(b"|");
        hasher.update(created_at.as_bytes());
        hasher.update(b"|");
        hasher.update(expires_at.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WaiverManager {
    pub leases: Vec<DerogationLease>,
    #[serde(skip)]
    storage_path: Option<PathBuf>,
}

impl WaiverManager {
    pub fn default_path() -> PathBuf {
        PathBuf::from(".mizan/waivers.json")
    }

    pub fn load_or_default() -> Self {
        Self::load_or_recover(&Self::default_path())
    }

    pub fn load_or_recover(path: &Path) -> Self {
        if !path.exists() {
            return Self {
                leases: Vec::new(),
                storage_path: Some(path.to_path_buf()),
            };
        }
        match Self::load_from_path(path) {
            Ok(mgr) => mgr,
            Err(e) => {
                let ts = Utc::now().timestamp();
                let base_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("waivers.json");
                let bak_path = path.with_file_name(format!("{base_name}.corrupt.{ts}.bak"));
                tracing::warn!(
                    "Corrupt waivers file at '{}' ({e}); preserving forensic backup at '{}'",
                    path.display(),
                    bak_path.display()
                );
                let _ = fs::rename(path, &bak_path);
                Self {
                    leases: Vec::new(),
                    storage_path: Some(path.to_path_buf()),
                }
            }
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                leases: Vec::new(),
                storage_path: Some(path.to_path_buf()),
            });
        }
        let content = read_bounded_utf8_file(path, MAX_OSCAL_FILE_BYTES).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to read waivers from {}: {e}",
                path.display()
            ))
        })?;
        let mut manager: Self = serde_json::from_str(&content).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to parse waivers JSON from {}: {e}",
                path.display()
            ))
        })?;
        manager.storage_path = Some(path.to_path_buf());
        manager.reconcile_expired();
        Ok(manager)
    }

    pub fn save_default(&self) -> Result<()> {
        let target = self.storage_path.clone().unwrap_or_else(Self::default_path);
        self.save_to_path(&target)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        atomic_write_bytes(path, json_str.as_bytes()).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to write waivers to {}: {e}",
                path.display()
            ))
        })?;
        Ok(())
    }

    pub fn reconcile_expired(&mut self) {
        let now = Utc::now();
        for lease in &mut self.leases {
            if lease.status == WaiverStatus::Active
                && let Ok(exp) = DateTime::parse_from_rfc3339(&lease.expires_at)
                && now >= exp.with_timezone(&Utc)
            {
                lease.status = WaiverStatus::Expired;
            }
        }
    }

    pub fn create_waiver(
        &mut self,
        rule_id: &str,
        reason: &str,
        ttl_str: &str,
        scope: Option<&str>,
        author: Option<&str>,
    ) -> Result<DerogationLease> {
        let duration = parse_ttl(ttl_str)?;
        let now = Utc::now();
        let expires_dt = now.checked_add_signed(duration).ok_or_else(|| {
            AppError::InvalidArgument(format!("TTL duration '{ttl_str}' overflows timestamp"))
        })?;
        let expires_at = expires_dt.to_rfc3339();
        let created_at = now.to_rfc3339();
        let scope_val = scope.unwrap_or("*").to_string();
        let author_val = author
            .map(str::to_string)
            .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "mizan-dev".to_string()));

        let short_uuid: String = uuid::Uuid::new_v4().to_string().chars().take(8).collect();
        let id = format!("waiver-{short_uuid}");
        let fingerprint = DerogationLease::compute_fingerprint(
            rule_id,
            reason,
            &scope_val,
            &author_val,
            &created_at,
            &expires_at,
        );

        let lease = DerogationLease {
            id,
            rule_id: rule_id.to_string(),
            reason: reason.to_string(),
            scope: scope_val,
            author: author_val,
            created_at,
            expires_at,
            status: WaiverStatus::Active,
            fingerprint,
        };

        self.leases.push(lease.clone());
        self.save_default()?;
        Ok(lease)
    }

    pub fn revoke_waiver(&mut self, waiver_id: &str) -> Result<DerogationLease> {
        let lease = self
            .leases
            .iter_mut()
            .find(|l| l.id == waiver_id || l.id.trim_start_matches("waiver-") == waiver_id)
            .ok_or_else(|| {
                AppError::Configuration(format!("Waiver lease '{waiver_id}' not found"))
            })?;
        lease.status = WaiverStatus::Revoked;
        let cloned = lease.clone();
        self.save_default()?;
        Ok(cloned)
    }

    pub fn find_active_waiver(&self, rule_id: &str, target: &str) -> Option<&DerogationLease> {
        self.leases.iter().find(|l| {
            l.is_valid()
                && (l.rule_id == rule_id || l.rule_id == "*")
                && (l.scope == "*" || l.scope == target || target.contains(&l.scope))
        })
    }

    pub fn list(&self) -> &[DerogationLease] {
        &self.leases
    }
}

pub fn parse_ttl(ttl: &str) -> Result<TimeDelta> {
    let ttl = ttl.trim();
    if ttl.is_empty() {
        return Err(AppError::InvalidArgument("TTL cannot be empty".to_string()));
    }

    let max_delta = TimeDelta::try_days(MAX_WAIVER_TTL_DAYS).ok_or_else(|| {
        AppError::InvalidArgument("Internal max TTL constant invalid".to_string())
    })?;

    // Check if string looks like an un-suffixed integer (optional +/- followed by digits)
    let digits_part = ttl.strip_prefix(['+', '-']).unwrap_or(ttl);
    if !digits_part.is_empty() && digits_part.chars().all(|c| c.is_ascii_digit()) {
        let days = ttl.parse::<i64>().map_err(|_| {
            AppError::InvalidArgument(format!(
                "TTL duration '{ttl}' overflows supported range (max {MAX_WAIVER_TTL_DAYS}d)"
            ))
        })?;
        if days <= 0 {
            return Err(AppError::InvalidArgument(
                "TTL duration must be greater than zero".to_string(),
            ));
        }
        if days > MAX_WAIVER_TTL_DAYS {
            return Err(AppError::InvalidArgument(format!(
                "TTL duration '{ttl}' exceeds maximum cap of {MAX_WAIVER_TTL_DAYS} days"
            )));
        }
        return TimeDelta::try_days(days).ok_or_else(|| {
            AppError::InvalidArgument(format!("TTL duration '{ttl}' overflows TimeDelta"))
        });
    }

    // Char-boundary-safe split of unit suffix
    let mut chars = ttl.chars();
    let unit_char = chars
        .next_back()
        .ok_or_else(|| AppError::InvalidArgument("TTL cannot be empty".to_string()))?;
    let num_str = chars.as_str();

    let num: i64 = num_str.parse().map_err(|_| {
        AppError::InvalidArgument(format!(
            "Invalid TTL duration number '{num_str}'. Example: 30m, 24h, 7d"
        ))
    })?;

    if num <= 0 {
        return Err(AppError::InvalidArgument(
            "TTL duration must be greater than zero".to_string(),
        ));
    }

    let delta = match unit_char.to_ascii_lowercase() {
        'm' => TimeDelta::try_minutes(num),
        'h' => TimeDelta::try_hours(num),
        'd' => TimeDelta::try_days(num),
        'w' => TimeDelta::try_weeks(num),
        _ => {
            return Err(AppError::InvalidArgument(format!(
                "Unknown TTL unit '{unit_char}'. Use 'm' (minutes), 'h' (hours), 'd' (days), or 'w' (weeks)"
            )));
        }
    }
    .ok_or_else(|| {
        AppError::InvalidArgument(format!(
            "TTL duration '{ttl}' overflows supported time delta"
        ))
    })?;

    if delta > max_delta {
        return Err(AppError::InvalidArgument(format!(
            "TTL duration '{ttl}' exceeds maximum permitted cap of {MAX_WAIVER_TTL_DAYS} days"
        )));
    }

    Ok(delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ttl() {
        assert_eq!(
            parse_ttl("30m").unwrap(),
            TimeDelta::try_minutes(30).unwrap()
        );
        assert_eq!(parse_ttl("24h").unwrap(), TimeDelta::try_hours(24).unwrap());
        assert_eq!(parse_ttl("7d").unwrap(), TimeDelta::try_days(7).unwrap());
        assert!(parse_ttl("-5h").is_err());
        assert!(parse_ttl("invalid").is_err());
    }

    #[test]
    fn test_parse_ttl_rejects_overflow_and_multibyte_utf8_without_panic() {
        for bad in [
            "999999999d",
            "9999999999999999999d",
            "99999999999999999999999999999",
            "3651d",
            "600w",
            "10日",
            "5µ",
            "🚀",
            "0d",
        ] {
            let err = parse_ttl(bad).expect_err(&format!("expected error for TTL '{bad}'"));
            assert!(
                matches!(err, AppError::InvalidArgument(_)),
                "expected AppError::InvalidArgument for '{bad}', got {err:?}"
            );
        }
        // Boundary: 3650d is accepted
        assert!(parse_ttl("3650d").is_ok());
    }

    #[test]
    fn test_waiver_lifecycle_and_corrupt_backup() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-waiver-{}", uuid::Uuid::new_v4()));
        let waiver_file = temp_dir.join(".mizan/waivers.json");
        fs::create_dir_all(waiver_file.parent().unwrap()).unwrap();

        let mut mgr = WaiverManager::load_from_path(&waiver_file).unwrap();
        assert_eq!(mgr.list().len(), 0);

        let lease = mgr
            .create_waiver(
                "cis-k8s-5.2.1",
                "Legacy ingress controller",
                "24h",
                Some("workload.yaml"),
                Some("dev-team"),
            )
            .unwrap();

        // Must persist to custom waiver_file rather than polluting CWD
        assert!(waiver_file.exists());
        let reloaded = WaiverManager::load_from_path(&waiver_file).unwrap();
        assert_eq!(reloaded.list().len(), 1);

        assert_eq!(lease.rule_id, "cis-k8s-5.2.1");
        assert!(lease.is_valid());
        assert!(lease.fingerprint.starts_with("sha256:"));

        // Match active waiver
        let matched = mgr.find_active_waiver("cis-k8s-5.2.1", "workload.yaml");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, lease.id);

        // Does not match different rule
        assert!(
            mgr.find_active_waiver("cis-k8s-5.2.6", "workload.yaml")
                .is_none()
        );

        // Revoke
        mgr.revoke_waiver(&lease.id).unwrap();
        assert!(
            mgr.find_active_waiver("cis-k8s-5.2.1", "workload.yaml")
                .is_none()
        );

        // Corrupt file recovery preserves a .corrupt.<ts>.bak backup
        fs::write(&waiver_file, "{corrupt-waiver-json").unwrap();
        let recovered = WaiverManager::load_or_recover(&waiver_file);
        assert!(recovered.list().is_empty());
        let backups: Vec<_> = fs::read_dir(waiver_file.parent().unwrap())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".corrupt."))
            .collect();
        assert_eq!(backups.len(), 1, "corrupt backup must be preserved");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
