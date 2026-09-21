#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_dedup(args: &DedupArgs, format: OutputFormat) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let output_target = if args.dry_run {
        None
    } else {
        args.output.as_deref()
    };
    let (_deduped_doc, report) = deduplicate_document(&doc, output_target)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&report)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            } else {
                serde_json::to_string(&report)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            };
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for dedup report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Deduplication Report ({})", report.kind);
            if let Some(f) = &report.file {
                println!("  Input File:      {f}");
            }
            if let Some(out_p) = &args.output {
                println!("  Saved To:        {}", out_p.display());
            } else if args.dry_run {
                println!("  Mode:            Dry Run (No files written)");
            }
            println!("────────────────────────────────────────────────────────────────────────");
            println!(
                "  Controls Deduplicated:   {}",
                report.controls_deduplicated
            );
            println!(
                "  Components Deduplicated: {}",
                report.components_deduplicated
            );
            println!("  Parties Deduplicated:    {}", report.parties_deduplicated);
            println!(
                "  Resources Deduplicated:  {}",
                report.resources_deduplicated
            );
            println!(
                "  Total Removed:           {}",
                report.total_duplicates_removed
            );
            println!("────────────────────────────────────────────────────────────────────────");

            if !report.modifications.is_empty() {
                println!("\nModifications Applied:");
                for m in &report.modifications {
                    println!("  \u{2713} {m}");
                }
            } else {
                println!("\nClean! No duplicate entities detected.");
            }
        }
    }
    Ok(())
}
