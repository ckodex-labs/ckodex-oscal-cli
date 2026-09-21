#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_federate(args: &FederateCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        FederateCliAction::AutoPoam {
            ssp,
            assessment,
            title,
            output,
        } => {
            let ssp_doc = OscalDocument::from_file(ssp)?;
            let assessment_doc = OscalDocument::from_file(assessment)?;

            let (_poam_doc, report) = ComplianceFederator::auto_generate_poam_from_findings(
                &ssp_doc,
                &assessment_doc,
                title,
                output.as_deref(),
            )?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Horizontal Document Federation (Assessment -> POA&M)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Source SSP:          {}", ssp.display());
                    println!("  Assessment Findings: {}", report.total_findings);
                    println!("  POA&M Items Created: {}", report.poam_items_created);
                    if let Some(out) = output {
                        println!("  Generated POA&M:     {}", out.display());
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
