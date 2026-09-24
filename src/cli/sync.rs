#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_sync(args: &SyncArgs, format: OutputFormat) -> Result<()> {
    if let Some(matrix_path) = &args.matrix {
        if !matrix_path.exists() {
            return Err(AppError::Configuration(format!(
                "Matrix CSV file does not exist: {}",
                matrix_path.display()
            )));
        }
        let target_file = if let Some(local_path) = &args.local {
            local_path
        } else if let Some(out) = &args.output {
            out
        } else {
            return Err(AppError::Configuration(
                "Must specify --local <DOCUMENT.json> or -o <OUTPUT.json> to sync matrix with"
                    .to_string(),
            ));
        };

        let target_doc = OscalDocument::from_file(target_file)?;
        let csv_content = fs::read_to_string(matrix_path)
            .map_err(|e| AppError::Configuration(format!("Failed to read matrix CSV: {e}")))?;

        let _merged_doc = crate::document::tabular::sync_matrix_csv(
            &target_doc,
            &csv_content,
            args.output.as_deref().or(Some(target_file)),
        )?;

        match format {
            OutputFormat::Json | OutputFormat::Jsonl => {
                let rep = serde_json::json!({
                    "matrix_source": matrix_path.display().to_string(),
                    "target_document": target_file.display().to_string(),
                    "output_file": args.output.as_ref().unwrap_or(target_file).display().to_string(),
                    "status": "SYNCHRONIZED"
                });
                println!("{rep}");
            }
            _ => {
                println!("Auditor Matrix Synchronization & AST Reconciliation");
                println!(
                    "────────────────────────────────────────────────────────────────────────"
                );
                println!("  Matrix Source:    {}", matrix_path.display());
                println!("  Target Document:  {}", target_file.display());
                println!(
                    "  Output File:      {}",
                    args.output.as_ref().unwrap_or(target_file).display()
                );
                println!("  Status:           SYNCHRONIZED");
                println!(
                    "────────────────────────────────────────────────────────────────────────"
                );
                println!("[OK] Auditor statuses and notes merged into OSCAL AST preserving UUIDs.");
            }
        }
        return Ok(());
    }

    let base_path = args.base.as_ref().ok_or_else(|| {
        AppError::Configuration("Missing required --base <DOCUMENT> for 3-way merge".to_string())
    })?;
    let upstream_path = args.upstream.as_ref().ok_or_else(|| {
        AppError::Configuration(
            "Missing required --upstream <DOCUMENT> for 3-way merge".to_string(),
        )
    })?;
    let local_path = args.local.as_ref().ok_or_else(|| {
        AppError::Configuration("Missing required --local <DOCUMENT> for 3-way merge".to_string())
    })?;

    let base_doc = OscalDocument::from_file(base_path)?;
    let upstream_doc = OscalDocument::from_file(upstream_path)?;
    let local_doc = OscalDocument::from_file(local_path)?;

    let strat = MergeStrategy::from_str_name(&args.strategy).ok_or_else(|| {
        AppError::Configuration(format!(
            "Unknown merge strategy: {} (use manual, ours, theirs)",
            args.strategy
        ))
    })?;

    let (_merged_doc, report) = sync_and_merge(
        &base_doc,
        &upstream_doc,
        &local_doc,
        strat,
        args.output.as_deref(),
        args.split_to.as_deref(),
    )?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("OSCAL 3-Way GitOps AST Sync & Merge");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Base:            {}", base_path.display());
            println!("  Upstream:        {}", upstream_path.display());
            println!("  Local:           {}", local_path.display());
            println!("  Strategy:        {}", report.strategy);
            println!(
                "  Status:          {}",
                if report.is_clean {
                    "CLEAN MERGE"
                } else {
                    "CONFLICTS DETECTED"
                }
            );
            println!("  Controls Merged: {}", report.controls_merged);
            println!("  Added (Upstream): {}", report.added_from_upstream.len());
            println!(
                "  Preserved Local:  {}",
                report.preserved_local_additions.len()
            );
            println!("  Updated Upstream: {}", report.updated_from_upstream.len());
            println!(
                "  Retained Local:   {}",
                report.retained_local_modifications.len()
            );
            if let Some(out) = &report.output_file {
                println!("  Saved Output:    {out}");
            }
            println!("────────────────────────────────────────────────────────────────────────");

            if !report.conflicts.is_empty() {
                println!("\nMerge Conflicts ({}):", report.conflicts.len());
                for c in &report.conflicts {
                    println!("  \u{26a0} Control: {}", c.control_id);
                    println!("     Local:      {}", c.local_summary);
                    println!("     Upstream:   {}", c.upstream_summary);
                    println!("     Resolution: {}", c.resolution);
                }
            } else {
                println!("\nSynchronization and merge completed cleanly.");
            }
        }
    }
    Ok(())
}
