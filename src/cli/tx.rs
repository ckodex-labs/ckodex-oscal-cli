#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_tx(args: &TxCliArgs, format: OutputFormat) -> Result<()> {
    match &args.action {
        TxCliAction::Begin { tx_id } => {
            let manager = ComplianceTransactionManager::default();
            let session = manager.begin_transaction(tx_id.as_deref())?;

            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&session.wal)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Compliance ACID Transaction Session");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Transaction ID:      {}", session.tx_id);
                    println!("  Status:              ACTIVE");
                    println!("  WAL Journal Path:    {}", session.wal_path.display());
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
    }
    Ok(())
}
