use crate::{
    document::CasStore,
    error::{AppError, Result},
    mcp::{execute_tool, tools_list},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const UTCP_SPEC_VERSION: &str = "1.0.1";
pub const UTCP_MANUAL_VERSION: &str = "1.0.0";
pub const UTCP_LEGACY_VERSION: &str = "1.0";

/// Supported UTCP transport / call-template types aligned with
/// `utcp-specification`, `rs-utcp`, `typescript-utcp`, `go-utcp`, `python-utcp`, and `utcp-mcp`.
/// Note: WebSockets are intentionally excluded per CKODEX §10 (Explicit Transport).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UtcpTransportType {
    Cli,
    Http,
    Grpc,
    Mcp,
    Sse,
    Text,
}

impl UtcpTransportType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Http => "http",
            Self::Grpc => "grpc",
            Self::Mcp => "mcp",
            Self::Sse => "sse",
            Self::Text => "text",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cli" => Some(Self::Cli),
            "http" | "https" | "rest" | "openapi" => Some(Self::Http),
            "grpc" => Some(Self::Grpc),
            "mcp" | "utcp-mcp" => Some(Self::Mcp),
            "sse" => Some(Self::Sse),
            "text" | "file" => Some(Self::Text),
            _ => None,
        }
    }
}

/// Official UTCP SDK targets across the `universal-tool-calling-protocol` organization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UtcpSdkLanguage {
    Rust,
    TypeScript,
    Go,
    Python,
    Mcp,
}

impl UtcpSdkLanguage {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" | "rs-utcp" => Some(Self::Rust),
            "typescript" | "ts" | "js" | "typescript-utcp" => Some(Self::TypeScript),
            "go" | "golang" | "go-utcp" => Some(Self::Go),
            "python" | "py" | "python-utcp" => Some(Self::Python),
            "mcp" | "utcp-mcp" => Some(Self::Mcp),
            _ => None,
        }
    }

    pub fn repo_url(self) -> &'static str {
        match self {
            Self::Rust => "https://github.com/universal-tool-calling-protocol/rs-utcp",
            Self::TypeScript => {
                "https://github.com/universal-tool-calling-protocol/typescript-utcp"
            }
            Self::Go => "https://github.com/universal-tool-calling-protocol/go-utcp",
            Self::Python => "https://github.com/universal-tool-calling-protocol/python-utcp",
            Self::Mcp => "https://github.com/universal-tool-calling-protocol/utcp-mcp",
        }
    }
}

/// UTCP v1.0.1 `tool_call_template` definition (`utcp-specification`, `rs-utcp`,
/// `typescript-utcp`, `go-utcp`, `python-utcp`, and `utcp-mcp`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpCallTemplate {
    pub call_template_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_communication_protocols: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_ssl: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grpc_service: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grpc_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_server: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
}

/// UTCP v1.0 `tool_provider` definition for cross-version SDK compatibility
/// (`python-utcp`, `go-utcp`, `rs-utcp`, `typescript-utcp`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpToolProvider {
    pub provider_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_tool_name: Option<String>,
}

/// A single UTCP tool descriptor with input/output JSON Schemas and native call bindings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpTool {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub read_only: bool,
    pub inputs: Value,
    pub outputs: Value,
    pub tool_call_template: UtcpCallTemplate,
    pub tool_provider: UtcpToolProvider,
}

/// Complete UTCP Manual document (`utcp-specification` v1.0 / v1.0.1).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpManual {
    pub manual_version: String,
    pub utcp_version: String,
    pub version: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub info: Value,
    #[serde(default)]
    pub allowed_communication_protocols: Vec<String>,
    pub homepage: String,
    pub specification_url: String,
    pub supported_repositories: Vec<String>,
    pub tools: Vec<UtcpTool>,
    pub manual_digest: String,
}

/// Validation report for a UTCP Manual.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpValidationReport {
    pub is_valid: bool,
    pub utcp_version: String,
    pub manual_version: String,
    pub tools_count: usize,
    pub digest_verified: bool,
    pub errors: Vec<String>,
}

/// Proof-carrying execution receipt emitted after a UTCP tool invocation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpCallReceipt {
    pub receipt_id: String,
    pub tool_name: String,
    pub transport: String,
    pub timestamp: String,
    pub input_digest: String,
    pub output_digest: String,
    pub status: String,
    pub output: Value,
}

/// Unified `.utcp_config.json` configuration compatible with `rs-utcp`,
/// `typescript-utcp`, `go-utcp`, `python-utcp`, and `utcp-mcp`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtcpConfig {
    pub utcp_version: String,
    pub manual_version: String,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(default)]
    pub manual_call_templates: Vec<Value>,
    pub providers: Vec<Value>,
    pub manuals: Vec<Value>,
    #[serde(default)]
    pub tool_repository: Value,
    #[serde(default)]
    pub tool_search_strategy: Value,
    pub mcp_servers: BTreeMap<String, Value>,
    pub sdk_endpoints: BTreeMap<String, String>,
}

impl UtcpManual {
    /// Build the canonical Mizan UTCP Manual for the requested primary transport.
    pub fn build_mizan_manual(transport: UtcpTransportType, base_url: &str) -> Self {
        let raw_mcp = tools_list();
        let mcp_tools = raw_mcp
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let normalized_base = base_url.trim_end_matches('/');
        let mut tools = Vec::with_capacity(mcp_tools.len());

        for t in mcp_tools {
            let name = t
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let description = t
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let inputs = t
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
            let read_only = description.contains("(read-only)");

            let mut tags = vec!["oscal".to_string(), "compliance".to_string()];
            if read_only {
                tags.push("read-only".to_string());
            } else {
                tags.push("governed-mutation".to_string());
            }
            if name.contains("fedramp") {
                tags.push("fedramp".to_string());
            }
            if name.contains("slsa") {
                tags.push("slsa".to_string());
            }
            if name.contains("fabric") || name.contains("spiffe") {
                tags.push("zero-trust-fabric".to_string());
            }
            if name.contains("utcp") {
                tags.push("utcp".to_string());
            }

            let (template, provider) = build_transport_bindings(transport, &name, normalized_base);

            let outputs = json!({
                "type": "object",
                "properties": {
                    "receipt_id": { "type": "string", "description": "SHA-256 execution receipt identifier" },
                    "tool_name": { "type": "string" },
                    "status": { "type": "string", "enum": ["VERIFIED", "OK", "ERROR"] },
                    "output": { "type": ["object", "string", "array"] }
                },
                "required": ["tool_name", "status", "output"]
            });

            tools.push(UtcpTool {
                name,
                description,
                tags,
                read_only,
                inputs,
                outputs,
                tool_call_template: template,
                tool_provider: provider,
            });
        }

        let desc = "Mizan High-Assurance OSCAL v1.2.3 Compliance Kernel & Workbench — Universal Tool Calling Protocol (UTCP) Manual".to_string();
        let mut manual = Self {
            manual_version: UTCP_MANUAL_VERSION.to_string(),
            utcp_version: UTCP_SPEC_VERSION.to_string(),
            version: UTCP_LEGACY_VERSION.to_string(),
            name: "mizan-oscal".to_string(),
            description: desc.clone(),
            info: json!({
                "title": "mizan-oscal",
                "version": env!("CARGO_PKG_VERSION"),
                "description": desc
            }),
            allowed_communication_protocols: vec![
                "cli".to_string(),
                "http".to_string(),
                "grpc".to_string(),
                "mcp".to_string(),
                "sse".to_string(),
                "text".to_string(),
            ],
            homepage: "https://github.com/universal-tool-calling-protocol".to_string(),
            specification_url:
                "https://github.com/universal-tool-calling-protocol/utcp-specification".to_string(),
            supported_repositories: vec![
                "https://github.com/universal-tool-calling-protocol".to_string(),
                "https://github.com/universal-tool-calling-protocol/utcp-specification".to_string(),
                "https://github.com/universal-tool-calling-protocol/utcp-mcp".to_string(),
                "https://github.com/universal-tool-calling-protocol/rs-utcp".to_string(),
                "https://github.com/universal-tool-calling-protocol/typescript-utcp".to_string(),
                "https://github.com/universal-tool-calling-protocol/go-utcp".to_string(),
                "https://github.com/universal-tool-calling-protocol/python-utcp".to_string(),
            ],
            tools,
            manual_digest: String::new(),
        };
        manual.manual_digest = manual.compute_digest();
        manual
    }

    /// Compute a canonical SHA-256 digest over the manual tools and versions.
    pub fn compute_digest(&self) -> String {
        let canonical = json!({
            "manual_version": self.manual_version,
            "utcp_version": self.utcp_version,
            "version": self.version,
            "name": self.name,
            "tools": self.tools,
        });
        let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
        CasStore::compute_digest(&bytes)
    }

    /// Look up a single UTCP tool descriptor by exact or suffix tool name.
    pub fn tool_info(&self, name: &str) -> Option<&UtcpTool> {
        let target = name.trim();
        let stripped = target.rsplit('.').next().unwrap_or(target);
        self.tools
            .iter()
            .find(|t| t.name == target || t.name == stripped)
    }

    /// Search UTCP tools using the `tag_and_description_word_match` strategy aligned with
    /// `utcp-mcp`, `rs-utcp`, `typescript-utcp`, `go-utcp`, and `python-utcp`.
    pub fn search_tools(&self, query: &str, limit: usize) -> Vec<UtcpTool> {
        if limit == 0 {
            return Vec::new();
        }
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.tools.iter().take(limit).cloned().collect();
        }
        let words: Vec<&str> = q
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
            .filter(|w| !w.is_empty())
            .collect();

        let mut scored: Vec<(usize, &UtcpTool)> = self
            .tools
            .iter()
            .filter_map(|tool| {
                let name_lower = tool.name.to_ascii_lowercase();
                let desc_lower = tool.description.to_ascii_lowercase();
                let tags_lower: Vec<String> =
                    tool.tags.iter().map(|t| t.to_ascii_lowercase()).collect();

                let mut score = 0usize;
                if name_lower == q {
                    score += 100;
                } else if name_lower.contains(&q) {
                    score += 50;
                }
                if tags_lower.iter().any(|t| t == &q || t.contains(&q)) {
                    score += 30;
                }
                if desc_lower.contains(&q) {
                    score += 20;
                }

                for &w in &words {
                    if name_lower.contains(w) {
                        score += 15;
                    }
                    if tags_lower.iter().any(|t| t.contains(w)) {
                        score += 10;
                    }
                    if desc_lower.contains(w) {
                        score += 5;
                    }
                }

                if score > 0 { Some((score, tool)) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, t)| t.clone())
            .collect()
    }

    /// Validate a UTCP manual JSON document against `utcp-specification` and CKODEX invariants.
    pub fn validate_json(val: &Value) -> UtcpValidationReport {
        Self::validate_json_with_options(val, false)
    }

    /// Validate a UTCP manual JSON document with optional strict schema checks (`--strict`).
    pub fn validate_json_with_options(val: &Value, strict: bool) -> UtcpValidationReport {
        let mut errors = Vec::new();

        let utcp_version = val
            .get("utcp_version")
            .or_else(|| val.get("version"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if utcp_version.is_empty() {
            errors.push("missing required `utcp_version` or `version` field".to_string());
        }

        let manual_version = val
            .get("manual_version")
            .or_else(|| val.get("version"))
            .and_then(Value::as_str)
            .unwrap_or("1.0.0")
            .to_string();

        if strict {
            match val.get("info").and_then(Value::as_object) {
                Some(info_obj) if !info_obj.is_empty() => {
                    if info_obj
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .is_empty()
                    {
                        errors.push("strict mode requires non-empty `info.title`".to_string());
                    }
                    if info_obj
                        .get("version")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .is_empty()
                    {
                        errors.push("strict mode requires non-empty `info.version`".to_string());
                    }
                }
                _ => {
                    errors
                        .push("strict mode requires non-empty `info` metadata object".to_string());
                }
            }

            match val
                .get("allowed_communication_protocols")
                .and_then(Value::as_array)
            {
                Some(protos) if !protos.is_empty() => {
                    for p in protos {
                        let p_str = p.as_str().unwrap_or("");
                        let lower = p_str.trim().to_ascii_lowercase();
                        if lower == "websocket" || lower == "ws" || lower == "wss" {
                            errors.push(format!(
                                "`allowed_communication_protocols` contains forbidden transport `{p_str}`"
                            ));
                        } else if UtcpTransportType::from_str_name(&lower).is_none() {
                            errors.push(format!(
                                "`allowed_communication_protocols` contains unsupported transport `{p_str}`"
                            ));
                        }
                    }
                }
                _ => {
                    errors.push(
                        "strict mode requires non-empty `allowed_communication_protocols` array"
                            .to_string(),
                    );
                }
            }
        }

        let tools_arr = match val.get("tools").and_then(Value::as_array) {
            Some(arr) => arr,
            None => {
                errors.push("missing or non-array `tools` field".to_string());
                return UtcpValidationReport {
                    is_valid: false,
                    utcp_version,
                    manual_version,
                    tools_count: 0,
                    digest_verified: false,
                    errors,
                };
            }
        };

        if tools_arr.is_empty() {
            errors.push("`tools` array must contain at least one tool definition".to_string());
        }

        for (idx, tool) in tools_arr.iter().enumerate() {
            let name = tool.get("name").and_then(Value::as_str).unwrap_or("");
            if name.trim().is_empty() {
                errors.push(format!("tools[{idx}] has empty or missing `name`"));
            }
            let desc = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            if desc.trim().is_empty() {
                errors.push(format!("tools[{idx}] ({name}) has empty `description`"));
            }
            if tool.get("inputs").is_none() && tool.get("inputSchema").is_none() {
                errors.push(format!("tools[{idx}] ({name}) is missing `inputs` schema"));
            }

            if strict {
                match tool.get("tags").and_then(Value::as_array) {
                    Some(tags) if !tags.is_empty() => {}
                    _ => {
                        errors.push(format!(
                            "tools[{idx}] ({name}) is missing non-empty `tags` array in strict mode"
                        ));
                    }
                }
                match tool.get("outputs").and_then(Value::as_object) {
                    Some(outputs) if !outputs.is_empty() => {}
                    _ => {
                        errors.push(format!(
                            "tools[{idx}] ({name}) is missing non-empty `outputs` schema in strict mode"
                        ));
                    }
                }
            }

            let template_type = tool
                .get("tool_call_template")
                .and_then(|t| t.get("call_template_type"))
                .and_then(Value::as_str);
            let provider_type = tool
                .get("tool_provider")
                .and_then(|p| p.get("provider_type"))
                .and_then(Value::as_str);

            if template_type.is_none() && provider_type.is_none() {
                errors.push(format!(
                    "tools[{idx}] ({name}) requires `tool_call_template.call_template_type` or `tool_provider.provider_type`"
                ));
            }

            for t_str in [template_type, provider_type].into_iter().flatten() {
                let lower = t_str.trim().to_ascii_lowercase();
                if lower == "websocket" || lower == "ws" || lower == "wss" {
                    errors.push(format!(
                        "tools[{idx}] ({name}) uses forbidden transport `{t_str}` (WebSockets are prohibited by CKODEX §10; use `http`, `grpc`, `cli`, `mcp`, or `sse`)"
                    ));
                } else if UtcpTransportType::from_str_name(&lower).is_none() {
                    errors.push(format!(
                        "tools[{idx}] ({name}) specifies unsupported UTCP transport `{t_str}`"
                    ));
                }
            }
        }

        let mut digest_verified = true;
        if let Some(expected_digest) = val.get("manual_digest").and_then(Value::as_str)
            && !expected_digest.is_empty()
        {
            if let Ok(parsed) = serde_json::from_value::<UtcpManual>(val.clone()) {
                let actual = parsed.compute_digest();
                if actual != expected_digest {
                    digest_verified = false;
                    errors.push(format!(
                        "manual_digest mismatch: expected {expected_digest}, computed {actual}"
                    ));
                }
            } else {
                digest_verified = false;
                errors.push("failed to deserialize UtcpManual for digest verification".to_string());
            }
        }

        UtcpValidationReport {
            is_valid: errors.is_empty() && digest_verified,
            utcp_version,
            manual_version,
            tools_count: tools_arr.len(),
            digest_verified,
            errors,
        }
    }
}

fn build_transport_bindings(
    transport: UtcpTransportType,
    tool_name: &str,
    base_url: &str,
) -> (UtcpCallTemplate, UtcpToolProvider) {
    let all_protocols = vec![
        "cli".to_string(),
        "http".to_string(),
        "grpc".to_string(),
        "mcp".to_string(),
        "sse".to_string(),
        "text".to_string(),
    ];
    match transport {
        UtcpTransportType::Cli => (
            UtcpCallTemplate {
                call_template_type: "cli".to_string(),
                name: Some(format!("mizan_cli_{tool_name}")),
                allowed_communication_protocols: Some(all_protocols),
                command: Some("mizan".to_string()),
                command_name: Some("mizan".to_string()),
                args: Some(vec![
                    "utcp".to_string(),
                    "call".to_string(),
                    tool_name.to_string(),
                    "--args".to_string(),
                    "{inputs_json}".to_string(),
                ]),
                commands: Some(vec![json!({
                    "command": format!("mizan utcp call {tool_name} --args 'UTCP_ARG_inputs_json_UTCP_END'"),
                    "append_to_final_output": true
                })]),
                param_style: Some("json".to_string()),
                output_format: Some("json".to_string()),
                url: None,
                http_method: None,
                content_type: None,
                host: None,
                port: None,
                service_name: None,
                method_name: None,
                use_ssl: None,
                grpc_service: None,
                grpc_method: None,
                mcp_server: None,
                mcp_tool_name: None,
                config: None,
                file_path: None,
                base_path: None,
                headers: BTreeMap::new(),
            },
            UtcpToolProvider {
                provider_type: "cli".to_string(),
                url: None,
                http_method: None,
                command_name: Some(format!("mizan utcp call {tool_name}")),
                grpc_service: None,
                grpc_method: None,
                mcp_tool_name: None,
            },
        ),
        UtcpTransportType::Http | UtcpTransportType::Sse => {
            let mut headers = BTreeMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let endpoint = format!("{base_url}/api/utcp?tool={tool_name}");
            (
                UtcpCallTemplate {
                    call_template_type: transport.as_str().to_string(),
                    name: Some(format!("mizan_{}_{tool_name}", transport.as_str())),
                    allowed_communication_protocols: Some(all_protocols),
                    command: None,
                    command_name: None,
                    args: None,
                    commands: None,
                    param_style: None,
                    output_format: Some("json".to_string()),
                    url: Some(endpoint.clone()),
                    http_method: Some("POST".to_string()),
                    content_type: Some("application/json".to_string()),
                    host: None,
                    port: None,
                    service_name: None,
                    method_name: None,
                    use_ssl: None,
                    grpc_service: None,
                    grpc_method: None,
                    mcp_server: None,
                    mcp_tool_name: None,
                    config: None,
                    file_path: None,
                    base_path: None,
                    headers,
                },
                UtcpToolProvider {
                    provider_type: transport.as_str().to_string(),
                    url: Some(endpoint),
                    http_method: Some("POST".to_string()),
                    command_name: None,
                    grpc_service: None,
                    grpc_method: None,
                    mcp_tool_name: None,
                },
            )
        }
        UtcpTransportType::Grpc => (
            UtcpCallTemplate {
                call_template_type: "grpc".to_string(),
                name: Some(format!("mizan_grpc_{tool_name}")),
                allowed_communication_protocols: Some(all_protocols),
                command: None,
                command_name: None,
                args: None,
                commands: None,
                param_style: None,
                output_format: Some("protobuf".to_string()),
                url: Some(base_url.to_string()),
                http_method: None,
                content_type: None,
                host: Some("127.0.0.1".to_string()),
                port: Some(50051),
                service_name: Some("oscal.services.v1.OscalService".to_string()),
                method_name: Some(tool_name.to_string()),
                use_ssl: Some(false),
                grpc_service: Some("oscal.services.v1.OscalService".to_string()),
                grpc_method: Some(tool_name.to_string()),
                mcp_server: None,
                mcp_tool_name: None,
                config: None,
                file_path: None,
                base_path: None,
                headers: BTreeMap::new(),
            },
            UtcpToolProvider {
                provider_type: "grpc".to_string(),
                url: Some(base_url.to_string()),
                http_method: None,
                command_name: None,
                grpc_service: Some("oscal.services.v1.OscalService".to_string()),
                grpc_method: Some(tool_name.to_string()),
                mcp_tool_name: None,
            },
        ),
        UtcpTransportType::Mcp => (
            UtcpCallTemplate {
                call_template_type: "mcp".to_string(),
                name: Some(format!("mizan_mcp_{tool_name}")),
                allowed_communication_protocols: Some(all_protocols),
                command: Some("mizan".to_string()),
                command_name: Some("mizan".to_string()),
                args: Some(vec!["mcp".to_string()]),
                commands: None,
                param_style: None,
                output_format: Some("json".to_string()),
                url: None,
                http_method: None,
                content_type: None,
                host: None,
                port: None,
                service_name: None,
                method_name: None,
                use_ssl: None,
                grpc_service: None,
                grpc_method: None,
                mcp_server: Some("mizan-mcp".to_string()),
                mcp_tool_name: Some(tool_name.to_string()),
                config: Some(json!({
                    "mcpServers": {
                        "mizan-mcp": {
                            "transport": "stdio",
                            "command": "mizan",
                            "args": ["mcp"]
                        }
                    }
                })),
                file_path: None,
                base_path: None,
                headers: BTreeMap::new(),
            },
            UtcpToolProvider {
                provider_type: "mcp".to_string(),
                url: None,
                http_method: None,
                command_name: Some("mizan mcp".to_string()),
                grpc_service: None,
                grpc_method: None,
                mcp_tool_name: Some(tool_name.to_string()),
            },
        ),
        UtcpTransportType::Text => (
            UtcpCallTemplate {
                call_template_type: "text".to_string(),
                name: Some(format!("mizan_text_{tool_name}")),
                allowed_communication_protocols: Some(all_protocols),
                command: None,
                command_name: None,
                args: None,
                commands: None,
                param_style: None,
                output_format: Some("json".to_string()),
                url: None,
                http_method: None,
                content_type: None,
                host: None,
                port: None,
                service_name: None,
                method_name: None,
                use_ssl: None,
                grpc_service: None,
                grpc_method: None,
                mcp_server: None,
                mcp_tool_name: None,
                config: None,
                file_path: Some("./mizan.tool.json".to_string()),
                base_path: Some(".".to_string()),
                headers: BTreeMap::new(),
            },
            UtcpToolProvider {
                provider_type: "text".to_string(),
                url: Some("./mizan.tool.json".to_string()),
                http_method: None,
                command_name: None,
                grpc_service: None,
                grpc_method: None,
                mcp_tool_name: None,
            },
        ),
    }
}

/// Execute a Mizan tool through the Universal Tool Calling Protocol and emit a signed/digested receipt.
pub fn execute_utcp_tool(tool_name: &str, args: &Value) -> Result<UtcpCallReceipt> {
    let trimmed = tool_name.trim();
    let normalized_name = trimmed.rsplit('.').next().unwrap_or(trimmed);
    let input_bytes = serde_json::to_vec(args)?;
    let input_digest = CasStore::compute_digest(&input_bytes);

    let raw_output = execute_tool(normalized_name, args)
        .map_err(|e| AppError::Configuration(format!("UTCP tool execution failed: {e}")))?;
    let output_digest = CasStore::compute_digest(raw_output.as_bytes());
    let output_val: Value = serde_json::from_str(&raw_output).unwrap_or(Value::String(raw_output));

    let timestamp = chrono::Utc::now().to_rfc3339();
    let receipt_seed = format!("{normalized_name}:{input_digest}:{output_digest}:{timestamp}");
    let receipt_id = CasStore::compute_digest(receipt_seed.as_bytes());

    Ok(UtcpCallReceipt {
        receipt_id,
        tool_name: normalized_name.to_string(),
        transport: "utcp-native".to_string(),
        timestamp,
        input_digest,
        output_digest,
        status: "VERIFIED".to_string(),
        output: output_val,
    })
}

/// Build a unified `.utcp_config.json` document compatible with `rs-utcp`, `typescript-utcp`,
/// `go-utcp`, `python-utcp`, and `utcp-mcp`.
pub fn build_utcp_config(base_url: &str) -> UtcpConfig {
    let normalized_base = base_url.trim_end_matches('/');
    let mut variables = BTreeMap::new();
    variables.insert("MIZAN_BASE_URL".to_string(), normalized_base.to_string());

    let mut mcp_servers = BTreeMap::new();
    mcp_servers.insert(
        "mizan-oscal".to_string(),
        json!({
            "command": "mizan",
            "args": ["mcp"],
            "transport": "stdio"
        }),
    );

    let mut sdk_endpoints = BTreeMap::new();
    sdk_endpoints.insert(
        "rs-utcp".to_string(),
        UtcpSdkLanguage::Rust.repo_url().to_string(),
    );
    sdk_endpoints.insert(
        "typescript-utcp".to_string(),
        UtcpSdkLanguage::TypeScript.repo_url().to_string(),
    );
    sdk_endpoints.insert(
        "go-utcp".to_string(),
        UtcpSdkLanguage::Go.repo_url().to_string(),
    );
    sdk_endpoints.insert(
        "python-utcp".to_string(),
        UtcpSdkLanguage::Python.repo_url().to_string(),
    );
    sdk_endpoints.insert(
        "utcp-mcp".to_string(),
        UtcpSdkLanguage::Mcp.repo_url().to_string(),
    );

    let allowed_protocols = json!(["cli", "http", "grpc", "mcp", "sse", "text"]);
    let manual_call_templates = vec![
        json!({
            "name": "mizan_cli",
            "provider_type": "cli",
            "call_template_type": "cli",
            "allowed_communication_protocols": allowed_protocols,
            "command": "mizan",
            "command_name": "mizan",
            "args": ["utcp", "manual", "--transport", "cli", "--format", "json"],
            "commands": [
                {
                    "command": "mizan utcp manual --transport cli --format json",
                    "append_to_final_output": true
                }
            ]
        }),
        json!({
            "name": "mizan_http",
            "provider_type": "http",
            "call_template_type": "http",
            "allowed_communication_protocols": allowed_protocols,
            "url": format!("{normalized_base}/api/utcp"),
            "http_method": "GET",
            "content_type": "application/json"
        }),
        json!({
            "name": "mizan_mcp_bridge",
            "provider_type": "mcp",
            "call_template_type": "mcp",
            "allowed_communication_protocols": allowed_protocols,
            "command": "mizan",
            "command_name": "mizan",
            "args": ["mcp"],
            "config": {
                "mcpServers": {
                    "mizan-mcp": {
                        "transport": "stdio",
                        "command": "mizan",
                        "args": ["mcp"]
                    }
                }
            }
        }),
    ];

    UtcpConfig {
        utcp_version: UTCP_SPEC_VERSION.to_string(),
        manual_version: UTCP_MANUAL_VERSION.to_string(),
        variables,
        providers: manual_call_templates.clone(),
        manual_call_templates,
        manuals: vec![
            json!({
                "name": "mizan-oscal-http",
                "url": format!("{normalized_base}/api/utcp"),
                "transport": "http"
            }),
            json!({
                "name": "mizan-oscal-cli",
                "command": "mizan utcp manual --transport cli",
                "transport": "cli"
            }),
        ],
        tool_repository: json!({
            "tool_repository_type": "in_memory"
        }),
        tool_search_strategy: json!({
            "tool_search_strategy_type": "tag_and_description_word_match"
        }),
        mcp_servers,
        sdk_endpoints,
    }
}

/// Generate an idiomatic UTCP SDK integration snippet for the specified official UTCP repository.
pub fn generate_sdk_snippet(lang: UtcpSdkLanguage, base_url: &str) -> String {
    let normalized_base = base_url.trim_end_matches('/');
    match lang {
        UtcpSdkLanguage::Rust => format!(
            r#"// Universal Tool Calling Protocol — Rust SDK (`rs-utcp`)
// Repository: https://github.com/universal-tool-calling-protocol/rs-utcp
// Specification: https://github.com/universal-tool-calling-protocol/utcp-specification

use mizan_oscal::utcp::{{UtcpManual, UtcpTransportType, execute_utcp_tool}};
use serde_json::json;

// Note: With external `rs_utcp`:
// let client = rs_utcp::UtcpClient::create(rs_utcp::UtcpClientConfig::from_file(".utcp_config.json")).await?;
// let res = client.call_tool("mizan_http.fabric_status", json!({{}})).await?;

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    // 1. Discover the Mizan UTCP v1.0.1 Manual (CLI or HTTP `{normalized_base}/api/utcp`)
    let manual = UtcpManual::build_mizan_manual(UtcpTransportType::Cli, "{normalized_base}");
    println!("Loaded UTCP manual `{{}}` ({{}} tools, digest={{}})",
        manual.name, manual.tools.len(), manual.manual_digest);

    // 2. Invoke a tool directly via UTCP without middleman proxy overhead
    let receipt = execute_utcp_tool("fabric_status", &json!({{}}))?;
    println!("UTCP Receipt: {{}} -> {{}}", receipt.receipt_id, receipt.output);
    Ok(())
}}
"#
        ),
        UtcpSdkLanguage::TypeScript => format!(
            r#"// Universal Tool Calling Protocol — TypeScript SDK (`typescript-utcp`)
// Repository: https://github.com/universal-tool-calling-protocol/typescript-utcp
// Specification: https://github.com/universal-tool-calling-protocol/utcp-specification

import {{ UtcpClient }} from "@utcp/sdk";

export async function runMizanUtcpClient() {{
  // 1. Register Mizan's native UTCP Manual endpoint (`GET {normalized_base}/api/utcp`)
  // (Supports `manual_call_templates` and legacy `manuals` configuration)
  const client = await UtcpClient.create(process.cwd(), {{
    manual_call_templates: [
      {{
        name: "mizan_oscal",
        call_template_type: "http",
        url: "{normalized_base}/api/utcp",
        http_method: "GET",
      }},
    ],
  }});

  // 2. Call Mizan compliance tools directly over UTCP
  const result = await client.callTool("mizan_oscal.fabric_status", {{}});
  console.log("Mizan UTCP Result:", result);
  return result;
}}
"#
        ),
        UtcpSdkLanguage::Go => format!(
            r#"// Universal Tool Calling Protocol — Go SDK (`go-utcp`)
// Repository: https://github.com/universal-tool-calling-protocol/go-utcp
// Specification: https://github.com/universal-tool-calling-protocol/utcp-specification

package main

import (
	"context"
	"fmt"
	utcp "github.com/universal-tool-calling-protocol/go-utcp"
)

func main() {{
	ctx := context.Background()
	cfg := &utcp.UtcpClientConfig{{
		ProvidersFilePath: ".utcp_config.json",
	}}
	// Compatible with utcp.NewUTCPClient (and alias NewUtcpClient)
	client, err := utcp.NewUTCPClient(ctx, cfg, nil, nil)
	if err != nil {{
		panic(err)
	}}
	res, err := client.CallTool(ctx, "mizan_http.fabric_status", map[string]any{{}})
	if err != nil {{
		panic(err)
	}}
	fmt.Printf("Mizan UTCP Result (%s/api/utcp): %+v\n", "{normalized_base}", res)
}}
"#
        ),
        UtcpSdkLanguage::Python => format!(
            r#"# Universal Tool Calling Protocol — Python SDK (`python-utcp`)
# Repository: https://github.com/universal-tool-calling-protocol/python-utcp
# Specification: https://github.com/universal-tool-calling-protocol/utcp-specification

import asyncio
from utcp.utcp_client import UtcpClient

async def main() -> None:
    client = await UtcpClient.create(
        config={{
            "manual_call_templates": [
                {{
                    "name": "mizan_oscal",
                    "call_template_type": "http",
                    "url": "{normalized_base}/api/utcp",
                    "http_method": "GET",
                }}
            ]
        }}
    )
    result = await client.call_tool("mizan_oscal.fabric_status", {{}})
    print("Mizan UTCP Result:", result)

if __name__ == "__main__":
    asyncio.run(main())
"#
        ),
        UtcpSdkLanguage::Mcp => format!(
            r#"{{
  "$comment": "Universal Tool Calling Protocol — MCP Bridge (`utcp-mcp`) — https://github.com/universal-tool-calling-protocol/utcp-mcp",
  "mcpServers": {{
    "mizan-mcp": {{
      "command": "mizan",
      "args": ["mcp"],
      "env": {{
        "MIZAN_UTCP_MANUAL_URL": "{normalized_base}/api/utcp"
      }}
    }},
    "utcp-mcp-bridge": {{
      "command": "npx",
      "args": ["-y", "@utcp/mcp-bridge"],
      "env": {{
        "UTCP_CONFIG_FILE": ".utcp_config.json",
        "MIZAN_UTCP_MANUAL_URL": "{normalized_base}/api/utcp"
      }}
    }}
  }}
}}
"#
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utcp_manual_generation_and_validation() {
        for transport in [
            UtcpTransportType::Cli,
            UtcpTransportType::Http,
            UtcpTransportType::Grpc,
            UtcpTransportType::Mcp,
            UtcpTransportType::Sse,
            UtcpTransportType::Text,
        ] {
            let manual = UtcpManual::build_mizan_manual(transport, "http://localhost:3000");
            assert_eq!(manual.utcp_version, UTCP_SPEC_VERSION);
            assert_eq!(manual.manual_version, UTCP_MANUAL_VERSION);
            assert_eq!(manual.version, UTCP_LEGACY_VERSION);
            assert!(manual.tools.len() >= 19);
            assert!(manual.manual_digest.starts_with("sha256:"));
            assert_eq!(manual.supported_repositories.len(), 7);
            assert_eq!(manual.info["title"], "mizan-oscal");
            assert_eq!(manual.allowed_communication_protocols.len(), 6);

            // Check Finding 1: outputs enum has no leading space in "VERIFIED"
            let status_enum = manual.tools[0].outputs["properties"]["status"]["enum"]
                .as_array()
                .expect("status enum");
            assert_eq!(
                status_enum,
                &vec![json!("VERIFIED"), json!("OK"), json!("ERROR")]
            );

            // Check Finding 3 transport-specific fields
            let tpl = &manual.tools[0].tool_call_template;
            match transport {
                UtcpTransportType::Cli => {
                    assert_eq!(tpl.command.as_deref(), Some("mizan"));
                    assert!(tpl.commands.is_some());
                }
                UtcpTransportType::Http | UtcpTransportType::Sse => {
                    assert_eq!(tpl.content_type.as_deref(), Some("application/json"));
                    assert!(tpl.url.as_deref().unwrap().contains("?tool="));
                }
                UtcpTransportType::Grpc => {
                    assert_eq!(tpl.host.as_deref(), Some("127.0.0.1"));
                    assert_eq!(tpl.port, Some(50051));
                    assert_eq!(
                        tpl.service_name.as_deref(),
                        Some("oscal.services.v1.OscalService")
                    );
                    assert_eq!(tpl.use_ssl, Some(false));
                }
                UtcpTransportType::Mcp => {
                    assert_eq!(tpl.command.as_deref(), Some("mizan"));
                    assert!(tpl.config.is_some());
                }
                UtcpTransportType::Text => {
                    assert_eq!(tpl.file_path.as_deref(), Some("./mizan.tool.json"));
                    assert_eq!(tpl.base_path.as_deref(), Some("."));
                }
            }

            let val = serde_json::to_value(&manual).expect("serialize manual");
            let report = UtcpManual::validate_json_with_options(&val, true);
            assert!(
                report.is_valid,
                "manual for {:?} must be valid under strict mode: {:?}",
                transport, report.errors
            );
            assert!(report.digest_verified);
        }
    }

    #[test]
    fn test_utcp_validation_rejects_websocket_transport_and_tampered_digest() {
        let manual =
            UtcpManual::build_mizan_manual(UtcpTransportType::Http, "http://localhost:3000");
        let mut val = serde_json::to_value(&manual).unwrap();

        // 1. Forbidden WebSocket transport must be rejected per CKODEX §10
        val["tools"][0]["tool_call_template"]["call_template_type"] = json!("websocket");
        let report_ws = UtcpManual::validate_json(&val);
        assert!(!report_ws.is_valid);
        assert!(
            report_ws
                .errors
                .iter()
                .any(|e| e.contains("forbidden transport"))
        );

        // 2. Tampered digest must fail verification
        let mut val_tampered = serde_json::to_value(&manual).unwrap();
        val_tampered["manual_digest"] =
            json!("sha256:0000000000000000000000000000000000000000000000000000000000000000");
        let report_digest = UtcpManual::validate_json(&val_tampered);
        assert!(!report_digest.is_valid);
        assert!(!report_digest.digest_verified);

        // 3. Strict validation catches missing info / allowed_communication_protocols
        let mut val_non_strict = serde_json::to_value(&manual).unwrap();
        val_non_strict.as_object_mut().unwrap().remove("info");
        val_non_strict
            .as_object_mut()
            .unwrap()
            .remove("allowed_communication_protocols");
        assert!(UtcpManual::validate_json(&val_non_strict).is_valid);
        assert!(!UtcpManual::validate_json_with_options(&val_non_strict, true).is_valid);
    }

    #[test]
    fn test_utcp_direct_tool_execution_emits_verified_receipt() {
        let receipt =
            execute_utcp_tool("fabric_status", &json!({})).expect("execute fabric_status");
        assert_eq!(receipt.tool_name, "fabric_status");
        assert_eq!(receipt.status, "VERIFIED");
        assert!(receipt.receipt_id.starts_with("sha256:"));
        assert!(receipt.input_digest.starts_with("sha256:"));
        assert!(receipt.output_digest.starts_with("sha256:"));
        assert_eq!(receipt.output["status"], "OPERATIONAL");

        let prefixed_receipt = execute_utcp_tool("mizan_oscal.fabric_status", &json!({}))
            .expect("execute provider-prefixed mizan_oscal.fabric_status");
        assert_eq!(prefixed_receipt.tool_name, "fabric_status");
        assert_eq!(prefixed_receipt.status, "VERIFIED");
    }

    #[test]
    fn test_utcp_search_and_tool_info() {
        let manual =
            UtcpManual::build_mizan_manual(UtcpTransportType::Cli, "http://localhost:3000");
        let results = manual.search_tools("fedramp", 5);
        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.name == "validate_fedramp"));

        let info = manual
            .tool_info("mizan_oscal.fabric_status")
            .expect("resolve prefixed tool_info");
        assert_eq!(info.name, "fabric_status");
        assert!(manual.tool_info("nonexistent_tool_xyz").is_none());
    }

    #[test]
    fn test_utcp_config_and_multi_language_sdk_snippets() {
        let cfg = build_utcp_config("http://localhost:3000");
        assert_eq!(cfg.utcp_version, UTCP_SPEC_VERSION);
        assert_eq!(cfg.providers.len(), 3);
        assert_eq!(cfg.manual_call_templates.len(), 3);
        assert_eq!(cfg.tool_repository["tool_repository_type"], "in_memory");
        assert_eq!(
            cfg.tool_search_strategy["tool_search_strategy_type"],
            "tag_and_description_word_match"
        );
        assert!(cfg.sdk_endpoints.contains_key("rs-utcp"));
        assert!(cfg.sdk_endpoints.contains_key("typescript-utcp"));
        assert!(cfg.sdk_endpoints.contains_key("go-utcp"));
        assert!(cfg.sdk_endpoints.contains_key("python-utcp"));
        assert!(cfg.sdk_endpoints.contains_key("utcp-mcp"));

        for (lang, expected_marker) in [
            (UtcpSdkLanguage::Rust, "rs-utcp"),
            (UtcpSdkLanguage::TypeScript, "typescript-utcp"),
            (UtcpSdkLanguage::Go, "go-utcp"),
            (UtcpSdkLanguage::Python, "python-utcp"),
            (UtcpSdkLanguage::Mcp, "utcp-mcp"),
        ] {
            let snippet = generate_sdk_snippet(lang, "http://localhost:3000");
            assert!(snippet.contains(expected_marker));
        }
    }
}
