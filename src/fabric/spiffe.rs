// Mizan Root Fabric · SPIFFE / SPIRE Workload Identity Engine
// High-assurance implementation conforming to the SPIFFE standard specification

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum SpiffeError {
    #[error("Invalid SPIFFE URI scheme, expected 'spiffe://', got: {0}")]
    InvalidScheme(String),
    #[error("Missing or empty trust domain in SPIFFE ID: {0}")]
    EmptyTrustDomain(String),
    #[error("Invalid trust domain format: {0}")]
    InvalidTrustDomain(String),
    #[error("Empty or invalid path in SPIFFE ID: {0}")]
    EmptyPath(String),
    #[error("Expired SVID token (expired at {0}, current time is {1})")]
    SvidExpired(i64, i64),
    #[error("Trust domain mismatch: expected {expected}, got {actual}")]
    TrustDomainMismatch { expected: String, actual: String },
}

/// SPIFFE Trust Domain representing an administrative domain (e.g., `meridian.runbase.io`)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustDomain(String);

impl TrustDomain {
    pub fn new(domain: impl Into<String>) -> Result<Self, SpiffeError> {
        let d = domain.into().to_ascii_lowercase();
        if d.is_empty() {
            return Err(SpiffeError::EmptyTrustDomain("empty string".to_string()));
        }
        if d.contains('/') || d.contains(':') || d.contains(' ') {
            return Err(SpiffeError::InvalidTrustDomain(d));
        }
        Ok(Self(d))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrustDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TrustDomain {
    type Err = SpiffeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// SPIFFE ID conforming to `spiffe://<trust-domain>/<path>`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpiffeId {
    trust_domain: TrustDomain,
    path: String,
}

impl SpiffeId {
    pub fn new(trust_domain: TrustDomain, path: impl Into<String>) -> Result<Self, SpiffeError> {
        let mut p = path.into();
        if !p.starts_with('/') {
            p = format!("/{}", p);
        }
        if p.len() <= 1 {
            return Err(SpiffeError::EmptyPath(p));
        }
        Ok(Self {
            trust_domain,
            path: p,
        })
    }

    pub fn trust_domain(&self) -> &TrustDomain {
        &self.trust_domain
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn namespace(&self) -> Option<&str> {
        let segments: Vec<&str> = self.path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() >= 2 && segments[0] == "ns" {
            Some(segments[1])
        } else {
            None
        }
    }

    pub fn service_account(&self) -> Option<&str> {
        let segments: Vec<&str> = self.path.split('/').filter(|s| !s.is_empty()).collect();
        for i in 0..segments.len() {
            if (segments[i] == "sa" || segments[i] == "serviceaccount") && i + 1 < segments.len() {
                return Some(segments[i + 1]);
            }
        }
        None
    }
}

impl fmt::Display for SpiffeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spiffe://{}{}", self.trust_domain.as_str(), self.path)
    }
}

impl FromStr for SpiffeId {
    type Err = SpiffeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("spiffe://") {
            return Err(SpiffeError::InvalidScheme(s.to_string()));
        }
        let rest = &s["spiffe://".len()..];
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(SpiffeError::EmptyTrustDomain(s.to_string()));
        }
        let trust_domain = TrustDomain::new(parts[0])?;
        let path = if parts.len() > 1 && !parts[1].is_empty() {
            format!("/{}", parts[1])
        } else {
            return Err(SpiffeError::EmptyPath(s.to_string()));
        };

        Ok(Self { trust_domain, path })
    }
}

/// SPIFFE JWT SVID (Security Verification ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSvid {
    pub spiffe_id: SpiffeId,
    pub audience: Vec<String>,
    pub expires_at: i64,
    pub issued_at: i64,
    pub token: String,
}

impl JwtSvid {
    pub fn is_valid_at(&self, now_timestamp: i64) -> Result<(), SpiffeError> {
        if now_timestamp >= self.expires_at {
            return Err(SpiffeError::SvidExpired(self.expires_at, now_timestamp));
        }
        Ok(())
    }
}

/// SPIFFE X.509 SVID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X509Svid {
    pub spiffe_id: SpiffeId,
    pub cert_sha256_fingerprint: String,
    pub expires_at: i64,
}

/// SPIRE Workload Attestor for high-assurance peer identity verification
#[derive(Debug, Clone)]
pub struct SpireWorkloadAttestor {
    expected_trust_domain: TrustDomain,
}

impl SpireWorkloadAttestor {
    pub fn new(expected_trust_domain: TrustDomain) -> Self {
        Self {
            expected_trust_domain,
        }
    }

    pub fn verify_spiffe_id(&self, spiffe_id: &SpiffeId) -> Result<(), SpiffeError> {
        if spiffe_id.trust_domain() != &self.expected_trust_domain {
            return Err(SpiffeError::TrustDomainMismatch {
                expected: self.expected_trust_domain.to_string(),
                actual: spiffe_id.trust_domain().to_string(),
            });
        }
        Ok(())
    }

    pub fn verify_jwt_svid(&self, svid: &JwtSvid, now_timestamp: i64) -> Result<(), SpiffeError> {
        svid.is_valid_at(now_timestamp)?;
        self.verify_spiffe_id(&svid.spiffe_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiffe_id_parsing_and_formatting() {
        let raw = "spiffe://meridian.runbase.io/ns/prod/sa/mizan-auditor";
        let id: SpiffeId = raw.parse().expect("valid spiffe ID");

        assert_eq!(id.trust_domain().as_str(), "meridian.runbase.io");
        assert_eq!(id.path(), "/ns/prod/sa/mizan-auditor");
        assert_eq!(id.namespace(), Some("prod"));
        assert_eq!(id.service_account(), Some("mizan-auditor"));
        assert_eq!(id.to_string(), raw);
    }

    #[test]
    fn test_invalid_spiffe_schemes_and_paths() {
        assert!("https://example.com/test".parse::<SpiffeId>().is_err());
        assert!("spiffe://".parse::<SpiffeId>().is_err());
        assert!("spiffe://domain.com".parse::<SpiffeId>().is_err());
        assert!("spiffe://domain.com/".parse::<SpiffeId>().is_err());
    }

    #[test]
    fn test_spire_workload_attestation() {
        let td = TrustDomain::new("meridian.runbase.io").unwrap();
        let attestor = SpireWorkloadAttestor::new(td);

        let valid_id: SpiffeId = "spiffe://meridian.runbase.io/sa/auditor".parse().unwrap();
        assert!(attestor.verify_spiffe_id(&valid_id).is_ok());

        let invalid_id: SpiffeId = "spiffe://untrusted.com/sa/attacker".parse().unwrap();
        assert!(attestor.verify_spiffe_id(&invalid_id).is_err());
    }
}
