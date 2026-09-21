use std::{fs, process::Command};

fn mizan_cmd() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mizan"));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));
    command
}

#[test]
fn test_cli_validate_catalog() {
    let output = mizan_cmd()
        .args(["validate", "examples/sample-catalog.json"])
        .output()
        .expect("failed to execute mizan binary");

    assert!(
        output.status.success(),
        "validation must succeed for sample-catalog.json: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VALID: catalog"));
}

#[test]
fn test_cli_inspect() {
    let output = mizan_cmd()
        .args(["inspect", "examples/sample-catalog.json"])
        .output()
        .expect("failed to execute mizan binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NIST Special Publication 800-53"));
    assert!(stdout.contains("Structure & Metrics:"));
}

#[test]
fn test_cli_convert_roundtrip_csv_proto_yaml() {
    let temporary_directory = create_test_directory("convert_test");
    let csv_file_path = temporary_directory.join("matrix.csv");
    let json_file_path = temporary_directory.join("roundtrip.json");
    let proto_file_path = temporary_directory.join("catalog.pb");
    let yaml_file_path = temporary_directory.join("catalog.yaml");

    // 1. JSON -> CSV
    let status = mizan_cmd()
        .args([
            "convert",
            "examples/sample-catalog.json",
            "--to",
            "csv",
            "-o",
            csv_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(csv_file_path.exists());

    // 2. CSV -> JSON
    let status = mizan_cmd()
        .args([
            "convert",
            csv_file_path.to_str().unwrap(),
            "--to",
            "json",
            "-o",
            json_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(json_file_path.exists());

    // 3. JSON -> Proto
    let status = mizan_cmd()
        .args([
            "convert",
            "examples/sample-catalog.json",
            "--to",
            "proto",
            "-o",
            proto_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(proto_file_path.exists());

    // 4. JSON -> YAML
    let status = mizan_cmd()
        .args([
            "convert",
            "examples/sample-catalog.json",
            "--to",
            "yaml",
            "-o",
            yaml_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(yaml_file_path.exists());

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_blast_radius() {
    let output = mizan_cmd()
        .args([
            "blast-radius",
            "examples/sample-catalog.json",
            "--target",
            "ac-1",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute blast-radius");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("risk_exposure_score"));
    assert!(stdout.contains("ac-1"));
}

#[test]
fn test_cli_3way_sync() {
    let temporary_directory = create_test_directory("sync_test");
    let merged_file_path = temporary_directory.join("merged.json");

    let output = mizan_cmd()
        .args([
            "sync",
            "--base",
            "examples/sample-catalog.json",
            "--upstream",
            "examples/sample-catalog.json",
            "--local",
            "examples/sample-catalog.json",
            "-o",
            merged_file_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute sync");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("is_clean"));
    assert!(merged_file_path.exists());

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_template_and_split_assemble() {
    let temporary_directory = create_test_directory("authoring_test");
    let ssp_file_path = temporary_directory.join("scaffolded-ssp.json");
    let workspace_directory = temporary_directory.join("workspace");
    let assembled_file_path = temporary_directory.join("assembled-ssp.json");

    // 1. Scaffold template
    let status = mizan_cmd()
        .args([
            "template",
            "ssp",
            "--standard",
            "fedramp-moderate",
            "-o",
            ssp_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(ssp_file_path.exists());

    // 2. Split into Markdown GitOps workspace
    let status = mizan_cmd()
        .args([
            "split",
            ssp_file_path.to_str().unwrap(),
            "-o",
            workspace_directory.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(workspace_directory.join("_metadata.yaml").exists());

    // 3. Assemble back to OSCAL JSON
    let status = mizan_cmd()
        .args([
            "assemble",
            workspace_directory.to_str().unwrap(),
            "-o",
            assembled_file_path.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(assembled_file_path.exists());

    // 4. Validate assembled SSP with FedRAMP PMO rules
    let output = mizan_cmd()
        .args([
            "fedramp",
            "validate",
            assembled_file_path.to_str().unwrap(),
            "--baseline",
            "moderate",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute fedramp validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("total_rules_checked"));

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_k8s_audit_fails_closed_without_cluster() {
    let temporary_directory = create_test_directory("k8s_audit_test");
    let assessment_output = temporary_directory.join("k8s-assessment.json");

    let output = mizan_cmd()
        // Make the negative-path assertion independent of the developer's
        // ambient kubeconfig or an attached local cluster.
        .env(
            "KUBECONFIG",
            temporary_directory
                .join("missing-kubeconfig")
                .to_str()
                .unwrap(),
        )
        .args([
            "audit",
            "-n",
            "production-workloads",
            "-o",
            assessment_output.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute mizan audit");

    // De-fabricated behavior: with no reachable cluster the audit must fail
    // closed with a typed negative result instead of emitting synthetic pods.
    assert!(
        !output.status.success(),
        "k8s audit must not succeed without a reachable cluster"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("negative") || stderr.contains("error"),
        "expected a typed negative error, got: {stderr}"
    );
    assert!(
        !assessment_output.exists(),
        "no assessment artifact may be produced from fabricated data"
    );

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_policy_eval_regorus() {
    let temporary_directory = create_test_directory("policy_eval_test");
    let policy_path = temporary_directory.join("container_security.rego");
    let input_path = temporary_directory.join("pod.json");

    let rego_code = r#"
        package oscal.container_security

        default allow = false
        allow if { input.readOnly == true }

        deny contains msg if {
            input.readOnly != true
            msg := "Root filesystem must be read-only (CM-7)"
        }
    "#;
    fs::write(&policy_path, rego_code).unwrap();

    let input_json = r#"{"readOnly": true}"#;
    fs::write(&input_path, input_json).unwrap();

    let output = mizan_cmd()
        .args([
            "policy",
            "eval",
            "-p",
            policy_path.to_str().unwrap(),
            "-i",
            input_path.to_str().unwrap(),
            "--query",
            "oscal.container_security",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute policy eval");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"passed\": true"));

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_fsm_transition() {
    let output = mizan_cmd()
        .args([
            "fsm",
            "transition",
            "--from",
            "draft",
            "--evidence",
            "e1",
            "--event",
            "submit_for_review",
            "--actor",
            "author-alice",
            "--rationale",
            "Initial catalog schema verified",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute fsm transition");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("under_review"));
    assert!(stdout.contains("author-alice"));
}

#[test]
fn test_cli_tx_begin() {
    let output = mizan_cmd()
        .args([
            "tx",
            "begin",
            "--tx-id",
            "test-tx-integration-001",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute tx begin");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-tx-integration-001"));
    assert!(stdout.contains("active"));
}

#[test]
fn test_cli_cas_put_get() {
    let temporary_directory = create_test_directory("cas_test");
    let test_file = temporary_directory.join("test_blob.txt");
    fs::write(
        &test_file,
        "Mizan CAS content addressable storage with RustFS",
    )
    .unwrap();

    let output = mizan_cmd()
        .args([
            "cas",
            "put",
            test_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute cas put");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sha256:"));

    let json_val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let digest = json_val["digest"].as_str().unwrap();

    let out_file = temporary_directory.join("retrieved_blob.txt");
    let get_output = mizan_cmd()
        .args(["cas", "get", digest, "-o", out_file.to_str().unwrap()])
        .output()
        .expect("failed to execute cas get");

    assert!(get_output.status.success());
    let retrieved_content = fs::read_to_string(out_file).unwrap();
    assert_eq!(
        retrieved_content,
        "Mizan CAS content addressable storage with RustFS"
    );

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_evidence_bundle_and_verify() {
    let temporary_directory = create_test_directory("evidence_test");
    let catalog_file = temporary_directory.join("catalog.json");
    let bundle_file = temporary_directory.join("evidence_bundle.json");

    let sample_catalog = r#"{
        "catalog": {
            "uuid": "8b788647-767a-4ecb-ba3a-f2b7f719602a",
            "metadata": {
                "title": "FedRAMP Catalog",
                "published": "2026-08-28T00:00:00Z",
                "last-modified": "2026-08-28T00:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "controls": [
                { "id": "ac-1", "title": "Access Control Policy" }
            ]
        }
    }"#;
    fs::write(&catalog_file, sample_catalog).unwrap();

    let output = mizan_cmd()
        .args([
            "evidence",
            "bundle",
            catalog_file.to_str().unwrap(),
            "--level",
            "e3",
            "--sign-as",
            "secops-lead",
            "-o",
            bundle_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute evidence bundle");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle-"));
    assert!(stdout.contains("secops-lead"));

    let verify_output = mizan_cmd()
        .args([
            "evidence",
            "verify-bundle",
            bundle_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute evidence verify-bundle");

    assert!(verify_output.status.success());
    let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_stdout.contains("\"is_valid\": true"));
    assert!(verify_stdout.contains("\"signature_verified\": true"));

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_catalog_tri_jurisdiction_and_extend() {
    let temporary_directory = create_test_directory("catalog_test");

    // 1. List
    let list_output = mizan_cmd()
        .args(["catalog", "list", "--format", "json"])
        .output()
        .expect("failed to execute catalog list");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("United States"));
    assert!(list_stdout.contains("Canada"));
    assert!(list_stdout.contains("European Union"));

    // 2. Export Canada ITSG-33
    let ca_file = temporary_directory.join("itsg33.json");
    let exp_output = mizan_cmd()
        .args([
            "catalog",
            "export",
            "-j",
            "ca",
            "-o",
            ca_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute catalog export");
    assert!(exp_output.status.success());
    assert!(ca_file.exists());

    // 3. Extend with Enterprise Control
    let ent_file = temporary_directory.join("enterprise_catalog.json");
    let ext_output = mizan_cmd()
        .args([
            "catalog",
            "extend",
            "-b",
            "us",
            "--title",
            "Sovereign Banking Cloud Baseline",
            "--add-control-id",
            "bank-sec-01",
            "--add-control-title",
            "HSM Hardware Isolation",
            "--add-control-desc",
            "FIPS 140-3 Level 4 key management",
            "-o",
            ent_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute catalog extend");
    assert!(ext_output.status.success());
    assert!(ent_file.exists());

    let ent_content = fs::read_to_string(ent_file).unwrap();
    assert!(ent_content.contains("bank-sec-01"));
    assert!(ent_content.contains("HSM Hardware Isolation"));

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_slsa_v1_2_attestation_and_verify() {
    let temporary_directory = create_test_directory("slsa_test");
    let stmt_file = temporary_directory.join("provenance.json");

    let output = mizan_cmd()
        .args([
            "attest",
            "slsa",
            "--subject",
            "ghcr.io/mizan/security-kernel:1.0.0",
            "--digest",
            "sha256:4a8f9c0e2b1d3f5a7e9c1b3d5f7a9e1c3b5d7f9a1b3c5d7e9f1a3b5c7d9e1f3a",
            "--version",
            "v1.2",
            "-o",
            stmt_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute attest slsa");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://slsa.dev/provenance/v1.2"));

    let verify_output = mizan_cmd()
        .args([
            "attest",
            "verify",
            stmt_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute attest verify");

    assert!(verify_output.status.success());
    let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_stdout.contains("\"is_valid\": true"));
    assert!(verify_stdout.contains("\"slsa_version\": \"v1-2\""));

    let _ = fs::remove_dir_all(temporary_directory);
}

#[test]
fn test_cli_gui_status() {
    let output = mizan_cmd()
        .args(["gui", "--format", "json"])
        .output()
        .expect("failed to execute gui command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mizan"));
    assert!(stdout.contains("http://localhost:3000"));
}

fn create_test_directory(test_prefix: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mizan-test-{test_prefix}-{}", std::process::id()));
    let _ = fs::create_dir_all(&directory);
    directory
}
