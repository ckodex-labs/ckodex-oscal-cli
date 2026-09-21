#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_fedramp(args: &FedrampCliArgs, format: OutputFormat) -> Result<()> {
    let (file_path, baseline_str) = match &args.action {
        FedrampAction::Validate { file, baseline } => (file.as_path(), baseline.as_str()),
    };

    let doc = OscalDocument::from_file(file_path)?;
    let baseline = FedrampBaseline::from_str_name(baseline_str).ok_or_else(|| {
        AppError::Configuration(format!(
            "Unknown FedRAMP baseline: {baseline_str} (use low, moderate, high)"
        ))
    })?;
    let report = validate_fedramp(&doc, baseline)?;

    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let json_str = serde_json::to_string_pretty(&report)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("FedRAMP PMO Baseline Validation ({})", report.baseline);
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  File:            {}", report.file);
            println!("  Document Kind:   {}", report.kind);
            println!(
                "  Status:          {}",
                if report.is_compliant {
                    "COMPLIANT"
                } else {
                    "NON-COMPLIANT"
                }
            );
            println!("  Rules Checked:   {}", report.total_rules_checked);
            println!("  Passed Rules:    {}", report.passed_rules);
            println!("  Failed Rules:    {}", report.failed_rules);
            println!("────────────────────────────────────────────────────────────────────────");

            if !report.findings.is_empty() {
                println!("\nPMO Rule Violations:");
                for f in &report.findings {
                    println!(
                        "  \u{26a0} [{}] {} (Severity: {})",
                        f.rule_id, f.title, f.severity
                    );
                    println!("     Detail: {}", f.detail);
                }
            } else {
                println!("\nAll FedRAMP PMO baseline rules passed.");
            }
        }
    }
    Ok(())
}
