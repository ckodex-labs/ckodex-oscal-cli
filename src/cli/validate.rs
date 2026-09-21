#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_validate(args: &ValidateArgs, format: OutputFormat) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let report = validate_document(
        &doc,
        &ValidationOptions {
            strict_constraints: args.strict,
            quiet: args.quiet,
        },
    )?;

    if args.quiet {
        if !report.is_valid {
            return Err(AppError::Configuration(format!(
                "Validation failed with {} error(s)",
                report.error_count()
            )));
        }
        return Ok(());
    }

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
                "--format proto is not supported for local document validation".to_owned(),
            ));
        }
        OutputFormat::Table => {
            let status = if report.is_valid { "VALID" } else { "INVALID" };
            println!("{status}: {} (OSCAL 1.2.3)", report.kind);
            if let Some(file) = &report.file {
                println!("  File: {file}");
            }
            println!("  Schema valid: {}", report.schema_valid);
            println!("  Constraints valid: {}", report.constraints_valid);
            if !report.diagnostics.is_empty() {
                println!("\nDiagnostics ({}):", report.diagnostics.len());
                for diag in &report.diagnostics {
                    let level_str = match diag.level {
                        DiagnosticLevel::Error => "ERROR",
                        DiagnosticLevel::Warning => "WARN ",
                        DiagnosticLevel::Info => "INFO ",
                    };
                    println!(
                        "  [{level_str}] [{}] {}: {}",
                        diag.code, diag.path, diag.message
                    );
                }
            }
        }
    }

    if !report.is_valid {
        return Err(AppError::Configuration(format!(
            "Document validation failed with {} error(s)",
            report.error_count()
        )));
    }

    Ok(())
}
