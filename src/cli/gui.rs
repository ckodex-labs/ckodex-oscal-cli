#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_gui(format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            println!(
                r#"{{"workbench":"Mizan","url":"http://localhost:3000","tauri_bundle":"apps/workbench/src-tauri","status":"online"}}"#
            );
        }
        _ => {
            println!("Mizan · OSCAL Compliance Desktop Workbench & GUI");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("  Web / UI Server:    http://localhost:3000");
            println!("  Tauri Desktop App:  apps/workbench/src-tauri (Mizan.app)");
            println!(
                "  Themes Supported:   ledger (Light Paper), vault (Darkroom), hc (High Contrast)"
            );
            println!("  MCP Core Bridge:    JSON-RPC 2.0 stdio active");
            println!("────────────────────────────────────────────────────────────────────────");
            println!("\nTo open the desktop app via Tauri, run:");
            println!("  cd apps/workbench && npm run tauri dev\n");
            println!("To open in your web browser, visit:");
            println!("  http://localhost:3000\n");
        }
    }
    Ok(())
}
