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
            name: "inspect_project".to_string(),
            description: "Return a compact project snapshot: top-level structure, manifests, and current git status. Use this as the first inspection for unfamiliar repositories.".to_string(),
            parameters: vec![],
        },
        ToolDef {
            name: "read_file".to_string(),
            description: "Read a file, optionally restricting output to an inclusive line range.".to_string(),
            parameters: vec![
                ToolParam {
                    name: "path".to_string(),
                    description: "Absolute or relative file path to read.".to_string(),
                    required: true,
                },
                ToolParam {
                    name: "start_line".to_string(),
                    description: "Optional 1-based first line.".to_string(),
                    required: false,
                },
                ToolParam {
                    name: "end_line".to_string(),
                    description: "Optional 1-based last line; at most 300 lines are returned.".to_string(),
                    required: false,
                },
            ],
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
        ToolDef {
            name: "get_file_owners".to_string(),
            description: "Query the Temporal Graph to find out who really owns a file and their commit history.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "The path of the file to query (e.g., core/src/engine.rs)".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "get_hidden_coupling".to_string(),
            description: "Find out if files are secretly coupled and always change together based on historical commit data.".to_string(),
            parameters: vec![],
        },
        ToolDef {
            name: "get_file_timeline".to_string(),
            description: "Get the full temporal story of a file: authors, staleness, and recent history.".to_string(),
            parameters: vec![ToolParam {
                name: "path".to_string(),
                description: "The path of the file to query".to_string(),
                required: true,
            }],
        },
        ToolDef {
            name: "get_semantic_concepts".to_string(),
            description: "Retrieve Astra's high-level understanding of the project's risks, patterns, and architectural issues.".to_string(),
            parameters: vec![],
        },
    ]
}

/// Build a compact text description of available tools for injection into the system prompt.
pub fn tools_prompt_block() -> String {
    String::from(
        r#"## Tools

To call tools, respond ONLY with this JSON (no other text):
{"tool_calls": [{"name": "TOOL", "arguments": {"key": "val"}}]}

Multiple tools allowed per response. When done, reply with plain text (no JSON).

Tools:
- inspect_project() — Compact structure, manifests, and git status
- read_file(path, start_line?, end_line?) — Read a file or targeted line range
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
- get_file_owners(path) — Find out who really owns a file
- get_hidden_coupling() — See files that frequently change together
- get_file_timeline(path) — Get a file's history, authors, and staleness
- get_semantic_concepts() — Learn the codebase's identified patterns, risks, and architectural rules
"#,
    )
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
        "inspect_project" => exec_inspect_project(project_root),
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
        "get_file_owners" => exec_get_file_owners(call, project_root),
        "get_hidden_coupling" => exec_get_hidden_coupling(project_root),
        "get_file_timeline" => exec_get_file_timeline(call, project_root),
        "get_semantic_concepts" => exec_get_semantic_concepts(project_root),
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

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let mut vec: Vec<usize> = (0..=s2.len()).collect();
    for (i, c1) in s1.chars().enumerate() {
        let mut prev = i;
        let mut cur;
        vec[0] = i + 1;
        for (j, c2) in s2.chars().enumerate() {
            cur = vec[j + 1];
            vec[j + 1] = std::cmp::min(
                prev + if c1 == c2 { 0 } else { 1 },
                std::cmp::min(vec[j] + 1, vec[j + 1] + 1),
            );
            prev = cur;
        }
    }
    *vec.last().unwrap_or(&0)
}

fn resolve_path(raw: &str, project_root: &Path) -> PathBuf {
    // Strip Windows drive letters (C:\) or Unix absolute roots (/)
    let mut clean_raw = raw.trim();
    if let Some(stripped) = clean_raw.strip_prefix('/') {
        clean_raw = stripped;
    }
    if let Some(stripped) = clean_raw.strip_prefix('\\') {
        clean_raw = stripped;
    }
    // Also handle Windows Drive Prefix (e.g. C:/ or C:\)
    if clean_raw.len() >= 2 && clean_raw.chars().nth(1) == Some(':') {
        clean_raw = &clean_raw[2..];
        if let Some(stripped) = clean_raw.strip_prefix('/') {
            clean_raw = stripped;
        }
        if let Some(stripped) = clean_raw.strip_prefix('\\') {
            clean_raw = stripped;
        }
    }

    let p = PathBuf::from(clean_raw);
    let mut current_resolved = project_root.to_path_buf();

    for comp in p.components() {
        match comp {
            std::path::Component::Normal(comp_os) => {
                let comp_str = comp_os.to_string_lossy().to_string();
                let next_path = current_resolved.join(&comp_str);

                if next_path.exists() {
                    current_resolved = next_path;
                } else {
                    // Try fuzzy matching against siblings
                    let mut best_match = None;
                    let mut best_dist = usize::MAX;

                    if let Ok(entries) = std::fs::read_dir(&current_resolved) {
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                let dist = levenshtein_distance(&comp_str, &name);
                                // For short strings, only allow distance 1. For longer strings, allow distance 2.
                                let allowed_dist = std::cmp::min(2, comp_str.len() / 2);
                                if dist <= allowed_dist && dist < best_dist {
                                    best_dist = dist;
                                    best_match = Some(name);
                                }
                            }
                        }
                    }

                    if let Some(matched) = best_match {
                        current_resolved = current_resolved.join(matched);
                    } else {
                        current_resolved = next_path;
                    }
                }
            }
            std::path::Component::ParentDir => {
                // Ensure we do not pop beyond project_root
                if current_resolved.starts_with(project_root) && current_resolved != project_root {
                    current_resolved.pop();
                }
            }
            std::path::Component::CurDir => {}
            _ => {}
        }
    }

    current_resolved.canonicalize().unwrap_or(current_resolved)
}

fn get_str_arg(args: &serde_json::Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Missing required argument: {}", key))
}

fn get_optional_str_arg(args: &serde_json::Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_optional_usize_arg(args: &serde_json::Value, key: &str) -> Option<usize> {
    args.get(key)
        .and_then(|value| value.as_u64())
        .and_then(|value| usize::try_from(value).ok())
}

// ── Individual Tool Implementations ─────────────────────────────────────

fn load_graph(root: &Path) -> Result<crate::semantic_graph::TemporalGraph> {
    let global_brain = crate::config::get_global_brain_path(root);
    crate::semantic_graph::TemporalGraph::load(&global_brain.join("temporal_graph.json"))
        .map_err(|e| anyhow!("Temporal graph not available (run :index first): {}", e))
}

fn exec_get_file_owners(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path = get_str_arg(&call.arguments, "path")?;
    let graph = load_graph(project_root)?;
    if let Some(history) = graph.file_timeline(&path) {
        let mut report = format!(
            "File: {}\nPrimary Owner: {}\nTotal Commits: {}\nAuthors:\n",
            history.path,
            history.primary_owner.as_deref().unwrap_or("unknown"),
            history.total_changes
        );
        for a in &history.authors {
            report.push_str(&format!(
                "  - {} ({} commits, {:.1}%)\n",
                a.name, a.commit_count, a.percentage
            ));
        }
        Ok(report)
    } else {
        Ok(format!("Could not find temporal history for {}", path))
    }
}

fn exec_get_hidden_coupling(project_root: &Path) -> Result<String> {
    let graph = load_graph(project_root)?;
    Ok(graph.coupling_report())
}

fn exec_get_file_timeline(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path = get_str_arg(&call.arguments, "path")?;
    let graph = load_graph(project_root)?;
    if let Some(report) = graph.why_report(&path) {
        Ok(report)
    } else {
        Ok(format!("No timeline available for {}", path))
    }
}

fn exec_get_semantic_concepts(project_root: &Path) -> Result<String> {
    let global_brain = crate::config::get_global_brain_path(project_root);
    let concepts = crate::semantic_memory::ConceptStore::load(&global_brain.join("concepts.json"))
        .map_err(|e| anyhow!("Concepts not available (run :analyze first): {}", e))?;
    Ok(format!(
        "{}\n\n{}",
        concepts.concepts_report(),
        concepts.watches_report()
    ))
}

fn exec_inspect_project(project_root: &Path) -> Result<String> {
    let skip = [
        ".git",
        ".astra",
        ".codex",
        "node_modules",
        "target",
        ".next",
    ];
    let mut entries = Vec::new();
    for entry in fs::read_dir(project_root)?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        let suffix = if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            "/"
        } else {
            ""
        };
        entries.push(format!("{}{}", name, suffix));
    }
    entries.sort();
    entries.truncate(30);

    let manifest_names = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "Dockerfile",
        "docker-compose.yml",
    ];
    let manifests = manifest_names
        .iter()
        .filter(|name| project_root.join(name).exists())
        .copied()
        .collect::<Vec<_>>();

    let git_status = Command::new("git")
        .args(["status", "--short", "--branch"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|status| !status.is_empty())
        .map(|status| {
            let lines = status.lines().collect::<Vec<_>>();
            let mut compact = lines
                .iter()
                .take(40)
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            if lines.len() > 40 {
                compact.push_str(&format!("\n... and {} more changes", lines.len() - 40));
            }
            compact
        })
        .unwrap_or_else(|| "not a git repository or git unavailable".to_string());

    Ok(format!(
        "Project snapshot\nRoot: {}\nManifests: {}\nTop level (max 30): {}\nGit:\n{}",
        project_root.display(),
        if manifests.is_empty() {
            "none detected".to_string()
        } else {
            manifests.join(", ")
        },
        if entries.is_empty() {
            "empty".to_string()
        } else {
            entries.join(", ")
        },
        git_status
    ))
}

fn exec_read_file(call: &ToolCall, project_root: &Path) -> Result<String> {
    let path_str = get_str_arg(&call.arguments, "path")?;
    let path = resolve_path(&path_str, project_root);

    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    let content = fs::read_to_string(&path)?;
    let start_line = get_optional_usize_arg(&call.arguments, "start_line");
    let end_line = get_optional_usize_arg(&call.arguments, "end_line");

    if start_line.is_some() || end_line.is_some() {
        let lines = content.lines().collect::<Vec<_>>();
        let start = start_line.unwrap_or(1).max(1);
        if start > lines.len().max(1) {
            return Err(anyhow!(
                "start_line {} is beyond {} lines in {}",
                start,
                lines.len(),
                path.display()
            ));
        }
        let end = end_line
            .unwrap_or(start.saturating_add(299))
            .max(start)
            .min(start.saturating_add(299))
            .min(lines.len());
        let selected = lines[start.saturating_sub(1)..end]
            .iter()
            .enumerate()
            .map(|(offset, line)| format!("{:>5} | {}", start + offset, line))
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(format!(
            "{} (lines {}-{} of {})\n{}",
            path.display(),
            start,
            end,
            lines.len(),
            selected
        ));
    }

    let char_count = content.chars().count();
    if char_count > 12_000 {
        let truncated = content.chars().take(12_000).collect::<String>();
        return Ok(format!(
            "{}\n\n... [TRUNCATED — {} characters; use start_line/end_line for targeted reads]",
            truncated, char_count
        ));
    }
    Ok(content)
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
    Ok(format!(
        "✅ Wrote {} bytes to {}",
        content.len(),
        path.display()
    ))
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
    let mut modified = original.clone();

    if original.contains(&search) {
        modified = original.replacen(&search, &replace, 1);
    } else {
        // Fallback: Fuzzy whitespace-agnostic line match
        let search_lines: Vec<&str> = search
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let orig_lines: Vec<&str> = original.lines().collect();
        let orig_trimmed: Vec<&str> = orig_lines.iter().map(|l| l.trim()).collect();

        let mut match_found = false;
        if !search_lines.is_empty() {
            for i in 0..=orig_trimmed.len().saturating_sub(search_lines.len()) {
                let mut matches = true;
                let mut s_idx = 0;
                let mut j_offset = 0;

                while s_idx < search_lines.len() && (i + j_offset) < orig_trimmed.len() {
                    if orig_trimmed[i + j_offset].is_empty() {
                        j_offset += 1;
                        continue; // skip blank lines in original
                    }
                    if search_lines[s_idx] != orig_trimmed[i + j_offset] {
                        matches = false;
                        break;
                    }
                    s_idx += 1;
                    j_offset += 1;
                }

                if matches && s_idx == search_lines.len() {
                    let mut new_lines = Vec::new();
                    new_lines.extend_from_slice(&orig_lines[..i]);
                    new_lines.push(replace.as_str());
                    if i + j_offset < orig_lines.len() {
                        new_lines.extend_from_slice(&orig_lines[(i + j_offset)..]);
                    }
                    modified = new_lines.join("\n");
                    match_found = true;
                    break;
                }
            }
        }

        if !match_found {
            return Err(anyhow!(
                 "Search string not found in {}. Exact and whitespace-agnostic matching both failed.",
                 path.display()
             ));
        }
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
        "node_modules",
        "target",
        ".git",
        "__pycache__",
        ".venv",
        "venv",
        "dist",
        "build",
        ".next",
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
        "node_modules",
        "target",
        ".git",
        "__pycache__",
        ".venv",
        "venv",
        "dist",
        "build",
        ".next",
        ".astra",
        ".codex",
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
                            matches.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
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
        eprintln!(
            "  ⚠️  Agent wants to DELETE directory and all contents: {}",
            path.display()
        );
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
        eprintln!(
            "  ⚠️  Agent wants to move/rename: {} -> {}",
            src.display(),
            dst.display()
        );
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

#[cfg(test)]
mod tests {
    use super::{exec_inspect_project, exec_read_file, ToolCall};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("astra_tools_test_{unique}"));
        fs::create_dir_all(&root).expect("create test root");
        root
    }

    #[test]
    fn project_snapshot_is_compact_and_detects_manifests() {
        let root = temp_root();
        fs::write(root.join("Cargo.toml"), "[package]\nname='demo'\n").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let snapshot = exec_inspect_project(&root).unwrap();

        assert!(snapshot.contains("Cargo.toml"));
        assert!(snapshot.contains("src/"));
        assert!(snapshot.len() < 4_000);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn read_file_supports_targeted_line_ranges() {
        let root = temp_root();
        fs::write(root.join("sample.txt"), "one\ntwo\nthree\nfour\n").unwrap();
        let call = ToolCall {
            name: "read_file".to_string(),
            arguments: serde_json::json!({
                "path": "sample.txt",
                "start_line": 2,
                "end_line": 3
            }),
        };

        let output = exec_read_file(&call, &root).unwrap();

        assert!(output.contains("2 | two"));
        assert!(output.contains("3 | three"));
        assert!(!output.contains("4 | four"));
        let _ = fs::remove_dir_all(root);
    }
}
