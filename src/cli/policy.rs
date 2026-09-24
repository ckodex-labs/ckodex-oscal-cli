#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_policy(args: &PolicyCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        PolicyAction::Compile {
            file,
            target,
            output_dir,
        } => {
            let doc = OscalDocument::from_file(file)?;
            let p_target = PolicyTarget::from_str_name(target).ok_or_else(|| {
                AppError::Configuration(format!(
                    "Unknown policy target: {target} (use rego, kyverno, all)"
                ))
            })?;
            let report = compile_policies(&doc, p_target, output_dir)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("OSCAL Compliance-to-Policy (C2P) Compiler");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Source Document: {}", report.source_file);
                    println!("  Target Engine:   {}", report.target);
                    println!("  Rules Compiled:  {}", report.policies_generated);
                    println!("  Output Dir:      {}", output_dir.display());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    for f in &report.generated_files {
                        println!("  \u{2713} Generated {f}");
                    }
                }
            }
        }
        PolicyAction::Ingest {
            log_file,
            title,
            output,
        } => {
            let (_doc, report) = ingest_policy_results(log_file, title, output.as_deref())?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&report)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("OSCAL Policy Results Ingestion");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Total Evaluated: {}", report.total_evaluated);
                    println!("  Passed Rules:    {}", report.passed_checks);
                    println!("  Failed Rules:    {}", report.failed_checks);
                    println!("  Findings Made:   {}", report.generated_findings);
                    if let Some(out) = &report.assessment_results_file {
                        println!("  Saved To:        {out}");
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        PolicyAction::Eval {
            policy,
            input,
            query,
        } => {
            let input_str = std::fs::read_to_string(input).map_err(|e| io_error(input, e))?;
            let input_json: serde_json::Value = if input.extension().and_then(|e| e.to_str())
                == Some("yaml")
                || input.extension().and_then(|e| e.to_str()) == Some("yml")
            {
                serde_yaml::from_str(&input_str)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            } else {
                serde_json::from_str(&input_str)
                    .map_err(|e| AppError::Configuration(e.to_string()))?
            };

            let mut evaluator = crate::document::policy::evaluator::RegorusEvaluator::new();
            evaluator.add_policy_dir(policy)?;
            let eval_res = evaluator.evaluate_compliance_rule(query, input_json)?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&eval_res)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan In-Process Rego Evaluation (Microsoft Regorus)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Policy Target:    {}", policy.display());
                    println!("  Query Package:    {query}");
                    println!(
                        "  Compliance State: {}",
                        if eval_res.passed {
                            "PASSED (Clean)"
                        } else {
                            "VIOLATIONS DETECTED"
                        }
                    );
                    if !eval_res.findings.is_empty() {
                        println!("\nRule Violations ({}):", eval_res.findings.len());
                        for f in &eval_res.findings {
                            println!("  \u{26a0} {f}");
                        }
                    } else {
                        println!("\nInput complies with evaluated OSCAL Rego rules.");
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        PolicyAction::Rulepack { action } => match action {
            RulepackAction::List => {
                let rules = BuiltinRulepack::get_rules();
                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        let json_str = serde_json::to_string_pretty(&rules)
                            .map_err(|e| AppError::Configuration(e.to_string()))?;
                        println!("{json_str}");
                    }
                    _ => {
                        println!("Mizan Built-in Compliance Rego Rulepacks");
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        for r in rules {
                            println!("  [{}] {}", r.id, r.name);
                            println!("    Benchmark: {}", r.benchmark);
                            println!(
                                "    Severity:  {} | Controls: {}",
                                r.severity,
                                r.target_controls.join(", ")
                            );
                            println!("    Desc:      {}", r.description);
                            println!();
                        }
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!(
                            "  Run with: `mizan policy rulepack eval -r <rule-id> -i <workload.json>`"
                        );
                    }
                }
            }
            RulepackAction::Eval { rule, input } => {
                let input_str = std::fs::read_to_string(input).map_err(|e| io_error(input, e))?;
                let input_json: serde_json::Value = if input.extension().and_then(|e| e.to_str())
                    == Some("yaml")
                    || input.extension().and_then(|e| e.to_str()) == Some("yml")
                {
                    serde_yaml::from_str(&input_str)
                        .map_err(|e| AppError::Configuration(e.to_string()))?
                } else {
                    serde_json::from_str(&input_str)
                        .map_err(|e| AppError::Configuration(e.to_string()))?
                };

                let res = BuiltinRulepack::evaluate_rule(rule, &input_json)?;

                match format {
                    OutputFormat::Json | OutputFormat::Jsonl => {
                        let json_str = serde_json::to_string_pretty(&res)
                            .map_err(|e| AppError::Configuration(e.to_string()))?;
                        println!("{json_str}");
                    }
                    _ => {
                        println!("Mizan Rulepack Evaluation: {rule}");
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Target Workload:  {}", input.display());
                        println!(
                            "  Status:           {}",
                            if res.passed {
                                "ALLOWED / PASSED"
                            } else {
                                "VIOLATION DETECTED"
                            }
                        );
                        if !res.findings.is_empty() {
                            println!("\n  Violations:");
                            for v in &res.findings {
                                println!("    \u{26a0} {v}");
                            }
                        }
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
            }
        },
    }
    Ok(())
}
