#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn emit_names<M: Message + std::fmt::Debug>(
    config: &AppConfig,
    full_name: &str,
    response: &M,
    names: &[String],
) -> Result<()> {
    if config.output == OutputFormat::Table {
        output::table(
            &["NAME"],
            &names
                .iter()
                .map(|name| vec![name.clone()])
                .collect::<Vec<_>>(),
        );
        Ok(())
    } else {
        output::emit_message(config.output, full_name, response)
    }
}

pub(super) fn resource_row(
    kind: &str,
    id: String,
    title: String,
    version: String,
    count: usize,
) -> ResourceRow {
    ResourceRow {
        kind: kind.to_owned(),
        id,
        title,
        version,
        count,
    }
}

pub(super) fn emit_collection<M: Message + std::fmt::Debug>(
    format: OutputFormat,
    full_name: &str,
    response: &M,
    rows: Vec<ResourceRow>,
    next_page_token: &str,
) -> Result<()> {
    if format == OutputFormat::Table {
        let table_rows = rows
            .iter()
            .map(|row| {
                vec![
                    row.kind.clone(),
                    row.id.clone(),
                    row.title.clone(),
                    row.version.clone(),
                    row.count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        output::table(&["TYPE", "ID", "TITLE", "VERSION", "COUNT"], &table_rows);
        output::page_token(next_page_token);
        Ok(())
    } else if format == OutputFormat::Proto {
        output::emit_message(format, full_name, response)
    } else {
        output::emit_json(
            format,
            &serde_json::json!({
                "items": rows,
                "nextPageToken": next_page_token,
            }),
        )
    }
}

pub(super) async fn get_model<Req, Resp, Fut>(
    config: &AppConfig,
    method: &str,
    full_name: &str,
    request: Req,
    future: Fut,
) -> Result<()>
where
    Req: Message + Clone,
    Resp: Message + std::fmt::Debug,
    Fut: std::future::Future<Output = Result<Resp>>,
{
    let response = future.await?;
    capture_if_enabled(config, method, &request, &response)?;
    output::emit_message(config.output, full_name, &response)
}

pub(super) fn validate_positive(name: &str, value: i32) -> Result<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(AppError::InvalidArgument(format!(
            "{name} must be greater than zero"
        )))
    }
}

pub(super) fn validate_non_empty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(AppError::InvalidArgument(format!(
            "{name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

pub(crate) fn mapping_count(item: &crate::proto::oscal::mapping::v1::MappingCollection) -> usize {
    if item.mappings.is_empty() {
        #[allow(deprecated)]
        let legacy_count = item.maps.len();
        legacy_count
    } else {
        item.mappings.len()
    }
}

pub(super) fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

pub(super) fn capture_if_enabled<M1: Message, M2: Message>(
    config: &AppConfig,
    method: &str,
    request: &M1,
    response: &M2,
) -> Result<()> {
    if config.capture_enabled {
        match crate::capture::write(config, method, request, response) {
            Ok(manifest) => eprintln!(
                "generated capture {} ({})",
                manifest.id, manifest.response_sha256
            ),
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn _unused_surface_anchor(
    _: GetSnapshotRequest,
    _: ListSnapshotsRequest,
    _: ListReleasesRequest,
    _: GetEvidenceRequest,
    _: VerifyEvidenceRequest,
) {
}
