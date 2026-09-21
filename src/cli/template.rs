#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_template(args: &TemplateArgs, format: OutputFormat) -> Result<()> {
    let (_doc, report) = scaffold_template(
        &args.standard,
        &args.kind,
        args.title.as_deref(),
        args.output.as_deref(),
        args.split_to.as_deref(),
    )?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for template".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Artifact Template Generator");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Standard:        {}", report.standard);
            println!("  Kind:            {}", report.kind);
            println!("  Title:           {}", report.title);
            if let Some(out) = &report.output_path {
                println!("  Output File:     {out}");
            }
            if report.split_to_dir {
                println!("  Workspace:       Scaffolded into Markdown directory");
            }
            println!("────────────────────────────────────────────────────────────────────────");
        }
    }
    Ok(())
}
