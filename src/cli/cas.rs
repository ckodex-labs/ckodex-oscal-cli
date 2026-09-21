#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_cas(args: &CasCliArgs, format: OutputFormat) -> Result<()> {
    let cas = CasStore::default();
    match &args.action {
        CasCliAction::Stats => {
            let stats = cas.stats()?;
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let json_str = serde_json::to_string_pretty(&stats)
                        .map_err(|e| AppError::Configuration(e.to_string()))?;
                    println!("{json_str}");
                }
                _ => {
                    println!("Mizan Content-Addressable Storage (CAS / RustFS)");
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                    println!("  Root Directory:      {}", stats.root_dir.display());
                    println!("  Total Objects:       {}", stats.total_objects);
                    println!("  Total Bytes:         {}", stats.total_bytes);
                    println!(
                        "────────────────────────────────────────────────────────────────────────"
                    );
                }
            }
        }
        CasCliAction::Put { file } => {
            let bytes = std::fs::read(file).map_err(|e| io_error(file, e))?;
            let digest = cas.put_bytes(&bytes)?;
            match format {
                OutputFormat::Json | OutputFormat::Jsonl => {
                    let res = serde_json::json!({
                        "file": file.display().to_string(),
                        "digest": digest,
                        "bytes": bytes.len()
                    });
                    println!("{res}");
                }
                _ => {
                    println!("Stored file in Content-Addressable Storage:");
                    println!("  File:   {}", file.display());
                    println!("  Digest: {digest}");
                    println!("  Bytes:  {}", bytes.len());
                }
            }
        }
        CasCliAction::Get { digest, output } => {
            let bytes = cas.get_bytes(digest)?;
            if let Some(out_p) = output {
                if let Some(parent) = out_p.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(out_p, &bytes).map_err(|e| io_error(out_p, e))?;
                println!("Retrieved {digest} -> {}", out_p.display());
            } else {
                let text = String::from_utf8_lossy(&bytes);
                println!("{text}");
            }
        }
        CasCliAction::Prune => {
            let pruned_count = cas.prune()?;
            println!("Pruned {pruned_count} objects from CAS store.");
        }
    }
    Ok(())
}
