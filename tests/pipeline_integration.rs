use std::{fs, process::Command};

fn mizan_cmd() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mizan"));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));
    command
}

#[test]
fn test_cli_pipeline_run_end_to_end() {
    let temp_dir = std::env::temp_dir().join(format!("mizan-cli-pipe-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let sbom_file = temp_dir.join("cyclonedx.json");
    fs::write(
        &sbom_file,
        serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "metadata": {
                "component": {
                    "name": "payment-auth-service",
                    "version": "2.1.0",
                    "type": "application"
                }
            },
            "components": [
                { "name": "axum", "version": "0.7.5", "type": "library" },
                { "name": "rustls", "version": "0.23.0", "type": "library" }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let workload_file = temp_dir.join("payment-pod.yaml");
    fs::write(
        &workload_file,
        r#"apiVersion: v1
kind: Pod
metadata:
  name: payment-auth
spec:
  containers:
  - name: payment-api
    securityContext:
      privileged: false
      runAsNonRoot: true
      readOnlyRootFilesystem: true
"#,
    )
    .unwrap();

    let out_dir = temp_dir.join("artifacts");

    let output = mizan_cmd()
        .args([
            "pipeline",
            "run",
            "-j",
            "us",
            "--sbom",
            sbom_file.to_str().unwrap(),
            "-w",
            workload_file.to_str().unwrap(),
            "--rule",
            "cis-k8s-5.2.1",
            "--rule",
            "cis-k8s-5.2.6",
            "--rule",
            "fedramp-ac-2",
            "--subject",
            "payment-auth-service:v2.1.0",
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute mizan binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "pipeline must succeed: stdout: {stdout}, stderr: {stderr}"
    );

    assert!(stdout.contains("Mizan End-to-End Compliance Pipeline Execution"));
    assert!(stdout.contains("SUCCESS / GREEN"));
    assert!(stdout.contains("SLSA v1.2 Statement:"));
    assert!(stdout.contains("SARIF v2.1.0 Report:"));
    assert!(stdout.contains("GitLab Sec Report:"));

    assert!(out_dir.join("oscal-catalog.json").exists());
    assert!(out_dir.join("oscal-component-definition.json").exists());
    assert!(out_dir.join("evidence-bundle.json").exists());
    assert!(out_dir.join("slsa-provenance.json").exists());
    assert!(out_dir.join("oscal-assessment-results.json").exists());
    assert!(out_dir.join("sarif-report.json").exists());
    assert!(out_dir.join("gl-security-report.json").exists());

    let _ = fs::remove_dir_all(temp_dir);
}
