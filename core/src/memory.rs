use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::git::GitRepo;
use crate::model::EmbeddingProvider;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
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
    #[serde(skip)]
    pub global: Option<Box<MemoryStore>>,
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
            global: None,
        };
        store.trim_to_max();
        store
    }

    /// Attach a global memory store (loaded from ~/.astra/memory.json).
    pub fn attach_global(&mut self, global: MemoryStore) {
        self.global = Some(Box::new(global));
    }

    /// Returns the global memory path: ~/.astra/memory.json
    pub fn global_memory_path() -> Option<PathBuf> {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .ok()?;
        Some(PathBuf::from(home).join(".astra").join("memory.json"))
    }

    /// Load the global memory store from ~/.astra/memory.json.
    pub fn load_global() -> Self {
        if let Some(path) = Self::global_memory_path() {
            Self::load(path)
        } else {
            Self::default()
        }
    }

    /// Check if this is the very first run (no global memory file exists).
    pub fn is_first_run() -> bool {
        match Self::global_memory_path() {
            Some(path) => !path.exists(),
            None => true,
        }
    }

    /// Add a fact to global memory (identity, preferences).
    pub fn add_global(&mut self, kind: &str, content: String) {
        if let Some(global) = self.global.as_mut() {
            global.add(kind, content);
        }
    }

    /// Get the user's stored name from global memory, if any.
    pub fn user_name(&self) -> Option<String> {
        if let Some(name) = self.user_identity("name") {
            return Some(name);
        }
        if let Some(global) = &self.global {
            if let Some(entry) = global.entries.iter().rev().find(|e| e.kind == "user-identity") {
                if let Some(name) = entry.content.strip_prefix("name: ") {
                    return Some(name.to_string());
                }
            }
            // Fallback: check "fact" entries that mention name
            for entry in global.entries.iter().rev() {
                if entry.kind == "fact" {
                    let lower = entry.content.to_ascii_lowercase();
                    if lower.starts_with("my name is ") {
                        let name = entry.content["my name is ".len()..].trim().to_string();
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }
        }
        // Also check local memory
        for entry in self.entries.iter().rev() {
            if entry.kind == "user-identity" {
                if let Some(name) = entry.content.strip_prefix("name: ") {
                    return Some(name.to_string());
                }
            }
            if entry.kind == "fact" {
                let lower = entry.content.to_ascii_lowercase();
                if lower.starts_with("my name is ") {
                    let name = entry.content["my name is ".len()..].trim().to_string();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    pub fn user_identity(&self, key: &str) -> Option<String> {
        let normalized = key.trim().to_ascii_lowercase();
        if let Some(global) = &self.global {
            for entry in global.entries.iter().rev() {
                if entry.kind == "user-identity" {
                    if let Some((entry_key, value)) = parse_key_value(&entry.content) {
                        if entry_key == normalized {
                            return Some(value);
                        }
                    }
                }
            }
        }
        for entry in self.entries.iter().rev() {
            if entry.kind == "user-identity" {
                if let Some((entry_key, value)) = parse_key_value(&entry.content) {
                    if entry_key == normalized {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    pub fn user_preference(&self, key: &str) -> Option<String> {
        let normalized = key.trim().to_ascii_lowercase();
        if let Some(global) = &self.global {
            for entry in global.entries.iter().rev() {
                if entry.kind == "user-preference" {
                    if let Some((entry_key, value)) = parse_key_value(&entry.content) {
                        if entry_key == normalized {
                            return Some(value);
                        }
                    }
                }
            }
        }
        for entry in self.entries.iter().rev() {
            if entry.kind == "user-preference" {
                if let Some((entry_key, value)) = parse_key_value(&entry.content) {
                    if entry_key == normalized {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    pub fn remember_user_identity(&mut self, key: &str, value: &str) {
        self.upsert_global_keyed("user-identity", key, value);
    }

    pub fn remember_user_preference(&mut self, key: &str, value: &str) {
        self.upsert_global_keyed("user-preference", key, value);
    }

    pub fn remember_project_fact(&mut self, key: &str, value: &str) {
        self.upsert_local_keyed("project-fact", key, value);
    }

    pub fn remember_style_fact(&mut self, key: &str, value: &str) {
        self.upsert_local_keyed("style-memory", key, value);
    }

    pub fn user_facts(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(global) = &self.global {
            collect_keyed_entries(&global.entries, "user-identity", &mut out);
            collect_keyed_entries(&global.entries, "user-preference", &mut out);
        }
        collect_keyed_entries(&self.entries, "user-identity", &mut out);
        collect_keyed_entries(&self.entries, "user-preference", &mut out);
        dedupe_keyed_pairs(out)
    }

    pub fn project_facts(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        collect_keyed_entries(&self.entries, "project-fact", &mut out);
        dedupe_keyed_pairs(out)
    }

    pub fn style_facts(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        collect_keyed_entries(&self.entries, "style-memory", &mut out);
        dedupe_keyed_pairs(out)
    }

    pub fn profile_report(&self) -> String {
        let facts = self.user_facts();
        if facts.is_empty() {
            return "No user profile facts stored yet.".to_string();
        }
        let mut out = String::from("User profile memory:\n");
        for (key, value) in facts {
            out.push_str(&format!("- {}: {}\n", key, value));
        }
        out.trim_end().to_string()
    }

    pub fn project_report(&self) -> String {
        let facts = self.project_facts();
        let styles = self.style_facts();
        if facts.is_empty() && styles.is_empty() {
            return "No project memory facts stored yet.".to_string();
        }
        let mut out = String::from("Project memory:\n");
        for (key, value) in facts {
            out.push_str(&format!("- {}: {}\n", key, value));
        }
        if !styles.is_empty() {
            out.push_str("Style memory:\n");
            for (key, value) in styles {
                out.push_str(&format!("- {}: {}\n", key, value));
            }
        }
        out.trim_end().to_string()
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
            embedding: None,
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
            embedding: None,
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

    pub fn search(&self, query: &str, query_embedding: Option<&[f32]>, limit: usize) -> Vec<MemoryEntry> {
        let trimmed = query.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let now = now_secs();

        // Collect all entries: local + global
        let mut all_entries: Vec<&MemoryEntry> = self.entries.iter().collect();
        if let Some(global) = &self.global {
            all_entries.extend(global.entries.iter());
        }

        // Tokenize query, skipping common stop words
        let stop_words = [
            "what", "is", "my", "the", "a", "an", "for", "from", "now", "on", "was", "did",
            "how", "why", "where", "to", "of", "in", "at", "with", "and", "or", "me", "you",
            "we", "this", "that",
        ];
        let query_tokens: Vec<&str> = trimmed
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-'))
            .filter(|w| !w.is_empty())
            .filter(|w| !stop_words.contains(w) && w.len() > 1)
            .collect();

        if query_tokens.is_empty() {
            // Fallback to basic substring if only stop words were provided
            let needle = trimmed;
            let mut matches = Vec::new();
            for entry in all_entries.iter().rev() {
                if entry.content.to_ascii_lowercase().contains(&needle) {
                    matches.push((*entry).clone());
                    if matches.len() >= limit { break; }
                }
            }
            return matches;
        }

        // Rank by match count OR cosine similarity
        let mut scored_matches: Vec<(f32, MemoryEntry)> = all_entries.iter().rev().map(|entry| {
            let mut score = 0.0;
            let content_lower = entry.content.to_ascii_lowercase();

            let age_secs = now.saturating_sub(entry.timestamp);
            let age_days = (age_secs / 86_400) as f32;
            let recency_boost = 0.25 / (1.0 + (age_days / 14.0));
            score += recency_boost;
            
            // Keyword match
            for token in &query_tokens {
                if content_lower.contains(token) {
                    score += 0.2; // base score for keywords
                }
            }
            if entry.kind == "user-identity" || entry.kind == "user-preference" || entry.kind == "project-fact" || entry.kind == "fact" {
                score += 0.5;
            }
            if entry.kind == "style-memory" {
                score += 0.35;
            }

            if let Some((key, value)) = parse_key_value(&entry.content) {
                for token in &query_tokens {
                    if key == *token {
                        score += 0.6;
                    } else if value.to_ascii_lowercase().contains(token) {
                        score += 0.25;
                    }
                }
            }

            // Vector match
            if let Some(q_vec) = query_embedding {
                if let Some(entry_vec) = &entry.embedding {
                    let sim = cosine_similarity(q_vec, entry_vec);
                    if sim > 0.6 {
                        score += sim; // boost significantly
                    }
                }
            }

            (score, (*entry).clone())
        }).filter(|(score, _)| *score > 0.0).collect();

        // Sort by score (descending)
        scored_matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored_matches.into_iter().take(limit).map(|(_, e)| e).collect()
    }

    /// Iterates over all memories without embeddings and calls the Gemini API to get them.
    /// This happens asynchronously or in a background pass so the user doesn't wait forever.
    pub fn embed_all(&mut self, embedder: &dyn EmbeddingProvider) {
        let mut updated = false;
        let mut count = 0;
        
        // Reverse because we want to embed the newest memories first
        for entry in self.entries.iter_mut().rev() {
            if entry.embedding.is_none() && entry.content.len() > 10 {
                if let Ok(vec) = embedder.get_embedding(&entry.content) {
                    entry.embedding = Some(vec);
                    updated = true;
                    count += 1;
                    if count >= 1 { // batch size limit to 1 so we don't block the REPL
                        break;
                    }
                }
            }
        }
        if updated {
            let _ = self.save();
        }
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

    fn upsert_global_keyed(&mut self, kind: &str, key: &str, value: &str) {
        if let Some(global) = self.global.as_mut() {
            global.upsert_local_keyed(kind, key, value);
        } else {
            self.upsert_local_keyed(kind, key, value);
        }
    }

    fn upsert_local_keyed(&mut self, kind: &str, key: &str, value: &str) {
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return;
        }
        let content = format!("{}: {}", key, value);
        let mut updated = false;
        if let Some(existing) = self.entries.iter_mut().rev().find(|entry| {
            entry.kind == kind
                && parse_key_value(&entry.content)
                    .map(|(existing_key, _)| existing_key == key)
                    .unwrap_or(false)
        }) {
            if existing.content != content {
                existing.content = content;
                existing.timestamp = now_secs();
                updated = true;
            }
        } else {
            self.push_entry(MemoryEntry {
                kind: kind.to_string(),
                content,
                timestamp: now_secs(),
                event: None,
                embedding: None,
            });
            return;
        }
        if updated {
            let _ = self.save();
        }
    }
}

fn parse_key_value(content: &str) -> Option<(String, String)> {
    let (key, value) = content.split_once(':')?;
    let key = key.trim().to_ascii_lowercase();
    let value = value.trim().to_string();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key, value))
    }
}

fn collect_keyed_entries(entries: &[MemoryEntry], kind: &str, out: &mut Vec<(String, String)>) {
    for entry in entries.iter().rev() {
        if entry.kind == kind {
            if let Some((key, value)) = parse_key_value(&entry.content) {
                out.push((key, value));
            }
        }
    }
}

fn dedupe_keyed_pairs(pairs: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut seen: HashMap<String, String> = HashMap::new();
    for (key, value) in pairs {
        seen.entry(key).or_insert(value);
    }
    let mut out: Vec<(String, String)> = seen.into_iter().collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (va, vb) in a.iter().zip(b.iter()) {
        dot += va * vb;
        norm_a += va * va;
        norm_b += vb * vb;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}
