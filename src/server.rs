/// MCP Server — minimal JSON-RPC stdio implementation.

use crate::adapter::RulesMcpServer;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{stdin, stdout, BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>,
}

impl RulesMcpServer {
    /// Run JSON-RPC stdio server loop.
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = stdin();
        let mut reader = stdin.lock();
        let stdout = stdout();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let response = self.handle_request(line).await;
                    let output = serde_json::to_string(&response)?;

                    let mut out = stdout.lock();
                    writeln!(out, "{}", output)?;
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, line: &str) -> JsonRpcResponse {
        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: Some(e.to_string()),
                    }),
                };
            }
        };

        let result = match req.method.as_str() {
            "tools/list" => self.list_tools().await,
            "tools/call" => self.call_tool(&req.params).await,
            "initialize" => self.initialize(&req.params).await,
            _ => Err(anyhow!("Unknown method: {}", req.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(e.to_string()),
                }),
            },
        }
    }

    async fn initialize(&self, _params: &Option<serde_json::Value>) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "serverInfo": {
                "name": "rules-mcp",
                "version": "0.1.0"
            }
        }))
    }

    async fn list_tools(&self) -> Result<serde_json::Value> {
        let tools = vec![
            json_tool("help", "Get started with the RulesMCP server", vec![]),
            json_tool(
                "search_rules",
                "Search rules by keyword",
                vec![
                    ("query", "string", true, "Search terms"),
                    ("category", "string", false, "Filter by category"),
                    ("limit", "integer", false, "Max results (default 10)"),
                ],
            ),
            json_tool(
                "get_rule",
                "Get full markdown content of a rule file",
                vec![("file", "string", true, "Path relative to repo root")],
            ),
            json_tool(
                "get_context",
                "Get combined rules context for given languages",
                vec![
                    ("languages", "array", true, "Language categories"),
                    ("topics", "array", false, "Optional concept filter"),
                ],
            ),
            json_tool(
                "get_learning_path",
                "Get rules in implementation order",
                vec![
                    ("languages", "array", true, "Language categories"),
                    ("phase", "integer", false, "Optional phase number (1-based)"),
                ],
            ),
            json_tool(
                "list_rules",
                "List available rule files",
                vec![("category", "string", false, "Filter by category")],
            ),
            json_tool(
                "get_related",
                "Get related rules by following edges",
                vec![("file", "string", true, "Path relative to repo root")],
            ),
        ];

        Ok(serde_json::json!({
            "tools": tools
        }))
    }

    async fn call_tool(&self, params: &Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = params.as_ref().ok_or_else(|| anyhow!("Missing params"))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;

        let default_args = serde_json::json!({});
        let arguments = params.get("arguments").unwrap_or(&default_args);

        let result = match name {
            "help" => self.help().await,
            "search_rules" => {
                let query = arguments
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let category = arguments.get("category").and_then(|v| v.as_str());
                let limit = arguments
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                self.search_rules(query, category, limit).await
            }
            "get_rule" => {
                let file = arguments
                    .get("file")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file argument"))?;
                self.get_rule(file).await
            }
            "get_context" => {
                let default_arr = vec![];
                let languages: Vec<&str> = arguments
                    .get("languages")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&default_arr)
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect();
                self.get_context(&languages).await
            }
            "get_learning_path" => {
                let default_arr = vec![];
                let languages: Vec<&str> = arguments
                    .get("languages")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&default_arr)
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect();
                let phase = arguments.get("phase").and_then(|v| v.as_u64()).map(|v| v as u32);
                self.get_learning_path(&languages, phase).await
            }
            "list_rules" => {
                let category = arguments.get("category").and_then(|v| v.as_str());
                self.list_rules(category).await
            }
            "get_related" => {
                let file = arguments
                    .get("file")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing file argument"))?;
                self.get_related(file).await
            }
            _ => return Err(anyhow!("Unknown tool: {}", name)),
        };

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": result
            }]
        }))
    }
}

fn json_tool(
    name: &str,
    description: &str,
    input_schema: Vec<(&str, &str, bool, &str)>,
) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = vec![];

    for (key, ty, is_required, desc) in input_schema {
        if is_required {
            required.push(key.to_string());
        }
        properties.insert(
            key.to_string(),
            serde_json::json!({
                "type": ty,
                "description": desc
            }),
        );
    }

    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required
        }
    })
}
