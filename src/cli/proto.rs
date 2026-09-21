#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_proto(format: OutputFormat) -> Result<()> {
    if format == OutputFormat::Proto {
        std::io::stdout()
            .write_all(crate::PROTO_DESCRIPTOR_SET)
            .map_err(|error| AppError::Io {
                path: "stdout".into(),
                source: error,
            })?;
        return Ok(());
    }
    let source = option_env!("OSCAL_PROTO_SOURCE")
        .unwrap_or("../ckodex-oscalify/proto/oscal")
        .to_owned();
    let value = serde_json::json!({
        "source": source,
        "services": [
            "OscalService",
            "GovernanceService",
            "TransparencyExchangeService",
            "TransparencyGraphService"
        ],
        "descriptor_bytes": crate::PROTO_DESCRIPTOR_SET.len(),
        "descriptor_sha256": digest(crate::PROTO_DESCRIPTOR_SET),
        "proto_lock_sha256": digest(include_bytes!("../../proto.lock")),
        "proto_file_count": crate::OSCAL_PROTO_FILE_COUNT
            .parse::<usize>()
            .expect("build script emitted a numeric proto file count"),
        "oscal_schema_version": crate::OSCAL_SCHEMA_VERSION,
        "oscal_schema_manifest_sha256": crate::OSCAL_SCHEMA_MANIFEST_SHA256,
        "oscal_schema_source_commit": crate::OSCAL_SCHEMA_SOURCE_COMMIT,
        "oscal_schema_release_zip_sha256": crate::OSCAL_SCHEMA_RELEASE_ZIP_SHA256,
        "capability_negotiation": "not_declared_by_protocol",
        "read_only": true
    });
    output::emit_json(format, &value)
}
