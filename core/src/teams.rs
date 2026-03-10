use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::git::GitRepo;

/// Teams configuration and state.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TeamState {
    pub team_name: String,
    pub api_key: String,
    pub tasks: HashMap<String, Task>,
    pub sessions: Vec<Session>,
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
}

pub struct TeamManager {
    state_path: PathBuf,
    repo_path: PathBuf,
}

impl TeamManager {
    pub fn new(repo_path: &Path) -> Self {
        let state_path = repo_path.join(".codex").join("teams.json");
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
        Ok(state)
    }

    /// Save the team state to disk.
    fn save_state(&self, state: &TeamState) -> Result<()> {
        if let Some(parent) = self.state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_path, content)?;
        Ok(())
    }

    /// Initialize a new team.
    pub fn init_team(&self, name: &str, api_key: &str) -> Result<()> {
        let state = TeamState {
            team_name: name.to_string(),
            api_key: api_key.to_string(),
            tasks: HashMap::new(),
            sessions: Vec::new(),
        };
        self.save_state(&state)
    }

    /// Assign a task to a developer.
    pub fn assign_task(&self, task_id: &str, description: &str, assignee: &str) -> Result<()> {
        let mut state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized. Run `codex team init` first."));
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

    /// Start working on a task.
    pub fn start_task(&self, task_id: &str, developer: &str) -> Result<()> {
        let mut state = self.load_state()?;
        
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
        if task.assignee != developer {
            return Err(anyhow!("Task is assigned to {}, not {}.", task.assignee, developer));
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
        });

        self.save_state(&state)
    }

    /// Finish working on a task and calculate productivity metrics.
    pub fn finish_task(&self, task_id: &str, developer: &str) -> Result<Session> {
        let mut state = self.load_state()?;
        
        let task = state.tasks.get_mut(task_id).ok_or_else(|| anyhow!("Task not found."))?;
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

        Ok(session)
    }

    /// Generate an end-of-week productivity report.
    pub fn generate_report(&self) -> Result<String> {
        let state = self.load_state()?;
        if state.team_name.is_empty() {
            return Err(anyhow!("Team not initialized."));
        }

        let mut report = format!("=== Team Report: {} ===\n", state.team_name);
        
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
        for (id, task) in &state.tasks {
            report.push_str(&format!("[{:?}] {}: {} (Assigned to {})\n", task.status, id, task.description, task.assignee));
        }

        Ok(report)
    }
}
