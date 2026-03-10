// TODO: use std::fmt::Write;
// TODO: use std::fs;
// TODO: use std::path::Path;

// TODO: use crate::index::CodeIndex;
// TODO: use crate::memory::{HealthScores, MemoryEvent, MemoryStore};
// TODO: use crate::teams::TeamManager;

export interface HealthReport {
  scores: any;
  prev_scores: any;
  details: any;
}


export interface HealthDetails {
  todo_count: number;
  total_lines: number;
  test_files: number;
  total_files: number;
  language_count: number;
  migration_count: number;
  security_files: number;
  uncommitted_changes: number;
  recent_commits: number;
  tasks_done: number;
  tasks_total: number;
}


// const SECURITY_KEYWORDS: &[&str] = &[
// "password", "secret", "token", "auth", "credential",
// "api_key", "apikey", "private_key", "jwt",
// ];

// const CODE_LANGUAGES: &[&str] = &[
// "rust", "typescript", "javascript", "python", "go", "java",
// "c", "cpp", "csharp", "ruby", "swift", "kotlin", "scala",
// ];

// pub fn compute_health(

// root: &Path,
// index: &CodeIndex,
// memory: &MemoryStore,
// team_mgr: Option<&TeamManager>,
// ) -> HealthReport {
// let stats = index.stats();
// let files = index.files();
// let by_lang = index.files_by_language();

// // --- Code Quality: scan for TODO/FIXME/HACK ---
// let mut todo_count: usize = 0;
// for (path, _summary) in files {
// let abs = if path.is_absolute() {
// path.clone()
// } else {
// root.join(path)
// };
// if let Ok(contents) = fs::read_to_string(&abs) {
// for line in contents.lines() {
// let upper = line.to_uppercase();
// if upper.contains("TODO") || upper.contains("FIXME") || upper.contains("HACK") {
// todo_count += 1;
// }
// }
// }
// }
// let todo_ratio = if stats.total_lines > 0 {
// todo_count as f64 / stats.total_lines as f64
// } else {
// 0.0
// };
// let code_quality = (100.0 - todo_ratio * 500.0).clamp(0.0, 100.0) as u32;

// // --- Test Health: ratio of test files ---
// let mut test_files: usize = 0;
// for path in files.keys() {
// let name = path.to_string_lossy().to_lowercase();
// if name.contains("test") || name.contains("spec") || name.contains("tests") {
// test_files += 1;
// }
// }
// let test_ratio = if stats.file_count > 0 {
// test_files as f64 / stats.file_count as f64
// } else {
// 0.0
// };
// let test_health = (test_ratio * 500.0).clamp(0.0, 100.0) as u32;

// // --- Cross-Lang Drift (only count real code languages) ---
// let language_count = by_lang.keys()
// .filter(|lang| CODE_LANGUAGES.contains(&lang.as_str()))
// .count();
// let migration_events = memory.events_of_kind("migration");
// let migration_count = migration_events.len();
// let drift_penalty = (language_count as i32 - 1).max(0) * 15;
// let migration_bonus = (migration_count as i32) * 5;
// let cross_lang_drift = (100 - drift_penalty + migration_bonus).clamp(0, 100) as u32;

// // --- Security Surface ---
// let mut security_files: usize = 0;
// for (path, _) in files {
// let abs = if path.is_absolute() {
// path.clone()
// } else {
// root.join(path)
// };
// if let Ok(contents) = fs::read_to_string(&abs) {
// let lower = contents.to_lowercase();
// if SECURITY_KEYWORDS.iter().any(|kw| lower.contains(kw)) {
// security_files += 1;
// }
// }
// }
// let sec_ratio = if stats.file_count > 0 {
// security_files as f64 / stats.file_count as f64
// } else {
// 0.0
// };
// let security_surface = (100.0 - sec_ratio * 200.0).clamp(0.0, 100.0) as u32;

// // --- Git Health ---
// let mut uncommitted_changes: usize = 0;
// let mut recent_commits: usize = 0;
// if let Ok(repo) = crate::git::GitRepo::discover(root) {
// uncommitted_changes = repo.uncommitted_file_count();
// recent_commits = repo.recent_commit_count(30);
// }
// let commit_score = (recent_commits as f64 * 3.3).clamp(0.0, 100.0);
// let uncommitted_penalty = (uncommitted_changes as f64 * 2.0).clamp(0.0, 40.0);
// let git_health = (commit_score - uncommitted_penalty).clamp(0.0, 100.0) as u32;

// // --- Team Velocity ---
// let mut tasks_done: usize = 0;
// let mut tasks_total: usize = 0;
// if let Some(mgr) = team_mgr {
// if let Ok(state) = mgr.load_state() {
// tasks_total = state.tasks.len();
// tasks_done = state
// .tasks
// .values()
// .filter(|t| t.status == crate::teams::TaskStatus::Done)
// .count();
// }
// }
// let team_velocity = if tasks_total > 0 {
// ((tasks_done as f64 / tasks_total as f64) * 100.0) as u32
// } else {
// 50 // neutral when no team data
// };

// let scores = HealthScores {
// code_quality,
// test_health,
// cross_lang_drift,
// security_surface,
// git_health,
// team_velocity,
// };

// // --- Load previous snapshot for trend ---
// let prev_scores = memory.latest_event("health").and_then(|entry| {
// if let Some(MemoryEvent::HealthSnapshot { scores: ref s }) = entry.event {
// Some(s.clone())
// } else {
// None
// }
// });

// HealthReport {
// scores,
// prev_scores,
// details: HealthDetails {
// todo_count,
// total_lines: stats.total_lines,
// test_files,
// total_files: stats.file_count,
// language_count,
// migration_count,
// security_files,
// uncommitted_changes,
// recent_commits,
// tasks_done,
// tasks_total,
// },
// }
// }

export function trend_arrow(current: number, prev: number): any {
  // if current > prev + 3 {
  // "▲"
  // } else if current + 3 < prev {
  // "▼"
  // } else {
  // "━"
  // }
}


export function trend_delta(current: number, prev: number): string {
  // let diff = current as i32 - prev as i32;
  // if diff > 0 {
  // format!("(+{})", diff)
  // } else if diff < 0 {
  // format!("({})", diff)
  // } else {
  // String::new()
  // }
}


export function score_bar(score: number): any {
  // match score {
  // 90..=100 => "██████████",
  // 80..=89 => "████████░░",
  // 70..=79 => "███████░░░",
  // 60..=69 => "██████░░░░",
  // 50..=59 => "█████░░░░░",
  // 40..=49 => "████░░░░░░",
  // 30..=39 => "███░░░░░░░",
  // 20..=29 => "██░░░░░░░░",
  // 10..=19 => "█░░░░░░░░░",
  // _ => "░░░░░░░░░░",
  // }
}


// impl HealthReport {
export function render(&self: any): string {
  // let mut out = String::new();
  // let _ = writeln!(&mut out, "╔══════════════════════════════════════════════════════╗");
  // let _ = writeln!(&mut out, "║            CODEBASE HEALTH REPORT                   ║");
  // let _ = writeln!(&mut out, "╠══════════════════════════════════════════════════════╣");
  // 
  // let metrics = [
  // ("Code Quality", self.scores.code_quality),
  // ("Test Health", self.scores.test_health),
  // ("Cross-Lang Drift", self.scores.cross_lang_drift),
  // ("Security Surface", self.scores.security_surface),
  // ("Git Health", self.scores.git_health),
  // ("Team Velocity", self.scores.team_velocity),
  // ];
  // 
  // let prev_vals: Option<[u32; 6]> = self.prev_scores.as_ref().map(|p| {
  // [
  // p.code_quality,
  // p.test_health,
  // p.cross_lang_drift,
  // p.security_surface,
  // p.git_health,
  // p.team_velocity,
  // ]
  // });
  // 
  // for (i, (name, score)) in metrics.iter().enumerate() {
  // let bar = score_bar(*score);
  // let trend = if let Some(ref prev) = prev_vals {
  // let arrow = trend_arrow(*score, prev[i]);
  // let delta = trend_delta(*score, prev[i]);
  // format!(" {} {}", arrow, delta)
  // } else {
  // String::new()
  // };
  // let _ = writeln!(
  // &mut out,
  // "║  {:<18} {:>3}/100  {} {}",
  // name, score, bar, trend
  // );
  // }
  // 
  // let _ = writeln!(&mut out, "╠══════════════════════════════════════════════════════╣");
  // let _ = writeln!(&mut out, "║  TOP ITEMS TO FIX THIS WEEK                         ║");
  // let _ = writeln!(&mut out, "╠══════════════════════════════════════════════════════╣");
  // 
  // let mut suggestions: Vec<(u32, String)> = Vec::new();
  // 
  // if self.scores.test_health < 60 {
  // suggestions.push((
  // self.scores.test_health,
  // format!(
  // "Add tests — only {}/{} files are test files",
  // self.details.test_files, self.details.total_files
  // ),
  // ));
  // }
  // if self.scores.code_quality < 80 {
  // suggestions.push((
  // self.scores.code_quality,
  // format!(
  // "Resolve {} TODO/FIXME/HACK comments across {} lines",
  // self.details.todo_count, self.details.total_lines
  // ),
  // ));
  // }
  // if self.scores.security_surface < 70 {
  // suggestions.push((
  // self.scores.security_surface,
  // format!(
  // "{} files touch auth/secrets/tokens — audit them",
  // self.details.security_files
  // ),
  // ));
  // }
  // if self.scores.git_health < 60 && self.details.uncommitted_changes > 0 {
  // suggestions.push((
  // self.scores.git_health,
  // format!(
  // "{} uncommitted changes — commit or stash them",
  // self.details.uncommitted_changes
  // ),
  // ));
  // }
  // if self.scores.git_health < 60 && self.details.recent_commits < 5 {
  // suggestions.push((
  // self.scores.git_health,
  // "Low commit frequency — commit smaller, more often".to_string(),
  // ));
  // }
  // if self.scores.cross_lang_drift < 60 {
  // suggestions.push((
  // self.scores.cross_lang_drift,
  // format!(
  // "{} languages detected — consider consolidating via migration",
  // self.details.language_count
  // ),
  // ));
  // }
  // if self.scores.team_velocity < 50 {
  // suggestions.push((
  // self.scores.team_velocity,
  // format!(
  // "Only {}/{} tasks completed — push to close open items",
  // self.details.tasks_done, self.details.tasks_total
  // ),
  // ));
  // }
  // 
  // suggestions.sort_by_key(|(score, _)| *score);
  // 
  // if suggestions.is_empty() {
  // let _ = writeln!(&mut out, "║  ✅ Looking good! No urgent items.                   ║");
  // } else {
  // for (i, (_, suggestion)) in suggestions.iter().take(3).enumerate() {
  // let _ = writeln!(&mut out, "║  {}. {}", i + 1, suggestion);
  // }
  // }
  // 
  // let _ = writeln!(&mut out, "╚══════════════════════════════════════════════════════╝");
  // out
}

// }
