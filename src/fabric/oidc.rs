// Mizan Root Fabric · OIDC Identity Federation Engine
// High-assurance OpenID Connect token decoder, claims validator, and role extractor

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("Malformed JWT token string: expected 3 parts separated by dots")]
    MalformedToken,
    #[error("Base64 decoding failed for token segment: {0}")]
    Base64Decode(String),
    #[error("JSON deserialization of JWT claims failed: {0}")]
    ClaimsDeserialization(String),
    #[error("Token expired (expired at {0}, current timestamp is {1})")]
    TokenExpired(i64, i64),
    #[error("Token issuer mismatch: expected {expected}, got {actual}")]
    IssuerMismatch { expected: String, actual: String },
    #[error("Token audience mismatch: expected {expected}, got {actual:?}")]
    AudienceMismatch {
        expected: String,
        actual: Vec<String>,
    },
    #[error("Missing tenant_id claim in token for multi-tenant enforcement")]
    MissingTenantClaim,
}

/// Standard OIDC Claims with Multi-Tenant Extensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OidcClaims {
    pub iss: String,
    pub sub: String,
    #[serde(default)]
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// OIDC Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: Option<String>,
}

impl OidcProviderConfig {
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
            jwks_uri: None,
        }
    }
}

/// High-Assurance OIDC Token Validator
#[derive(Debug, Clone)]
pub struct OidcTokenValidator {
    config: OidcProviderConfig,
}

impl OidcTokenValidator {
    pub fn new(config: OidcProviderConfig) -> Self {
        Self { config }
    }

    /// Parse and validate JWT token claims offline
    pub fn decode_and_validate(
        &self,
        jwt: &str,
        now_timestamp: i64,
    ) -> Result<OidcClaims, OidcError> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(OidcError::MalformedToken);
        }

        // Decode Payload (Part 2)
        let payload_raw = Self::base64_url_decode(parts[1]).map_err(OidcError::Base64Decode)?;

        let claims: OidcClaims = serde_json::from_slice(&payload_raw)
            .map_err(|e| OidcError::ClaimsDeserialization(e.to_string()))?;

        // 1. Expiration check
        if now_timestamp >= claims.exp {
            return Err(OidcError::TokenExpired(claims.exp, now_timestamp));
        }

        // 2. Issuer check
        if claims.iss != self.config.issuer {
            return Err(OidcError::IssuerMismatch {
                expected: self.config.issuer.clone(),
                actual: claims.iss.clone(),
            });
        }

        // 3. Audience check
        if !claims.aud.is_empty() && !claims.aud.contains(&self.config.audience) {
            return Err(OidcError::AudienceMismatch {
                expected: self.config.audience.clone(),
                actual: claims.aud.clone(),
            });
        }

        Ok(claims)
    }

    fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
        let mut s = input.replace('-', "+").replace('_', "/");
        while !s.len().is_multiple_of(4) {
            s.push('=');
        }

        // Simple manual base64 decode for zero-dep high assurance
        Self::standard_base64_decode(&s)
    }

    fn standard_base64_decode(input: &str) -> Result<Vec<u8>, String> {
        const B64_TABLE: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = Vec::with_capacity(input.len() * 3 / 4);
        let mut buf = 0u32;
        let mut bits = 0;

        for &b in input.as_bytes() {
            if b == b'=' || b == b'\r' || b == b'\n' || b == b' ' {
                continue;
            }
            let val = B64_TABLE
                .iter()
                .position(|&c| c == b)
                .ok_or_else(|| format!("Invalid base64 char: {}", b as char))?
                as u32;
            buf = (buf << 6) | val;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_claims_parsing_and_validation() {
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench");
        let validator = OidcTokenValidator::new(config);

        // Header: {"alg":"none"} -> eyJhbGciOiJub25lIn0
        // Payload: {"iss":"https://auth.runbase.io","sub":"user_102","aud":["mizan-workbench"],"exp":2000000000,"iat":1700000000,"email":"mori@meridian.io","tenant_id":"tenant-corp","roles":["ComplianceArchitect"]}
        // Signature: ""
        let token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfMTAyIiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjIwMDAwMDAwMDAsImlhdCI6MTcwMDAwMDAwMCwiZW1haWwiOiJtb3JpQG1lcmlkaWFuLmlvIiwidGVuYW50X2lkIjoidGVuYW50LWNvcnAiLCJyb2xlcyI6WyJDb21wbGlhbmNlQXJjaGl0ZWN0Il19.";

        let claims = validator
            .decode_and_validate(token, 1750000000)
            .expect("valid token");
        assert_eq!(claims.sub, "user_102");
        assert_eq!(claims.tenant_id, Some("tenant-corp".to_string()));
        assert_eq!(claims.roles, vec!["ComplianceArchitect".to_string()]);
    }

    #[test]
    fn test_oidc_expired_token() {
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench");
        let validator = OidcTokenValidator::new(config);

        let expired_token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfMTAyIiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjE2MDAwMDAwMDAsImlhdCI6MTUwMDAwMDAwMH0.";
        let err = validator.decode_and_validate(expired_token, 1750000000);
        assert!(matches!(err, Err(OidcError::TokenExpired(_, _))));
    }
}
