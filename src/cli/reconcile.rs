#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_reconcile(args: &ReconcileArgs, format: OutputFormat) -> Result<()> {
    let ssp_doc = match &args.ssp {
        Some(p) => Some(OscalDocument::from_file(p)?),
        None => None,
    };
    let inventory_val = match &args.inventory {
        Some(p) => {
            let content = fs::read_to_string(p).map_err(|e| crate::error::io_error(p, e))?;
            let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                AppError::Configuration(format!("Failed to parse inventory JSON: {e}"))
            })?;
            Some(val)
        }
        None => None,
    };
    let results_doc = match &args.results {
        Some(p) => Some(OscalDocument::from_file(p)?),
        None => None,
    };
    let poam_doc = match &args.poam {
        Some(p) => Some(OscalDocument::from_file(p)?),
        None => None,
    };

    if ssp_doc.is_none() && inventory_val.is_none() && results_doc.is_none() && poam_doc.is_none() {
        return Err(AppError::Configuration(
            "reconcile requires at least --ssp and --inventory, or --results and --poam".to_owned(),
        ));
    }

    let report = reconcile_compliance(
        ssp_doc.as_ref(),
        inventory_val.as_ref(),
        results_doc.as_ref(),
        poam_doc.as_ref(),
    )?;

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
                "--format proto is not supported for reconcile report".to_owned(),
            ));
        }
        OutputFormat::Table => {
            println!("OSCAL Compliance Reconciliation");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Verdict:         {}", report.verdict.as_str());
            println!("────────────────────────────────────────────────────────────────────────");

            if let Some(comp_rec) = &report.component_reconciliation {
                println!("Component Inventory Reconciliation:");
                println!("  Declared Components:     {}", comp_rec.total_declared);
                println!("  Observed Components:     {}", comp_rec.total_observed);
                println!(
                    "  Matching Components:     {}",
                    comp_rec.matching_components.len()
                );
                println!(
                    "  Shadow Components:       {}",
                    comp_rec.shadow_components.len()
                );
                println!(
                    "  Phantom Components:      {}",
                    comp_rec.phantom_components.len()
                );
                println!(
                    "  Version Drifts:          {}",
                    comp_rec.version_drifts.len()
                );
            }

            if let Some(f_rec) = &report.finding_poam_reconciliation {
                println!("\nAssessment Finding \u{2194} POA&M Reconciliation:");
                println!("  Total Findings:          {}", f_rec.total_findings);
                println!(
                    "  Mitigated via POA&M:     {}",
                    f_rec.mitigated_findings.len()
                );
                println!(
                    "  Unmitigated Findings:    {}",
                    f_rec.unmitigated_findings.len()
                );
                println!(
                    "  Stale POA&M Items:       {}",
                    f_rec.stale_poam_items.len()
                );
            }

            if !report.action_items.is_empty() {
                println!("\nAction Items ({}):", report.action_items.len());
                for item in &report.action_items {
                    println!("  \u{26a0} {item}");
                }
            } else {
                println!("\nAll compliance records are fully reconciled and aligned.");
            }
        }
    }

    if args.strict && report.verdict != ReconciliationVerdict::Aligned {
        return Err(AppError::Configuration(format!(
            "Reconciliation failed in strict mode with verdict '{}'",
            report.verdict.as_str()
        )));
    }

    Ok(())
}
