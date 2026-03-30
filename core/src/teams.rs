use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::git::GitRepo;
use crate::config::get_global_config_path;

/// Teams configuration and state.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TeamState {
    pub team_name: String,
    #[serde(default)]
    pub admin_key: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub members: HashMap<String, TeamMember>,
    pub tasks: HashMap<String, Task>,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TeamRole {
    Admin,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMember {
    pub name: String,
    pub role: TeamRole,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub assignee: String,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub task_id: String,
    pub developer: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub start_commit: String,
    pub end_commit: Option<String>,
    pub lines_added: usize,
    pub lines_deleted: usize,
    #[serde(default)]
    pub prompts_asked: Vec<String>,
    #[serde(default)]
    pub files_touched: Vec<String>,
}

pub struct TeamManager {
    state_path: PathBuf,
    repo_path: PathBuf,
}

impl TeamManager {
    pub fn new(repo_path: &Path) -> Self {
        let state_path = resolve_state_path(repo_path);
        Self {
            state_path,
            repo_path: repo_path.to_path_buf(),
        }
    }

    /// Load the current team state from disk.
    pub fn load_state(&self) -> Result<TeamState> {
        if !self.state_path.exists() {
            return Ok(TeamState::default());
        }
        let content = fs::read_to_string(&self.state_path)?;
        let state: TeamState = serde_json::from_str(&content)?;
        Ok(normalize_state(state))
    }

    /// Save the team state to disk.
    fn save_state(&self, state: &TeamState) -> Result<()> {
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_path, content)?;
        
        // Asynchronously try to push the state so it doesn't block local workflow
        let _ = self.push_state();
        
        Ok(())
    }

    /// Pull and merge internal state from `origin/astra-state`.
    pub fn sync(&self) -> Result<()> {
        // 1. Fetch `astra-state` from origin (ignore err if it doesn't exist yet)
        let _ = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["fetch", "origin", "astra-state"])
            .output();

        // 2. Read `teams.json` from `origin/astra-state`
        let output = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["show", "origin/astra-state:teams.json"])
            .output()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            
            if let Ok(mut remote_state) = serde_json::from_str::<TeamState>(&content) {
                if let Ok(local_state) = self.load_state() {
                    // Simple merge: keep remote state but retain local active sessions 
                    // that haven't synchronized to remote yet.
                    for local_session in local_state.sessions {
                        if !remote_state.sessions.iter().any(|s| s.task_id == local_session.task_id && s.developer == local_session.developer) {
                            remote_state.sessions.push(local_session);
                        }
                    }
                }
                
                // Write the merged state locally bypassing save_state to avoid an infinite push loop
                if let Some(parent) = self.state_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let new_content = serde_json::to_string_pretty(&remote_state)?;
                fs::write(&self.state_path, new_content)?;
            }
        }
        
        // Update local ref
        let _ = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["update-ref", "refs/heads/astra-state", "origin/astra-state"])
            .output();

        Ok(())
    }

    /// Saves the current state to the local `astra-state` branch and pushes to origin.
    pub fn push_state(&self) -> Result<()> {
        let content = fs::read_to_string(&self.state_path)?;
        
        let mut child = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
        }
        
        let output = child.wait_with_output()?;
        if !output.status.success() {
             return Err(anyhow::anyhow!("Failed to create git blob"));
        }
        let blob_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let tree_input = format!("100644 blob {}\tteams.json\n", blob_sha);
        let mut child = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["mktree"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(tree_input.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        let tree_sha = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let mut parent_args = vec![];
        let rev_parse = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rev-parse", "refs/heads/astra-state"])
            .output()?;
            
        if rev_parse.status.success() {
            let parent_sha = String::from_utf8_lossy(&rev_parse.stdout).trim().to_string();
            parent_args.push("-p".to_string());
            parent_args.push(parent_sha);
        }

        let mut commit_args = vec!["commit-tree".to_string(), tree_sha];
        commit_args.extend(parent_args);
        commit_args.push("-m".to_string());
        commit_args.push("Update Astra team state".to_string());

        let commit_out = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&commit_args)
            .output()?;
        let commit_sha = String::from_utf8_lossy(&commit_out.stdout).trim().to_string();

        std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["update-ref", "refs/heads/astra-state", &commit_sha])
            .output()?;

        // Quietly try to push to remote (ignore error if no remote configured)
        let _ = std::process::Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["push", "origin", "astra-state"])
            .output();

        Ok(())
    }

    /// Initialize a new team.
    pub fn init_team(&self, name: &str, admin_key: Option<&str>) -> Result<String> {
        let key = admin_key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| generate_key("admin"));
        let mut members = HashMap::new();
        members.insert(
            "admin".to_string(),
            TeamMember {
                name: "admin".to_string(),
                role: TeamRole::Admin,
                key: key.clone(),
            },
        );
        let state = TeamState {
            team_name: name.to_string(),
            admin_key: key.clone(),
            api_key: key.clone(),
            members,
            tasks: HashMap::new(),
            sessions: Vec::new(),
        };
        self.save_state(&state)?;
        Ok(key)
    }

    pub fn add_member(
        &self,
        admin_key: &str,
        name: &str,
        role: TeamRole,
        member_key: Option<&str>,
    ) -> Result<String> {
        let mut state = self.load_state()?;
        self.require_admin(&state, admin_key)?;
        if state.members.contains_key(name) {
            return Err(anyhow!("Member {} already exists.", name));
        }
        let key = member_key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| generate_key("member"));
        state.members.insert(
            name.to_string(),
            TeamMember {
                name: name.to_string(),
                role,
                key: key.clone(),
            },
        );
        self.save_state(&state)?;
        Ok(key)
    }

    /// Assign a task to a developer.
    pub fn assign_task(
        &self,
        admin_key: &str,
        task_id: &str,
        description: &str,
        assignee: &str,
    ) -> Result<()> {
        let mut state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized. Run `astra team init` first."));
        }
        self.require_admin(&state, admin_key)?;
        if state.tasks.contains_key(task_id) {
            return Err(anyhow!("Task {} already exists.", task_id));
        }
        self.require_member(&state, assignee)?;

        let task = Task {
            id: task_id.to_string(),
            description: description.to_string(),
            assignee: assignee.to_string(),
            status: TaskStatus::Pending,
        };
        state.tasks.insert(task_id.to_string(), task);
        self.save_state(&state)
    }

    /// Start working on a task.
    pub fn start_task(&self, member_key: &str, task_id: &str, developer: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized. Run `astra team init` first."));
        }
        self.require_member_key(&state, developer, member_key)?;
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
        if task.assignee != developer {
            return Err(anyhow!("Task is assigned to {}, not {}.", task.assignee, developer));
        }
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Task {} is already completed.", task_id));
        }
        if state
            .sessions
            .iter()
            .any(|s| s.task_id == task_id && s.end_time.is_none())
        {
            return Err(anyhow!("Task {} already has an active session.", task_id));
        }
        task.status = TaskStatus::InProgress;

        let start_commit = GitRepo::discover(&self.repo_path)
            .and_then(|repo| repo.get_head_commit())
            .unwrap_or_else(|_| "unknown".to_string());

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Check if there's already an active session for this dev
        if state.sessions.iter().any(|s| s.developer == developer && s.end_time.is_none()) {
            return Err(anyhow!("Developer {} already has an active session.", developer));
        }

        state.sessions.push(Session {
            task_id: task_id.to_string(),
            developer: developer.to_string(),
            start_time: now,
            end_time: None,
            start_commit,
            end_commit: None,
            lines_added: 0,
            lines_deleted: 0,
            prompts_asked: Vec::new(),
            files_touched: Vec::new(),
        });

        // Dynamic Context Injection for AI Editors
        let _ = create_cursorrules(&self.repo_path, task_id, &task.description, developer);

        self.save_state(&state)
    }

    /// Finish working on a task and calculate productivity metrics.
    pub fn finish_task(
        &self,
        member_key: &str,
        task_id: &str,
        developer: &str,
    ) -> Result<Session> {
        let mut state = self.load_state()?;
        self.require_member_key(&state, developer, member_key)?;
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
        if task.assignee != developer {
            return Err(anyhow!("Task is assigned to {}, not {}.", task.assignee, developer));
        }
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Task {} is already completed.", task_id));
        }
        task.status = TaskStatus::Done;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let end_commit = GitRepo::discover(&self.repo_path)
            .and_then(|repo| repo.get_head_commit())
            .unwrap_or_else(|_| "unknown".to_string());

        let session_idx = state.sessions.iter().position(|s| s.task_id == task_id && s.developer == developer && s.end_time.is_none())
            .ok_or_else(|| anyhow!("No active session found for task {} and developer {}.", task_id, developer))?;

        let mut session = state.sessions[session_idx].clone();
        
        // Calculate lines changed by diffing start_commit and current state
        let (added, deleted) = if session.start_commit != "unknown" {
            GitRepo::discover(&self.repo_path)
                .ok()
                .and_then(|repo| repo.get_diff_stats(&session.start_commit).ok())
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        session.end_time = Some(now);
        session.end_commit = Some(end_commit.clone());
        session.lines_added = added;
        session.lines_deleted = deleted;

        state.sessions[session_idx] = session.clone();
        self.save_state(&state)?;

        // Append to offline sync queue
        let _ = enqueue_session_for_sync(&session);

        Ok(session)
    }

    /// Generate an end-of-week productivity report.
    pub fn generate_report(&self, admin_key: &str) -> Result<String> {
        let state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized."));
        }
        self.require_admin(&state, admin_key)?;

        let mut report = format!("=== Team Report: {} ===\n", state.team_name);
        let active_sessions = state
            .sessions
            .iter()
            .filter(|s| s.end_time.is_none())
            .count();
        report.push_str(&format!("Active sessions: {}\n", active_sessions));
        
        // Group sessions by developer
        let mut dev_stats: HashMap<String, (u64, usize, usize)> = HashMap::new();
        
        for session in &state.sessions {
            if let Some(end) = session.end_time {
                let duration = end.saturating_sub(session.start_time);
                let entry = dev_stats.entry(session.developer.clone()).or_insert((0, 0, 0));
                entry.0 += duration;
                entry.1 += session.lines_added;
                entry.2 += session.lines_deleted;
            }
        }

        report.push_str("\n--- Productivity Metrics ---\n");
        for (dev, (time_secs, added, deleted)) in dev_stats {
            let hours = time_secs / 3600;
            let mins = (time_secs % 3600) / 60;
            let total_changed = added + deleted;
            
            report.push_str(&format!("Developer: {}\n", dev));
            report.push_str(&format!("  Time Logged: {}h {}m\n", hours, mins));
            report.push_str(&format!("  Lines Changed: +{} -{} (Total: {})\n", added, deleted, total_changed));
            
            // Flag if time spent is high but lines changed is very low
            if hours > 2 && total_changed < 10 {
                report.push_str("  ⚠️ Warning: High time logged with very low code output.\n");
            }
            report.push('\n');
        }

        report.push_str("--- Task Status ---\n");
        let mut pending = 0usize;
        let mut in_progress = 0usize;
        let mut done = 0usize;
        for (id, task) in &state.tasks {
            match task.status {
                TaskStatus::Pending => pending += 1,
                TaskStatus::InProgress => in_progress += 1,
                TaskStatus::Done => done += 1,
            }
            report.push_str(&format!("[{:?}] {}: {} (Assigned to {})\n", task.status, id, task.description, task.assignee));
        }
        report.push_str(&format!(
            "\nSummary: {} pending, {} in progress, {} done\n",
            pending, in_progress, done
        ));

        Ok(report)
    }

    pub fn assign_task_implicit(
        &self,
        task_id: &str,
        description: &str,
        assignee: &str,
    ) -> Result<()> {
        let mut state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized. Run `astra team init` first."));
        }
        if state.tasks.contains_key(task_id) {
            return Err(anyhow!("Task {} already exists.", task_id));
        }
        let task = Task {
            id: task_id.to_string(),
            description: description.to_string(),
            assignee: assignee.to_string(),
            status: TaskStatus::Pending,
        };
        state.tasks.insert(task_id.to_string(), task);
        self.save_state(&state)
    }

    pub fn start_task_implicit(&self, task_id: &str, developer: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized. Run `astra team init` first."));
        }
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
        if task.assignee != developer {
            return Err(anyhow!("Task is assigned to {}, not {}.", task.assignee, developer));
        }
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Task {} is already completed.", task_id));
        }
        if state
            .sessions
            .iter()
            .any(|s| s.task_id == task_id && s.end_time.is_none())
        {
            return Err(anyhow!("Task {} already has an active session.", task_id));
        }
        task.status = TaskStatus::InProgress;

        let start_commit = GitRepo::discover(&self.repo_path)
            .and_then(|repo| repo.get_head_commit())
            .unwrap_or_else(|_| "unknown".to_string());

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        if state.sessions.iter().any(|s| s.developer == developer && s.end_time.is_none()) {
            return Err(anyhow!("Developer {} already has an active session.", developer));
        }

        state.sessions.push(Session {
            task_id: task_id.to_string(),
            developer: developer.to_string(),
            start_time: now,
            end_time: None,
            start_commit,
            end_commit: None,
            lines_added: 0,
            lines_deleted: 0,
            prompts_asked: Vec::new(),
            files_touched: Vec::new(),
        });

        // Dynamic Context Injection for AI Editors
        let _ = create_cursorrules(&self.repo_path, task_id, &task.description, developer);

        self.save_state(&state)
    }

    pub fn finish_task_implicit(&self, task_id: &str, developer: &str) -> Result<Session> {
        let mut state = self.load_state()?;
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
        if task.assignee != developer {
            return Err(anyhow!("Task is assigned to {}, not {}.", task.assignee, developer));
        }
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Task {} is already completed.", task_id));
        }
        task.status = TaskStatus::Done;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let end_commit = GitRepo::discover(&self.repo_path)
            .and_then(|repo| repo.get_head_commit())
            .unwrap_or_else(|_| "unknown".to_string());

        let session_idx = state.sessions.iter().position(|s| s.task_id == task_id && s.developer == developer && s.end_time.is_none())
            .ok_or_else(|| anyhow!("No active session found for task {} and developer {}.", task_id, developer))?;

        let mut session = state.sessions[session_idx].clone();

        let (added, deleted) = if session.start_commit != "unknown" {
            GitRepo::discover(&self.repo_path)
                .ok()
                .and_then(|repo| repo.get_diff_stats(&session.start_commit).ok())
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        session.end_time = Some(now);
        session.end_commit = Some(end_commit.clone());
        session.lines_added = added;
        session.lines_deleted = deleted;

        state.sessions[session_idx] = session.clone();
        self.save_state(&state)?;

        // Append to offline sync queue
        let _ = enqueue_session_for_sync(&session);

        Ok(session)
    }

    pub fn log_prompt(&self, developer: &str, prompt: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if let Some(session) = state.sessions.iter_mut().find(|s| s.developer == developer && s.end_time.is_none()) {
            session.prompts_asked.push(prompt.to_string());
            self.save_state(&state)?;
        }
        Ok(())
    }

    fn require_admin(&self, state: &TeamState, admin_key: &str) -> Result<()> {
        if state.admin_key.is_empty() {
            return Err(anyhow!("Team admin key is not set."));
        }
        if state.admin_key != admin_key {
            return Err(anyhow!("Invalid admin key."));
        }
        Ok(())
    }

    fn require_member(&self, state: &TeamState, name: &str) -> Result<()> {
        if state.members.contains_key(name) {
            Ok(())
        } else {
            Err(anyhow!("Member {} is not registered.", name))
        }
    }

    fn require_member_key(&self, state: &TeamState, name: &str, key: &str) -> Result<()> {
        let member = state
            .members
            .get(name)
            .ok_or_else(|| anyhow!("Member {} is not registered.", name))?;
        if member.key != key {
            return Err(anyhow!("Invalid key for member {}.", name));
        }
        Ok(())
    }
}

fn resolve_state_path(repo_path: &Path) -> PathBuf {
    let preferred = repo_path.join(".astra").join("teams.json");
    if preferred.exists() {
        return preferred;
    }
    let previous = repo_path.join(".forge").join("teams.json");
    if previous.exists() {
        return previous;
    }
    let legacy = repo_path.join(".codex").join("teams.json");
    if legacy.exists() {
        return legacy;
    }
    preferred
}

fn normalize_state(mut state: TeamState) -> TeamState {
    if state.admin_key.is_empty() && !state.api_key.is_empty() {
        state.admin_key = state.api_key.clone();
    }
    if state.members.is_empty() && !state.admin_key.is_empty() {
        state.members.insert(
            "admin".to_string(),
            TeamMember {
                name: "admin".to_string(),
                role: TeamRole::Admin,
                key: state.admin_key.clone(),
            },
        );
    }
    state
}

fn create_cursorrules(repo_path: &Path, task_id: &str, description: &str, assignee: &str) -> Result<()> {
    let rules_path = repo_path.join(".cursorrules");
    let content = format!(
        "# ASTRA TEAM CONTEXT\n\
         # You are assisting {} on TASK: {}\n\
         # DESCRIPTION: {}\n\n\
         # Context:\n\
         # Please keep your suggestions aligned with the team's objectives mentioned above.\n",
        assignee, task_id, description
    );
    fs::write(rules_path, content)?;
    Ok(())
}

fn generate_key(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    format!("astra_{}_{}_{}", prefix, pid, now)
}

pub fn enqueue_session_for_sync(session: &Session) -> Result<()> {
    let mut path = get_global_config_path();
    path.set_file_name("sync_queue.json");
    
    let mut queue: Vec<Session> = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    queue.push(session.clone());
    
    let content = serde_json::to_string_pretty(&queue)?;
    std::fs::write(path, content)?;
    Ok(())
}

