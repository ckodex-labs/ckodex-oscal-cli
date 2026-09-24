use serde_json::{Map, Value, json};

use crate::{
    document::{catalog::jurisdiction::Jurisdiction, parser::OscalDocument, schema::DocumentKind},
    error::Result,
};

pub struct EmbeddedCatalogProvider;

impl EmbeddedCatalogProvider {
    pub fn get_catalog(jurisdiction: Jurisdiction) -> Result<OscalDocument> {
        let (title, uuid, controls) = match jurisdiction {
            Jurisdiction::UsNist800_53Rev5 => (
                "NIST Special Publication 800-53 Revision 5: Security and Privacy Controls for Information Systems and Organizations",
                "8b788647-767a-4ecb-ba3a-f2b7f719602a",
                vec![
                    json!({
                        "id": "ac-1",
                        "title": "Policy and Procedures",
                        "class": "SP800-53",
                        "params": [{ "id": "ac-1_prm_1", "values": ["at least annually"] }]
                    }),
                    json!({
                        "id": "ac-2",
                        "title": "Account Management",
                        "class": "SP800-53",
                        "params": [{ "id": "ac-2_prm_1", "values": ["automated system account management"] }]
                    }),
                    json!({
                        "id": "ac-6",
                        "title": "Least Privilege",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "au-2",
                        "title": "Event Logging",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "cm-2",
                        "title": "Baseline Configuration",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "cm-7",
                        "title": "Least Functionality",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "ia-2",
                        "title": "Identification and Authentication (Organizational Users)",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "sc-7",
                        "title": "Boundary Protection",
                        "class": "SP800-53"
                    }),
                    json!({
                        "id": "si-4",
                        "title": "Information System Monitoring",
                        "class": "SP800-53"
                    }),
                ],
            ),
            Jurisdiction::CaItsg33Pbmm => (
                "Canadian Centre for Cyber Security ITSG-33: IT Security Risk Management Baseline (PBMM)",
                "ca788647-767a-4ecb-ba3a-f2b7f719602b",
                vec![
                    json!({
                        "id": "itsg-ac-1",
                        "title": "Access Control Management for Protected B",
                        "class": "ITSG-33",
                        "params": [{ "id": "pbmm_auth", "values": ["two-factor multi-zone authentication"] }]
                    }),
                    json!({
                        "id": "itsg-au-1",
                        "title": "Audit and Accountability for GC Protected Cloud",
                        "class": "ITSG-33"
                    }),
                    json!({
                        "id": "itsg-sc-1",
                        "title": "Data Residency and Cryptographic Boundary (Canada Sovereign)",
                        "class": "ITSG-33"
                    }),
                    json!({
                        "id": "itsg-si-1",
                        "title": "Continuous Threat Monitoring (CCCS Sensor Feed)",
                        "class": "ITSG-33"
                    }),
                ],
            ),
            Jurisdiction::EuEucsIso27001 => (
                "European Union Cybersecurity Scheme (EUCS) & ISO/IEC 27001:2022 Controls Baseline",
                "eu788647-767a-4ecb-ba3a-f2b7f719602c",
                vec![
                    json!({
                        "id": "iso-a.5.15",
                        "title": "Access Control",
                        "class": "ISO-27001:2022"
                    }),
                    json!({
                        "id": "iso-a.8.9",
                        "title": "Configuration Management",
                        "class": "ISO-27001:2022"
                    }),
                    json!({
                        "id": "eucs-sec-01",
                        "title": "EU Sovereign Cloud Isolation and Key Custody",
                        "class": "EUCS-High"
                    }),
                    json!({
                        "id": "iso-a.8.28",
                        "title": "Secure Coding and Container Provenance",
                        "class": "ISO-27001:2022"
                    }),
                ],
            ),
            Jurisdiction::EnterpriseCustom => (
                "Enterprise Custom Baseline Catalog",
                "ee788647-767a-4ecb-ba3a-f2b7f719602d",
                vec![json!({
                    "id": "corp-sec-01",
                    "title": "Zero Trust Identity & Microsegmentation",
                    "class": "Enterprise-Core"
                })],
            ),
        };

        let now = chrono::Utc::now().to_rfc3339();

        let mut metadata = Map::new();
        metadata.insert("title".to_string(), json!(title));
        metadata.insert("published".to_string(), json!(now));
        metadata.insert("last-modified".to_string(), json!(now));
        metadata.insert("version".to_string(), json!("1.0.0"));
        metadata.insert("oscal-version".to_string(), json!("1.2.3"));

        let mut root_obj = Map::new();
        root_obj.insert("uuid".to_string(), json!(uuid));
        root_obj.insert("metadata".to_string(), Value::Object(metadata));
        root_obj.insert("controls".to_string(), Value::Array(controls));

        let mut doc_json = Map::new();
        doc_json.insert(
            DocumentKind::Catalog.root_key().to_string(),
            Value::Object(root_obj),
        );

        OscalDocument::from_value(Value::Object(doc_json), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_catalogs_tri_jurisdiction() {
        let us_doc = EmbeddedCatalogProvider::get_catalog(Jurisdiction::UsNist800_53Rev5).unwrap();
        assert_eq!(us_doc.kind, DocumentKind::Catalog);
        let us_root = us_doc.root_object().unwrap();
        assert!(
            us_root["metadata"]["title"]
                .as_str()
                .unwrap()
                .contains("NIST")
        );

        let ca_doc = EmbeddedCatalogProvider::get_catalog(Jurisdiction::CaItsg33Pbmm).unwrap();
        assert_eq!(ca_doc.kind, DocumentKind::Catalog);
        let ca_root = ca_doc.root_object().unwrap();
        assert!(
            ca_root["metadata"]["title"]
                .as_str()
                .unwrap()
                .contains("ITSG-33")
        );

        let eu_doc = EmbeddedCatalogProvider::get_catalog(Jurisdiction::EuEucsIso27001).unwrap();
        assert_eq!(eu_doc.kind, DocumentKind::Catalog);
        let eu_root = eu_doc.root_object().unwrap();
        assert!(
            eu_root["metadata"]["title"]
                .as_str()
                .unwrap()
                .contains("ISO/IEC 27001")
        );
    }
}
