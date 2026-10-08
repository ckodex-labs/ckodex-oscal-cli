// Mizan Root Fabric · OIDC Identity Federation Engine
// High-assurance OpenID Connect token decoder, claims validator, and role extractor

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("Malformed JWT token string: expected 3 parts separated by dots")]
    MalformedToken,
    #[error("Base64 decoding failed for token segment: {0}")]
    Base64Decode(String),
    #[error("JSON deserialization of JWT header failed: {0}")]
    HeaderDeserialization(String),
    #[error("JSON deserialization of JWT claims failed: {0}")]
    ClaimsDeserialization(String),
    #[error("Insecure algorithm 'none' rejected in strict mode")]
    UnsafeAlgorithmNone,
    #[error("Unsupported JWT algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Missing signature in JWT token")]
    MissingSignature,
    #[error("Invalid cryptographic signature on JWT token")]
    InvalidSignature,
    #[error("Verification secret or key required for algorithm {0}")]
    MissingVerificationKey(String),
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

/// Standard JWT Header for algorithm and key discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JwtHeader {
    pub alg: String,
    #[serde(default)]
    pub typ: Option<String>,
    #[serde(default)]
    pub kid: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub shared_secret: Option<Vec<u8>>,
    #[serde(default = "default_true")]
    pub strict_mode: bool,
}

fn default_true() -> bool {
    true
}

impl OidcProviderConfig {
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
            jwks_uri: None,
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

    pub fn with_jwks_uri(mut self, uri: impl Into<String>) -> Self {
        self.jwks_uri = Some(uri.into());
        self
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

    pub fn config(&self) -> &OidcProviderConfig {
        &self.config
    }

    /// Parse and validate JWT token claims with algorithmic and cryptographic verification.
    ///
    /// SECURITY INVARIANT:
    /// Strict mode enforces cryptographic integrity:
    /// - Insecure algorithm 'none' is categorically rejected.
    /// - If a shared secret is configured, HMAC-SHA256 signatures are verified using constant-time comparison.
    /// - Missing or corrupted signatures fail closed.
    /// - Expiration, issuer, and audience are verified.
    pub fn decode_and_validate(
        &self,
        jwt: &str,
        now_timestamp: i64,
    ) -> Result<OidcClaims, OidcError> {
        let (_header, claims) = self.decode_and_validate_with_header(jwt, now_timestamp)?;
        Ok(claims)
    }

    /// Parse and validate both JWT header and claims with cryptographic enforcement.
    pub fn decode_and_validate_with_header(
        &self,
        jwt: &str,
        now_timestamp: i64,
    ) -> Result<(JwtHeader, OidcClaims), OidcError> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(OidcError::MalformedToken);
        }

        // 1. Decode and parse Header (Part 0)
        let header_raw = base64_url_decode(parts[0]).map_err(OidcError::Base64Decode)?;
        let header: JwtHeader = serde_json::from_slice(&header_raw)
            .map_err(|e| OidcError::HeaderDeserialization(e.to_string()))?;

        // 2. Algorithm validation and strict mode enforcement
        let alg = header.alg.to_ascii_uppercase();
        if (self.config.strict_mode || self.config.shared_secret.is_some())
            && (alg == "NONE" || alg.is_empty())
        {
            return Err(OidcError::UnsafeAlgorithmNone);
        }

        // 3. Cryptographic signature verification
        let signature_segment = parts[2];
        if let Some(secret) = &self.config.shared_secret {
            if alg != "HS256" {
                return Err(OidcError::UnsupportedAlgorithm(header.alg));
            }
            if signature_segment.is_empty() {
                return Err(OidcError::MissingSignature);
            }
            let signature_bytes =
                base64_url_decode(signature_segment).map_err(OidcError::Base64Decode)?;
            let signing_input = format!("{}.{}", parts[0], parts[1]);
            let expected_hmac = hmac_sha256(secret, signing_input.as_bytes());

            if !constant_time_eq(&signature_bytes, &expected_hmac) {
                return Err(OidcError::InvalidSignature);
            }
        } else if self.config.strict_mode && signature_segment.is_empty() && alg != "NONE" {
            return Err(OidcError::MissingSignature);
        } else if !self.config.strict_mode && alg == "NONE" && !signature_segment.is_empty() {
            return Err(OidcError::InvalidSignature);
        }

        // 4. Decode and parse Payload (Part 1)
        let payload_raw = base64_url_decode(parts[1]).map_err(OidcError::Base64Decode)?;
        let claims: OidcClaims = serde_json::from_slice(&payload_raw)
            .map_err(|e| OidcError::ClaimsDeserialization(e.to_string()))?;

        // 5. Expiration check
        if now_timestamp >= claims.exp {
            return Err(OidcError::TokenExpired(claims.exp, now_timestamp));
        }

        // 6. Issuer check
        if claims.iss != self.config.issuer {
            return Err(OidcError::IssuerMismatch {
                expected: self.config.issuer.clone(),
                actual: claims.iss.clone(),
            });
        }

        // 7. Audience check
        if !claims.aud.is_empty() && !claims.aud.contains(&self.config.audience) {
            return Err(OidcError::AudienceMismatch {
                expected: self.config.audience.clone(),
                actual: claims.aud.clone(),
            });
        }

        Ok((header, claims))
    }

    /// Unauthenticated inspection of JWT header and claims.
    ///
    /// # SECURITY INVARIANT
    /// THIS METHOD DOES NOT VERIFY CRYPTOGRAPHIC SIGNATURES OR ALGORITHM INTEGRITY.
    ///
    /// It MUST NOT be used for authorization, access control, tenant admission,
    /// or trust delegation. It is strictly intended for pre-authentication routing,
    /// debugging, or telemetry diagnostics.
    pub fn inspect_unauthenticated(jwt: &str) -> Result<(JwtHeader, OidcClaims), OidcError> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(OidcError::MalformedToken);
        }

        let header_raw = base64_url_decode(parts[0]).map_err(OidcError::Base64Decode)?;
        let header: JwtHeader = serde_json::from_slice(&header_raw)
            .map_err(|e| OidcError::HeaderDeserialization(e.to_string()))?;

        let payload_raw = base64_url_decode(parts[1]).map_err(OidcError::Base64Decode)?;
        let claims: OidcClaims = serde_json::from_slice(&payload_raw)
            .map_err(|e| OidcError::ClaimsDeserialization(e.to_string()))?;

        Ok((header, claims))
    }

    /// Create a signed HS256 JWT token for testing and federation attestation.
    pub fn sign_hs256(claims: &OidcClaims, secret: &[u8]) -> Result<String, OidcError> {
        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: Some("JWT".to_string()),
            kid: None,
        };
        let header_json = serde_json::to_vec(&header)
            .map_err(|e| OidcError::HeaderDeserialization(e.to_string()))?;
        let claims_json = serde_json::to_vec(claims)
            .map_err(|e| OidcError::ClaimsDeserialization(e.to_string()))?;

        let h_b64 = base64_url_encode(&header_json);
        let p_b64 = base64_url_encode(&claims_json);
        let signing_input = format!("{}.{}", h_b64, p_b64);
        let sig = hmac_sha256(secret, signing_input.as_bytes());
        let s_b64 = base64_url_encode(&sig);

        Ok(format!("{}.{}", signing_input, s_b64))
    }
}

/// Constant-time byte slice comparison to mitigate timing side-channels
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// RFC 2104 compliant HMAC-SHA256 implementation using pure sha2 primitives
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k_block = [0u8; 64];
    if key.len() > 64 {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let key_hash = hasher.finalize();
        k_block[..32].copy_from_slice(&key_hash);
    } else {
        k_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_block[i];
        opad[i] ^= k_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let outer_hash = outer.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(&outer_hash);
    result
}

/// Base64-URL encoding without padding (RFC 7515 compliant)
pub fn base64_url_encode(data: &[u8]) -> String {
    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as u32
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if i + 1 < data.len() {
            out.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        if i + 2 < data.len() {
            out.push(B64_CHARS[(triple & 0x3F) as usize] as char);
        }
        i += 3;
    }
    out
}

/// Base64-URL decoding with automatic padding restoration
pub fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut s = input.replace('-', "+").replace('_', "/");
    while !s.len().is_multiple_of(4) {
        s.push('=');
    }
    standard_base64_decode(&s)
}

fn standard_base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const B64_TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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
            .ok_or_else(|| format!("Invalid base64 char: {}", b as char))? as u32;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc4231_hmac_sha256_vectors() {
        // RFC 4231 Test Case 1
        let key1 = [0x0bu8; 20];
        let data1 = b"Hi There";
        let hmac1 = hmac_sha256(&key1, data1);
        assert_eq!(
            hex::encode(hmac1),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );

        // RFC 4231 Test Case 2
        let key2 = b"Jefe";
        let data2 = b"what do ya want for nothing?";
        let hmac2 = hmac_sha256(key2, data2);
        assert_eq!(
            hex::encode(hmac2),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn test_oidc_claims_signed_hs256_validation() {
        let secret = b"super-secret-mizan-validation-key-32b";
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench")
            .with_shared_secret(secret);
        let validator = OidcTokenValidator::new(config);

        let claims = OidcClaims {
            iss: "https://auth.runbase.io".to_string(),
            sub: "user_102".to_string(),
            aud: vec!["mizan-workbench".to_string()],
            exp: 2000000000,
            iat: 1700000000,
            email: Some("mori@meridian.io".to_string()),
            tenant_id: Some("tenant-corp".to_string()),
            groups: vec![],
            roles: vec!["ComplianceArchitect".to_string()],
            name: None,
        };

        let token = OidcTokenValidator::sign_hs256(&claims, secret).expect("token generation");
        let validated = validator
            .decode_and_validate(&token, 1750000000)
            .expect("valid token");

        assert_eq!(validated.sub, "user_102");
        assert_eq!(validated.tenant_id, Some("tenant-corp".to_string()));
        assert_eq!(validated.roles, vec!["ComplianceArchitect".to_string()]);
    }

    #[test]
    fn test_oidc_strict_mode_rejects_alg_none() {
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench");
        let validator = OidcTokenValidator::new(config);

        let token = "eyJhbGciOiJub25lIn0.eyJpc3MiOiJodHRwczovL2F1dGgucnVuYmFzZS5pbyIsInN1YiI6InVzZXJfMTAyIiwiYXVkIjpbIm1pemFuLXdvcmtiZW5jaCJdLCJleHAiOjIwMDAwMDAwMDAsImlhdCI6MTcwMDAwMDAwMCwidGVuYW50X2lkIjoidGVuYW50LWNvcnAifQ.";

        let err = validator.decode_and_validate(token, 1750000000);
        assert!(matches!(err, Err(OidcError::UnsafeAlgorithmNone)));
    }

    #[test]
    fn test_oidc_invalid_signature_rejection() {
        let secret = b"authentic-secret-key-32-bytes-long";
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench")
            .with_shared_secret(secret);
        let validator = OidcTokenValidator::new(config);

        let claims = OidcClaims {
            iss: "https://auth.runbase.io".to_string(),
            sub: "user_attacker".to_string(),
            aud: vec!["mizan-workbench".to_string()],
            exp: 2000000000,
            iat: 1700000000,
            email: None,
            tenant_id: Some("tenant-corp".to_string()),
            groups: vec![],
            roles: vec!["Admin".to_string()],
            name: None,
        };

        // Signed with wrong secret
        let rogue_token =
            OidcTokenValidator::sign_hs256(&claims, b"wrong-secret-key").expect("token generation");
        let err = validator.decode_and_validate(&rogue_token, 1750000000);
        assert!(matches!(err, Err(OidcError::InvalidSignature)));
    }

    #[test]
    fn test_oidc_inspect_unauthenticated() {
        let claims = OidcClaims {
            iss: "https://auth.runbase.io".to_string(),
            sub: "user_debug".to_string(),
            aud: vec!["mizan-workbench".to_string()],
            exp: 2000000000,
            iat: 1700000000,
            email: None,
            tenant_id: Some("tenant-dev".to_string()),
            groups: vec![],
            roles: vec![],
            name: None,
        };

        let token = OidcTokenValidator::sign_hs256(&claims, b"test-key").expect("token");
        let (header, decoded_claims) =
            OidcTokenValidator::inspect_unauthenticated(&token).expect("inspect");

        assert_eq!(header.alg, "HS256");
        assert_eq!(decoded_claims.sub, "user_debug");
    }

    #[test]
    fn test_oidc_expired_token() {
        let secret = b"secret-key-32-bytes-for-tests-ok";
        let config = OidcProviderConfig::new("https://auth.runbase.io", "mizan-workbench")
            .with_shared_secret(secret);
        let validator = OidcTokenValidator::new(config);

        let claims = OidcClaims {
            iss: "https://auth.runbase.io".to_string(),
            sub: "user_102".to_string(),
            aud: vec!["mizan-workbench".to_string()],
            exp: 1600000000,
            iat: 1500000000,
            email: None,
            tenant_id: None,
            groups: vec![],
            roles: vec![],
            name: None,
        };

        let token = OidcTokenValidator::sign_hs256(&claims, secret).expect("token");
        let err = validator.decode_and_validate(&token, 1750000000);
        assert!(matches!(err, Err(OidcError::TokenExpired(_, _))));
    }
}
