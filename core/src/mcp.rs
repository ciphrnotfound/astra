use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde_json::{json, Value};

use crate::coworker::{CoworkJobStatus, CoworkStore};
use crate::engine::CodexEngine;

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Astra's stdio MCP server. Each editor may launch its own process; durable
/// cowork jobs and memory remain shared through the project `.astra` folder.
pub fn run_mcp_server(engine: &mut CodexEngine) -> Result<()> {
    eprintln!("Astra MCP coworker ready on stdio.");
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(&line) {
            Ok(request) => {
                if let Some(response) = handle_mcp_request(engine, request) {
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(error) => {
                let response = rpc_error(Value::Null, -32700, &format!("Parse error: {}", error));
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}

fn handle_mcp_request(engine: &mut CodexEngine, request: Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let id = request.get("id").cloned();

    // JSON-RPC notifications never receive a response.
    if id.is_none() {
        return None;
    }
    let id = id.unwrap_or(Value::Null);
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

    let response = match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": { "name": "astra-mcp", "version": env!("CARGO_PKG_VERSION") }
            }
        }),
        "ping" => rpc_result(id, json!({})),
        "tools/list" => rpc_result(id, json!({ "tools": tool_definitions() })),
        "tools/call" => handle_tool_call(engine, id, &params),
        "resources/list" => rpc_result(id, json!({ "resources": resource_definitions() })),
        "resources/read" => handle_resource_read(engine, id, &params),
        "prompts/list" => rpc_result(id, json!({ "prompts": prompt_definitions() })),
        "prompts/get" => handle_prompt_get(engine, id, &params),
        _ => rpc_error(id, -32601, &format!("Method not found: {}", method)),
    };
    Some(response)
}

fn handle_tool_call(engine: &mut CodexEngine, id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let store = CoworkStore::new(engine.root());

    let result: Result<Value> = (|| match name {
        "ask_astra" => {
            let prompt = required_string(&args, "prompt")?;
            // handle_input already performs compact retrieval and grounding. The old
            // MCP bridge duplicated a large context block into the user message.
            Ok(json!({ "answer": engine.handle_input(prompt)? }))
        }
        "astra_project_context" => {
            let query = args
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("current project and active work");
            let max_chars = args
                .get("max_chars")
                .and_then(Value::as_u64)
                .unwrap_or(3_500)
                .clamp(800, 8_000) as usize;
            Ok(json!({ "context": engine.build_cowork_context(query, max_chars) }))
        }
        "astra_create_job" => {
            let goal = required_string(&args, "goal")?;
            let worker = args.get("worker").and_then(Value::as_str);
            let acceptance = string_array(&args, "acceptance");
            let job = store.create_job(goal, worker, acceptance)?;
            engine.memory_mut().add(
                "cowork-job",
                format!(
                    "Created {} for {:?}: {}",
                    job.id, job.preferred_worker, job.goal
                ),
            );
            Ok(json!({ "job": job, "prompt": job.worker_prompt() }))
        }
        "astra_claim_job" => {
            let worker = required_string(&args, "worker")?;
            match store.claim_next(worker)? {
                Some(job) => {
                    engine
                        .memory_mut()
                        .add("cowork-job", format!("{} claimed {}", worker, job.id));
                    Ok(json!({ "job": job, "prompt": job.worker_prompt() }))
                }
                None => Ok(json!({ "job": null, "message": "No matching queued job." })),
            }
        }
        "astra_report_job" => {
            let job_id = required_string(&args, "job_id")?;
            let status = parse_job_status(required_string(&args, "status")?)?;
            let summary = required_string(&args, "summary")?;
            let worker = args.get("worker").and_then(Value::as_str);
            let job = store.report(
                job_id,
                worker,
                status,
                summary,
                string_array(&args, "files_changed"),
                string_array(&args, "verification"),
            )?;
            let issue_id = args
                .get("issue_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| extract_issue_id(&job.goal));
            if let Some(issue_id) = issue_id {
                let issue_store = crate::issues::IssueStore::new(engine.root());
                if let Some(mut issue) = issue_store.get(&issue_id)? {
                    issue.status = match job.status {
                        CoworkJobStatus::Completed => crate::issues::IssueStatus::Verified,
                        CoworkJobStatus::Failed => crate::issues::IssueStatus::Failed,
                        CoworkJobStatus::Blocked => crate::issues::IssueStatus::Blocked,
                        CoworkJobStatus::Claimed => crate::issues::IssueStatus::InProgress,
                        _ => issue.status,
                    };
                    issue.fix_summary = job.summary.clone();
                    issue.changed_files = job.files_changed.clone();
                    issue.verification = job.verification.clone();
                    issue.updated_at = job.updated_at;
                    issue_store.save(&issue)?;
                }
            }
            engine.memory_mut().add(
                "cowork-result",
                format!("{} {:?}: {}", job.id, job.status, summary),
            );
            Ok(json!({ "job": job }))
        }
        "astra_job_status" => {
            if let Some(job_id) = args.get("job_id").and_then(Value::as_str) {
                Ok(json!({ "job": store.get(job_id)? }))
            } else {
                Ok(json!({ "jobs": store.list(30)? }))
            }
        }
        "astra_issue_status" => {
            let issue_store = crate::issues::IssueStore::new(engine.root());
            if let Some(issue_id) = args.get("issue_id").and_then(Value::as_str) {
                Ok(json!({ "issue": issue_store.get(issue_id)? }))
            } else {
                Ok(json!({ "issues": issue_store.list(30)? }))
            }
        }
        "astra_remember_decision" => {
            let key = required_string(&args, "key")?;
            let value = required_string(&args, "value")?;
            engine.memory_mut().remember_project_fact(key, value);
            Ok(json!({ "remembered": true, "key": key, "value": value }))
        }
        // Backward-compatible names for existing Astra MCP clients.
        "get_next_task" => {
            let goal = args.get("goal").and_then(Value::as_str);
            Ok(json!({ "plan": engine.get_next_task(goal)? }))
        }
        "report_task_result" => {
            let task_id = required_string(&args, "task_id")?;
            let success = args
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let details = required_string(&args, "details")?;
            Ok(json!({ "message": engine.report_task_result(task_id, success, details)? }))
        }
        _ => Err(anyhow::anyhow!("Unknown Astra tool '{}'.", name)),
    })();

    match result {
        Ok(value) => tool_result(id, value, false),
        Err(error) => tool_result(id, json!({ "error": error.to_string() }), true),
    }
}

fn handle_resource_read(engine: &mut CodexEngine, id: Value, params: &Value) -> Value {
    let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
    let store = CoworkStore::new(engine.root());
    let text = match uri {
        "astra://project/context" => engine.build_cowork_context("project architecture", 4_000),
        "astra://cowork/jobs" => serde_json::to_string_pretty(&store.list(50).unwrap_or_default())
            .unwrap_or_else(|_| "[]".to_string()),
        "astra://memory/project" => engine.memory().project_report(),
        "astra://issues" => serde_json::to_string_pretty(
            &crate::issues::IssueStore::new(engine.root())
                .list(50)
                .unwrap_or_default(),
        )
        .unwrap_or_else(|_| "[]".to_string()),
        _ => return rpc_error(id, -32602, &format!("Unknown resource URI: {}", uri)),
    };
    rpc_result(
        id,
        json!({ "contents": [{ "uri": uri, "mimeType": "text/plain", "text": text }] }),
    )
}

fn handle_prompt_get(engine: &mut CodexEngine, id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    if name != "ship_feature" {
        return rpc_error(id, -32602, &format!("Unknown prompt: {}", name));
    }
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let goal = args
        .get("goal")
        .and_then(Value::as_str)
        .unwrap_or("Ship the highest-priority user-facing improvement");
    let worker = args
        .get("worker")
        .and_then(Value::as_str)
        .unwrap_or("this editor");
    let context = engine.build_cowork_context(goal, 2_500);
    let text = format!(
        "You are {} working with Astra as the persistent tech lead.\n\nGoal: {}\n\n{}\n\nInspect before editing, preserve unrelated changes, implement completely, verify with relevant tests/builds, then report the outcome through `astra_report_job` if a job ID was provided.",
        worker, goal, context
    );
    rpc_result(
        id,
        json!({
            "description": "A grounded feature-shipping prompt from Astra",
            "messages": [{ "role": "user", "content": { "type": "text", "text": text } }]
        }),
    )
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool("ask_astra", "Ask Astra a grounded project question. Use for decisions and analysis; use cowork jobs for delegated implementation.", json!({
            "type": "object", "properties": { "prompt": { "type": "string" } }, "required": ["prompt"]
        })),
        tool("astra_project_context", "Get a compact, query-focused project snapshot and relevant durable memory before editing.", json!({
            "type": "object", "properties": {
                "query": { "type": "string" },
                "max_chars": { "type": "integer", "minimum": 800, "maximum": 8000 }
            }
        })),
        tool("astra_create_job", "Create a durable implementation job for Codex, Claude, Cursor, or any MCP worker.", json!({
            "type": "object", "properties": {
                "goal": { "type": "string" },
                "worker": { "type": "string", "description": "codex, claude, cursor, or any" },
                "acceptance": { "type": "array", "items": { "type": "string" } }
            }, "required": ["goal"]
        })),
        tool("astra_claim_job", "Claim the oldest queued job assigned to this worker and receive its execution prompt.", json!({
            "type": "object", "properties": { "worker": { "type": "string" } }, "required": ["worker"]
        })),
        tool("astra_report_job", "Report a cowork job outcome. Completed jobs require verification evidence.", json!({
            "type": "object", "properties": {
                "job_id": { "type": "string" },
                "issue_id": { "type": "string", "description": "Optional linked Astra issue; inferred from the job goal when omitted" },
                "worker": { "type": "string" },
                "status": { "type": "string", "enum": ["claimed", "blocked", "completed", "failed", "cancelled"] },
                "summary": { "type": "string" },
                "files_changed": { "type": "array", "items": { "type": "string" } },
                "verification": { "type": "array", "items": { "type": "string" } }
            }, "required": ["job_id", "status", "summary"]
        })),
        tool("astra_job_status", "Read one job or the recent shared job board.", json!({
            "type": "object", "properties": { "job_id": { "type": "string" } }
        })),
        tool("astra_issue_status", "Read a tracked bug report, its Git evidence, reproduction gate, and worker job.", json!({
            "type": "object", "properties": { "issue_id": { "type": "string" } }
        })),
        tool("astra_remember_decision", "Store a durable project decision so every connected editor receives it when relevant.", json!({
            "type": "object", "properties": {
                "key": { "type": "string" }, "value": { "type": "string" }
            }, "required": ["key", "value"]
        })),
        tool("get_next_task", "Compatibility tool for Astra's original orchestrator.", json!({
            "type": "object", "properties": { "goal": { "type": "string" } }
        })),
        tool("report_task_result", "Compatibility tool for reporting an original orchestrator task.", json!({
            "type": "object", "properties": {
                "task_id": { "type": "string" }, "success": { "type": "boolean" }, "details": { "type": "string" }
            }, "required": ["task_id", "success", "details"]
        })),
    ]
}

fn resource_definitions() -> Vec<Value> {
    vec![
        json!({ "uri": "astra://project/context", "name": "Astra project context", "description": "Compact architecture and working-state snapshot", "mimeType": "text/plain" }),
        json!({ "uri": "astra://cowork/jobs", "name": "Astra cowork jobs", "description": "Shared cross-editor job board", "mimeType": "application/json" }),
        json!({ "uri": "astra://memory/project", "name": "Astra project memory", "description": "Durable project facts and decisions", "mimeType": "text/plain" }),
        json!({ "uri": "astra://issues", "name": "Astra issue ledger", "description": "Tracked bug reports and verification state", "mimeType": "application/json" }),
    ]
}

fn prompt_definitions() -> Vec<Value> {
    vec![json!({
        "name": "ship_feature",
        "description": "Create a grounded implementation brief for shipping a feature with Astra",
        "arguments": [
            { "name": "goal", "description": "Feature or outcome to ship", "required": true },
            { "name": "worker", "description": "Worker identity (Codex, Claude, Cursor)", "required": false }
        ]
    })]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": input_schema })
}

fn required_string<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required string argument '{}'.", key))
}

fn string_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn extract_issue_id(goal: &str) -> Option<String> {
    goal.split_whitespace()
        .map(|token| token.trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '-'))
        .find(|token| token.starts_with("astra-issue-") && token.len() <= 120)
        .map(str::to_owned)
}

fn parse_job_status(value: &str) -> Result<CoworkJobStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "claimed" | "in_progress" => Ok(CoworkJobStatus::Claimed),
        "blocked" => Ok(CoworkJobStatus::Blocked),
        "completed" | "done" => Ok(CoworkJobStatus::Completed),
        "failed" => Ok(CoworkJobStatus::Failed),
        "cancelled" | "canceled" => Ok(CoworkJobStatus::Cancelled),
        other => Err(anyhow::anyhow!("Unsupported job status '{}'.", other)),
    }
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tool_result(id: Value, value: Value, is_error: bool) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
    rpc_result(
        id,
        json!({
            "content": [{ "type": "text", "text": text }],
            "structuredContent": value,
            "isError": is_error
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::handle_mcp_request;
    use crate::engine::CodexEngine;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn engine() -> CodexEngine {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("astra_mcp_test_{}", unique));
        fs::create_dir_all(&root).unwrap();
        CodexEngine::with_root(root)
    }

    #[test]
    fn notifications_do_not_produce_json_rpc_responses() {
        let mut engine = engine();
        assert!(handle_mcp_request(
            &mut engine,
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })
        )
        .is_none());
    }

    #[test]
    fn lists_cowork_tools_resources_and_prompts() {
        let mut engine = engine();
        let tools = handle_mcp_request(
            &mut engine,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
        )
        .unwrap();
        let names = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"astra_claim_job"));
        assert!(names.contains(&"astra_report_job"));
        assert!(names.contains(&"astra_issue_status"));

        let resources = handle_mcp_request(
            &mut engine,
            json!({ "jsonrpc": "2.0", "id": 2, "method": "resources/list" }),
        )
        .unwrap();
        assert_eq!(
            resources["result"]["resources"].as_array().unwrap().len(),
            4
        );

        let prompts = handle_mcp_request(
            &mut engine,
            json!({ "jsonrpc": "2.0", "id": 3, "method": "prompts/list" }),
        )
        .unwrap();
        assert_eq!(prompts["result"]["prompts"][0]["name"], "ship_feature");
    }

    #[test]
    fn editor_can_create_claim_and_report_a_verified_job() {
        let mut engine = engine();
        let created = handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "astra_create_job", "arguments": {
                    "goal": "Ship profile settings", "worker": "cursor",
                    "acceptance": ["Settings persist"]
                }}
            }),
        )
        .unwrap();
        let job_id = created["result"]["structuredContent"]["job"]["id"]
            .as_str()
            .unwrap()
            .to_string();

        let claimed = handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": { "name": "astra_claim_job", "arguments": { "worker": "cursor" }}
            }),
        )
        .unwrap();
        assert_eq!(claimed["result"]["structuredContent"]["job"]["id"], job_id);

        let reported = handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 3, "method": "tools/call",
                "params": { "name": "astra_report_job", "arguments": {
                    "job_id": job_id, "worker": "cursor", "status": "completed",
                    "summary": "Implemented settings", "files_changed": ["src/settings.ts"],
                    "verification": ["npm test passed"]
                }}
            }),
        )
        .unwrap();
        assert_eq!(
            reported["result"]["structuredContent"]["job"]["status"],
            "completed"
        );
        assert_eq!(reported["result"]["isError"], false);
    }

    #[test]
    fn verified_worker_report_updates_linked_issue_ledger() {
        let mut engine = engine();
        let issue = crate::issues::IssueStore::new(engine.root())
            .create("checkout returns 500 for expired cards")
            .unwrap();
        let created = handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 10, "method": "tools/call",
                "params": { "name": "astra_create_job", "arguments": {
                    "goal": format!("Resolve {}: checkout returns 500", issue.id),
                    "worker": "codex"
                }}
            }),
        )
        .unwrap();
        let job_id = created["result"]["structuredContent"]["job"]["id"]
            .as_str()
            .unwrap()
            .to_string();
        handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 11, "method": "tools/call",
                "params": { "name": "astra_claim_job", "arguments": { "worker": "codex" }}
            }),
        );
        handle_mcp_request(
            &mut engine,
            json!({
                "jsonrpc": "2.0", "id": 12, "method": "tools/call",
                "params": { "name": "astra_report_job", "arguments": {
                    "job_id": job_id, "worker": "codex", "status": "completed",
                    "summary": "Added regression test and fixed expiry handling",
                    "files_changed": ["src/checkout.rs"],
                    "verification": ["cargo test checkout::expiry"]
                }}
            }),
        );
        let updated = crate::issues::IssueStore::new(engine.root())
            .get(&issue.id)
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, crate::issues::IssueStatus::Verified);
        assert!(updated.fix_summary.unwrap().contains("regression test"));
        assert_eq!(updated.verification, vec!["cargo test checkout::expiry"]);
    }
}
