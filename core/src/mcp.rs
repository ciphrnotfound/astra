use std::io::{self, BufRead, Write};
use anyhow::Result;
use serde_json::{json, Value};
use crate::engine::CodexEngine;

/// A simple stdio Model Context Protocol (MCP) server for Astra.
/// This allows AI editors like Cursor to connect to Astra and use its semantic graph and team features.
pub fn run_mcp_server(engine: &mut CodexEngine) -> Result<()> {
    // Write initialization handshake to stderr
    eprintln!("Astra MCP Server initialized. Ready for connections.");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(&line) {
            Ok(request) => {
                let response = handle_mcp_request(engine, request);
                let response_json = serde_json::to_string(&response).unwrap_or_default();
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("Invalid JSON RPC: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_mcp_request(engine: &mut CodexEngine, req: Value) -> Value {
    let _method = req["method"].as_str().unwrap_or("unknown");
    let req_id = req["id"].clone();
    
    // In a full implementation, we would register tools like `query_semantic_graph`
    // or `run_migration_agent` via the 'initialize' MCP lifecycle.
    // For this prototype, we'll implement a basic proxy query.
    
    if _method == "tools/call" {
        if let Some(params) = req.get("params") {
            if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                if name == "ask_astra" {
                    // Auto-index if not yet indexed, only when asked a question
                    if engine.index().stats().file_count == 0 {
                        let _ = engine.handle_input(":index");
                    }

                    // Inject grounding context safely
                    let grounding = engine.build_grounding_context();

                    let raw_prompt = params["arguments"]["prompt"].as_str().unwrap_or("");
                    let prompt = format!("{}\n\nUser Query: {}", grounding, raw_prompt);
                    match engine.handle_input(&prompt) {
                        Ok(res) => return json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "content": [{"type": "text", "text": res}]
                            }
                        }),
                        Err(e) => return json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "error": { "code": -32603, "message": e.to_string() }
                        })
                    }
                } else if name == "get_next_task" {
                    let raw_goal = params.get("arguments")
                        .and_then(|a| a.get("goal"))
                        .and_then(|g| g.as_str());
                    match engine.get_next_task(raw_goal) {
                        Ok(task_json) => return json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": { "content": [{"type": "text", "text": task_json}] }
                        }),
                        Err(e) => return json!({ "jsonrpc": "2.0", "id": req_id, "error": { "code": -32603, "message": e.to_string() } })
                    }
                } else if name == "report_task_result" {
                    let empty = json!({});
                    let args = params.get("arguments").unwrap_or(&empty);
                    let task_id = args.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
                    let success = args.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let details = args.get("details").and_then(|v| v.as_str()).unwrap_or("");
                    match engine.report_task_result(task_id, success, details) {
                        Ok(msg) => return json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": { "content": [{"type": "text", "text": msg}] }
                        }),
                        Err(e) => return json!({ "jsonrpc": "2.0", "id": req_id, "error": { "code": -32603, "message": e.to_string() } })
                    }
                }
            }
        }

    } else if _method == "initialize" {
        return json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": true }
                },
                "serverInfo": {
                    "name": "astra-mcp",
                    "version": "0.1.0"
                }
            }
        });
    } else if _method == "tools/list" {
        return json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": [
                    {
                        "name": "ask_astra",
                        "description": "Send a natural language query or command directly to the Astra Codex engine to analyze the codebase, summarize tasks, or time travel debug.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "prompt": {
                                    "type": "string",
                                    "description": "The command or question for Astra (e.g., ':summary', 'how does auth work?')"
                                }
                            },
                            "required": ["prompt"]
                        }
                    },
                    {
                        "name": "get_next_task",
                        "description": "Ask Astra for the next architectural step. Astra will analyze the codebase health and return a structured JSON plan with actionable steps for you (the IDE) to execute.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "goal": {
                                    "type": "string",
                                    "description": "Optional high-level goal (e.g., 'refactor backend'). If omitted, Astra will suggest the top priority fix based on health scores."
                                }
                            }
                        }
                    },
                    {
                        "name": "report_task_result",
                        "description": "Report the outcome of an executed Orchestrator task back to Astra so it can learn and update the codebase health.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "task_id": { "type": "string", "description": "The ID of the task provided by get_next_task" },
                                "success": { "type": "boolean", "description": "Whether the task was completed successfully" },
                                "details": { "type": "string", "description": "What was done or any errors encountered" }
                            },
                            "required": ["task_id", "success", "details"]
                        }
                    }
                ]
            }
        });
    }

    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {}
    })
}
