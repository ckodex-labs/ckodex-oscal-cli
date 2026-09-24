use super::*;
use crate::document::fix::FixEngine;

pub(super) fn run_fix(args: &FixCliArgs, format: OutputFormat) -> Result<()> {
    let report = FixEngine::fix_rule(&args.rule, &args.file, args.dry_run)?;

    match format {
        OutputFormat::Json => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("Mizan Automated Remediation Engine");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Rule ID:     {}", report.rule_id);
            println!("  Target File: {}", report.target_file.display());
            println!(
                "  Applied:     {}",
                if report.applied { "YES" } else { "NO" }
            );
            println!(
                "  Mode:        {}",
                if report.dry_run {
                    "DRY-RUN (Preview)"
                } else {
                    "MUTATE (In-Place)"
                }
            );
            println!("  Action:      {}", report.description);
            println!("────────────────────────────────────────────────────────────────────────");

            if !report.diff.is_empty() {
                println!("Diff Patch Preview:");
                println!("{}", report.diff);
            }

            if report.applied && !report.dry_run {
                println!("[OK] Manifest remediated and written to disk successfully.");
            } else if report.applied && report.dry_run {
                println!("[NOTE] Run without --dry-run to apply this remediation patch.");
            }
        }
    }

    Ok(())
}
