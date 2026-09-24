use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IncoherenceKind {
    DigestMismatch,
    ChainInvalid,
    PartialCrud,
    ProtocolDrift,
    FsmContradiction,
    EvidenceConflict,
}

impl fmt::Display for IncoherenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::DigestMismatch => "digest-mismatch",
            Self::ChainInvalid => "chain-invalid",
            Self::PartialCrud => "partial-crud",
            Self::ProtocolDrift => "protocol-drift",
            Self::FsmContradiction => "fsm-contradiction",
            Self::EvidenceConflict => "evidence-conflict",
        };
        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("connection to {endpoint} failed: {message}. {hint}")]
    Connection {
        endpoint: String,
        message: String,
        hint: &'static str,
    },
    #[error("RPC error: {0}")]
    Rpc(Box<tonic::Status>),
    #[error(
        "read path unavailable: {method}; the server returned UNIMPLEMENTED. Use a matching OSCALify protocol revision"
    )]
    UnsupportedReadPath { method: &'static str },
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("protobuf error: {0}")]
    Protobuf(#[from] prost::DecodeError),
    #[error("descriptor error: {0}")]
    Descriptor(String),
    #[error("anti-policy: method `{method}` (class: {class}) is not permitted: {reason}")]
    Anti {
        method: String,
        class: &'static str,
        reason: String,
    },
    #[error("negative result: class `{class}` failed with status `{status}`")]
    Negative { class: &'static str, status: String },
    #[allow(dead_code)]
    #[error("incoherent state: {kind} — {detail}")]
    Incoherent {
        kind: IncoherenceKind,
        detail: String,
    },
    #[error("capture integrity error: {0}")]
    CaptureIntegrity(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<tonic::Status> for AppError {
    fn from(status: tonic::Status) -> Self {
        Self::Rpc(Box::new(status))
    }
}

impl AppError {
    pub(crate) fn from_unimplemented(method: &'static str, status: tonic::Status) -> Self {
        if status.code() == tonic::Code::Unimplemented {
            Self::UnsupportedReadPath { method }
        } else {
            Self::Rpc(Box::new(status))
        }
    }
}

pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> AppError {
    AppError::Io {
        path: path.into(),
        source,
    }
}

pub(crate) fn redact_endpoint(endpoint: &str) -> String {
    let Some((scheme, remainder)) = endpoint.split_once("://") else {
        return "<invalid-endpoint>".to_owned();
    };
    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let authority = &remainder[..authority_end];
    let host = authority.rsplit('@').next().unwrap_or(authority);
    format!("{scheme}://{host}")
}

#[cfg(test)]
mod tests {
    use super::{AppError, IncoherenceKind, redact_endpoint};
    use tonic::{Code, Status};

    #[test]
    fn endpoint_redaction_removes_userinfo_and_query() {
        assert_eq!(
            redact_endpoint("https://observer:secret@example.test:443/rpc?token=hidden"),
            "https://example.test:443"
        );
    }

    #[test]
    fn unsupported_read_path_is_distinct_from_transport_failure() {
        let status = Status::new(Code::Unimplemented, "method unavailable");
        let error =
            AppError::from_unimplemented("TransparencyGraphService.ListProjectionEvents", status);
        assert_eq!(
            error.to_string(),
            "read path unavailable: TransparencyGraphService.ListProjectionEvents; the server returned UNIMPLEMENTED. Use a matching OSCALify protocol revision"
        );
    }

    #[test]
    fn incoherence_kind_renders_kebab_case() {
        assert_eq!(IncoherenceKind::ChainInvalid.to_string(), "chain-invalid");
        assert_eq!(
            IncoherenceKind::EvidenceConflict.to_string(),
            "evidence-conflict"
        );
    }

    #[test]
    fn anti_error_includes_method_class_and_reason() {
        let error = AppError::Anti {
            method: "oscal.create.catalog".to_owned(),
            class: "create",
            reason: "read-only gate denies mutating RPC".to_owned(),
        };
        let text = error.to_string();
        assert!(text.contains("oscal.create.catalog"));
        assert!(text.contains("create"));
        assert!(text.contains("read-only"));
    }

    #[test]
    fn negative_error_includes_class_and_status() {
        let error = AppError::Negative {
            class: "chain",
            status: "invalid".to_owned(),
        };
        let text = error.to_string();
        assert!(text.contains("chain"));
        assert!(text.contains("invalid"));
    }

    #[test]
    fn incoherent_error_includes_kind_and_detail() {
        let error = AppError::Incoherent {
            kind: IncoherenceKind::DigestMismatch,
            detail: "sha256 mismatch".to_owned(),
        };
        let text = error.to_string();
        assert!(text.contains("digest-mismatch"));
        assert!(text.contains("sha256 mismatch"));
    }
}
