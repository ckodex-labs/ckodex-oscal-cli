use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{AppError, Result};

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
}

impl WaiverManager {
    pub fn default_path() -> PathBuf {
        PathBuf::from(".mizan/waivers.json")
    }

    pub fn load_or_default() -> Self {
        Self::load_from_path(&Self::default_path()).unwrap_or_default()
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|e| {
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
        manager.reconcile_expired();
        Ok(manager)
    }

    pub fn save_default(&self) -> Result<()> {
        Self::save_to_path(self, &Self::default_path())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Configuration(format!(
                    "Failed to create directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        fs::write(path, json_str).map_err(|e| {
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
        let expires_at = (now + duration).to_rfc3339();
        let created_at = now.to_rfc3339();
        let scope_val = scope.unwrap_or("*").to_string();
        let author_val = author
            .map(str::to_string)
            .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "mizan-dev".to_string()));

        let id = format!("waiver-{}", &uuid::Uuid::new_v4().to_string()[..8]);
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

pub fn parse_ttl(ttl: &str) -> Result<Duration> {
    let ttl = ttl.trim();
    if ttl.is_empty() {
        return Err(AppError::Configuration("TTL cannot be empty".to_string()));
    }
    let (num_str, unit) = ttl.split_at(ttl.len() - 1);
    let num: i64 = num_str.parse().map_err(|_| {
        AppError::Configuration(format!(
            "Invalid TTL duration number '{num_str}'. Example: 30m, 24h, 7d"
        ))
    })?;

    if num <= 0 {
        return Err(AppError::Configuration(
            "TTL duration must be greater than zero".to_string(),
        ));
    }

    match unit.to_lowercase().as_str() {
        "m" => Ok(Duration::minutes(num)),
        "h" => Ok(Duration::hours(num)),
        "d" => Ok(Duration::days(num)),
        "w" => Ok(Duration::weeks(num)),
        _ => Err(AppError::Configuration(format!(
            "Unknown TTL unit '{unit}'. Use 'm' (minutes), 'h' (hours), 'd' (days), or 'w' (weeks)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ttl() {
        assert_eq!(parse_ttl("30m").unwrap(), Duration::minutes(30));
        assert_eq!(parse_ttl("24h").unwrap(), Duration::hours(24));
        assert_eq!(parse_ttl("7d").unwrap(), Duration::days(7));
        assert!(parse_ttl("-5h").is_err());
        assert!(parse_ttl("invalid").is_err());
    }

    #[test]
    fn test_waiver_lifecycle() {
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

        let _ = fs::remove_dir_all(temp_dir);
    }
}
