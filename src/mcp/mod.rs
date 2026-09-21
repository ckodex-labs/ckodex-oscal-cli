use crate::error::{AppError, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

mod protocol;
mod tools;

pub use protocol::{error_response, success_response, McpError};
pub use tools::execute_tool;

pub fn run_mcp_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut reader = stdin.lock();
    let mut line = String::new();

    while reader.read_line(&mut line).map_err(|e| AppError::Io {
        path: "stdin".into(),
        source: e,
    })? > 0
    {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            match serde_json::from_str::<Value>(trimmed) {
                Ok(req) => {
                    let id = req.get("id").unwrap_or(&Value::Null);
                    match handle_mcp_request(&req) {
                        Ok(Some(resp)) => write_response(&mut stdout, &resp)?,
                        Ok(None) => {}
                        Err(e) => {
                            let err_resp = error_response(Some(id), e.code(), e.to_string());
                            write_response(&mut stdout, &err_resp)?;
                        }
                    }
                }
                Err(e) => {
                    let err_resp = error_response(None, -32700, e.to_string());
                    write_response(&mut stdout, &err_resp)?;
                }
            }
        }
        line.clear();
    }
    Ok(())
}

fn write_response(stdout: &mut io::StdoutLock, resp: &Value) -> Result<()> {
    let fallback = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal response serialization error"}}"#;
    let resp_str = match serde_json::to_string(resp) {
        Ok(s) => s,
        Err(_) => fallback.to_string(),
    };
    writeln!(stdout, "{resp_str}").map_err(|e| AppError::Io {
        path: "stdout".into(),
        source: e,
    })?;
    stdout.flush().map_err(|e| AppError::Io {
        path: "stdout".into(),
        source: e,
    })?;
    Ok(())
}

fn handle_mcp_request(req: &Value) -> std::result::Result<Option<Value>, McpError> {
    let id = req.get("id").unwrap_or(&Value::Null);
    let Some(method) = req.get("method").and_then(Value::as_str) else {
        return Ok(Some(error_response(
            Some(id),
            -32600,
            "method must be a string",
        )));
    };

    match method {
        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "mizan-mcp", "version": env!("CARGO_PKG_VERSION") }
            });
            Ok(Some(success_response(id, result)))
        }
        "notifications/initialized" => Ok(None),
        "tools/list" => Ok(Some(success_response(id, protocol::tools_list()))),
        "tools/call" => {
            let params = req.get("params").unwrap_or(&Value::Null);
            let name = params.get("name").and_then(Value::as_str).ok_or_else(|| {
                McpError::InvalidRequest("tools/call requires params.name".to_string())
            })?;
            let args = params.get("arguments").unwrap_or(&Value::Null);
            match tools::execute_tool(name, args) {
                Ok(content) => {
                    let result = json!({
                        "content": [{ "type": "text", "text": content }]
                    });
                    Ok(Some(success_response(id, result)))
                }
                Err(e) => Ok(Some(error_response(Some(id), e.code(), e.to_string()))),
            }
        }
        _ => Ok(Some(error_response(
            Some(id),
            -32601,
            format!("Method not found: {method}"),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_initialize_and_tools_list() {
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let init_resp = handle_mcp_request(&init_req)
            .unwrap()
            .expect("should handle initialize");
        assert_eq!(init_resp["result"]["serverInfo"]["name"], "mizan-mcp");

        let list_req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });
        let list_resp = handle_mcp_request(&list_req)
            .unwrap()
            .expect("should handle tools/list");
        let tools = list_resp["result"]["tools"]
            .as_array()
            .expect("tools array");
        assert!(tools.len() >= 12);

        let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(tool_names.contains(&"audit_kubernetes_cluster"));
        assert!(tool_names.contains(&"evaluate_rego_policy"));
        assert!(tool_names.contains(&"validate_fedramp"));
        assert!(tool_names.contains(&"sync_3way_merge"));
        assert!(tool_names.contains(&"export_sarif"));
        assert!(tool_names.contains(&"export_gitlab"));
        assert!(tool_names.contains(&"import_sbom_cyclonedx"));
        assert!(tool_names.contains(&"evaluate_policy_rulepack"));
    }

    #[test]
    fn test_mcp_rejects_missing_method() {
        let req = json!({ "jsonrpc": "2.0", "id": 42 });
        let resp = handle_mcp_request(&req)
            .unwrap()
            .expect("should return error");
        assert_eq!(resp["error"]["code"], -32600);
        assert_eq!(resp["id"], 42);
    }

    #[test]
    fn test_mcp_rejects_unknown_method() {
        let req = json!({ "jsonrpc": "2.0", "id": 7, "method": "nope" });
        let resp = handle_mcp_request(&req)
            .unwrap()
            .expect("should return error");
        assert_eq!(resp["error"]["code"], -32601);
    }
}
