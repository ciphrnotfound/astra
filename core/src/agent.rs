use std::fmt::Write as FmtWrite;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::model::{CodexModel, SearchProvider};
use crate::tools::{self, ToolCall, ToolResult};

/// Maximum iterations before the agent loop is forcefully stopped.
const MAX_ITERATIONS: usize = 50;
const CONTEXT_TAIL_MESSAGES: usize = 10;

/// A message in the agent conversation.
#[derive(Clone, Debug)]
pub enum AgentMessage {
    System(String),
    User(String),
    Assistant(String),
    ToolResults(Vec<ToolResult>),
}

/// Configuration for an agent run.
pub struct AgentConfig {
    pub auto_approve: bool,
    pub max_iterations: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            auto_approve: false,
            max_iterations: MAX_ITERATIONS,
        }
    }
}

/// Run the agentic tool-use loop.
///
/// The agent sends the user's task to the LLM along with tool definitions.
/// If the LLM responds with tool calls, the tools are executed and results
/// fed back. This repeats until the LLM gives a final text answer or the
/// max iteration limit is hit.
pub fn run_agent_loop(
    model: &dyn CodexModel,
    task: &str,
    project_root: &Path,
    config: &AgentConfig,
    system_context: &str,
    searcher: Option<&dyn SearchProvider>,
) -> Result<String> {
    // Build a compact system prompt with tools
    let tools_block = tools::tools_prompt_block();
    let system = format!(
        "{}\n\n{}\n\nRules (CRACKED SENIOR ENGINEER MODE):\n\
        1. REASON: Calibrate your thoughts first. Use `reason` to plan architecture.\n\
        2. NO PLACEHOLDERS: NEVER use `// rest of code here` or lazy placeholders. Write complete, production-ready code with full implementations. If writing a file, write the WHOLE file.\n\
        3. PATHS: Never assume where a file is. If `read_file` fails, use `search_codebase` or `list_dir` to find the correct path.\n\
        4. FOLDERS vs FILES: Only use `create_dir` for FOLDERS. For files, rely on `write_file` (creates parents automatically).\n\
        5. EFFICIENCY: Minimize model calls by batching multiple independent tool calls into one JSON response when possible.\n\
        6. EXCELLENCE: Write expert-level, highly-optimized, properly typed and error-handled code. Do not skip edge cases.\n\
        7. IMPLEMENT: Use JSON `tool_calls` for all actions.\n\
        8. FINAL CHECK: Before giving a final response, ensure no recent tool call failed. If a tool failed, fix it first.\n\n\
        Example Turn:\n\
        PLAN: I will find the main file first.\n\
        {{\"tool_calls\": [{{\"name\": \"search_codebase\", \"arguments\": {{\"pattern\": \"main.py\"}}}}]}}\n",
        system_context, tools_block
    );

    let mut conversation: Vec<AgentMessage> = vec![
        AgentMessage::System(system.clone()),
        AgentMessage::User(task.to_string()),
    ];

    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > config.max_iterations {
            return Ok(format!(
                "⚠️ Agent stopped after {} iterations. Here's what was accomplished so far.",
                config.max_iterations
            ));
        }

        // Add a delay between iterations to avoid rate limiting
        if iteration > 1 {
            thread::sleep(Duration::from_millis(4000));
        }

        trim_conversation(&mut conversation, CONTEXT_TAIL_MESSAGES);

        // Build the prompt from conversation history
        let prompt = build_prompt(&conversation);

        // Call the LLM
        let response = model.complete_chat(&system, &prompt)?;
        let trimmed = response.trim().to_string();

        // Try to parse tool calls from the response
        if let Some(tool_calls) = parse_tool_calls(&trimmed) {
            for call in &tool_calls {
                eprintln!("  🔧 Calling tool: {} {:?}", call.name, summarize_args(&call.arguments));
            }

            let mut results = Vec::new();
            for call in &tool_calls {
                let result = tools::execute_tool(call, project_root, config.auto_approve, searcher);
                if result.success {
                    eprintln!("  ✅ {} succeeded", result.tool_name);
                } else {
                    eprintln!("  ❌ {} failed: {}", result.tool_name, &result.output[..result.output.len().min(200)]);
                }
                results.push(result);
            }

            conversation.push(AgentMessage::Assistant(trimmed));
            conversation.push(AgentMessage::ToolResults(results));
        } else if trimmed.contains("{\"tool_calls\"") || trimmed.contains("\"tool_calls\":") {
            // Repair Turn: The response looks like JSON but failed to parse
            eprintln!("  ⚠️  Malformed tool call detected. Asking agent to fix formatting...");
            conversation.push(AgentMessage::Assistant(trimmed));
            conversation.push(AgentMessage::User("Error: Your last response contained malformed JSON tool calls. Please fix the formatting (ensure all quotes are closed and keys are correct) and try again. Respond ONLY with the valid JSON block.".to_string()));
        } else {
            if has_recent_tool_error(&conversation) {
                eprintln!("  ⚠️  Final response blocked because recent tool calls failed.");
                conversation.push(AgentMessage::Assistant(trimmed));
                conversation.push(AgentMessage::User(
                    "A recent tool call failed. Do not finalize yet. Diagnose the failure, call the required tools to fix it, and only then provide the final answer.".to_string(),
                ));
                continue;
            }
            eprintln!("\n✅ **Task complete!** Astra has finished its autonomous work.\n");
            return Ok(trimmed);
        }
    }
}

/// Build a single user-message string from the conversation history
/// for models that only support system + user format.
fn build_prompt(conversation: &[AgentMessage]) -> String {
    let mut prompt = String::new();

    for msg in conversation {
        match msg {
            AgentMessage::System(_) => {
                // System is sent separately via complete_chat
            }
            AgentMessage::User(text) => {
                let _ = writeln!(&mut prompt, "USER: {}\n", text);
            }
            AgentMessage::Assistant(text) => {
                let _ = writeln!(&mut prompt, "ASSISTANT: {}\n", text);
            }
            AgentMessage::ToolResults(results) => {
                let _ = writeln!(&mut prompt, "TOOL RESULTS:");
                for r in results {
                    let status = if r.success { "SUCCESS" } else { "ERROR" };
                    let truncated_output = safe_truncate(&r.output, 2000);
                    let _ = writeln!(
                        &mut prompt,
                        "[{}] {}: {}\n",
                        status, r.tool_name, truncated_output
                    );
                }
                let _ = writeln!(&mut prompt, "Continue with the task. Call more tools if needed, or give your final answer if done.\n");
            }
        }
    }

    prompt
}

/// Try to extract tool calls from an LLM response.
///
/// The LLM is instructed to respond with:
/// ```json
/// {"tool_calls": [{"name": "...", "arguments": {...}}]}
/// ```
fn parse_tool_calls(response: &str) -> Option<Vec<ToolCall>> {
    // Try to find a JSON block in the response
    let trimmed = response.trim();

    // Case 1: The entire response is JSON
    if let Some(calls) = try_parse_tool_calls_json(trimmed) {
        return Some(calls);
    }

    // Case 2: JSON is inside a markdown code block
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            let json_str = after[..end].trim();
            if let Some(calls) = try_parse_tool_calls_json(json_str) {
                return Some(calls);
            }
        }
    }

    // Case 3: JSON is inside a plain code block
    if let Some(start) = trimmed.find("```\n") {
        let after = &trimmed[start + 4..];
        if let Some(end) = after.find("```") {
            let json_str = after[..end].trim();
            if let Some(calls) = try_parse_tool_calls_json(json_str) {
                return Some(calls);
            }
        }
    }

    // Case 4: Find a JSON object anywhere in the text
    if let Some(start) = trimmed.find("{\"tool_calls\"") {
        // Go forward to find the matching closing brace
        let substr = &trimmed[start..];
        if let Some(calls) = try_parse_tool_calls_json(substr) {
            return Some(calls);
        }
        // Try to find the end by brace matching
        if let Some(end) = find_matching_brace(substr) {
            let json_str = &substr[..=end];
            if let Some(calls) = try_parse_tool_calls_json(json_str) {
                return Some(calls);
            }
        }
    }

    None
}

fn try_parse_tool_calls_json(json_str: &str) -> Option<Vec<ToolCall>> {
    #[derive(serde::Deserialize)]
    struct ToolCallsWrapper {
        tool_calls: Vec<ToolCall>,
    }

    serde_json::from_str::<ToolCallsWrapper>(json_str)
        .ok()
        .map(|w| w.tool_calls)
        .filter(|calls| !calls.is_empty())
}

/// Find the index of the closing brace that matches the opening brace at position 0.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, ch) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Summarize tool arguments for display (don't dump massive content strings).
fn summarize_args(args: &serde_json::Value) -> String {
    if let Some(obj) = args.as_object() {
        let mut parts = Vec::new();
        for (k, v) in obj {
            if k == "content" {
                let len = v.as_str().map(|s| s.len()).unwrap_or(0);
                parts.push(format!("content: [{} chars]", len));
            } else if let Some(s) = v.as_str() {
                if s.len() > 60 {
                    parts.push(format!("{}: \"{}...\"", k, &s[..57]));
                } else {
                    parts.push(format!("{}: \"{}\"", k, s));
                }
            } else {
                parts.push(format!("{}: {}", k, v));
            }
        }
        format!("{{{}}}", parts.join(", "))
    } else {
        format!("{}", args)
    }
}

/// Trim conversation history to keep only the System, User (first), and last N messages.
/// This prevents the prompt from growing too large and hitting rate/token limits.
fn trim_conversation(conversation: &mut Vec<AgentMessage>, keep_last: usize) {
    if conversation.len() <= keep_last + 2 {
        return; // Already small enough
    }
    // Keep: System (0), User (1), then the last `keep_last` messages
    let tail: Vec<AgentMessage> = conversation
        .iter()
        .skip(conversation.len() - keep_last)
        .cloned()
        .collect();
    conversation.truncate(2);
    conversation.extend(tail);
}

fn has_recent_tool_error(conversation: &[AgentMessage]) -> bool {
    conversation
        .iter()
        .rev()
        .take(8)
        .any(|msg| match msg {
            AgentMessage::ToolResults(results) => results.iter().any(|r| !r.success),
            _ => false,
        })
}

/// Safely truncate a string to a maximum byte length without breaking UTF-8 character boundaries.
fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        s
    } else {
        let mut end = max_bytes;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}
