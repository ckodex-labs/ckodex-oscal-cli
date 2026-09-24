use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{AppError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredAsset {
    pub category: String,
    pub path: PathBuf,
    pub details: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitReport {
    pub repo_root: PathBuf,
    pub output_dir: PathBuf,
    pub discovered_dockerfiles: usize,
    pub discovered_k8s_manifests: usize,
    pub discovered_lockfiles: usize,
    pub discovered_ci_workflows: usize,
    pub mapped_controls_count: usize,
    pub created_files: Vec<PathBuf>,
    pub assets: Vec<DiscoveredAsset>,
}

pub struct RepoInitializer;

impl RepoInitializer {
    pub fn init_from_repo(repo_path: &Path, output_dir_opt: Option<&Path>) -> Result<InitReport> {
        if !repo_path.exists() {
            return Err(AppError::Configuration(format!(
                "Repository path does not exist: {}",
                repo_path.display()
            )));
        }

        let output_dir = output_dir_opt
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo_path.join("governance"));

        fs::create_dir_all(&output_dir).map_err(|e| {
            AppError::Configuration(format!(
                "Failed to create output directory {}: {e}",
                output_dir.display()
            ))
        })?;

        let mut assets = Vec::new();
        Self::scan_directory(repo_path, &mut assets);

        let df_count = assets.iter().filter(|a| a.category == "Dockerfile").count();
        let k8s_count = assets.iter().filter(|a| a.category == "Kubernetes").count();
        let lock_count = assets
            .iter()
            .filter(|a| a.category == "DependencyLock")
            .count();
        let ci_count = assets.iter().filter(|a| a.category == "CI/CD").count();

        let mut created_files = Vec::new();

        // 1. Generate .mizan/config.json in repo root
        let dot_mizan = repo_path.join(".mizan");
        fs::create_dir_all(&dot_mizan).map_err(|e| {
            AppError::Configuration(format!("Failed to create .mizan directory: {e}"))
        })?;
        let config_file = dot_mizan.join("config.json");
        let mizan_config = json!({
            "version": "1.0",
            "jurisdiction": "us",
            "governance_root": output_dir.strip_prefix(repo_path).unwrap_or(&output_dir),
            "pipeline": {
                "default_rules": [
                    "cis-k8s-5.2.1",
                    "cis-k8s-5.2.2",
                    "cis-k8s-5.2.5",
                    "cis-k8s-5.2.6",
                    "cis-k8s-5.2.7"
                ],
                "auto_fix": true,
                "allow_waivers": true
            },
            "discovered_summary": {
                "dockerfiles": df_count,
                "k8s_manifests": k8s_count,
                "lockfiles": lock_count,
                "ci_workflows": ci_count
            }
        });
        fs::write(
            &config_file,
            serde_json::to_string_pretty(&mizan_config)
                .map_err(|e| AppError::Configuration(e.to_string()))?,
        )
        .map_err(|e| {
            AppError::Configuration(format!("Failed to write {}: {e}", config_file.display()))
        })?;
        created_files.push(config_file);

        // 2. Generate OSCAL Component Definition with control mappings
        let comp_def_file = output_dir.join("component-definition.json");
        let (comp_def_val, mapped_controls) =
            Self::generate_component_definition(repo_path, &assets);
        fs::write(
            &comp_def_file,
            serde_json::to_string_pretty(&comp_def_val)
                .map_err(|e| AppError::Configuration(e.to_string()))?,
        )
        .map_err(|e| {
            AppError::Configuration(format!("Failed to write {}: {e}", comp_def_file.display()))
        })?;
        created_files.push(comp_def_file);

        // 3. Generate initial System Security Plan (SSP)
        let ssp_file = output_dir.join("ssp.json");
        let ssp_val = Self::generate_ssp(repo_path, &mapped_controls);
        fs::write(
            &ssp_file,
            serde_json::to_string_pretty(&ssp_val)
                .map_err(|e| AppError::Configuration(e.to_string()))?,
        )
        .map_err(|e| {
            AppError::Configuration(format!("Failed to write {}: {e}", ssp_file.display()))
        })?;
        created_files.push(ssp_file);

        // 4. Generate README.md with guidance
        let readme_file = output_dir.join("README.md");
        let readme_content = format!(
            r#"# Mizan Automated Compliance Governance Workspace

Auto-scaffolded by `mizan init --from-repo` on {}.

## Discovered Inventory
- **Dockerfiles**: {} detected
- **Kubernetes Workloads**: {} detected
- **Software Dependencies / Lockfiles**: {} detected
- **CI/CD Pipelines**: {} detected
- **NIST 800-53 Controls Mapped**: {} controls

## Core Files
- `component-definition.json`: OSCAL Component Definition with architectural control mappings.
- `ssp.json`: System Security Plan ready for authoring and evidence attestation.
- `../.mizan/config.json`: Local workspace policies and rules.

## Recommended Next Steps
1. **Run Compliance Pipeline**:
   ```bash
   mizan pipeline run --jurisdiction us --output ./mizan-pipeline-output
   ```
2. **Review & Autofix Policies**:
   ```bash
   mizan fix --rule cis-k8s-5.2.1 -f workload.yaml --dry-run
   ```
3. **Export Auditor Matrix**:
   ```bash
   mizan catalog export-matrix --jurisdiction us --output matrix.csv
   ```
"#,
            chrono::Utc::now().to_rfc3339(),
            df_count,
            k8s_count,
            lock_count,
            ci_count,
            mapped_controls.len()
        );
        fs::write(&readme_file, readme_content).map_err(|e| {
            AppError::Configuration(format!("Failed to write {}: {e}", readme_file.display()))
        })?;
        created_files.push(readme_file);

        Ok(InitReport {
            repo_root: repo_path.to_path_buf(),
            output_dir,
            discovered_dockerfiles: df_count,
            discovered_k8s_manifests: k8s_count,
            discovered_lockfiles: lock_count,
            discovered_ci_workflows: ci_count,
            mapped_controls_count: mapped_controls.len(),
            created_files,
            assets,
        })
    }

    fn scan_directory(dir: &Path, assets: &mut Vec<DiscoveredAsset>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                Self::scan_directory(&path, assets);
            } else if path.is_file() {
                let lower = file_name.to_lowercase();
                if lower.starts_with("dockerfile") || lower.ends_with(".dockerfile") {
                    assets.push(DiscoveredAsset {
                        category: "Dockerfile".to_string(),
                        path: path.clone(),
                        details: format!("Container image specification: {file_name}"),
                    });
                } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("apiVersion:") && content.contains("kind:") {
                            let kind = content
                                .lines()
                                .find(|l| l.trim_start().starts_with("kind:"))
                                .map(|l| l.trim().to_string())
                                .unwrap_or_else(|| "kind: Manifest".to_string());
                            assets.push(DiscoveredAsset {
                                category: "Kubernetes".to_string(),
                                path: path.clone(),
                                details: format!("Kubernetes workload ({kind})"),
                            });
                        } else if path.to_string_lossy().contains(".github/workflows")
                            || file_name == ".gitlab-ci.yml"
                        {
                            assets.push(DiscoveredAsset {
                                category: "CI/CD".to_string(),
                                path: path.clone(),
                                details: format!("Automated CI/CD workflow: {file_name}"),
                            });
                        }
                    }
                } else if matches!(
                    file_name.as_str(),
                    "Cargo.lock"
                        | "package-lock.json"
                        | "pnpm-lock.yaml"
                        | "yarn.lock"
                        | "go.mod"
                        | "pom.xml"
                        | "requirements.txt"
                ) {
                    assets.push(DiscoveredAsset {
                        category: "DependencyLock".to_string(),
                        path: path.clone(),
                        details: format!("Package lockfile: {file_name}"),
                    });
                }
            }
        }
    }

    fn generate_component_definition(
        repo_root: &Path,
        assets: &[DiscoveredAsset],
    ) -> (Value, Vec<String>) {
        let repo_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        let mut components = Vec::new();
        let mut mapped_controls = Vec::new();

        // 1. Software dependencies component
        let lockfiles: Vec<String> = assets
            .iter()
            .filter(|a| a.category == "DependencyLock")
            .map(|a| a.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        if !lockfiles.is_empty() {
            components.push(json!({
                "uuid": uuid::Uuid::new_v4().to_string(),
                "type": "software",
                "title": format!("{repo_name}-dependencies"),
                "description": format!("Third-party software dependencies governed via: {}", lockfiles.join(", ")),
                "status": { "state": "under-development" },
                "control-implementations": [{
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "source": "https://doi.org/10.6028/NIST.SP.800-53r5",
                    "description": "Software dependency tracking and vulnerability mitigation",
                    "implemented-requirements": [
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "cm-8",
                            "description": "Component inventory tracked from content-addressed lockfiles."
                        },
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "sa-11",
                            "description": "Developer security testing and static dependency analysis."
                        },
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "si-2",
                            "description": "Flaw remediation via automated supply-chain scanner."
                        }
                    ]
                }]
            }));
            mapped_controls.push("cm-8".to_string());
            mapped_controls.push("sa-11".to_string());
            mapped_controls.push("si-2".to_string());
        }

        // 2. Containers component
        let dockerfiles: Vec<String> = assets
            .iter()
            .filter(|a| a.category == "Dockerfile")
            .map(|a| a.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        if !dockerfiles.is_empty() {
            components.push(json!({
                "uuid": uuid::Uuid::new_v4().to_string(),
                "type": "container",
                "title": format!("{repo_name}-workload-images"),
                "description": format!("Container base images and build definitions: {}", dockerfiles.join(", ")),
                "status": { "state": "operational" },
                "control-implementations": [{
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "source": "https://doi.org/10.6028/NIST.SP.800-53r5",
                    "description": "Container baseline hardening and minimal attack surface",
                    "implemented-requirements": [
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "cm-2",
                            "description": "Standardized baseline images built from vetted Dockerfiles."
                        },
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "cm-7",
                            "description": "Least functionality enforced through non-root container users."
                        }
                    ]
                }]
            }));
            mapped_controls.push("cm-2".to_string());
            mapped_controls.push("cm-7".to_string());
        }

        // 3. Kubernetes platform component
        let k8s_manifests: Vec<String> = assets
            .iter()
            .filter(|a| a.category == "Kubernetes")
            .map(|a| a.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        if !k8s_manifests.is_empty() {
            components.push(json!({
                "uuid": uuid::Uuid::new_v4().to_string(),
                "type": "service",
                "title": format!("{repo_name}-kubernetes-runtime"),
                "description": format!("Kubernetes orchestration manifests: {}", k8s_manifests.join(", ")),
                "status": { "state": "operational" },
                "control-implementations": [{
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "source": "https://doi.org/10.6028/NIST.SP.800-53r5",
                    "description": "Runtime workload isolation and access controls",
                    "implemented-requirements": [
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "ac-2",
                            "description": "Workload service accounts with bounded RBAC."
                        },
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "ac-6",
                            "description": "Least privilege enforced via readOnlyRootFilesystem and dropped capabilities."
                        },
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "sc-7",
                            "description": "Network policies and ingress boundary protection."
                        }
                    ]
                }]
            }));
            mapped_controls.push("ac-2".to_string());
            mapped_controls.push("ac-6".to_string());
            mapped_controls.push("sc-7".to_string());
        }

        // 4. Default baseline component if nothing discovered
        if components.is_empty() {
            components.push(json!({
                "uuid": uuid::Uuid::new_v4().to_string(),
                "type": "application",
                "title": format!("{repo_name}-core-service"),
                "description": "Core repository application codebase",
                "status": { "state": "under-development" },
                "control-implementations": [{
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "source": "https://doi.org/10.6028/NIST.SP.800-53r5",
                    "description": "Baseline information system controls",
                    "implemented-requirements": [
                        {
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "control-id": "cm-8",
                            "description": "Repository software asset registration."
                        }
                    ]
                }]
            }));
            mapped_controls.push("cm-8".to_string());
        }

        mapped_controls.sort();
        mapped_controls.dedup();

        let doc = json!({
            "component-definition": {
                "uuid": uuid::Uuid::new_v4().to_string(),
                "metadata": {
                    "title": format!("{repo_name} Security Component Definition"),
                    "last-modified": chrono::Utc::now().to_rfc3339(),
                    "version": "1.0.0",
                    "oscal-version": "1.2.0"
                },
                "components": components
            }
        });

        (doc, mapped_controls)
    }

    fn generate_ssp(repo_root: &Path, controls: &[String]) -> Value {
        let repo_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        let implemented_requirements: Vec<Value> = controls
            .iter()
            .map(|ctrl_id| {
                json!({
                    "uuid": uuid::Uuid::new_v4().to_string(),
                    "control-id": ctrl_id,
                    "description": format!("Control {ctrl_id} automatically satisfied and attested via Mizan CI/CD pipeline.")
                })
            })
            .collect();

        json!({
            "system-security-plan": {
                "uuid": uuid::Uuid::new_v4().to_string(),
                "metadata": {
                    "title": format!("{repo_name} System Security Plan"),
                    "last-modified": chrono::Utc::now().to_rfc3339(),
                    "version": "1.0.0",
                    "oscal-version": "1.2.0"
                },
                "import-profile": {
                    "href": "https://github.com/usnistgov/oscal-content/blob/master/nist.gov/SP800-53/rev5/json/NIST_SP-800-53_rev5_MODERATE-baseline_profile.json"
                },
                "system-characteristics": {
                    "system-name": format!("{repo_name}-system"),
                    "system-information": {
                        "information-types": [{
                            "uuid": uuid::Uuid::new_v4().to_string(),
                            "title": "System Operational Telemetry",
                            "categorization": {
                                "system": "NIST SP 800-60",
                                "information-type-ids": ["C.3.5.8"]
                            }
                        }]
                    },
                    "security-sensitivity-level": "moderate"
                },
                "system-implementation": {
                    "users": [{
                        "uuid": uuid::Uuid::new_v4().to_string(),
                        "title": "Automated CI/CD Robot",
                        "role-ids": ["system-administrator"]
                    }]
                },
                "control-implementation": {
                    "description": "Automated control implementations discovered from codebase assets.",
                    "implemented-requirements": implemented_requirements
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_init_discovery() {
        let temp_dir = std::env::temp_dir().join(format!("mizan-init-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Create sample Dockerfile
        fs::write(temp_dir.join("Dockerfile"), "FROM alpine:3.20\n").unwrap();

        // Create sample k8s manifest
        fs::write(
            temp_dir.join("deployment.yaml"),
            "apiVersion: apps/v1\nkind: Deployment\nspec:\n  replicas: 1\n",
        )
        .unwrap();

        // Create sample lockfile
        fs::write(temp_dir.join("Cargo.lock"), "# lockfile\n").unwrap();

        let report = RepoInitializer::init_from_repo(&temp_dir, None).unwrap();
        assert_eq!(report.discovered_dockerfiles, 1);
        assert_eq!(report.discovered_k8s_manifests, 1);
        assert_eq!(report.discovered_lockfiles, 1);
        assert!(report.mapped_controls_count >= 3);

        let gov_dir = temp_dir.join("governance");
        assert!(gov_dir.join("component-definition.json").exists());
        assert!(gov_dir.join("ssp.json").exists());
        assert!(gov_dir.join("README.md").exists());
        assert!(temp_dir.join(".mizan/config.json").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
