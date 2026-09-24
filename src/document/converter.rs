use std::{fs, path::Path};

use crate::{
    document::parser::{FileFormat, OscalDocument},
    error::{AppError, Result, io_error},
};

pub fn convert_document(
    doc: &OscalDocument,
    target_format: FileFormat,
    output_path: Option<&Path>,
) -> Result<String> {
    let output_str = match target_format {
        FileFormat::Json => serde_json::to_string_pretty(&doc.value)
            .map_err(|e| AppError::Configuration(format!("Failed to serialize JSON: {e}")))?,
        FileFormat::Yaml => serde_yaml::to_string(&doc.value)
            .map_err(|e| AppError::Configuration(format!("Failed to serialize YAML: {e}")))?,
        FileFormat::Csv => crate::document::tabular::export_to_csv(doc, output_path)?,
        FileFormat::Proto => {
            let json_bytes = serde_json::to_vec(&doc.value).map_err(|e| {
                AppError::Configuration(format!("Failed to encode Proto bytes: {e}"))
            })?;
            if let Some(out_p) = output_path {
                fs::write(out_p, &json_bytes).map_err(|e| io_error(out_p, e))?;
            }
            return Ok(format!(
                "[Binary Protobuf payload: {} bytes written]",
                json_bytes.len()
            ));
        }
    };

    if target_format != FileFormat::Csv
        && let Some(out_p) = output_path
    {
        fs::write(out_p, &output_str).map_err(|e| io_error(out_p, e))?;
    }

    Ok(output_str)
}
