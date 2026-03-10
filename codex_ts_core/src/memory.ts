// TODO: use std::collections::HashMap;
// TODO: use std::fs;
// TODO: use std::path::PathBuf;
// TODO: use std::time::{SystemTime, UNIX_EPOCH};

// TODO: use anyhow::Result;
// TODO: use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Clone, Debug)]
export interface MemoryEntry {
  kind: string;
  content: string;
  timestamp: number;
  #[serde(default, skip_serializing_if = "Option: any;
  event: any;
}


// #[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "type")]
export type MemoryEvent =
  | "file_count: usize"
  | "total_lines: usize"
  | "languages: HashMap<String, usize>";

// MigrationRun {
// from: String,
// to: String,
// file_count: usize,
// },
// TeamSession {
// developer: String,
// task_id: String,
// duration_secs: u64,
// lines_added: usize,
// lines_deleted: usize,
// },
// HealthSnapshot {
// scores: HealthScores,
// },
// }

// #[derive(Serialize, Deserialize, Clone, Debug, Default)]
export interface HealthScores {
  code_quality: number;
  test_health: number;
  cross_lang_drift: number;
  security_surface: number;
  git_health: number;
  team_velocity: number;
}


// #[derive(Serialize, Deserialize, Default, Clone)]
export interface MemoryStore {
  entries: any[];
  path: any;
}


export function now_secs(): number {
  // SystemTime::now()
  // .duration_since(UNIX_EPOCH)
  // .map(|d| d.as_secs())
  // .unwrap_or(0)
}


// impl MemoryStore {
export function load(path: any): any {
  // let entries = if let Ok(data) = fs::read_to_string(&path) {
  // serde_json::from_str(&data).unwrap_or_default()
  // } else {
  // Vec::new()
  // };
  // Self {
  // entries,
  // path: Some(path),
  // }
}


export function add(&mut self: any, kind: string, content: string): void {
  // self.entries.push(MemoryEntry {
  // kind: kind.to_string(),
  // content,
  // timestamp: now_secs(),
  // event: None,
  // });
  // let _ = self.save();
}


export function add_event(&mut self: any, kind: string, content: string, event: any): void {
  // self.entries.push(MemoryEntry {
  // kind: kind.to_string(),
  // content,
  // timestamp: now_secs(),
  // event: Some(event),
  // });
  // let _ = self.save();
}


export function recent(&self: any, limit: number): any[] {
  // let len = self.entries.len();
  // let start = len.saturating_sub(limit);
  // self.entries[start..].to_vec()
}


export function events_of_kind(&self: any, kind: string): any[] {
  // self.entries.iter().filter(|e| e.kind == kind).collect()
}


export function latest_event(&self: any, kind: string): any {
  // self.entries.iter().rev().find(|e| e.kind == kind)
}


export function events_since(&self: any, timestamp: number): any[] {
  // self.entries
  // .iter()
  // .filter(|e| e.timestamp >= timestamp)
  // .collect()
}


export function save(&self: any): any {
  // if let Some(path) = &self.path {
  // if let Some(parent) = path.parent() {
  // fs::create_dir_all(parent)?;
  // }
  // let data = serde_json::to_string_pretty(&self.entries)?;
  // fs::write(path, data)?;
  // }
  // Ok(())
}

// }
