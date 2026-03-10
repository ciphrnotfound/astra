use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryEntry {
    pub kind: String,
    pub content: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<MemoryEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MemoryEvent {
    IndexSnapshot {
        file_count: usize,
        total_lines: usize,
        languages: HashMap<String, usize>,
    },
    MigrationRun {
        from: String,
        to: String,
        file_count: usize,
    },
    TeamSession {
        developer: String,
        task_id: String,
        duration_secs: u64,
        lines_added: usize,
        lines_deleted: usize,
    },
    HealthSnapshot {
        scores: HealthScores,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HealthScores {
    pub code_quality: u32,
    pub test_health: u32,
    pub cross_lang_drift: u32,
    pub security_surface: u32,
    pub git_health: u32,
    pub team_velocity: u32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MemoryStore {
    pub entries: Vec<MemoryEntry>,
    #[serde(skip)]
    path: Option<PathBuf>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl MemoryStore {
    pub fn load(path: PathBuf) -> Self {
        let entries = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self {
            entries,
            path: Some(path),
        }
    }

    pub fn add(&mut self, kind: &str, content: String) {
        self.entries.push(MemoryEntry {
            kind: kind.to_string(),
            content,
            timestamp: now_secs(),
            event: None,
        });
        let _ = self.save();
    }

    pub fn add_event(&mut self, kind: &str, content: String, event: MemoryEvent) {
        self.entries.push(MemoryEntry {
            kind: kind.to_string(),
            content,
            timestamp: now_secs(),
            event: Some(event),
        });
        let _ = self.save();
    }

    pub fn recent(&self, limit: usize) -> Vec<MemoryEntry> {
        let len = self.entries.len();
        let start = len.saturating_sub(limit);
        self.entries[start..].to_vec()
    }

    pub fn events_of_kind(&self, kind: &str) -> Vec<&MemoryEntry> {
        self.entries.iter().filter(|e| e.kind == kind).collect()
    }

    pub fn latest_event(&self, kind: &str) -> Option<&MemoryEntry> {
        self.entries.iter().rev().find(|e| e.kind == kind)
    }

    pub fn events_since(&self, timestamp: u64) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= timestamp)
            .collect()
    }

    fn save(&self) -> Result<()> {
        if let Some(path) = &self.path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(&self.entries)?;
            fs::write(path, data)?;
        }
        Ok(())
    }
}
