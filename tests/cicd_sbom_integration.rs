use std::{fs, path::PathBuf, process::Command};

fn mizan_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mizan"));
    cmd.env_remove("MIZAN_TOKEN");
    cmd
}

fn create_test_directory(test_prefix: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mizan-test-{test_prefix}-{}", std::process::id()));
    let _ = fs::create_dir_all(&directory);
    directory
}

#[test]
fn test_cli_export_sarif_and_gitlab() {
    let temp_dir = create_test_directory("export_test");
    let cat_file = temp_dir.join("catalog.json");

    // 1. Export CA catalog first
    let exp_cat = mizan_cmd()
        .args([
            "catalog",
            "export",
            "-j",
            "ca",
            "-o",
            cat_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to export catalog");
    assert!(exp_cat.status.success());

    // 2. Export SARIF
    let sarif_file = temp_dir.join("compliance.sarif");
    let sarif_out = mizan_cmd()
        .args([
            "export",
            "sarif",
            "-i",
            cat_file.to_str().unwrap(),
            "-o",
            sarif_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute export sarif");

    assert!(sarif_out.status.success());
    assert!(sarif_file.exists());
    let sarif_content = fs::read_to_string(&sarif_file).unwrap();
    assert!(sarif_content.contains("\"version\": \"2.1.0\""));
    assert!(sarif_content.contains("mizan-compliance-engine"));

    // 3. Export GitLab Security Report
    let gl_file = temp_dir.join("gl-security.json");
    let gl_out = mizan_cmd()
        .args([
            "export",
            "gitlab",
            "-i",
            cat_file.to_str().unwrap(),
            "-o",
            gl_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute export gitlab");

    assert!(gl_out.status.success());
    assert!(gl_file.exists());
    let gl_content = fs::read_to_string(&gl_file).unwrap();
    assert!(gl_content.contains("\"version\": \"15.0.0\""));
    assert!(gl_content.contains("mizan-compliance-scanner"));

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cli_sbom_cyclonedx_import() {
    let temp_dir = create_test_directory("sbom_test");
    let sbom_file = temp_dir.join("bom.json");
    let oscal_comp_file = temp_dir.join("oscal-components.json");

    let cyclonedx_json = r#"{
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "version": 1,
        "components": [
            {
                "type": "library",
                "name": "axum",
                "version": "0.8.1",
                "purl": "pkg:cargo/axum@0.8.1",
                "description": "Ergonomic and modular web framework built with Tokio, Tower, and Hyper"
            }
        ]
    }"#;
    fs::write(&sbom_file, cyclonedx_json).unwrap();

    let output = mizan_cmd()
        .args([
            "sbom",
            "import",
            "-i",
            sbom_file.to_str().unwrap(),
            "-o",
            oscal_comp_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute sbom import");

    assert!(output.status.success());
    assert!(oscal_comp_file.exists());

    let oscal_content = fs::read_to_string(oscal_comp_file).unwrap();
    assert!(oscal_content.contains("component-definition"));
    assert!(oscal_content.contains("axum@0.8.1"));
    assert!(oscal_content.contains("sa-11"));

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cli_policy_rulepack_list_and_eval() {
    let temp_dir = create_test_directory("rulepack_test");

    // 1. List
    let list_out = mizan_cmd()
        .args(["policy", "rulepack", "list", "--format", "json"])
        .output()
        .expect("failed to execute policy rulepack list");

    assert!(list_out.status.success());
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(list_stdout.contains("cis-k8s-5.2.1"));
    assert!(list_stdout.contains("fedramp-ac-2"));

    // 2. Eval bad pod
    let bad_pod_file = temp_dir.join("bad_pod.json");
    fs::write(
        &bad_pod_file,
        r#"{
            "spec": {
                "containers": [
                    {
                        "name": "database-root",
                        "securityContext": {
                            "privileged": true
                        }
                    }
                ]
            }
        }"#,
    )
    .unwrap();

    let eval_out = mizan_cmd()
        .args([
            "policy",
            "rulepack",
            "eval",
            "-r",
            "cis-k8s-5.2.1",
            "-i",
            bad_pod_file.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to execute policy rulepack eval");

    assert!(eval_out.status.success());
    let eval_stdout = String::from_utf8_lossy(&eval_out.stdout);
    assert!(eval_stdout.contains("\"passed\": false"));
    assert!(eval_stdout.contains("privileged: true"));

    let _ = fs::remove_dir_all(temp_dir);
}
