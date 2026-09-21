#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_lint(args: &LintArgs, format: OutputFormat) -> Result<()> {
    let mut doc = OscalDocument::from_file(&args.file)?;
    let report = lint_document(&mut doc, args.fix)?;

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
                "--format proto is not supported for lint report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Linter: {}", report.kind);
            if let Some(file) = &report.file {
                println!("  File: {file}");
            }
            if args.fix {
                println!(
                    "  Autofix applied: {} issue(s) resolved",
                    report.fixed_count
                );
            }
            if report.issues.is_empty() {
                println!("  Clean! No issues found.");
            } else {
                println!("\nIssues ({}):", report.issues.len());
                for issue in &report.issues {
                    println!(
                        "  [{}] [{}] {}: {} (autofixable: {})",
                        issue.severity, issue.code, issue.path, issue.message, issue.auto_fixable
                    );
                }
            }
        }
    }
    Ok(())
}
