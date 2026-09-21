#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_export(args: &ExportCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        ExportCliAction::Sarif { input, output } => {
            let doc = OscalDocument::from_file(input)?;
            let report = SarifExporter::export_from_oscal(&doc, input)?;
            SarifExporter::save_to_file(&report, output)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let rep = serde_json::json!({
                        "format": "SARIF v2.1.0",
                        "rules_count": report.runs.first().map_or(0, |r| r.tool.driver.rules.len()),
                        "results_count": report.runs.first().map_or(0, |r| r.results.len()),
                        "output_file": output.display().to_string(),
                    });
                    println!("{rep}");
                }
                _ => {
                    println!("Exported OASIS SARIF v2.1.0 Report");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Source Document: {}", input.display());
                    println!(
                        "  Rules Exported:  {}",
                        report.runs.first().map_or(0, |r| r.tool.driver.rules.len())
                    );
                    println!("  Output File:     {}", output.display());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        ExportCliAction::Gitlab { input, output } => {
            let doc = OscalDocument::from_file(input)?;
            let report = GitLabReportExporter::export_from_oscal(&doc, input)?;
            GitLabReportExporter::save_to_file(&report, output)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let rep = serde_json::json!({
                        "format": "GitLab Security Report 15.0.0",
                        "vulnerabilities_count": report.vulnerabilities.len(),
                        "output_file": output.display().to_string(),
                    });
                    println!("{rep}");
                }
                _ => {
                    println!("Exported GitLab Security Scanner Report");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Source Document: {}", input.display());
                    println!("  Vulnerabilities: {}", report.vulnerabilities.len());
                    println!("  Output File:     {}", output.display());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
    }
    Ok(())
}
