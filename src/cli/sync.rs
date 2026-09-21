#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_sync(args: &SyncArgs, format: OutputFormat) -> Result<()> {
    let base_doc = OscalDocument::from_file(&args.base)?;
    let upstream_doc = OscalDocument::from_file(&args.upstream)?;
    let local_doc = OscalDocument::from_file(&args.local)?;

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
            println!("  Base:            {}", args.base.display());
            println!("  Upstream:        {}", args.upstream.display());
            println!("  Local:           {}", args.local.display());
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
