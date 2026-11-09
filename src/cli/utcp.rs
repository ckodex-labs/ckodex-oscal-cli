#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;
use crate::utcp::{
    UtcpManual, UtcpSdkLanguage, UtcpTransportType, build_utcp_config, execute_utcp_tool,
    generate_sdk_snippet,
};

pub(super) fn run_utcp(args: &UtcpCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        UtcpCliAction::Manual {
            transport,
            base_url,
            output,
        } => {
            let t_type = UtcpTransportType::from_str_name(transport).ok_or_else(|| {
                AppError::Configuration(format!(
                    "Unsupported UTCP transport '{transport}'. Use: cli, http, grpc, mcp, sse, text"
                ))
            })?;
            let manual = UtcpManual::build_mizan_manual(t_type, base_url);
            let json_str = serde_json::to_string_pretty(&manual)?;
            if let Some(out_path) = output {
                fs::write(out_path, &json_str).map_err(|e| io_error(out_path, e))?;
            }
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    println!("{json_str}");
                }
                _ => {
                    if let Some(out_path) = output {
                        println!("Mizan UTCP v{} Manual Generated", manual.utcp_version);
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                        println!("  Manual Name:     {}", manual.name);
                        println!("  Transport:       {}", t_type.as_str());
                        println!("  Tools Count:     {}", manual.tools.len());
                        println!("  Manual Digest:   {}", manual.manual_digest);
                        println!("  Saved To:        {}", out_path.display());
                        println!(
                            "────────────────────────────────────────────────────────────────────────"
                        );
                    } else {
                        println!("{json_str}");
                    }
                }
            }
            Ok(())
        }
        UtcpCliAction::Config { base_url, output } => {
            let cfg = build_utcp_config(base_url);
            let json_str = serde_json::to_string_pretty(&cfg)?;
            if let Some(out_path) = output {
                fs::write(out_path, &json_str).map_err(|e| io_error(out_path, e))?;
            }
            println!("{json_str}");
            Ok(())
        }
        UtcpCliAction::Validate { file, strict } => {
            let content = fs::read_to_string(file).map_err(|e| io_error(file, e))?;
            let val: serde_json::Value = serde_json::from_str(&content)?;
            let report = UtcpManual::validate_json_with_options(&val, *strict);

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                _ => {
                    println!("Mizan UTCP Manual Specification Validation");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  File:            {}", file.display());
                    println!("  UTCP Version:    {}", report.utcp_version);
                    println!("  Manual Version:  {}", report.manual_version);
                    println!("  Tools Defined:   {}", report.tools_count);
                    println!(
                        "  Digest Verified: {}",
                        if report.digest_verified { "YES" } else { "NO" }
                    );
                    println!(
                        "  Status:          {}",
                        if report.is_valid {
                            "VALID (UTCP Conforming)"
                        } else {
                            "INVALID"
                        }
                    );
                    if !report.errors.is_empty() {
                        println!("\nValidation Errors:");
                        for err in &report.errors {
                            println!("  - {err}");
                        }
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }

            if !report.is_valid {
                return Err(AppError::Negative {
                    class: "utcp-validate",
                    status: format!(
                        "UTCP manual validation failed with {} error(s)",
                        report.errors.len()
                    ),
                });
            }
            Ok(())
        }
        UtcpCliAction::Call { tool, args } => {
            let parsed_args: serde_json::Value = serde_json::from_str(args).map_err(|e| {
                AppError::Configuration(format!("Invalid JSON in --args for UTCP call: {e}"))
            })?;
            let receipt = execute_utcp_tool(tool, &parsed_args)?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
            Ok(())
        }
        UtcpCliAction::Sdk {
            lang,
            base_url,
            output,
        } => {
            let sdk_lang = UtcpSdkLanguage::from_str_name(lang).ok_or_else(|| {
                AppError::Configuration(format!(
                    "Unsupported UTCP SDK target '{lang}'. Use: rust, typescript, go, python, mcp"
                ))
            })?;
            let snippet = generate_sdk_snippet(sdk_lang, base_url);
            if let Some(out_path) = output {
                fs::write(out_path, &snippet).map_err(|e| io_error(out_path, e))?;
            }
            print!("{snippet}");
            Ok(())
        }
        UtcpCliAction::Search { query, limit } => {
            let manual =
                UtcpManual::build_mizan_manual(UtcpTransportType::Cli, "http://localhost:3000");
            let matches = manual.search_tools(query, *limit);
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    println!("{}", serde_json::to_string_pretty(&matches)?);
                }
                _ => {
                    println!(
                        "Mizan UTCP Tool Search (`{query}`, {} match(es))",
                        matches.len()
                    );
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    for tool in &matches {
                        println!(
                            "  {:<32} [{}] {}",
                            tool.name,
                            tool.tags.join(", "),
                            tool.description
                        );
                    }
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
            Ok(())
        }
    }
}
