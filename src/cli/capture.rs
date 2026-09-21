#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn run_capture(
    capture_root: &Path,
    format: OutputFormat,
    action: CaptureAction,
) -> Result<()> {
    match action {
        CaptureAction::List => {
            let manifests = crate::capture::list(capture_root)?;
            if format == OutputFormat::Table {
                let rows = manifests
                    .iter()
                    .map(|manifest| {
                        vec![
                            manifest.id.clone(),
                            manifest.method.clone(),
                            manifest.captured_at_unix_ms.to_string(),
                            manifest.response_bytes.to_string(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(&["ID", "METHOD", "CAPTURED MS", "RESPONSE BYTES"], &rows);
            } else {
                output::emit_json(format, &manifests)?;
            }
        }
        CaptureAction::Verify { id } => {
            let manifests = crate::capture::verify(capture_root, id.as_deref())?;
            if format == OutputFormat::Table {
                let rows = manifests
                    .iter()
                    .map(|manifest| {
                        vec![
                            manifest.id.clone(),
                            manifest.method.clone(),
                            "verified".to_owned(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(&["ID", "METHOD", "INTEGRITY"], &rows);
            } else {
                output::emit_json(format, &manifests)?;
            }
        }
        CaptureAction::Prune {
            older_than_days,
            confirm,
        } => {
            let result = crate::capture::prune(capture_root, older_than_days, confirm)?;
            if format == OutputFormat::Table {
                let rows = result
                    .candidates
                    .iter()
                    .map(|manifest| {
                        let state = if result.deleted_ids.iter().any(|id| id == &manifest.id) {
                            "deleted"
                        } else {
                            "would_delete"
                        };
                        vec![
                            manifest.id.clone(),
                            manifest.method.clone(),
                            state.to_owned(),
                        ]
                    })
                    .collect::<Vec<_>>();
                output::table(&["ID", "METHOD", "ACTION"], &rows);
                if result.dry_run {
                    println!("dry_run  true  (pass --confirm to delete verified local captures)");
                }
            } else {
                output::emit_json(format, &result)?;
            }
        }
        CaptureAction::Show { id } => {
            let (manifest, _) = crate::capture::load(capture_root, &id)?;
            output::emit_json(format, &manifest)?;
        }
        CaptureAction::Export { id } => {
            let (_, response) = crate::capture::load(capture_root, &id)?;
            std::io::stdout()
                .write_all(&response)
                .map_err(|error| AppError::Io {
                    path: "stdout".into(),
                    source: error,
                })?;
        }
    }
    Ok(())
}
