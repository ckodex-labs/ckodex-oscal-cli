#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_pipeline(args: &PipelineCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        PipelineAction::Run {
            jurisdiction,
            sbom,
            workload,
            rules,
            subject,
            digest,
            output_dir,
        } => {
            let jur = Jurisdiction::from_str_name(jurisdiction).ok_or_else(|| {
                AppError::Configuration(format!(
                    "Unknown jurisdiction '{jurisdiction}'. Options: us, ca, eu, enterprise"
                ))
            })?;
            let cfg = PipelineConfig {
                jurisdiction: jur,
                sbom_path: sbom.clone(),
                workload_path: workload.clone(),
                rule_ids: rules.clone(),
                output_dir: output_dir.clone(),
                subject_name: subject.clone(),
                subject_digest: digest.clone(),
            };

            let report = PipelineOrchestrator::run(cfg)?;

            match format {
                OutputFormat::Json => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan End-to-End Compliance Pipeline Execution");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Jurisdiction:       {}", report.jurisdiction);
                    println!("  OSCAL Catalog UUID: {}", report.oscal_catalog_uuid);
                    println!("  SBOM Components:    {}", report.sbom_components_count);
                    println!("  Rules Evaluated:    {}", report.evaluated_rules_count);
                    println!("  Passed Rules:       {}", report.passed_rules_count);
                    println!("  Violations:         {}", report.violations_count);
                    println!("  Evidence Merkle:    {}", report.merkle_root);
                    println!("  CAS Objects Cached: {}", report.cas_objects_written);
                    println!(
                        "  SLSA v1.2 Statement:{}",
                        report.slsa_provenance_path.display()
                    );
                    println!(
                        "  SARIF v2.1.0 Report:{}",
                        report.sarif_report_path.display()
                    );
                    println!(
                        "  GitLab Sec Report:  {}",
                        report.gitlab_report_path.display()
                    );
                    println!(
                        "  OSCAL Assessment:   {}",
                        report.oscal_assessment_path.display()
                    );
                    println!(
                        "  Pipeline Result:    {}",
                        if report.all_passed {
                            "SUCCESS / GREEN"
                        } else {
                            "DEGRADED / VIOLATIONS DETECTED"
                        }
                    );
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
            Ok(())
        }
    }
}
