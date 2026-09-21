use std::{
    env,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

struct SnapshotMetadata {
    proto_file_count: usize,
    oscal_schema_version: String,
    oscal_schema_manifest_sha256: String,
    oscal_schema_source_commit: String,
    oscal_schema_release_zip_sha256: String,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local_proto_root = manifest_dir.join("proto");
    let vendored_proto_root = local_proto_root.join("oscal");
    let proto_root = env::var_os("OSCALIFY_PROTO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| vendored_proto_root.clone());
    let descriptor_path =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("oscal_descriptor.bin");

    let proto_files = [
        "common/v1/common.proto",
        "catalog/v1/catalog.proto",
        "profile/v1/profile.proto",
        "component_definition/v1/component.proto",
        "ssp/v1/ssp.proto",
        "assessment_plan/v1/assessment_plan.proto",
        "assessment_results/v1/assessment_results.proto",
        "poam/v1/poam.proto",
        "mapping/v1/mapping.proto",
        "services/v1/oscal_service.proto",
        "services/v1/governance_service.proto",
        "services/v1/transparency_exchange_service.proto",
        "services/v1/transparency_graph_service.proto",
    ];

    let paths: Vec<PathBuf> = proto_files
        .iter()
        .map(|file| proto_root.join(file))
        .collect();

    let snapshot = verify_snapshot(&manifest_dir.join("proto.lock"), &proto_root, &proto_files);

    println!(
        "cargo:rustc-env=OSCAL_PROTO_SOURCE={}",
        proto_root.display()
    );
    println!(
        "cargo:rustc-env=OSCAL_PROTO_FILE_COUNT={}",
        snapshot.proto_file_count
    );
    println!(
        "cargo:rustc-env=OSCAL_SCHEMA_VERSION={}",
        snapshot.oscal_schema_version
    );
    println!(
        "cargo:rustc-env=OSCAL_SCHEMA_MANIFEST_SHA256={}",
        snapshot.oscal_schema_manifest_sha256
    );
    println!(
        "cargo:rustc-env=OSCAL_SCHEMA_SOURCE_COMMIT={}",
        snapshot.oscal_schema_source_commit
    );
    println!(
        "cargo:rustc-env=OSCAL_SCHEMA_RELEASE_ZIP_SHA256={}",
        snapshot.oscal_schema_release_zip_sha256
    );
    println!("cargo:rerun-if-env-changed=OSCALIFY_PROTO_ROOT");
    println!("cargo:rerun-if-changed={}", local_proto_root.display());
    for path in &paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    tonic_prost_build::configure()
        .build_server(false)
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(&paths, &[proto_root, local_proto_root])
        .expect("failed to compile OSCALify protobuf definitions");
}

fn verify_snapshot(lock_path: &Path, proto_root: &Path, proto_files: &[&str]) -> SnapshotMetadata {
    let lock: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(lock_path).expect("failed to read proto.lock"),
    )
    .expect("proto.lock must be valid JSON");
    let files = lock
        .get("files")
        .and_then(serde_json::Value::as_object)
        .expect("proto.lock must contain a files object");

    for relative in proto_files {
        let expected = files
            .get(*relative)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("proto.lock is missing {relative}"));
        let path = proto_root.join(relative);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| {
            panic!("failed to read proto snapshot {}: {error}", path.display())
        });
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let actual = hex::encode(hasher.finalize());
        assert_eq!(
            actual,
            expected,
            "proto snapshot drift at {}",
            path.display()
        );
    }
    for relative in files.keys() {
        if !proto_files.contains(&relative.as_str()) {
            panic!("proto.lock contains an uncompiled file {relative}");
        }
    }
    let oscal_schema = lock
        .get("oscal_schema")
        .and_then(serde_json::Value::as_object)
        .expect("proto.lock must contain oscal_schema metadata");
    let required = |key: &str| {
        oscal_schema
            .get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| panic!("proto.lock oscal_schema is missing {key}"))
            .to_owned()
    };
    let manifest_sha256 = required("manifest_sha256");
    let release_zip_sha256 = required("release_zip_sha256");
    assert_sha256("oscal_schema.manifest_sha256", &manifest_sha256);
    assert_sha256("oscal_schema.release_zip_sha256", &release_zip_sha256);
    let source_commit = required("source_commit");
    assert!(
        source_commit.len() == 40 && source_commit.chars().all(|value| value.is_ascii_hexdigit()),
        "proto.lock oscal_schema.source_commit must be a 40-character hexadecimal Git commit"
    );
    let version = required("version");
    assert!(
        version.split('.').count() == 3
            && version
                .split('.')
                .all(|part| !part.is_empty()
                    && part.chars().all(|character| character.is_ascii_digit())),
        "proto.lock oscal_schema.version must be a three-part semantic version"
    );
    SnapshotMetadata {
        proto_file_count: files.len(),
        oscal_schema_version: version,
        oscal_schema_manifest_sha256: manifest_sha256,
        oscal_schema_source_commit: source_commit,
        oscal_schema_release_zip_sha256: release_zip_sha256,
    }
}

fn assert_sha256(label: &str, value: &str) {
    let digest = value
        .strip_prefix("sha256:")
        .unwrap_or_else(|| panic!("proto.lock {label} must use a sha256: prefix"));
    assert!(
        digest.len() == 64
            && digest
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
        "proto.lock {label} must contain a 64-character hexadecimal digest"
    );
}
