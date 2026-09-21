#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_inspect(args: &InspectArgs, format: OutputFormat) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let summary = inspect_document(&doc)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = if format == OutputFormat::Json {
                serde_json::to_string_pretty(&summary)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            } else {
                serde_json::to_string(&summary)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            };
            println!("{json_str}");
        }
        OutputFormat::Proto => {
            return Err(AppError::Configuration(
                "--format proto is not supported for inspect report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Model Summary");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Model Kind:      {}", summary.kind);
            println!("  Title:           {}", summary.title);
            println!("  Version:         {}", summary.version);
            println!("  OSCAL Version:   {}", summary.oscal_version);
            println!("  UUID:            {}", summary.uuid);
            println!("  Last Modified:   {}", summary.last_modified);
            if let Some(pub_date) = &summary.published {
                println!("  Published:       {pub_date}");
            }
            println!("────────────────────────────────────────────────────────────────────────");
            println!("Structure & Metrics:");
            println!("  Controls:        {}", summary.stats.total_controls);
            if !summary.stats.controls_by_family.is_empty() {
                let families: Vec<String> = summary
                    .stats
                    .controls_by_family
                    .iter()
                    .map(|(fam, count)| format!("{fam}: {count}"))
                    .collect();
                println!("    Families:      {}", families.join(", "));
            }
            println!("  Groups:          {}", summary.stats.total_groups);
            println!("  Parameters:      {}", summary.stats.total_params);
            println!("  Roles:           {}", summary.stats.total_roles);
            println!("  Parties:         {}", summary.stats.total_parties);
            if summary.stats.total_components > 0 {
                println!("  Components:      {}", summary.stats.total_components);
            }
            if summary.stats.total_findings > 0 || summary.stats.total_observations > 0 {
                println!("  Findings:        {}", summary.stats.total_findings);
                println!("  Observations:    {}", summary.stats.total_observations);
            }
            if summary.stats.total_poam_items > 0 {
                println!("  POA&M Items:     {}", summary.stats.total_poam_items);
            }
            if summary.stats.total_mappings > 0 {
                println!("  Mappings:        {}", summary.stats.total_mappings);
            }
            println!("────────────────────────────────────────────────────────────────────────");
        }
    }
    Ok(())
}
