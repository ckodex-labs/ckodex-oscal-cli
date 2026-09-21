#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_resolve(args: &ResolveArgs) -> Result<()> {
    let doc = OscalDocument::from_file(&args.file)?;
    let resolved = resolve_profile(&doc, args.output.as_deref())?;
    if args.output.is_none() {
        let json_str = serde_json::to_string_pretty(&resolved.value)
            .map_err(|e| AppError::Configuration(e.to_string()))?;
        println!("{json_str}");
    } else if let Some(out_p) = &args.output {
        println!("Resolved profile to catalog -> {}", out_p.display());
    }
    Ok(())
}
