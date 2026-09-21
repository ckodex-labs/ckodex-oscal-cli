#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_assemble(args: &AssembleArgs, format: OutputFormat) -> Result<()> {
    let (_doc, report) = assemble_directory(&args.input_dir, args.output.as_deref())?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for assemble".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Markdown Workspace Assembly");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Input Dir:       {}", report.input_dir);
            println!("  Document Kind:   {}", report.kind);
            println!("  Controls:        {}", report.controls_assembled);
            println!("  Components:      {}", report.components_assembled);
            println!(
                "  Schema Valid:    {}",
                if report.is_valid { "YES" } else { "NO" }
            );
            if let Some(out) = &report.output_file {
                println!("  Saved To:        {out}");
            }
            println!("────────────────────────────────────────────────────────────────────────");
        }
    }
    Ok(())
}
