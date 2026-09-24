use super::*;
use crate::document::init::RepoInitializer;

pub(super) fn run_init(args: &InitCliArgs, format: OutputFormat) -> Result<()> {
    let report = RepoInitializer::init_from_repo(&args.from_repo, args.output.as_deref())?;

    match format {
        OutputFormat::Json => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("Mizan Cold-Start Codebase Onboarding & Discovery");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Repository Root:     {}", report.repo_root.display());
            println!("  Governance Output:   {}", report.output_dir.display());
            println!("  Dockerfiles Found:   {}", report.discovered_dockerfiles);
            println!("  K8s Manifests Found: {}", report.discovered_k8s_manifests);
            println!("  Lockfiles Found:     {}", report.discovered_lockfiles);
            println!("  CI/CD Workflows:     {}", report.discovered_ci_workflows);
            println!("  NIST Controls Mapped:{}", report.mapped_controls_count);
            println!("────────────────────────────────────────────────────────────────────────");

            println!("Created Governance Artifacts:");
            for f in &report.created_files {
                println!("  [file] {}", f.display());
            }

            println!("\nNext Recommended Commands:");
            println!("  1. Run Zero-Trust Pipeline:  mizan pipeline run --jurisdiction us");
            println!(
                "  2. Inspect Component Def:    mizan inspect {}/component-definition.json",
                report.output_dir.display()
            );
            println!(
                "  3. Export Auditor Matrix:    mizan catalog export-matrix -j us -o matrix.csv"
            );
            println!("────────────────────────────────────────────────────────────────────────");
        }
    }

    Ok(())
}
