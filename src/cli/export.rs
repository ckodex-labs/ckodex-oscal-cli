#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_export(args: &ExportCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        ExportCliAction::Sarif {
            input,
            input_file,
            output,
        } => {
            let in_path = input
                .as_deref()
                .or(input_file.as_deref())
                .ok_or_else(|| {
                    AppError::Configuration(
                        "Missing input OSCAL document. Pass as argument or with -i/--input-file. Example: mizan export sarif examples/sample-ssp.json -o out.sarif".to_string(),
                    )
                })?;
            let doc = OscalDocument::from_file(in_path)?;
            let report = SarifExporter::export_from_oscal(&doc, in_path)?;

            if let Some(out_p) = output {
                SarifExporter::save_to_file(&report, out_p)?;

                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        let rep = serde_json::json!({
                            "format": "SARIF v2.1.0",
                            "rules_count": report.runs.first().map_or(0, |r| r.tool.driver.rules.len()),
                            "results_count": report.runs.first().map_or(0, |r| r.results.len()),
                            "output_file": out_p.display().to_string(),
                        });
                        println!("{rep}");
                    }
                    _ => {
                        println!("Exported OASIS SARIF v2.1.0 Report");
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Source Document: {}", in_path.display());
                        println!(
                            "  Rules Exported:  {}",
                            report.runs.first().map_or(0, |r| r.tool.driver.rules.len())
                        );
                        println!("  Output File:     {}", out_p.display());
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
            } else {
                let json_str = serde_json::to_string_pretty(&report)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                println!("{json_str}");
            }
        }
        ExportCliAction::Gitlab {
            input,
            input_file,
            output,
        } => {
            let in_path = input
                .as_deref()
                .or(input_file.as_deref())
                .ok_or_else(|| {
                    AppError::Configuration(
                        "Missing input OSCAL document. Pass as argument or with -i/--input-file. Example: mizan export gitlab examples/sample-ssp.json -o out.json".to_string(),
                    )
                })?;
            let doc = OscalDocument::from_file(in_path)?;
            let report = GitLabReportExporter::export_from_oscal(&doc, in_path)?;

            if let Some(out_p) = output {
                GitLabReportExporter::save_to_file(&report, out_p)?;

                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        let rep = serde_json::json!({
                            "format": "GitLab Security Report 15.0.0",
                            "vulnerabilities_count": report.vulnerabilities.len(),
                            "output_file": out_p.display().to_string(),
                        });
                        println!("{rep}");
                    }
                    _ => {
                        println!("Exported GitLab Security Scanner Report");
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Source Document: {}", in_path.display());
                        println!("  Vulnerabilities: {}", report.vulnerabilities.len());
                        println!("  Output File:     {}", out_p.display());
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
            } else {
                let json_str = serde_json::to_string_pretty(&report)
                    .map_err(|e| AppError::Configuration(e.to_string()))?;
                println!("{json_str}");
            }
        }
        ExportCliAction::Capsule {
            assessment,
            assessment_file,
            bundle,
            output,
        } => {
            let assess_path = assessment
                .as_deref()
                .or(assessment_file.as_deref())
                .ok_or_else(|| {
                    AppError::Configuration(
                        "Missing input assessment results JSON. Pass as argument or with -a/--assessment. Example: mizan export capsule results.json -o capsule.html".to_string(),
                    )
                })?;
            let default_out = std::path::PathBuf::from("capsule.html");
            let out_path = output.as_ref().unwrap_or(&default_out);
            let report = CapsuleExporter::export_capsule(assess_path, bundle.as_deref(), out_path)?;
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let rep = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{rep}");
                }
                _ => {
                    println!("Exported Offline Evidence Capsule");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Title:            {}", report.title);
                    println!("  Rules Evaluated:  {}", report.rules_evaluated);
                    println!("  Merkle Root:      {}", report.merkle_root);
                    println!("  File Size:        {} bytes", report.file_size_bytes);
                    println!("  Output File:      {}", report.output_path.display());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!(
                        "[OK] Standalone air-gap evidence capsule generated. Open in any browser."
                    );
                }
            }
        }
        ExportCliAction::Badge {
            label,
            message,
            color,
            variant,
            wcag,
            logo,
            link,
            from_document,
            output,
        } => {
            let mut cfg = ShieldcnBadgeConfig {
                label: label.clone(),
                message: message.clone(),
                color: color.clone(),
                variant: variant.clone(),
                wcag: Some(*wcag),
                logo: logo.clone(),
                provider_url: "https://shieldcn.dev".to_string(),
            };

            if let Some(doc_path) = from_document {
                let doc = OscalDocument::from_file(doc_path)?;
                if let Ok(fedramp_rep) = validate_fedramp(&doc, FedrampBaseline::Moderate) {
                    cfg = ShieldcnBadgeExporter::from_fedramp(
                        &fedramp_rep,
                        variant,
                        Some(*wcag),
                        "https://shieldcn.dev",
                    );
                }
            }

            let svg_url = ShieldcnBadgeExporter::build_url(&cfg);
            let md = ShieldcnBadgeExporter::to_markdown(&cfg, link.as_deref());

            if let Some(out) = output {
                fs::write(out, &md).map_err(|err| io_error(out, err))?;
            }

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let rep = serde_json::json!({
                        "label": cfg.label,
                        "message": cfg.message,
                        "color": cfg.color,
                        "variant": cfg.variant,
                        "wcag": cfg.wcag,
                        "svg_url": svg_url,
                        "markdown": md,
                        "output_file": output.as_ref().map(|p| p.display().to_string()),
                    });
                    println!("{rep}");
                }
                _ => {
                    println!("Generated shieldcn Badge");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Label:       {}", cfg.label);
                    println!("  Message:     {}", cfg.message);
                    println!("  Color:       {}", cfg.color);
                    println!("  Variant:     {}", cfg.variant);
                    println!(
                        "  WCAG 3.0:    {}",
                        cfg.wcag.map_or("disabled".to_string(), |w| format!(
                            "Mode {w} (APCA |Lc| >= 60)"
                        ))
                    );
                    println!("  SVG URL:     {svg_url}");
                    println!("  Markdown:    {md}");
                    if let Some(out) = output {
                        println!("  Output File: {}", out.display());
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
