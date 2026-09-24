use std::io::{self, Write};

use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, SerializeOptions};
use serde::Serialize;
use serde_json::Value;

use crate::{
    PROTO_DESCRIPTOR_SET,
    config::OutputFormat,
    error::{AppError, Result},
    valence::CrudResult,
};

/// Escape terminal control characters without changing structured output.
/// JSON and protobuf paths retain original values; this is for human-facing
/// terminal surfaces only.
pub fn terminal_text(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_control() {
            use std::fmt::Write;

            let _ = write!(sanitized, "\\u{{{:04x}}}", character as u32);
        } else {
            sanitized.push(character);
        }
    }
    sanitized
}

pub fn message_json<M: Message>(full_name: &str, message: &M) -> Result<Value> {
    let pool = DescriptorPool::decode(PROTO_DESCRIPTOR_SET)
        .map_err(|error| AppError::Descriptor(error.to_string()))?;
    let descriptor = pool.get_message_by_name(full_name).ok_or_else(|| {
        AppError::Descriptor(format!("message descriptor not found: {full_name}"))
    })?;
    let bytes = message.encode_to_vec();
    let dynamic = DynamicMessage::decode(descriptor, bytes.as_slice())
        .map_err(|error| AppError::Descriptor(error.to_string()))?;
    let mut serializer = serde_json::Serializer::new(Vec::new());
    dynamic.serialize_with_options(
        &mut serializer,
        &SerializeOptions::new().skip_default_fields(false),
    )?;
    serde_json::from_slice(&serializer.into_inner()).map_err(AppError::from)
}

pub fn emit_message<M: Message + std::fmt::Debug>(
    format: OutputFormat,
    full_name: &str,
    message: &M,
) -> Result<()> {
    match format {
        OutputFormat::Proto => emit_proto(message)?,
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&message_json(full_name, message)?)?
            );
        }
        OutputFormat::Jsonl => println!(
            "{}",
            serde_json::to_string(&message_json(full_name, message)?)?
        ),
        OutputFormat::Table => println!("{message:#?}"),
    }
    Ok(())
}

pub fn emit_crud_result<M: Message + std::fmt::Debug>(
    format: OutputFormat,
    full_name: &str,
    result: &CrudResult<M>,
    include_valence: bool,
) -> Result<()> {
    if !include_valence {
        return emit_message(format, full_name, &result.data);
    }
    match format {
        OutputFormat::Proto => emit_proto(&result.data)?,
        OutputFormat::Table => {
            table(&["VALENCE"], &[vec![result.valence.to_string()]]);
            println!("{:#?}", result.data);
        }
        OutputFormat::Json => {
            let mut value = message_json(full_name, &result.data)?;
            if let Value::Object(ref mut map) = value {
                map.insert(
                    "valence".to_owned(),
                    Value::String(result.valence.to_string()),
                );
            }
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        OutputFormat::Jsonl => {
            let mut value = message_json(full_name, &result.data)?;
            if let Value::Object(ref mut map) = value {
                map.insert(
                    "valence".to_owned(),
                    Value::String(result.valence.to_string()),
                );
            }
            println!("{}", serde_json::to_string(&value)?);
        }
    }
    Ok(())
}

pub fn emit_proto<M: Message>(message: &M) -> Result<()> {
    io::stdout()
        .write_all(&message.encode_to_vec())
        .map_err(|error| AppError::Io {
            path: "stdout".into(),
            source: error,
        })?;
    Ok(())
}

pub fn emit_json<T: Serialize>(format: OutputFormat, value: &T) -> Result<()> {
    match format {
        OutputFormat::Proto => Err(AppError::Configuration(
            "--format proto is only valid for protobuf responses".to_owned(),
        )),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
            Ok(())
        }
        OutputFormat::Jsonl | OutputFormat::Table => {
            println!("{}", serde_json::to_string(value)?);
            Ok(())
        }
    }
}

pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    let safe_headers = headers
        .iter()
        .map(|header| terminal_text(header))
        .collect::<Vec<_>>();
    let safe_rows = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| terminal_text(cell))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut widths: Vec<usize> = safe_headers.iter().map(|header| header.len()).collect();
    for row in &safe_rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.len());
            }
        }
    }

    let render = |row: &[String]| {
        row.iter()
            .enumerate()
            .map(|(index, cell)| format!("{cell:<width$}", width = widths[index]))
            .collect::<Vec<_>>()
            .join("  ")
    };

    println!("{}", render(&safe_headers));
    println!(
        "{}",
        widths
            .iter()
            .map(|width| "─".repeat(*width))
            .collect::<Vec<_>>()
            .join("──")
    );
    for row in &safe_rows {
        println!("{}", render(row));
    }
}

pub fn page_token(token: &str) {
    if !token.is_empty() {
        println!("next_page_token  {}", terminal_text(token));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::oscal::{
        common::v1::Uuid,
        services::v1::{SearchResponse, SearchResult},
    };

    #[test]
    fn protobuf_response_can_be_rendered_as_json() {
        let response = SearchResponse {
            results: vec![SearchResult {
                model_type: "catalog".to_owned(),
                uuid: Some(Uuid {
                    value: "fixture".to_owned(),
                }),
                title: "Fixture catalog".to_owned(),
                score: 0.75,
            }],
            next_page_token: String::new(),
        };
        let value = message_json("oscal.services.v1.SearchResponse", &response)
            .expect("descriptor-backed JSON should render");
        assert!(value.get("results").is_some());
        assert!(value.get("nextPageToken").is_some());
    }

    #[test]
    fn terminal_text_escapes_control_characters() {
        assert_eq!(
            terminal_text("safe\u{1b}[31m\ntext"),
            "safe\\u{001b}[31m\\u{000a}text"
        );
    }
}
