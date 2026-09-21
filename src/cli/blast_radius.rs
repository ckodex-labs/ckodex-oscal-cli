#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_blast_radius(args: &BlastRadiusArgs, format: OutputFormat) -> Result<()> {
    let primary = OscalDocument::from_file(&args.file)?;
    let mut context_docs = Vec::new();
    for ctx_p in &args.context_files {
        context_docs.push(OscalDocument::from_file(ctx_p)?);
    }

    let report = analyze_blast_radius(&primary, &context_docs, &args.target, args.depth)?;

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
                "--format proto is not supported for blast-radius report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Compliance Blast Radius Analysis");
            println!("────────────────────────────────────────────────────────────────────────");
            println!(
                "  Target:          {} ({})",
                report.target_id, report.target_kind
            );
            println!(
                "  Risk Score:      {:.1} / 10.0",
                report.risk_exposure_score
            );
            println!(
                "  Critical Path:   {}",
                if report.is_critical_path {
                    "YES (High Impact)"
                } else {
                    "No"
                }
            );
            println!(
                "  Docs Analyzed:   {}",
                report.documents_analyzed.join(", ")
            );
            println!("────────────────────────────────────────────────────────────────────────");
            println!("Impact Summary:");
            println!(
                "  Direct Dependents:      {}",
                report.direct_dependents.len()
            );
            println!(
                "  Transitive Dependents:  {}",
                report.transitive_dependents.len()
            );
            println!(
                "  Affected Components:    {}",
                report.affected_components.len()
            );
            println!(
                "  Affected Findings:      {}",
                report.affected_findings.len()
            );
            println!(
                "  Affected POA&M Items:   {}",
                report.affected_poam_items.len()
            );
            println!("────────────────────────────────────────────────────────────────────────");

            if !report.direct_dependents.is_empty() {
                println!("\nDirect Dependents:");
                for dep in &report.direct_dependents {
                    println!(
                        "  \u{25b6} [{}] {} ({}) in {}",
                        dep.kind, dep.id, dep.relation, dep.document
                    );
                }
            }

            if !report.transitive_dependents.is_empty() {
                println!("\nTransitive Dependents:");
                for dep in &report.transitive_dependents {
                    println!(
                        "    \u{21b3} [{}] {} ({}) in {}",
                        dep.kind, dep.id, dep.relation, dep.document
                    );
                }
            }
        }
    }
    Ok(())
}
