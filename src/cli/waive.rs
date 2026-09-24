use super::*;
use crate::document::waiver::{WaiverManager, WaiverStatus};

pub(super) fn run_waive(args: &WaiveCliArgs, format: OutputFormat) -> Result<()> {
    let mut manager = WaiverManager::load_or_default();

    if let Some(action) = &args.action {
        match action {
            WaiveCliAction::List => {
                match format {
                    OutputFormat::Json => {
                        let json_str = serde_json::to_string_pretty(&manager.list())
                            .map_err(|e| AppError::Configuration(e.to_string()))?;
                        println!("{json_str}");
                    }
                    _ => {
                        println!("Mizan Compliance Derogation Waivers");
                        println!(
                            "────────────────────────────────────────────────────────────────────────────────────────"
                        );
                        println!(
                            "{:<18} {:<16} {:<10} {:<14} {:<12} REASON",
                            "WAIVER ID", "RULE ID", "STATUS", "REMAINING", "SCOPE"
                        );
                        println!(
                            "────────────────────────────────────────────────────────────────────────────────────────"
                        );
                        if manager.list().is_empty() {
                            println!(
                                "  No active or historical waivers found in .mizan/waivers.json"
                            );
                        } else {
                            for w in manager.list() {
                                println!(
                                    "{:<18} {:<16} {:<10} {:<14} {:<12} {}",
                                    w.id,
                                    w.rule_id,
                                    w.status.to_string(),
                                    w.time_remaining_display(),
                                    if w.scope.len() > 10 {
                                        &w.scope[..10]
                                    } else {
                                        &w.scope
                                    },
                                    w.reason
                                );
                            }
                        }
                        println!(
                            "────────────────────────────────────────────────────────────────────────────────────────"
                        );
                    }
                }
                return Ok(());
            }
            WaiveCliAction::Revoke { id } => {
                let revoked = manager.revoke_waiver(id)?;
                match format {
                    OutputFormat::Json => {
                        let json_str = serde_json::to_string_pretty(&revoked)
                            .map_err(|e| AppError::Configuration(e.to_string()))?;
                        println!("{json_str}");
                    }
                    _ => {
                        println!(
                            "[OK] Derogation waiver '{}' revoked successfully.",
                            revoked.id
                        );
                        println!("  Rule:    {}", revoked.rule_id);
                        println!("  Status:  {}", revoked.status);
                    }
                }
                return Ok(());
            }
        }
    }

    // Direct invocation: mizan waive --rule <RULE> --reason <REASON> --ttl <TTL>
    let rule_id = args.rule.as_deref().ok_or_else(|| {
        AppError::Configuration("Missing required --rule argument. Example: mizan waive --rule cis-k8s-5.2.1 --reason \"Legacy ingress\" --ttl 7d".to_string())
    })?;

    let reason = args.reason.as_deref().ok_or_else(|| {
        AppError::Configuration(
            "Missing mandatory --reason argument justifying the derogation lease.".to_string(),
        )
    })?;

    let lease = manager.create_waiver(
        rule_id,
        reason,
        &args.ttl,
        Some(&args.scope),
        args.author.as_deref(),
    )?;

    match format {
        OutputFormat::Json => {
            let json_str = serde_json::to_string_pretty(&lease)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            println!("{json_str}");
        }
        _ => {
            println!("Mizan Compliance Derogation Lease Issued");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Waiver ID:      {}", lease.id);
            println!("  Rule ID:        {}", lease.rule_id);
            println!("  Status:         {}", lease.status);
            println!("  Scope:          {}", lease.scope);
            println!("  Author:         {}", lease.author);
            println!(
                "  Expires At:     {} (in {})",
                lease.expires_at,
                lease.time_remaining_display()
            );
            println!("  Reason:         \"{}\"", lease.reason);
            println!("  Fingerprint:    {}", lease.fingerprint);
            println!("────────────────────────────────────────────────────────────────────────");
            println!("Derogation active. Future pipeline runs targeting this rule will not block.");
        }
    }

    Ok(())
}
