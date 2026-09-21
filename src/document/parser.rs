use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    document::schema::DocumentKind,
    error::{io_error, AppError, Result},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileFormat {
    Json,
    Yaml,
    Csv,
    Proto,
}

impl FileFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Csv => "csv",
            Self::Proto => "pb",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OscalDocument {
    pub path: Option<PathBuf>,
    pub format: FileFormat,
    pub kind: DocumentKind,
    pub value: Value,
}

impl OscalDocument {
    pub fn from_value(value: Value, path: Option<PathBuf>) -> Result<Self> {
        let kind = detect_kind(&value)?;
        Ok(Self {
            path,
            format: FileFormat::Json,
            kind,
            value,
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) {
            if ext == "csv" {
                let content = fs::read_to_string(path_ref).map_err(|e| io_error(path_ref, e))?;
                return crate::document::tabular::import_from_csv(&content, None, None, None);
            }
        }
        let content = fs::read_to_string(path_ref).map_err(|e| io_error(path_ref, e))?;
        Self::from_str(&content, Some(path_ref.to_path_buf()))
    }

    pub fn from_str(content: &str, path: Option<PathBuf>) -> Result<Self> {
        let format = if let Some(p) = &path {
            match p.extension().and_then(|ext| ext.to_str()) {
                Some("yaml") | Some("yml") => FileFormat::Yaml,
                Some("csv") => FileFormat::Csv,
                Some("pb") | Some("proto") => FileFormat::Proto,
                _ => {
                    if serde_json::from_str::<Value>(content).is_ok() {
                        FileFormat::Json
                    } else if serde_yaml::from_str::<Value>(content).is_ok() {
                        FileFormat::Yaml
                    } else {
                        FileFormat::Json
                    }
                }
            }
        } else if serde_json::from_str::<Value>(content).is_ok() {
            FileFormat::Json
        } else {
            FileFormat::Yaml
        };

        let value: Value = match format {
            FileFormat::Json => serde_json::from_str(content)
                .map_err(|e| AppError::Configuration(format!("Failed to parse JSON: {e}")))?,
            FileFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| AppError::Configuration(format!("Failed to parse YAML: {e}")))?,
            FileFormat::Csv => {
                let doc = crate::document::tabular::import_from_csv(content, None, None, None)?;
                return Ok(doc);
            }
            FileFormat::Proto => serde_json::from_str(content)
                .map_err(|e| AppError::Configuration(format!("Failed to parse Proto text: {e}")))?,
        };

        let kind = detect_kind(&value)?;

        Ok(Self {
            path,
            format,
            kind,
            value,
        })
    }

    pub fn root_object(&self) -> Option<&serde_json::Map<String, Value>> {
        self.value
            .get(self.kind.root_key())
            .and_then(Value::as_object)
    }

    pub fn root_object_mut(&mut self) -> Option<&mut serde_json::Map<String, Value>> {
        self.value
            .get_mut(self.kind.root_key())
            .and_then(Value::as_object_mut)
    }

    pub fn metadata(&self) -> Option<&serde_json::Map<String, Value>> {
        self.root_object()?.get("metadata")?.as_object()
    }

    pub fn metadata_mut(&mut self) -> Option<&mut serde_json::Map<String, Value>> {
        self.root_object_mut()?.get_mut("metadata")?.as_object_mut()
    }

    pub fn uuid(&self) -> Option<&str> {
        self.root_object()?.get("uuid")?.as_str()
    }

    pub fn title(&self) -> Option<&str> {
        self.metadata()?.get("title")?.as_str()
    }

    pub fn version(&self) -> Option<&str> {
        self.metadata()?.get("version")?.as_str()
    }

    pub fn oscal_version(&self) -> Option<&str> {
        self.metadata()?.get("oscal-version")?.as_str()
    }

    pub fn last_modified(&self) -> Option<&str> {
        self.metadata()?.get("last-modified")?.as_str()
    }
}

pub fn detect_kind(value: &Value) -> Result<DocumentKind> {
    let obj = value.as_object().ok_or_else(|| {
        AppError::Configuration("OSCAL document must be a JSON/YAML object at root".to_owned())
    })?;

    for (k, _) in obj {
        if let Some(kind) = DocumentKind::from_root_key(k) {
            return Ok(kind);
        }
    }

    Err(AppError::Configuration(
        "Could not detect OSCAL root model (expected one of: catalog, profile, system-security-plan, component-definition, assessment-plan, assessment-results, plan-of-action-and-milestones, mapping-collection)".to_owned(),
    ))
}
