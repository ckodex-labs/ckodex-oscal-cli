#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_diff(args: &DiffArgs, format: OutputFormat) -> Result<()> {
    let doc_a = OscalDocument::from_file(&args.file_a)?;
    let doc_b = OscalDocument::from_file(&args.file_b)?;
    let diff = diff_documents(&doc_a, &doc_b)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&diff)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            } else {
                serde_json::to_string(&diff).map_err(|e| AppError::Configuration(e.to_string()))?
            };
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for diff report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Semantic Diff ({}):", diff.kind);
            println!("  Base:    {} (v{})", diff.title_a, diff.version_a);
            println!("  Revised: {} (v{})", diff.title_b, diff.version_b);
            if !diff.metadata_changes.is_empty() {
                println!("\nMetadata Changes:");
                for mc in &diff.metadata_changes {
                    println!("  ~ {}: '{}' -> '{}'", mc.field, mc.old_val, mc.new_val);
                }
            }
            if !diff.controls_added.is_empty() {
                println!("\nControls Added ({}):", diff.controls_added.len());
                for id in &diff.controls_added {
                    println!("  + {id}");
                }
            }
            if !diff.controls_removed.is_empty() {
                println!("\nControls Removed ({}):", diff.controls_removed.len());
                for id in &diff.controls_removed {
                    println!("  - {id}");
                }
            }
            if !diff.controls_modified.is_empty() {
                println!("\nControls Modified ({}):", diff.controls_modified.len());
                for cd in &diff.controls_modified {
                    println!(
                        "  * {} ({})",
                        cd.control_id,
                        cd.title_b.as_deref().unwrap_or("")
                    );
                }
            }
            if diff.controls_added.is_empty()
                && diff.controls_removed.is_empty()
                && diff.controls_modified.is_empty()
                && diff.metadata_changes.is_empty()
            {
                println!("\nNo semantic differences found.");
            }
        }
    }
    Ok(())
}
