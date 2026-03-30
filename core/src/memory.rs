use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LearningPhase {
    pub language: String,
    pub phase_number: u32,
    pub goal: String,
    pub path: String,
    pub proficiency_notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MemoryEntry {
    pub kind: String,
    pub content: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<MemoryEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    WorktreeSnapshot {
        changed_files: usize,
        files: Vec<String>,
    },
    HealthSnapshot {
        scores: HealthScores,
    },
    GitCommit {
        id: String,
        summary: String,
        author: String,
        date: String,
    },
    LearningProgress {
        phase: LearningPhase,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
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
    #[serde(skip)]
    max_entries: usize,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn default_max_entries() -> usize {
    0 // 0 denotes infinite entries
}

impl MemoryStore {
    pub fn load(path: PathBuf) -> Self {
        let entries = if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut store = Self {
            entries,
            path: Some(path),
            max_entries: default_max_entries(),
        };
        store.trim_to_max();
        store
    }

    pub fn add(&mut self, kind: &str, mut content: String) {
        if content.len() > 5000 {
            content = content.chars().take(5000).collect::<String>();
            content.push_str("... [TRUNCATED]");
        }
        self.push_entry(MemoryEntry {
            kind: kind.to_string(),
            content,
            timestamp: now_secs(),
            event: None,
        });
    }

    pub fn add_event(&mut self, kind: &str, mut content: String, event: MemoryEvent) {
        if content.len() > 5000 {
            content = content.chars().take(5000).collect::<String>();
            content.push_str("... [TRUNCATED]");
        }
        self.push_entry(MemoryEntry {
            kind: kind.to_string(),
            content,
            timestamp: now_secs(),
            event: Some(event),
        });
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

    pub fn search(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let trimmed = query.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Vec::new();
        }

        // Tokenize query, skipping common stop words
        let stop_words = ["what", "is", "my", "the", "a", "an", "for", "from", "now", "on", "was", "did", "how", "why", "where"];
        let query_tokens: Vec<&str> = trimmed
            .split_whitespace()
            .filter(|w| !stop_words.contains(w) && w.len() > 1)
            .collect();

        if query_tokens.is_empty() {
            // Fallback to basic substring if only stop words were provided
            let needle = trimmed;
            let mut matches = Vec::new();
            for entry in self.entries.iter().rev() {
                if entry.content.to_ascii_lowercase().contains(&needle) {
                    matches.push(entry.clone());
                    if matches.len() >= limit { break; }
                }
            }
            return matches;
        }

        // Rank by match count
        let mut scored_matches: Vec<(usize, MemoryEntry)> = self.entries.iter().rev().map(|entry| {
            let content_lower = entry.content.to_ascii_lowercase();
            let mut score = 0;
            for token in &query_tokens {
                if content_lower.contains(token) {
                    score += 1;
                }
            }
            (score, entry.clone())
        }).filter(|(score, _)| *score > 0).collect();

        // Sort by score (descending)
        scored_matches.sort_by(|a, b| b.0.cmp(&a.0));

        scored_matches.into_iter().take(limit).map(|(_, e)| e).collect()
    }

    pub fn compact_noise(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|entry| !matches!(
            entry.kind.as_str(),
            "qa" | "qa-memory" | "web-search" | "web-knowledge" | "autonomous-action"
        ));
        let removed = before.saturating_sub(self.entries.len());
        if removed > 0 {
            let _ = self.save();
        }
        removed
    }

    fn push_entry(&mut self, entry: MemoryEntry) {
        if self.is_duplicate_of_last(&entry) {
            return;
        }
        self.entries.push(entry);
        self.trim_to_max();
        let _ = self.save();
    }

    fn trim_to_max(&mut self) {
        if self.max_entries == 0 {
            return;
        }
        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.drain(0..excess);
        }
    }

    fn is_duplicate_of_last(&self, entry: &MemoryEntry) -> bool {
        match self.entries.last() {
            Some(last) => last.kind == entry.kind
                && last.content == entry.content
                && last.event == entry.event,
            None => false,
        }
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
