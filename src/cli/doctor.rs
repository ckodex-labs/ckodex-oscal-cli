#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[derive(Debug, Serialize)]
pub(super) struct DoctorReport {
    schema: &'static str,
    read_only: bool,
    network_checked: bool,
    status: &'static str,
    checks: Vec<DoctorCheck>,
    protocol: DoctorProtocol,
}

#[derive(Debug, Serialize)]
pub(super) struct DoctorCheck {
    id: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DoctorProtocol {
    descriptor_bytes: usize,
    descriptor_sha256: String,
    proto_lock_sha256: String,
    proto_file_count: usize,
    oscal_schema_version: &'static str,
    oscal_schema_manifest_sha256: &'static str,
    oscal_schema_source_commit: &'static str,
    oscal_schema_release_zip_sha256: &'static str,
    capability_negotiation: &'static str,
}

pub(super) struct EndpointAssessment {
    endpoint: Option<String>,
    check: DoctorCheck,
}

pub fn run_doctor(args: &crate::config::Cli) -> Result<()> {
    if args.format == OutputFormat::Proto {
        return Err(AppError::Configuration(
            "--format proto is not valid for the local doctor report".to_owned(),
        ));
    }
    let endpoint = assess_endpoint(&args.endpoint);
    let mut checks = vec![endpoint.check];
    checks.push(assess_timeout(args));
    checks.push(assess_tls(args, endpoint.endpoint.as_deref()));
    checks.push(assess_token(args));
    checks.push(assess_client_identity(args));
    checks.push(assess_capture_root(args));
    let status = if checks.iter().any(|check| check.status == "attention") {
        "attention"
    } else {
        "ready_for_network_check"
    };
    let report = DoctorReport {
        schema: "urn:oscalify:observer:doctor:v1",
        read_only: true,
        network_checked: false,
        status,
        checks,
        protocol: DoctorProtocol {
            descriptor_bytes: crate::PROTO_DESCRIPTOR_SET.len(),
            descriptor_sha256: digest(crate::PROTO_DESCRIPTOR_SET),
            proto_lock_sha256: digest(include_bytes!("../../proto.lock")),
            proto_file_count: crate::OSCAL_PROTO_FILE_COUNT
                .parse()
                .expect("build script emitted a numeric proto file count"),
            oscal_schema_version: crate::OSCAL_SCHEMA_VERSION,
            oscal_schema_manifest_sha256: crate::OSCAL_SCHEMA_MANIFEST_SHA256,
            oscal_schema_source_commit: crate::OSCAL_SCHEMA_SOURCE_COMMIT,
            oscal_schema_release_zip_sha256: crate::OSCAL_SCHEMA_RELEASE_ZIP_SHA256,
            capability_negotiation: "not_declared_by_protocol",
        },
    };
    if args.format == OutputFormat::Table {
        let rows = report
            .checks
            .iter()
            .map(|check| {
                vec![
                    check.id.to_owned(),
                    check.status.to_owned(),
                    check.detail.clone(),
                ]
            })
            .collect::<Vec<_>>();
        output::table(&["CHECK", "STATUS", "DETAIL"], &rows);
        println!("status  {}", report.status);
        println!("network_checked  false");
        println!(
            "oscal_schema_version  {}",
            report.protocol.oscal_schema_version
        );
        println!(
            "oscal_schema_manifest_sha256  {}",
            report.protocol.oscal_schema_manifest_sha256
        );
        println!(
            "capability_negotiation  {}",
            report.protocol.capability_negotiation
        );
        Ok(())
    } else {
        output::emit_json(args.format, &report)
    }
}

pub(super) fn assess_timeout(args: &crate::config::Cli) -> DoctorCheck {
    DoctorCheck {
        id: "timeout",
        status: if args.timeout_secs == 0 {
            "attention"
        } else {
            "pass"
        },
        detail: if args.timeout_secs == 0 {
            "timeout-secs must be greater than zero".to_owned()
        } else {
            format!("{} seconds; network not dialed", args.timeout_secs)
        },
    }
}

pub(super) fn assess_endpoint(raw_endpoint: &str) -> EndpointAssessment {
    let endpoint = match crate::config::normalized_endpoint(raw_endpoint) {
        Ok(endpoint) => endpoint,
        Err(error) => {
            return EndpointAssessment {
                endpoint: None,
                check: DoctorCheck {
                    id: "endpoint",
                    status: "attention",
                    detail: error.to_string(),
                },
            };
        }
    };
    if Endpoint::from_shared(endpoint.clone()).is_err() {
        return EndpointAssessment {
            endpoint: None,
            check: DoctorCheck {
                id: "endpoint",
                status: "attention",
                detail: "endpoint is not accepted by the gRPC transport parser".to_owned(),
            },
        };
    }
    EndpointAssessment {
        endpoint: Some(endpoint.clone()),
        check: DoctorCheck {
            id: "endpoint",
            status: "pass",
            detail: format!("{} (not dialed)", crate::error::redact_endpoint(&endpoint)),
        },
    }
}

pub(super) fn assess_tls(args: &crate::config::Cli, endpoint: Option<&str>) -> DoctorCheck {
    let has_tls_options = args.tls_domain.is_some()
        || args.ca_cert.is_some()
        || args.client_cert.is_some()
        || args.client_key.is_some();
    let Some(endpoint) = endpoint else {
        return DoctorCheck {
            id: "tls",
            status: "attention",
            detail: "cannot assess TLS until the endpoint is valid".to_owned(),
        };
    };
    let https = endpoint.starts_with("https://");
    if has_tls_options && !https {
        return DoctorCheck {
            id: "tls",
            status: "attention",
            detail: "TLS options require an https:// endpoint".to_owned(),
        };
    }
    if !https {
        return DoctorCheck {
            id: "tls",
            status: "not_configured",
            detail: "plaintext endpoint; no TLS options requested".to_owned(),
        };
    }
    let domain_ok = args
        .tls_domain
        .as_deref()
        .is_some_and(|domain| !domain.trim().is_empty())
        || crate::transport::infer_tls_domain(endpoint)
            .is_some_and(|domain| !domain.trim().is_empty());
    if !domain_ok {
        return DoctorCheck {
            id: "tls",
            status: "attention",
            detail: "HTTPS endpoint needs a resolvable host or --tls-domain".to_owned(),
        };
    }
    if let Some(path) = &args.ca_cert {
        if !readable_file(path) {
            return file_check("tls-ca", path, "CA certificate is not readable");
        }
    }
    DoctorCheck {
        id: "tls",
        status: "pass",
        detail: "HTTPS certificate verification is configured; server not dialed".to_owned(),
    }
}

pub(super) fn assess_token(args: &crate::config::Cli) -> DoctorCheck {
    if let Some(token) = args.token.as_deref() {
        return DoctorCheck {
            id: "token",
            status: if token.trim().is_empty() {
                "attention"
            } else {
                "pass"
            },
            detail: if token.trim().is_empty() {
                "token argument is empty".to_owned()
            } else {
                "bearer token configured from argument (value redacted)".to_owned()
            },
        };
    }
    if let Some(path) = &args.token_file {
        return match fs::read_to_string(path) {
            Ok(value) if !value.trim().is_empty() => DoctorCheck {
                id: "token",
                status: "pass",
                detail: format!("non-empty bearer token file: {}", path.display()),
            },
            Ok(_) => DoctorCheck {
                id: "token",
                status: "attention",
                detail: "bearer token file is empty".to_owned(),
            },
            Err(_) => file_check("token", path, "bearer token file is not readable"),
        };
    }
    DoctorCheck {
        id: "token",
        status: "not_configured",
        detail: "no bearer token configured; server authorization was not checked".to_owned(),
    }
}

pub(super) fn assess_client_identity(args: &crate::config::Cli) -> DoctorCheck {
    match (&args.client_cert, &args.client_key) {
        (None, None) => DoctorCheck {
            id: "client-identity",
            status: "not_configured",
            detail: "mTLS client certificate and key are not configured".to_owned(),
        },
        (Some(cert), Some(key)) if readable_file(cert) && readable_file(key) => DoctorCheck {
            id: "client-identity",
            status: "pass",
            detail: "mTLS client certificate and key are readable".to_owned(),
        },
        (Some(_), Some(_)) => DoctorCheck {
            id: "client-identity",
            status: "attention",
            detail: "mTLS client certificate or key is not readable".to_owned(),
        },
        _ => DoctorCheck {
            id: "client-identity",
            status: "attention",
            detail: "client-cert and client-key must be supplied together".to_owned(),
        },
    }
}

pub(super) fn assess_capture_root(args: &crate::config::Cli) -> DoctorCheck {
    let root = crate::config::capture_root_for_args(args);
    if root.is_dir() {
        return DoctorCheck {
            id: "capture-root",
            status: "pass",
            detail: format!("capture root exists: {} (not written)", root.display()),
        };
    }
    if root.exists() {
        return DoctorCheck {
            id: "capture-root",
            status: "attention",
            detail: format!("capture root is not a directory: {}", root.display()),
        };
    }
    let parent_invalid = root
        .parent()
        .is_some_and(|parent| parent.exists() && !parent.is_dir());
    DoctorCheck {
        id: "capture-root",
        status: if parent_invalid { "attention" } else { "pass" },
        detail: if parent_invalid {
            format!("capture root parent is unavailable: {}", root.display())
        } else {
            format!(
                "capture root will be created on first capture: {}",
                root.display()
            )
        },
    }
}

pub(super) fn file_check(id: &'static str, path: &Path, detail: &str) -> DoctorCheck {
    DoctorCheck {
        id,
        status: "attention",
        detail: format!("{detail}: {}", path.display()),
    }
}

pub(super) fn readable_file(path: &Path) -> bool {
    path.is_file() && fs::read(path).is_ok()
}
