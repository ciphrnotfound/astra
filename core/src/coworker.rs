//! Lightweight cross-editor job coordination for Astra.
//!
//! Every editor gets its own MCP connection, but all of them share the small
//! JSON job records in `.astra/cowork/jobs`. Astra remains the source of truth;
//! Codex, Claude Code, Cursor, or another MCP client can claim and report work.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoworkJobStatus {
    Queued,
    Claimed,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkJob {
    pub id: String,
    pub goal: String,
    pub acceptance: Vec<String>,
    pub preferred_worker: Option<String>,
    pub claimed_by: Option<String>,
    pub status: CoworkJobStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub summary: Option<String>,
    pub files_changed: Vec<String>,
    pub verification: Vec<String>,
}

impl CoworkJob {
    pub fn worker_prompt(&self) -> String {
        let acceptance = if self.acceptance.is_empty() {
            "- Implement the goal completely.\n- Follow existing project patterns.\n- Run relevant verification.".to_string()
        } else {
            self.acceptance
                .iter()
                .map(|item| format!("- {}", item))
                .collect::<Vec<_>>()
                .join("\n")
        };
        format!(
            "ASTRA COWORK JOB {id}\n\nGoal: {goal}\n\nAcceptance criteria:\n{acceptance}\n\nWorkflow:\n1. Call `astra_project_context` with a focused query before editing.\n2. Inspect the relevant files and preserve unrelated user changes.\n3. Implement the goal and run appropriate tests/checks.\n4. Call `astra_report_job` with this job ID, a concise summary, changed files, and verification evidence.\n\nDo not claim completion without verification.",
            id = self.id,
            goal = self.goal,
            acceptance = acceptance,
        )
    }
}

pub struct CoworkStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoworkSetupReport {
    pub configured: Vec<String>,
    pub executable: String,
    pub project_root: String,
}

impl CoworkSetupReport {
    pub fn render(&self) -> String {
        format!(
            "Astra coworker is configured for Codex, Claude Code, and Cursor.\n\nCreated or updated:\n- {}\n\nEach editor can now use Astra's MCP tools and the shared `.astra/cowork/jobs` board. Restart or reload the editors so they discover the server.",
            self.configured.join("\n- ")
        )
    }
}

/// Install project-scoped stdio MCP configuration while preserving unrelated
/// MCP servers already configured by the user.
pub fn install_editor_bridges(root: &Path, executable: &Path) -> Result<CoworkSetupReport> {
    let executable = executable
        .canonicalize()
        .unwrap_or_else(|_| executable.to_path_buf());
    let project_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let command = editor_path_string(&executable);
    let project = editor_path_string(&project_root);
    let server = json!({
        "type": "stdio",
        "command": command,
        "args": ["--root", project, "--mcp"]
    });

    let cursor_path = root.join(".cursor").join("mcp.json");
    merge_json_mcp_server(&cursor_path, &server)?;

    let claude_path = root.join(".mcp.json");
    merge_json_mcp_server(&claude_path, &server)?;

    let codex_path = root.join(".codex").join("config.toml");
    upsert_codex_mcp_server(&codex_path, &command, &project)?;

    let guide_path = root.join(".astra").join("cowork").join("README.md");
    if let Some(parent) = guide_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&guide_path, cowork_guide())?;

    Ok(CoworkSetupReport {
        configured: vec![
            relative_display(root, &cursor_path),
            relative_display(root, &claude_path),
            relative_display(root, &codex_path),
            relative_display(root, &guide_path),
        ],
        executable: command,
        project_root: project,
    })
}

fn merge_json_mcp_server(path: &Path, server: &Value) -> Result<()> {
    let mut document = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(path)?)
            .map_err(|error| anyhow!("Cannot update {}: {}", path.display(), error))?
    } else {
        json!({})
    };
    let root_object = document
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} must contain a JSON object.", path.display()))?;
    let servers = root_object
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    let servers = servers
        .as_object_mut()
        .ok_or_else(|| anyhow!("{}.mcpServers must be a JSON object.", path.display()))?;
    servers.insert("astra".to_string(), server.clone());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&document)?)?;
    Ok(())
}

fn upsert_codex_mcp_server(path: &Path, executable: &str, project_root: &str) -> Result<()> {
    const START: &str = "# ASTRA COWORKER MCP START";
    const END: &str = "# ASTRA COWORKER MCP END";
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let preserved = remove_marked_block(&existing, START, END);
    let command = serde_json::to_string(executable)?;
    let root = serde_json::to_string(project_root)?;
    let block = format!(
        "{START}\n[mcp_servers.astra]\ncommand = {command}\nargs = [\"--root\", {root}, \"--mcp\"]\n{END}\n"
    );
    let mut output = preserved.trim_end().to_string();
    if !output.is_empty() {
        output.push_str("\n\n");
    }
    output.push_str(&block);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, output)?;
    Ok(())
}

fn remove_marked_block(content: &str, start: &str, end: &str) -> String {
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in content.lines() {
        if line.trim() == start {
            skipping = true;
            continue;
        }
        if line.trim() == end {
            skipping = false;
            continue;
        }
        if !skipping {
            kept.push(line);
        }
    }
    kept.join("\n")
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn editor_path_string(path: &Path) -> String {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{}", rest);
    }
    value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
}

fn cowork_guide() -> &'static str {
    "# Astra Coworker\n\nAstra is the persistent project brain. Connected MCP workers (Codex, Claude Code, Cursor, or another client) share this workflow:\n\n1. `astra_claim_job` to receive assigned work.\n2. `astra_project_context` before editing.\n3. Implement and verify locally.\n4. `astra_report_job` with changed files and verification evidence.\n5. `astra_remember_decision` for durable architectural decisions.\n\nJobs are small JSON records in `jobs/`; large transcripts and code are deliberately not copied into them.\n"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerCommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkerRun {
    pub worker: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub output: String,
}

/// Build a documented non-interactive invocation. Arguments are passed
/// directly to the process and never interpreted by a shell.
pub fn worker_command_spec(worker: &str, prompt: &str) -> Result<WorkerCommandSpec> {
    match worker.trim().to_ascii_lowercase().as_str() {
        "codex" => Ok(WorkerCommandSpec {
            program: "codex".to_string(),
            args: vec![
                "exec".to_string(),
                "--sandbox".to_string(),
                "workspace-write".to_string(),
                prompt.to_string(),
            ],
        }),
        "claude" | "claude-code" => Ok(WorkerCommandSpec {
            program: "claude".to_string(),
            args: vec![
                "-p".to_string(),
                prompt.to_string(),
                "--permission-mode".to_string(),
                "acceptEdits".to_string(),
            ],
        }),
        "cursor" | "cursor-agent" => Ok(WorkerCommandSpec {
            program: "agent".to_string(),
            args: vec![
                "--print".to_string(),
                "--force".to_string(),
                prompt.to_string(),
            ],
        }),
        other => Err(anyhow!(
            "Unsupported worker '{}'. Use codex, claude, or cursor.",
            other
        )),
    }
}

pub fn dispatch_worker(root: &Path, worker: &str, prompt: &str) -> Result<WorkerRun> {
    let spec = worker_command_spec(worker, prompt)?;
    let output = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(root)
        .output()
        .map_err(|error| {
            anyhow!(
                "Could not start '{}' for {}: {}. Install and sign in to that editor CLI first.",
                spec.program,
                worker,
                error
            )
        })?;
    let mut combined = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() && !output.status.success() {
        if !combined.is_empty() {
            combined.push_str("\n\n");
        }
        combined.push_str(&stderr);
    }
    Ok(WorkerRun {
        worker: worker.to_ascii_lowercase(),
        success: output.status.success(),
        exit_code: output.status.code(),
        output: truncate_chars(&combined, 8_000),
    })
}

impl CoworkStore {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn create_job(
        &self,
        goal: &str,
        preferred_worker: Option<&str>,
        acceptance: Vec<String>,
    ) -> Result<CoworkJob> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(anyhow!("A cowork job needs a non-empty goal."));
        }
        let now = now_millis();
        let job = CoworkJob {
            id: format!("astra-job-{}", now_nanos()),
            goal: truncate_chars(goal, 2_000),
            acceptance: acceptance
                .into_iter()
                .map(|item| truncate_chars(item.trim(), 500))
                .filter(|item| !item.is_empty())
                .take(12)
                .collect(),
            preferred_worker: normalize_worker(preferred_worker),
            claimed_by: None,
            status: CoworkJobStatus::Queued,
            created_at: now,
            updated_at: now,
            summary: None,
            files_changed: Vec::new(),
            verification: Vec::new(),
        };
        self.save(&job)?;
        Ok(job)
    }

    pub fn claim_next(&self, worker: &str) -> Result<Option<CoworkJob>> {
        let _lock = self.acquire_lock()?;
        let worker = normalize_worker(Some(worker)).unwrap_or_else(|| "unknown".to_string());
        let mut jobs = self.list(200)?;
        jobs.sort_by_key(|job| job.created_at);
        if let Some(mut job) = jobs.into_iter().find(|job| {
            job.status == CoworkJobStatus::Queued
                && job
                    .preferred_worker
                    .as_ref()
                    .map(|preferred| preferred == "any" || preferred == &worker)
                    .unwrap_or(true)
        }) {
            job.status = CoworkJobStatus::Claimed;
            job.claimed_by = Some(worker);
            job.updated_at = now_millis();
            self.save(&job)?;
            return Ok(Some(job));
        }
        Ok(None)
    }

    pub fn claim(&self, job_id: &str, worker: &str) -> Result<CoworkJob> {
        let _lock = self.acquire_lock()?;
        let worker = normalize_worker(Some(worker)).unwrap_or_else(|| "unknown".to_string());
        let mut job = self
            .get(job_id)?
            .ok_or_else(|| anyhow!("Cowork job '{}' was not found.", job_id))?;
        if job.status != CoworkJobStatus::Queued {
            return Err(anyhow!(
                "Cowork job '{}' cannot be claimed from status {:?}.",
                job_id,
                job.status
            ));
        }
        if let Some(preferred) = &job.preferred_worker {
            if preferred != "any" && preferred != &worker {
                return Err(anyhow!(
                    "Cowork job '{}' is assigned to '{}', not '{}'.",
                    job_id,
                    preferred,
                    worker
                ));
            }
        }
        job.status = CoworkJobStatus::Claimed;
        job.claimed_by = Some(worker);
        job.updated_at = now_millis();
        self.save(&job)?;
        Ok(job)
    }

    pub fn report(
        &self,
        job_id: &str,
        worker: Option<&str>,
        status: CoworkJobStatus,
        summary: &str,
        files_changed: Vec<String>,
        verification: Vec<String>,
    ) -> Result<CoworkJob> {
        let _lock = self.acquire_lock()?;
        let mut job = self
            .get(job_id)?
            .ok_or_else(|| anyhow!("Cowork job '{}' was not found.", job_id))?;
        if status == CoworkJobStatus::Completed && verification.is_empty() {
            return Err(anyhow!(
                "Completion requires verification evidence (for example a passing test or build)."
            ));
        }
        if summary.trim().is_empty() {
            return Err(anyhow!("A job report needs a concise summary."));
        }
        if let Some(worker) = normalize_worker(worker) {
            if let Some(claimed_by) = &job.claimed_by {
                if claimed_by != &worker {
                    return Err(anyhow!(
                        "Job '{}' is claimed by '{}', not '{}'.",
                        job_id,
                        claimed_by,
                        worker
                    ));
                }
            } else {
                job.claimed_by = Some(worker);
            }
        }
        job.status = status;
        job.summary = Some(truncate_chars(summary.trim(), 2_000));
        job.files_changed = files_changed
            .into_iter()
            .map(|path| truncate_chars(path.trim(), 500))
            .filter(|path| !path.is_empty())
            .take(100)
            .collect();
        job.verification = verification
            .into_iter()
            .map(|item| truncate_chars(item.trim(), 700))
            .filter(|item| !item.is_empty())
            .take(20)
            .collect();
        job.updated_at = now_millis();
        self.save(&job)?;
        Ok(job)
    }

    pub fn get(&self, job_id: &str) -> Result<Option<CoworkJob>> {
        let safe_id = sanitize_id(job_id);
        if safe_id.is_empty() || safe_id != job_id {
            return Ok(None);
        }
        let path = self.jobs_dir().join(format!("{}.json", safe_id));
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).ok())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<CoworkJob>> {
        let dir = self.jobs_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut jobs = Vec::new();
        for entry in fs::read_dir(dir)?.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(job) = serde_json::from_str::<CoworkJob>(&content) {
                    jobs.push(job);
                }
            }
        }
        jobs.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        jobs.truncate(limit.min(200));
        Ok(jobs)
    }

    fn jobs_dir(&self) -> PathBuf {
        self.root.join(".astra").join("cowork").join("jobs")
    }

    fn acquire_lock(&self) -> Result<CoworkLock> {
        let cowork_dir = self.root.join(".astra").join("cowork");
        fs::create_dir_all(&cowork_dir)?;
        let path = cowork_dir.join("jobs.lock");
        for _ in 0..40 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(CoworkLock { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = fs::metadata(&path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                        .and_then(|modified| modified.elapsed().ok())
                        .map(|elapsed| elapsed.as_secs() > 30)
                        .unwrap_or(false);
                    if stale {
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                    thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(anyhow!(
            "The Astra cowork job board is busy; retry the operation."
        ))
    }

    fn save(&self, job: &CoworkJob) -> Result<()> {
        let dir = self.jobs_dir();
        fs::create_dir_all(&dir)?;
        let data = serde_json::to_string_pretty(job)?;
        fs::write(dir.join(format!("{}.json", sanitize_id(&job.id))), data)?;
        Ok(())
    }
}

struct CoworkLock {
    path: PathBuf,
}

impl Drop for CoworkLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn normalize_worker(worker: Option<&str>) -> Option<String> {
    worker
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .map(|value| sanitize_id(&value))
        .filter(|value| !value.is_empty())
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(120)
        .collect()
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut output = value
        .chars()
        .take(limit.saturating_sub(16))
        .collect::<String>();
    output.push_str("... [truncated]");
    output
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{install_editor_bridges, worker_command_spec, CoworkJobStatus, CoworkStore};
    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn store() -> CoworkStore {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("astra_cowork_test_{}", unique));
        fs::create_dir_all(&root).unwrap();
        CoworkStore::new(&root)
    }

    #[test]
    fn preferred_worker_claims_job_and_receives_compact_prompt() {
        let store = store();
        let created = store
            .create_job(
                "Ship OAuth login",
                Some("codex"),
                vec!["Login flow has tests".to_string()],
            )
            .unwrap();
        assert!(store.claim_next("cursor").unwrap().is_none());

        let claimed = store.claim_next("codex").unwrap().unwrap();
        assert_eq!(claimed.id, created.id);
        assert_eq!(claimed.status, CoworkJobStatus::Claimed);
        assert!(claimed.worker_prompt().contains("astra_project_context"));
        assert!(claimed.worker_prompt().contains("astra_report_job"));
    }

    #[test]
    fn completed_job_requires_verification_evidence() {
        let store = store();
        let job = store
            .create_job("Fix checkout", Some("claude"), Vec::new())
            .unwrap();
        store.claim_next("claude").unwrap();
        let error = store
            .report(
                &job.id,
                Some("claude"),
                CoworkJobStatus::Completed,
                "Implemented the fix",
                vec!["src/checkout.rs".to_string()],
                Vec::new(),
            )
            .unwrap_err();
        assert!(error.to_string().contains("verification evidence"));
    }

    #[test]
    fn installer_preserves_existing_mcp_servers_and_codex_settings() {
        let store = store();
        let root = &store.root;
        fs::create_dir_all(root.join(".cursor")).unwrap();
        fs::write(
            root.join(".cursor").join("mcp.json"),
            r#"{"mcpServers":{"existing":{"command":"keep-me"}}}"#,
        )
        .unwrap();
        fs::create_dir_all(root.join(".codex")).unwrap();
        fs::write(
            root.join(".codex").join("config.toml"),
            "model = \"keep-me\"\n",
        )
        .unwrap();

        install_editor_bridges(root, Path::new("astra-cli.exe")).unwrap();
        install_editor_bridges(root, Path::new("astra-cli.exe")).unwrap();

        let cursor: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(".cursor").join("mcp.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cursor["mcpServers"]["existing"]["command"], "keep-me");
        assert_eq!(cursor["mcpServers"]["astra"]["type"], "stdio");
        assert!(!cursor["mcpServers"]["astra"]["command"]
            .as_str()
            .unwrap()
            .starts_with(r"\\?\"));
        let codex = fs::read_to_string(root.join(".codex").join("config.toml")).unwrap();
        assert!(codex.contains("model = \"keep-me\""));
        assert_eq!(codex.matches("[mcp_servers.astra]").count(), 1);
        assert!(root.join(".mcp.json").exists());
    }

    #[test]
    fn worker_adapters_use_non_interactive_cli_modes() {
        let codex = worker_command_spec("codex", "ship it").unwrap();
        assert_eq!(codex.program, "codex");
        assert_eq!(codex.args[0], "exec");
        assert!(codex.args.contains(&"workspace-write".to_string()));

        let claude = worker_command_spec("claude", "ship it").unwrap();
        assert_eq!(claude.args[0], "-p");
        assert!(claude.args.contains(&"acceptEdits".to_string()));

        let cursor = worker_command_spec("cursor", "ship it").unwrap();
        assert_eq!(cursor.program, "agent");
        assert!(cursor.args.contains(&"--print".to_string()));
    }

    #[test]
    fn concurrent_workers_cannot_claim_the_same_job() {
        let store = store();
        let root = store.root.clone();
        store
            .create_job("One worker only", Some("any"), Vec::new())
            .unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let handles = ["codex", "claude"].map(|worker| {
            let root = root.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                CoworkStore::new(&root)
                    .claim_next(worker)
                    .unwrap()
                    .is_some()
            })
        });
        barrier.wait();
        let claims = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|claimed| *claimed)
            .count();
        assert_eq!(claims, 1);
    }
}
