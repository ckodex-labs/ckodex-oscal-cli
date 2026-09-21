#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_sbom(args: &SbomCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        SbomCliAction::Import { input, output } => {
            let (doc, summary) = SbomImporter::import_file(input)?;

            if let Some(out_p) = output {
                if let Some(parent) = out_p.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let json_str = serde_json::to_string_pretty(&doc.value)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                std::fs::write(out_p, json_str).map_err(|e| io_error(out_p, e))?;
            }

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&summary)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Software Bill of Materials (SBOM) Ingestion");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Source SBOM:     {}", input.display());
                    println!(
                        "  Format:          {} v{}",
                        summary.format, summary.spec_version
                    );
                    println!("  Components:      {}", summary.component_count);
                    println!("  Component UUID:  {}", summary.oscal_component_uuid);
                    if let Some(out) = output {
                        println!("  Saved To:        {}", out.display());
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
    }
    Ok(())
}
