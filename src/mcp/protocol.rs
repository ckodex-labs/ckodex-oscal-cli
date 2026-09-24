use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Tool error ({tool}): {message}")]
    Tool { tool: String, message: String },
    #[error("Path escape: {0}")]
    PathEscape(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl McpError {
    pub fn code(&self) -> i64 {
        match self {
            McpError::Parse(_) | McpError::Serialization(_) => -32700,
            McpError::InvalidRequest(_)
            | McpError::InvalidArgument(_)
            | McpError::MissingArgument(_) => -32600,
            McpError::MethodNotFound(_) => -32601,
            McpError::Tool { .. } => -32000,
            McpError::PathEscape(_) => -32001,
            McpError::Io(_) => -32002,
        }
    }
}

impl From<crate::error::AppError> for McpError {
    fn from(e: crate::error::AppError) -> Self {
        match e {
            crate::error::AppError::Serialization(s) => McpError::Serialization(s),
            crate::error::AppError::Io { source, .. } => McpError::Io(source),
            _ => McpError::Tool {
                tool: "unknown".to_string(),
                message: e.to_string(),
            },
        }
    }
}

pub fn error_response(id: Option<&Value>, code: i64, message: impl Into<String>) -> Value {
    let id = id.cloned().unwrap_or(Value::Null);
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.into() }
    })
}

pub fn success_response(id: &Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

pub fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "query_control",
                "description": "Query an OSCAL catalog for control text, parameters, and guidance (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "catalog_file": { "type": "string", "description": "Path to catalog JSON/YAML" },
                        "control_id": { "type": "string", "description": "Control identifier (e.g. ac-1, sc-7)" }
                    },
                    "required": ["catalog_file", "control_id"]
                }
            },
            {
                "name": "inspect_document",
                "description": "Inspect an OSCAL document summary, kinds, control counts, and metadata (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "OSCAL document file path" }
                    },
                    "required": ["file"]
                }
            },
            {
                "name": "compute_blast_radius",
                "description": "Calculate the downstream compliance blast radius for a control or component (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "Primary document path" },
                        "target": { "type": "string", "description": "Target ID (e.g. ac-1)" }
                    },
                    "required": ["file", "target"]
                }
            },
            {
                "name": "validate_fedramp",
                "description": "Run high-assurance FedRAMP PMO baseline validation on an SSP (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "ssp_file": { "type": "string", "description": "Path to SSP document" },
                        "baseline": { "type": "string", "enum": ["low", "moderate", "high"], "default": "moderate" }
                    },
                    "required": ["ssp_file"]
                }
            },
            {
                "name": "audit_kubernetes_cluster",
                "description": "Audit live Kubernetes cluster or workload against NIST 800-53 controls and optionally write OSCAL assessment results (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace to scan" },
                        "output_file": { "type": "string", "description": "Optional output path for OSCAL assessment-results.json" }
                    }
                }
            },
            {
                "name": "evaluate_rego_policy",
                "description": "Evaluate an input JSON artifact against compiled OSCAL Rego rules using Microsoft Regorus (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "policy_file": { "type": "string", "description": "Path to Rego policy file or directory" },
                        "input_data": { "type": "object", "description": "JSON payload to evaluate" },
                        "query_package": { "type": "string", "description": "Rego query package (e.g. oscal.ac_2 or oscal.k8s)" }
                    },
                    "required": ["policy_file", "input_data", "query_package"]
                }
            },
            {
                "name": "sync_3way_merge",
                "description": "Perform 3-way GitOps AST merge across base, upstream, and local documents (transform)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "base_file": { "type": "string", "description": "Path to base ancestor document" },
                        "upstream_file": { "type": "string", "description": "Path to upstream document" },
                        "local_file": { "type": "string", "description": "Path to local modified document" },
                        "strategy": { "type": "string", "enum": ["manual", "ours", "theirs"], "default": "manual" }
                    },
                    "required": ["base_file", "upstream_file", "local_file"]
                }
            },
            {
                "name": "split_oscal_markdown",
                "description": "Split a monolithic OSCAL document into an agile Markdown authoring workspace (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "Path to source OSCAL document" },
                        "output_dir": { "type": "string", "description": "Directory to populate with Markdown chapters" }
                    },
                    "required": ["file", "output_dir"]
                }
            },
            {
                "name": "assemble_oscal_markdown",
                "description": "Assemble an agile Markdown authoring workspace back into a validated OSCAL document (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input_dir": { "type": "string", "description": "Directory containing Markdown chapters" },
                        "output_file": { "type": "string", "description": "Output path for the assembled OSCAL document" }
                    },
                    "required": ["input_dir", "output_file"]
                }
            },
            {
                "name": "export_baseline_catalog",
                "description": "Retrieve an embedded built-in tri-jurisdictional compliance catalog (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "jurisdiction": { "type": "string", "enum": ["us", "ca", "eu", "enterprise"], "default": "us" }
                    },
                    "required": ["jurisdiction"]
                }
            },
            {
                "name": "extend_enterprise_catalog",
                "description": "Build an enterprise custom compliance catalog inheriting base standards with custom controls and parameter overlays (transform)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "base_jurisdiction": { "type": "string", "enum": ["us", "ca", "eu"], "default": "us" },
                        "title": { "type": "string", "description": "Enterprise catalog title" },
                        "custom_control_id": { "type": "string", "description": "Custom control ID (e.g. corp-sec-01)" },
                        "custom_control_title": { "type": "string", "description": "Custom control title" },
                        "custom_control_description": { "type": "string", "description": "Custom control description" }
                    },
                    "required": ["title"]
                }
            },
            {
                "name": "generate_slsa_provenance",
                "description": "Generate a SLSA v1.2 / v1.0 supply chain provenance attestation statement with OSCAL evidence Merkle proof (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "subject_name": { "type": "string", "description": "Artifact name (e.g. container image, binary)" },
                        "subject_digest": { "type": "string", "description": "SHA-256 digest of artifact" },
                        "slsa_version": { "type": "string", "enum": ["v1.2", "v1.0"], "default": "v1.2" },
                        "evidence_file": { "type": "string", "description": "Optional path to evidence bundle JSON" }
                    },
                    "required": ["subject_name", "subject_digest"]
                }
            },
            {
                "name": "verify_slsa_provenance",
                "description": "Cryptographically verify a SLSA v1.2 / v1.0 supply chain provenance statement (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "statement_json": { "type": "string", "description": "JSON string of the in-toto SLSA statement" }
                    },
                    "required": ["statement_json"]
                }
            },
            {
                "name": "query_cas_store",
                "description": "Query local Content-Addressable Storage (CAS) for object contents or stats (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "digest": { "type": "string", "description": "Cryptographic digest of object (sha256:...)" }
                    }
                }
            },
            {
                "name": "export_sarif",
                "description": "Convert an OSCAL compliance document into an OASIS SARIF v2.1.0 report, optionally saving to disk (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "Path to source OSCAL document" },
                        "output_file": { "type": "string", "description": "Optional output path to save the SARIF report" }
                    },
                    "required": ["file"]
                }
            },
            {
                "name": "export_gitlab",
                "description": "Convert an OSCAL compliance document into GitLab Security Scanner Report v15.0.0, optionally saving to disk (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "Path to source OSCAL document" },
                        "output_file": { "type": "string", "description": "Optional output path to save the GitLab report" }
                    },
                    "required": ["file"]
                }
            },
            {
                "name": "import_sbom_cyclonedx",
                "description": "Convert CycloneDX v1.5/v1.6 or SPDX SBOM JSON into an OSCAL component-definition, optionally saving to disk (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file": { "type": "string", "description": "Path to source SBOM JSON file" },
                        "output_file": { "type": "string", "description": "Optional output path to save the OSCAL component-definition" }
                    },
                    "required": ["file"]
                }
            },
            {
                "name": "evaluate_policy_rulepack",
                "description": "Evaluate verified built-in Rego compliance rules (CIS K8s 5.2.1, 5.2.6, FedRAMP AC-2, CCCS ITSG-33) against workload JSON/YAML (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "rule_id": { "type": "string", "enum": ["cis-k8s-5.2.1", "cis-k8s-5.2.6", "fedramp-ac-2", "itsg33-boundary-isolation"], "description": "Rule ID to evaluate" },
                        "input_data": { "type": "string", "description": "JSON string of Kubernetes/cloud workload resource" }
                    },
                    "required": ["rule_id", "input_data"]
                }
            },
            {
                "name": "execute_compliance_pipeline",
                "description": "Execute full end-to-end compliance pipeline: Ingest SBOM -> Evaluate Policies -> OSCAL Assessment -> SLSA v1.2 Provenance -> SARIF & GitLab Export (write)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "jurisdiction": { "type": "string", "enum": ["us", "ca", "eu", "enterprise"], "default": "us" },
                        "sbom_file": { "type": "string", "description": "Optional path to SBOM JSON" },
                        "workload_file": { "type": "string", "description": "Optional path to workload manifest JSON/YAML" },
                        "output_dir": { "type": "string", "description": "Output directory for pipeline artifacts", "default": "mizan-pipeline-output" },
                        "subject_name": { "type": "string", "description": "Subject artifact name", "default": "production-service" }
                    }
                }
            },
            {
                "name": "fabric_status",
                "description": "Inspect Root Fabric status: active tenants, SPIFFE trust domain, and isolated DataStore partitions (read-only)",
                "inputSchema": { "type": "object", "properties": {} }
            },
            {
                "name": "fabric_validate_spiffe",
                "description": "Validate a SPIFFE Workload Identity ID URI (spiffe://<trust-domain>/<path>) (read-only)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "spiffe_id": { "type": "string", "description": "SPIFFE ID URI string" }
                    },
                    "required": ["spiffe_id"]
                }
            }
        ]
    })
}
