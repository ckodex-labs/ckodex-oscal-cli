use crate::error::{AppError, Result};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

mod protocol;
mod tools;

pub use protocol::{McpError, error_response, success_response, tools_list};
pub use tools::execute_tool;

pub const MAX_MCP_FRAME_BYTES: usize = 16 * 1024 * 1024;

enum FrameRead {
    Eof,
    Line(String),
    Oversized,
    InvalidUtf8(String),
}

fn read_bounded_frame<R: BufRead>(reader: &mut R, max_bytes: usize) -> io::Result<FrameRead> {
    let mut buf = Vec::new();
    let mut oversized = false;

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if buf.is_empty() && !oversized {
                return Ok(FrameRead::Eof);
            }
            break;
        }

        if let Some(newline_pos) = available.iter().position(|&b| b == b'\n') {
            let chunk = &available[..=newline_pos];
            if !oversized {
                if buf.len().saturating_add(chunk.len()) > max_bytes {
                    oversized = true;
                    buf.clear();
                } else {
                    buf.extend_from_slice(chunk);
                }
            }
            reader.consume(newline_pos + 1);
            break;
        } else {
            let len = available.len();
            if !oversized {
                if buf.len().saturating_add(len) > max_bytes {
                    oversized = true;
                    buf.clear();
                } else {
                    buf.extend_from_slice(available);
                }
            }
            reader.consume(len);
        }
    }

    if oversized {
        return Ok(FrameRead::Oversized);
    }

    match String::from_utf8(buf) {
        Ok(line) => Ok(FrameRead::Line(line)),
        Err(e) => Ok(FrameRead::InvalidUtf8(e.to_string())),
    }
}

pub fn run_mcp_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut reader = stdin.lock();
    serve_mcp_stream(&mut reader, &mut stdout, MAX_MCP_FRAME_BYTES)
}

fn serve_mcp_stream<R: BufRead, W: Write>(
    reader: &mut R,
    stdout: &mut W,
    max_frame_bytes: usize,
) -> Result<()> {
    loop {
        match read_bounded_frame(reader, max_frame_bytes).map_err(|e| AppError::Io {
            path: "stdin".into(),
            source: e,
        })? {
            FrameRead::Eof => break,
            FrameRead::Oversized => {
                let err_resp = error_response(
                    None,
                    -32600,
                    "Invalid Request: frame exceeds 16 MiB limit".to_string(),
                );
                write_response(stdout, &err_resp)?;
            }
            FrameRead::InvalidUtf8(msg) => {
                let err_resp =
                    error_response(None, -32700, format!("Parse error: invalid UTF-8 ({msg})"));
                write_response(stdout, &err_resp)?;
            }
            FrameRead::Line(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    match serde_json::from_str::<Value>(trimmed) {
                        Ok(req) => {
                            let id = req.get("id").unwrap_or(&Value::Null);
                            match handle_mcp_request(&req) {
                                Ok(Some(resp)) => write_response(stdout, &resp)?,
                                Ok(None) => {}
                                Err(e) => {
                                    let err_resp =
                                        error_response(Some(id), e.code(), e.to_string());
                                    write_response(stdout, &err_resp)?;
                                }
                            }
                        }
                        Err(e) => {
                            let err_resp = error_response(None, -32700, e.to_string());
                            write_response(stdout, &err_resp)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_response<W: Write>(stdout: &mut W, resp: &Value) -> Result<()> {
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
                "capabilities": {
                    "tools": {},
                    "utcp": {
                        "utcpVersion": crate::utcp::UTCP_SPEC_VERSION,
                        "manualVersion": crate::utcp::UTCP_MANUAL_VERSION,
                        "bridge": "utcp-mcp"
                    }
                },
                "serverInfo": { "name": "mizan-mcp", "version": env!("CARGO_PKG_VERSION") }
            });
            Ok(Some(success_response(id, result)))
        }
        "notifications/initialized" => Ok(None),
        "tools/list" => Ok(Some(success_response(id, protocol::tools_list()))),
        "utcp/manual" => {
            let params = req.get("params").unwrap_or(&Value::Null);
            let transport_str = params
                .get("transport")
                .and_then(Value::as_str)
                .unwrap_or("mcp");
            let base_url = params
                .get("base_url")
                .and_then(Value::as_str)
                .unwrap_or("http://localhost:3000");
            let transport = crate::utcp::UtcpTransportType::from_str_name(transport_str)
                .unwrap_or(crate::utcp::UtcpTransportType::Mcp);
            let manual = crate::utcp::UtcpManual::build_mizan_manual(transport, base_url);
            let manual_val = serde_json::to_value(manual)?;
            Ok(Some(success_response(id, manual_val)))
        }
        "utcp/call" => {
            let params = req.get("params").unwrap_or(&Value::Null);
            let name = params.get("name").and_then(Value::as_str).ok_or_else(|| {
                McpError::InvalidRequest("utcp/call requires params.name".to_string())
            })?;
            let args = params
                .get("arguments")
                .or_else(|| params.get("inputs"))
                .unwrap_or(&Value::Null);
            match crate::utcp::execute_utcp_tool(name, args) {
                Ok(receipt) => {
                    let receipt_val = serde_json::to_value(receipt)?;
                    Ok(Some(success_response(id, receipt_val)))
                }
                Err(e) => Ok(Some(error_response(Some(id), -32000, e.to_string()))),
            }
        }
        "utcp/search" => {
            let params = req.get("params").unwrap_or(&Value::Null);
            let query = params
                .get("query")
                .or_else(|| params.get("q"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let limit = params
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10)
                .clamp(1, 100) as usize;
            let transport_str = params
                .get("transport")
                .and_then(Value::as_str)
                .unwrap_or("mcp");
            let base_url = params
                .get("base_url")
                .and_then(Value::as_str)
                .unwrap_or("http://localhost:3000");
            let transport = crate::utcp::UtcpTransportType::from_str_name(transport_str)
                .unwrap_or(crate::utcp::UtcpTransportType::Mcp);
            let manual = crate::utcp::UtcpManual::build_mizan_manual(transport, base_url);
            let tools = manual.search_tools(query, limit);
            let res = json!({
                "query": query,
                "count": tools.len(),
                "tools": tools
            });
            Ok(Some(success_response(id, res)))
        }
        "utcp/tool_info" => {
            let params = req.get("params").unwrap_or(&Value::Null);
            let name = params
                .get("name")
                .or_else(|| params.get("tool"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    McpError::InvalidRequest("utcp/tool_info requires params.name".to_string())
                })?;
            let transport_str = params
                .get("transport")
                .and_then(Value::as_str)
                .unwrap_or("mcp");
            let base_url = params
                .get("base_url")
                .and_then(Value::as_str)
                .unwrap_or("http://localhost:3000");
            let transport = crate::utcp::UtcpTransportType::from_str_name(transport_str)
                .unwrap_or(crate::utcp::UtcpTransportType::Mcp);
            let manual = crate::utcp::UtcpManual::build_mizan_manual(transport, base_url);
            match manual.tool_info(name) {
                Some(tool) => {
                    let tool_val = serde_json::to_value(tool)?;
                    Ok(Some(success_response(id, tool_val)))
                }
                None => Ok(Some(error_response(
                    Some(id),
                    -32602,
                    format!("UTCP tool not found: {name}"),
                ))),
            }
        }
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
        assert!(tool_names.contains(&"ingest_security_report_to_poam"));
        assert!(tool_names.contains(&"export_utcp_manual"));
        assert!(tool_names.contains(&"search_utcp_tools"));

        let utcp_manual_req = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "utcp/manual",
            "params": { "transport": "http" }
        });
        let utcp_manual_resp = handle_mcp_request(&utcp_manual_req)
            .unwrap()
            .expect("should handle utcp/manual");
        assert_eq!(utcp_manual_resp["result"]["utcp_version"], "1.0.1");
        assert!(
            utcp_manual_resp["result"]["tools"]
                .as_array()
                .unwrap()
                .len()
                >= 20
        );

        let utcp_call_req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "utcp/call",
            "params": { "name": "fabric_status", "inputs": {} }
        });
        let utcp_call_resp = handle_mcp_request(&utcp_call_req)
            .unwrap()
            .expect("should handle utcp/call");
        assert_eq!(utcp_call_resp["result"]["status"], "VERIFIED");
        assert_eq!(utcp_call_resp["result"]["output"]["status"], "OPERATIONAL");

        let utcp_search_req = json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "utcp/search",
            "params": { "query": "fedramp", "limit": 5 }
        });
        let utcp_search_resp = handle_mcp_request(&utcp_search_req)
            .unwrap()
            .expect("should handle utcp/search");
        assert!(utcp_search_resp["result"]["count"].as_u64().unwrap() >= 1);

        let utcp_info_req = json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "utcp/tool_info",
            "params": { "name": "fabric_status" }
        });
        let utcp_info_resp = handle_mcp_request(&utcp_info_req)
            .unwrap()
            .expect("should handle utcp/tool_info");
        assert_eq!(utcp_info_resp["result"]["name"], "fabric_status");
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

    #[test]
    fn test_mcp_rejects_oversized_frame_and_continues_serving() {
        // Test with a small synthetic frame cap (256 bytes) AND verify MAX_MCP_FRAME_BYTES is 16 MiB
        assert_eq!(MAX_MCP_FRAME_BYTES, 16 * 1024 * 1024);

        let mut input = Vec::new();
        // Frame 1: oversized (1024 bytes > 256 byte cap)
        input.extend_from_slice(&vec![b'a'; 1024]);
        input.push(b'\n');
        // Frame 2: valid initialize request after draining the oversized frame
        input.extend_from_slice(br#"{"jsonrpc":"2.0","id":99,"method":"initialize","params":{}}"#);
        input.push(b'\n');

        let mut reader = io::BufReader::with_capacity(64, input.as_slice());
        let mut output = Vec::new();
        serve_mcp_stream(&mut reader, &mut output, 256).expect("stream processing must succeed");

        let lines: Vec<&str> = std::str::from_utf8(&output).unwrap().lines().collect();
        assert_eq!(lines.len(), 2);

        let err_resp: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(err_resp["error"]["code"], -32600);
        assert!(
            err_resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("frame exceeds 16 MiB limit")
        );

        let ok_resp: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(ok_resp["id"], 99);
        assert_eq!(ok_resp["result"]["serverInfo"]["name"], "mizan-mcp");
    }
}
