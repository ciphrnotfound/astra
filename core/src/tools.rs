use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

// Re-use SearchProvider from model
use crate::model::SearchProvider;

// ── Tool Definitions ────────────────────────────────────────────────────

/// JSON schema for a tool parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Description of a tool the agent can invoke.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
}

/// A concrete invocation of a tool returned by the LLM.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Result of executing a tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
}

// ── Tool Registry ───────────────────────────────────────────────────────

pub fn all_tool_defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "read_file".to_string(),
            description: "Read the contents of a file at the given path.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "Absolute or relative file path to read.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "write_file".to_string(),
            description: "Create or overwrite a file with the given content. Creates parent directories if needed.".to_string(),
            parameters: vec![
                ToolParam {
                    name: "path".to_string(),
                    description: "File path to write to.".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "content".to_string(),
                    description: "The full content to write to the file.".to_string(),
                    required: true,
                },
            ],
        },
        ToolDef {
            name: "edit_file".to_string(),
            description: "Apply a targeted search-and-replace edit to a file. Replaces the first occurrence of 'search' with 'replace'.".to_string(),
            parameters: vec![
                ToolParam {
                    name: "path".to_string(),
                    description: "File path to edit.".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "search".to_string(),
                    description: "Exact string to find in the file.".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "replace".to_string(),
                    description: "String to replace the found text with.".to_string(),
                    required: true,
                },
            ],
        },
        ToolDef {
            name: "list_dir".to_string(),
            description: "List files and subdirectories in the given directory.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "Directory path to list.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "run_command".to_string(),
            description: "Execute a shell command and return its stdout and stderr. Use this for running builds, tests, installs, git commands, etc.".to_string(),
            parameters: vec![
                ToolParam {
                    name: "command".to_string(),
                    description: "The shell command to execute (e.g. 'npm install', 'cargo build').".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "cwd".to_string(),
                    description: "Optional working directory for the command. Defaults to the project root.".to_string(),
                    required: false,
                },
            ],
        },
        ToolDef {
            name: "search_codebase".to_string(),
            description: "Search for a text pattern across all indexed files in the codebase. Returns matching file paths and line numbers.".to_string(),
            parameters: vec![ToolParam {
                name: "pattern".to_string(),
                description: "Text pattern or keyword to search for.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "delete_file".to_string(),
            description: "Delete a file at the given path.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "File path to delete.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "create_dir".to_string(),
            description: "Create a new directory (and any parent directories) if they don't exist.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "Directory path to create.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "delete_dir".to_string(),
            description: "Delete an entire directory and all its contents.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "Directory path to delete.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "move_path".to_string(),
            description: "Move or rename a file or directory from src to dst. Overwrites dst if it exists.".to_string(),
            parameters: vec![
                ToolParam {
                    name: "src".to_string(),
                    description: "Source path.".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "dst".to_string(),
                    description: "Destination path.".to_string(),
                    required: true,
                },
            ],
        },
        ToolDef {
            name: "reason".to_string(),
            description: "Think through a problem before taking action. Use this to reason about architecture, logic, or complex refactors. Returns your own thoughts.".to_string(),
            parameters: vec![ToolParam {
                name: "thought".to_string(),
                description: "Your detailed reasoning or architectural plan.".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "research_web".to_string(),
            description: "Search the web for latest documentation, library syntax, or technical solutions. Returns a summary of findings.".to_string(),
            parameters: vec![ToolParam {
                name: "query".to_string(),
                description: "Search query (e.g. 'FastAPI latest sqlalchemy 2.0 syntax').".to_string(),
                required: true,
            }],
        },
    ]
}

/// Build a compact text description of available tools for injection into the system prompt.
pub fn tools_prompt_block() -> String {
    String::from(r#"## Tools

To call tools, respond ONLY with this JSON (no other text):
{"tool_calls": [{"name": "TOOL", "arguments": {"key": "val"}}]}

Multiple tools allowed per response. When done, reply with plain text (no JSON).

Tools:
- read_file(path) — Read file contents
- write_file(path, content) — Create/overwrite file (creates dirs)
- edit_file(path, search, replace) — Search & replace in file
- list_dir(path) — List directory contents
- run_command(command, cwd?) — Run shell command
- search_codebase(pattern) — Grep across project files
- delete_file(path) — Delete a file
- create_dir(path) — Create a directory
- delete_dir(path) — Delete a directory and all contents
- move_path(src, dst) — Move or rename a file/dir
- reason(thought) — SYSTEM 2: Think through logic before doing
- research_web(query) — Search the web for latest info
"#)
}

// ── Tool Execution ──────────────────────────────────────────────────────

/// Execute a tool call and return the result.
/// `project_root` is the root of the project for resolving relative paths.
pub fn execute_tool(
    call: &ToolCall,
    project_root: &Path,
    auto_approve: bool,
    searcher: Option<&dyn SearchProvider>,
) -> ToolResult {
    let result = match call.name.as_str() {
        "read_file" => exec_read_file(call, project_root),
        "write_file" => exec_write_file(call, project_root, auto_approve),
        "edit_file" => exec_edit_file(call, project_root, auto_approve),
        "list_dir" => exec_list_dir(call, project_root),
        "run_command" => exec_run_command(call, project_root, auto_approve),
        "search_codebase" => exec_search_codebase(call, project_root),
        "delete_file" => exec_delete_file(call, project_root, auto_approve),
        "create_dir" => exec_create_dir(call, project_root),
        "delete_dir" => exec_delete_dir(call, project_root, auto_approve),
        "move_path" => exec_move_path(call, project_root, auto_approve),
        "reason" => exec_reason(call),
        "research_web" => exec_research_web(call, searcher),
        _ => Err(anyhow!("Unknown tool: {}", call.name)),
    };

    match result {
        Ok(output) => ToolResult {
            tool_name: call.name.clone(),
            success: true,
            output,
        },
        Err(e) => ToolResult {
            tool_name: call.name.clone(),
            success: false,
            output: format!("Error: {}", e),
        },
    }
}

fn resolve_path(raw: &str, project_root: &Path) -> PathBuf {
    let p = PathBuf::from(raw);
    if p.is_absolute() {
        p
    } else {
        project_root.join(p)
    }
}

fn get_str_arg(args: &serde_json::Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Missing required argument: {}", key))
}

fn get_optional_str_arg(args: &serde_json::Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

// ── Individual Tool Implementations ─────────────────────────────────────

fn exec_read_file(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    let content = fs::read_to_string(&path)?;
    // Truncate very large files to avoid blowing up LLM context
    if content.len() > 50_000 {
        let truncated = &content[..50_000];
        Ok(format!(
            "{}\n\n... [TRUNCATED — file is {} bytes, showing first 50,000]",
            truncated,
            content.len()
        ))
    } else {
        Ok(content)
    }
}

fn exec_write_file(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let content = get_str_arg(&call.arguments, "content")?;
    let path = resolve_path(&path_str, project_root);

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to write file: {}", path.display());
        eprintln!("     ({} bytes)", content.len());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied file write.".to_string());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, &content)?;
    Ok(format!("✅ Wrote {} bytes to {}", content.len(), path.display()))
}

fn exec_edit_file(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let search = get_str_arg(&call.arguments, "search")?;
    let replace = get_str_arg(&call.arguments, "replace")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    let original = fs::read_to_string(&path)?;
    if !original.contains(&search) {
        return Err(anyhow!(
            "Search string not found in {}. The exact text must match.",
            path.display()
        ));
    }

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to edit file: {}", path.display());
        eprintln!("     Search:  {} chars", search.len());
        eprintln!("     Replace: {} chars", replace.len());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied file edit.".to_string());
        }
    }

    let modified = original.replacen(&search, &replace, 1);
    fs::write(&path, &modified)?;
    Ok(format!("✅ Edited {}", path.display()))
}

fn exec_list_dir(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("Not a directory: {}", path.display()));
    }

    let mut entries = Vec::new();
    let skip = [
        "node_modules", "target", ".git", "__pycache__", ".venv", "venv",
        "dist", "build", ".next",
    ];

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        let meta = entry.metadata()?;
        if meta.is_dir() {
            entries.push(format!("📁 {}/", name));
        } else {
            let size = meta.len();
            entries.push(format!("📄 {} ({} bytes)", name, size));
        }
    }

    entries.sort();
    if entries.is_empty() {
        Ok(format!("Directory {} is empty.", path.display()))
    } else {
        Ok(format!(
            "Contents of {}:\n{}",
            path.display(),
            entries.join("\n")
        ))
    }
}

fn exec_run_command(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let command_str = get_str_arg(&call.arguments, "command")?;
    let cwd = get_optional_str_arg(&call.arguments, "cwd")
        .map(|c| resolve_path(&c, project_root))
        .unwrap_or_else(|| project_root.to_path_buf());

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to run command: {}", command_str);
        eprintln!("     Working dir: {}", cwd.display());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied command execution.".to_string());
        }
    }

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command_str])
            .current_dir(&cwd)
            .output()?
    } else {
        Command::new("sh")
            .args(["-c", &command_str])
            .current_dir(&cwd)
            .output()?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    let mut result = String::new();
    result.push_str(&format!("Exit code: {}\n", exit_code));
    if !stdout.is_empty() {
        let truncated = if stdout.len() > 10_000 {
            format!("{}... [TRUNCATED]", &stdout[..10_000])
        } else {
            stdout.to_string()
        };
        result.push_str(&format!("STDOUT:\n{}\n", truncated));
    }
    if !stderr.is_empty() {
        let truncated = if stderr.len() > 5_000 {
            format!("{}... [TRUNCATED]", &stderr[..5_000])
        } else {
            stderr.to_string()
        };
        result.push_str(&format!("STDERR:\n{}\n", truncated));
    }
    Ok(result)
}

fn exec_search_codebase(call: &ToolCall, project_root: &Path) -> Result<String> {
    let pattern = get_str_arg(&call.arguments, "pattern")?;
    let pattern_lower = pattern.to_lowercase();

    let mut matches = Vec::new();
    let mut stack = vec![project_root.to_path_buf()];
    let skip = [
        "node_modules", "target", ".git", "__pycache__", ".venv", "venv",
        "dist", "build", ".next", ".astra", ".codex",
    ];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if skip.contains(&name) {
                    continue;
                }
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&pattern_lower) {
                            let rel = path.strip_prefix(project_root).unwrap_or(&path);
                            matches.push(format!(
                                "{}:{}: {}",
                                rel.display(),
                                i + 1,
                                line.trim()
                            ));
                            if matches.len() >= 50 {
                                break;
                            }
                        }
                    }
                }
                if matches.len() >= 50 {
                    break;
                }
            }
        }
        if matches.len() >= 50 {
            break;
        }
    }

    if matches.is_empty() {
        Ok(format!("No matches found for '{}'.", pattern))
    } else {
        Ok(format!(
            "Found {} matches for '{}':\n{}",
            matches.len(),
            pattern,
            matches.join("\n")
        ))
    }
}

fn exec_delete_file(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to DELETE file: {}", path.display());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied file deletion.".to_string());
        }
    }

    fs::remove_file(&path)?;
    Ok(format!("✅ Deleted {}", path.display()))
}

fn exec_create_dir(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if path.exists() {
        if path.is_dir() {
            return Ok(format!("✅ Directory {} already exists", path.display()));
        }
        return Err(anyhow!(
            "Cannot create directory at {} because a file already exists at that path.",
            path.display()
        ));
    }

    fs::create_dir_all(&path)?;
    Ok(format!("✅ Directory {} created", path.display()))
}

fn exec_delete_dir(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("Directory not found: {}", path.display()));
    }

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to DELETE directory and all contents: {}", path.display());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied directory deletion.".to_string());
        }
    }

    fs::remove_dir_all(&path)?;
    Ok(format!("✅ Deleted directory {}", path.display()))
}

fn exec_move_path(call: &ToolCall, project_root: &Path, auto_approve: bool) -> Result<String> {
    let src_str = get_str_arg(&call.arguments, "src")?;
    let dst_str = get_str_arg(&call.arguments, "dst")?;
    let src = resolve_path(&src_str, project_root);
    let dst = resolve_path(&dst_str, project_root);

    if !src.exists() {
        return Err(anyhow!("Source path not found: {}", src.display()));
    }

    if !auto_approve {
        eprintln!("  ⚠️  Agent wants to move/rename: {} -> {}", src.display(), dst.display());
        eprint!("     Allow? [y/n]: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("User denied move operation.".to_string());
        }
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&src, &dst)?;
    Ok(format!("✅ Moved {} to {}", src.display(), dst.display()))
}

fn exec_reason(call: &ToolCall) -> Result<String> {
    let thought = get_str_arg(&call.arguments, "thought")?;
    Ok(format!("🧠 Reasoning: {}", thought))
}

fn exec_research_web(call: &ToolCall, searcher: Option<&dyn SearchProvider>) -> Result<String> {
    let query = get_str_arg(&call.arguments, "query")?;
    if let Some(s) = searcher {
        let results = s.search(&query)?;
        Ok(format!("🌐 Search results for '{}':\n{}", query, results))
    } else {
        Err(anyhow!("Web search is not configured or available."))
    }
}
