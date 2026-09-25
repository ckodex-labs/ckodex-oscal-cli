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
        FederateCliAction::FromSecurityReport {
            report,
            input_format: fmt_hint,
            title,
            existing_poam,
            output,
        } => {
            let (_poam_doc, report_meta) = ComplianceFederator::generate_poam_from_security_report(
                report,
                fmt_hint,
                title,
                existing_poam.as_deref(),
                output.as_deref(),
            )?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report_meta)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Security Ingestion -> OSCAL POA&M");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Input Report:        {}", report.display());
                    println!("  Detected Format:     {}", report_meta.input_format);
                    println!("  Findings Ingested:   {}", report_meta.total_findings_read);
                    println!("  High/Critical Count: {}", report_meta.high_critical_count);
                    println!("  POA&M Items Added:   {}", report_meta.poam_items_created);
                    println!("  Total Items in POA&M:{}", report_meta.poam_items_total);
                    if let Some(ref out) = report_meta.output_file {
                        println!("  POA&M Document:      {out}");
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("[OK] Security report ingested into OSCAL POA&M successfully.");
                }
            }
        }
    }
    Ok(())
}
