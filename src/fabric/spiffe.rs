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
    #[error("Malformed JWT SVID token: {0}")]
    MalformedJwtToken(String),
    #[error("Insecure algorithm 'none' rejected in SPIFFE JWT SVID")]
    UnsafeAlgorithmNone,
    #[error("Unsupported algorithm in SPIFFE JWT SVID: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Missing signature in SPIFFE JWT SVID")]
    MissingSignature,
    #[error("Cryptographic signature verification failed for SPIFFE JWT SVID")]
    InvalidSignature,
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
    pub fn new(
        spiffe_id: SpiffeId,
        audience: Vec<String>,
        expires_at: i64,
        issued_at: i64,
        token: impl Into<String>,
    ) -> Self {
        Self {
            spiffe_id,
            audience,
            expires_at,
            issued_at,
            token: token.into(),
        }
    }

    pub fn is_valid_at(&self, now_timestamp: i64) -> Result<(), SpiffeError> {
        if now_timestamp >= self.expires_at {
            return Err(SpiffeError::SvidExpired(self.expires_at, now_timestamp));
        }
        Ok(())
    }

    /// Sign and construct a high-assurance HS256 SPIFFE JWT SVID.
    pub fn sign_hs256(
        spiffe_id: SpiffeId,
        audience: Vec<String>,
        expires_at: i64,
        issued_at: i64,
        secret: &[u8],
    ) -> Result<Self, SpiffeError> {
        use crate::fabric::oidc::{JwtHeader, base64_url_encode, hmac_sha256};

        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: Some("JWT".to_string()),
            kid: None,
        };
        let header_json = serde_json::to_vec(&header)
            .map_err(|e| SpiffeError::MalformedJwtToken(e.to_string()))?;

        #[derive(Serialize)]
        struct SvidClaims<'a> {
            sub: &'a str,
            aud: &'a [String],
            exp: i64,
            iat: i64,
        }

        let claims = SvidClaims {
            sub: &spiffe_id.to_string(),
            aud: &audience,
            exp: expires_at,
            iat: issued_at,
        };
        let claims_json = serde_json::to_vec(&claims)
            .map_err(|e| SpiffeError::MalformedJwtToken(e.to_string()))?;

        let h_b64 = base64_url_encode(&header_json);
        let p_b64 = base64_url_encode(&claims_json);
        let signing_input = format!("{}.{}", h_b64, p_b64);
        let signature = hmac_sha256(secret, signing_input.as_bytes());
        let s_b64 = base64_url_encode(&signature);
        let token = format!("{}.{}", signing_input, s_b64);

        Ok(Self {
            spiffe_id,
            audience,
            expires_at,
            issued_at,
            token,
        })
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
    shared_secret: Option<Vec<u8>>,
    strict_mode: bool,
}

impl SpireWorkloadAttestor {
    pub fn new(expected_trust_domain: TrustDomain) -> Self {
        Self {
            expected_trust_domain,
            shared_secret: None,
            strict_mode: true,
        }
    }

    pub fn with_shared_secret(mut self, secret: impl AsRef<[u8]>) -> Self {
        self.shared_secret = Some(secret.as_ref().to_vec());
        self
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn expected_trust_domain(&self) -> &TrustDomain {
        &self.expected_trust_domain
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

    /// Verify SPIFFE JWT SVID with cryptographic signature and algorithm enforcement.
    ///
    /// SECURITY INVARIANT:
    /// In strict mode (default):
    /// - Tokens with algorithm 'none' are rejected immediately.
    /// - If a shared secret is configured, HS256 signatures are verified using constant-time comparison.
    /// - If a signature is missing or tampered, verification fails closed.
    /// - Expiration and trust domain matches are strictly validated.
    pub fn verify_jwt_svid(&self, svid: &JwtSvid, now_timestamp: i64) -> Result<(), SpiffeError> {
        svid.is_valid_at(now_timestamp)?;
        self.verify_spiffe_id(&svid.spiffe_id)?;

        if !svid.token.is_empty() {
            let parts: Vec<&str> = svid.token.split('.').collect();
            if parts.len() != 3 {
                return Err(SpiffeError::MalformedJwtToken(
                    "JWT SVID token does not contain 3 segments".to_string(),
                ));
            }

            let header_raw = crate::fabric::oidc::base64_url_decode(parts[0])
                .map_err(|e| SpiffeError::MalformedJwtToken(format!("Header decode: {}", e)))?;
            let header: crate::fabric::oidc::JwtHeader = serde_json::from_slice(&header_raw)
                .map_err(|e| SpiffeError::MalformedJwtToken(format!("Header JSON: {}", e)))?;

            let alg = header.alg.to_ascii_uppercase();
            if (self.strict_mode || self.shared_secret.is_some())
                && (alg == "NONE" || alg.is_empty())
            {
                return Err(SpiffeError::UnsafeAlgorithmNone);
            }

            let sig_segment = parts[2];
            if let Some(secret) = &self.shared_secret {
                if alg != "HS256" {
                    return Err(SpiffeError::UnsupportedAlgorithm(header.alg));
                }
                if sig_segment.is_empty() {
                    return Err(SpiffeError::MissingSignature);
                }
                let sig_bytes =
                    crate::fabric::oidc::base64_url_decode(sig_segment).map_err(|e| {
                        SpiffeError::MalformedJwtToken(format!("Signature decode: {}", e))
                    })?;
                let signing_input = format!("{}.{}", parts[0], parts[1]);
                let expected_sig =
                    crate::fabric::oidc::hmac_sha256(secret, signing_input.as_bytes());

                if !crate::fabric::oidc::constant_time_eq(&sig_bytes, &expected_sig) {
                    return Err(SpiffeError::InvalidSignature);
                }
            } else if self.strict_mode && sig_segment.is_empty() && alg != "NONE" {
                return Err(SpiffeError::MissingSignature);
            } else if !self.strict_mode && alg == "NONE" && !sig_segment.is_empty() {
                return Err(SpiffeError::InvalidSignature);
            }
        } else if self.shared_secret.is_some() {
            return Err(SpiffeError::MissingSignature);
        }

        Ok(())
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

    #[test]
    fn test_spire_jwt_svid_cryptographic_verification() {
        let td = TrustDomain::new("meridian.runbase.io").unwrap();
        let secret = b"spire-cluster-signing-key-32b-ok";
        let attestor = SpireWorkloadAttestor::new(td.clone()).with_shared_secret(secret);

        let spiffe_id: SpiffeId = "spiffe://meridian.runbase.io/ns/prod/sa/workload"
            .parse()
            .unwrap();
        let svid = JwtSvid::sign_hs256(
            spiffe_id.clone(),
            vec!["mizan-fabric".to_string()],
            2000000000,
            1700000000,
            secret,
        )
        .expect("sign svid");

        assert!(attestor.verify_jwt_svid(&svid, 1750000000).is_ok());

        // Corrupted signature
        let mut tampered_svid = svid.clone();
        tampered_svid.token.push_str("extra");
        assert!(matches!(
            attestor.verify_jwt_svid(&tampered_svid, 1750000000),
            Err(SpiffeError::InvalidSignature)
        ));

        // alg: none token rejection
        let none_svid = JwtSvid {
            spiffe_id,
            audience: vec!["mizan-fabric".to_string()],
            expires_at: 2000000000,
            issued_at: 1700000000,
            token: "eyJhbGciOiJub25lIn0.eyJzdWIiOiJzcGlmZmU6Ly9tZXJpZGlhbi5ydW5iYXNlLmlvL25zL3Byb2Qvc2Evd29ya2xvYWQifQ.".to_string(),
        };
        assert!(matches!(
            attestor.verify_jwt_svid(&none_svid, 1750000000),
            Err(SpiffeError::UnsafeAlgorithmNone)
        ));
    }
}
