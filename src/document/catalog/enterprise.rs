use serde_json::{Map, Value, json};
use std::path::Path;

use crate::{
    document::{
        catalog::{embedded::EmbeddedCatalogProvider, jurisdiction::Jurisdiction},
        parser::OscalDocument,
        schema::DocumentKind,
    },
    error::{AppError, Result, io_error},
};

pub struct EnterpriseCatalogBuilder {
    title: String,
    version: String,
    base_jurisdiction: Option<Jurisdiction>,
    custom_controls: Vec<Value>,
    parameter_overlays: Vec<(String, String, String)>, // (control_id, param_id, value)
}

impl EnterpriseCatalogBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            version: "1.0.0".to_string(),
            base_jurisdiction: None,
            custom_controls: Vec::new(),
            parameter_overlays: Vec::new(),
        }
    }

    pub fn with_base_jurisdiction(mut self, jurisdiction: Jurisdiction) -> Self {
        self.base_jurisdiction = Some(jurisdiction);
        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn add_custom_control(
        mut self,
        control_id: &str,
        title: &str,
        description: &str,
        class: &str,
    ) -> Self {
        let ctrl = json!({
            "id": control_id,
            "title": title,
            "class": class,
            "description": description
        });
        self.custom_controls.push(ctrl);
        self
    }

    pub fn overlay_parameter(mut self, control_id: &str, param_id: &str, new_value: &str) -> Self {
        self.parameter_overlays.push((
            control_id.to_string(),
            param_id.to_string(),
            new_value.to_string(),
        ));
        self
    }

    pub fn build(self, output_path: Option<&Path>) -> Result<OscalDocument> {
        let mut controls = Vec::new();

        // 1. Inherit baseline controls if base is specified
        if let Some(base) = self.base_jurisdiction {
            let base_doc = EmbeddedCatalogProvider::get_catalog(base)?;
            if let Some(base_root) = base_doc.root_object()
                && let Some(base_ctrls) = base_root.get("controls").and_then(Value::as_array)
            {
                controls.extend(base_ctrls.clone());
            }
        }

        // 2. Apply parameter overlays
        for (ctrl_id, param_id, new_val) in &self.parameter_overlays {
            for ctrl in &mut controls {
                if ctrl.get("id").and_then(Value::as_str) == Some(ctrl_id)
                    && let Some(params) = ctrl.get_mut("params").and_then(Value::as_array_mut)
                {
                    for p in params {
                        if p.get("id").and_then(Value::as_str) == Some(param_id) {
                            p["values"] = json!([new_val]);
                        }
                    }
                }
            }
        }

        // 3. Append custom enterprise controls
        controls.extend(self.custom_controls);

        let now = chrono::Utc::now().to_rfc3339();
        let doc_uuid = uuid::Uuid::new_v4().to_string();

        let mut metadata = Map::new();
        metadata.insert("title".to_string(), json!(self.title));
        metadata.insert("published".to_string(), json!(now));
        metadata.insert("last-modified".to_string(), json!(now));
        metadata.insert("version".to_string(), json!(self.version));
        metadata.insert("oscal-version".to_string(), json!("1.2.3"));

        let mut root_obj = Map::new();
        root_obj.insert("uuid".to_string(), json!(doc_uuid));
        root_obj.insert("metadata".to_string(), Value::Object(metadata));
        root_obj.insert("controls".to_string(), Value::Array(controls));

        let mut doc_json = Map::new();
        doc_json.insert(
            DocumentKind::Catalog.root_key().to_string(),
            Value::Object(root_obj),
        );

        let doc = OscalDocument::from_value(
            Value::Object(doc_json),
            output_path.map(|p| p.to_path_buf()),
        )?;

        if let Some(out_p) = output_path {
            if let Some(parent) = out_p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let json_str = serde_json::to_string_pretty(&doc.value)
                .map_err(|e| AppError::Configuration(e.to_string()))?;
            std::fs::write(out_p, json_str).map_err(|e| io_error(out_p, e))?;
        }

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_catalog_inheritance_and_overlay() {
        let builder = EnterpriseCatalogBuilder::new("Acme Corp Secure Cloud Baseline")
            .with_base_jurisdiction(Jurisdiction::UsNist800_53Rev5)
            .add_custom_control(
                "corp-sec-01",
                "Mandatory Hardware Security Key MFA",
                "All engineers must use FIDO2 WebAuthn keys",
                "Corporate-Policy",
            )
            .overlay_parameter("ac-1", "ac-1_prm_1", "quarterly");

        let doc = builder.build(None).expect("build should succeed");
        let root = doc.root_object().unwrap();
        let ctrls = root["controls"].as_array().unwrap();

        // Check inherited + added
        assert!(ctrls.iter().any(|c| c["id"] == "ac-1"));
        assert!(ctrls.iter().any(|c| c["id"] == "corp-sec-01"));

        // Check overlaid parameter
        let ac1 = ctrls.iter().find(|c| c["id"] == "ac-1").unwrap();
        let params = ac1["params"].as_array().unwrap();
        assert_eq!(params[0]["values"][0].as_str().unwrap(), "quarterly");
    }
}
