#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_split(args: &SplitArgs, format: OutputFormat) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let report = split_document(&doc, &args.output_dir)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for split".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Agile Markdown Workspace Split");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Source File:     {}", args.file.display());
            println!("  Output Dir:      {}", report.output_dir);
            println!("  Document Kind:   {}", report.kind);
            println!("  Files Created:   {}", report.files_created);
            println!("  Controls Split:  {}", report.controls_split);
            println!("  Components:      {}", report.components_split);
            println!("────────────────────────────────────────────────────────────────────────");
            println!("Markdown authoring workspace initialized successfully.");
        }
    }
    Ok(())
}
