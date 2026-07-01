use std::fmt::Write as FmtWrite;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::git::GitRepo;

// ────────────────────────────────────────────────────────────────
//  NODE TYPES
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperNode {
    pub name: String,
    pub email: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub commit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitNode {
    pub id: String,
    pub summary: String,
    pub author: String,
    pub timestamp: i64,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistory {
    pub path: String,
    pub commits: Vec<CommitNode>,
    pub authors: Vec<AuthorContribution>,
    pub primary_owner: Option<String>,
    pub last_touched: i64,
    pub staleness_days: u64,
    pub total_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorContribution {
    pub name: String,
    pub commit_count: usize,
    pub percentage: f32,
    pub first_commit: i64,
    pub last_commit: i64,
}

// ────────────────────────────────────────────────────────────────
//  CO-CHANGE (HIDDEN COUPLING)
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoChangePair {
    pub file_a: String,
    pub file_b: String,
    pub co_change_count: usize,
    pub coupling_score: f32, // 0.0 – 1.0
}

// ────────────────────────────────────────────────────────────────
//  THE TEMPORAL GRAPH
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TemporalGraph {
    pub developers: HashMap<String, DeveloperNode>,
    pub commits: Vec<CommitNode>,
    pub file_histories: HashMap<String, FileHistory>,
    pub co_changes: Vec<CoChangePair>,
    pub enriched: bool,
    /// Watermark: the most recent commit ID we've already processed.
    /// Used by `enrich_incremental` to skip already-known history.
    #[serde(default)]
    pub last_commit_id: Option<String>,
}

impl TemporalGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Walk the git history and build the temporal dimension.
    /// This uses ZERO Gemini API calls — pure git2 analysis.
    pub fn enrich_from_git(&mut self, git: &GitRepo) {
        self.developers.clear();
        self.commits.clear();
        self.file_histories.clear();
        self.co_changes.clear();

        let commits = match git.all_commits() {
            Ok(c) => c,
            Err(_) => return,
        };

        for commit_summary in &commits {
            // 1. Create/update DeveloperNode
            let dev = self
                .developers
                .entry(commit_summary.author.clone())
                .or_insert_with(|| DeveloperNode {
                    name: commit_summary.author.clone(),
                    email: None,
                    first_seen: commit_summary.time,
                    last_seen: commit_summary.time,
                    commit_count: 0,
                });
            dev.commit_count += 1;
            if commit_summary.time < dev.first_seen {
                dev.first_seen = commit_summary.time;
            }
            if commit_summary.time > dev.last_seen {
                dev.last_seen = commit_summary.time;
            }

            // 2. Create CommitNode (files_changed populated later)
            self.commits.push(CommitNode {
                id: commit_summary.id.clone(),
                summary: commit_summary.summary.clone(),
                author: commit_summary.author.clone(),
                timestamp: commit_summary.time,
                files_changed: Vec::new(),
            });
        }

        // 3. Populate files_changed for each commit using git diff
        // We'll use the git log --name-only approach for efficiency
        self.populate_commit_files(git);

        // 4. Build per-file histories from the commit data
        self.build_file_histories();

        // 5. Detect co-change patterns
        self.detect_co_changes(3); // minimum 3 co-changes to be significant

        // Set watermark to the latest commit
        if let Some(latest) = commits.first() {
            self.last_commit_id = Some(latest.id.clone());
        }

        self.enriched = true;
    }

    /// Incremental enrichment: only process commits newer than our watermark.
    /// Falls back to full enrichment if no watermark exists.
    pub fn enrich_incremental(&mut self, git: &GitRepo) {
        let last_known = match &self.last_commit_id {
            Some(id) => id.clone(),
            None => {
                // No watermark — do a full build
                self.enrich_from_git(git);
                return;
            }
        };

        let all_commits = match git.all_commits() {
            Ok(c) => c,
            Err(_) => return,
        };

        if self.commits.len() < all_commits.len() {
            self.enrich_from_git(git);
            return;
        }

        // Find commits newer than our watermark
        let new_commits: Vec<_> = all_commits
            .iter()
            .take_while(|c| c.id != last_known)
            .collect();

        if new_commits.is_empty() {
            return; // Nothing new since last index
        }

        // Process only the new commits
        for commit_summary in &new_commits {
            let dev = self
                .developers
                .entry(commit_summary.author.clone())
                .or_insert_with(|| DeveloperNode {
                    name: commit_summary.author.clone(),
                    email: None,
                    first_seen: commit_summary.time,
                    last_seen: commit_summary.time,
                    commit_count: 0,
                });
            dev.commit_count += 1;
            if commit_summary.time < dev.first_seen {
                dev.first_seen = commit_summary.time;
            }
            if commit_summary.time > dev.last_seen {
                dev.last_seen = commit_summary.time;
            }

            self.commits.push(CommitNode {
                id: commit_summary.id.clone(),
                summary: commit_summary.summary.clone(),
                author: commit_summary.author.clone(),
                timestamp: commit_summary.time,
                files_changed: Vec::new(),
            });
        }

        // Re-populate file info and rebuild derived data
        self.populate_commit_files(git);
        self.build_file_histories();
        self.detect_co_changes(3);

        // Update watermark
        if let Some(latest) = all_commits.first() {
            self.last_commit_id = Some(latest.id.clone());
        }

        self.enriched = true;
    }

    /// Use `git log --name-only` to find which files each commit touched.
    fn populate_commit_files(&mut self, git: &GitRepo) {
        let output = std::process::Command::new("git")
            .current_dir(git.root_path())
            .args(&[
                "log",
                "--name-only",
                "--format=%H",
            ])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return,
        };

        let mut current_hash: Option<String> = None;
        let mut commit_files: HashMap<String, Vec<String>> = HashMap::new();

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Full SHA hashes are 40 chars of hex
            if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                current_hash = Some(trimmed[..8].to_string()); // We store short hashes
            } else if let Some(ref hash) = current_hash {
                commit_files
                    .entry(hash.clone())
                    .or_default()
                    .push(trimmed.to_string());
            }
        }

        // Map back to our commit nodes
        for commit in &mut self.commits {
            if let Some(files) = commit_files.get(&commit.id) {
                commit.files_changed = files.clone();
            }
        }
    }

    /// Build per-file histories from all commits.
    fn build_file_histories(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut file_commits: HashMap<String, Vec<&CommitNode>> = HashMap::new();

        for commit in &self.commits {
            for file in &commit.files_changed {
                file_commits.entry(file.clone()).or_default().push(commit);
            }
        }

        for (path, commits) in &file_commits {
            // Count contributions per author
            let mut author_counts: HashMap<String, (usize, i64, i64)> = HashMap::new();
            for c in commits {
                let entry = author_counts.entry(c.author.clone()).or_insert((0, c.timestamp, c.timestamp));
                entry.0 += 1;
                if c.timestamp < entry.1 {
                    entry.1 = c.timestamp;
                }
                if c.timestamp > entry.2 {
                    entry.2 = c.timestamp;
                }
            }

            let total_commits = commits.len();
            let mut authors: Vec<AuthorContribution> = author_counts
                .into_iter()
                .map(|(name, (count, first, last))| AuthorContribution {
                    name,
                    commit_count: count,
                    percentage: (count as f32 / total_commits as f32) * 100.0,
                    first_commit: first,
                    last_commit: last,
                })
                .collect();

            // Sort by commit count descending
            authors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

            let primary_owner = authors.first().map(|a| a.name.clone());
            let last_touched = commits.iter().map(|c| c.timestamp).max().unwrap_or(0);

            let staleness_days = if last_touched > 0 {
                ((now - last_touched) / 86400) as u64
            } else {
                0
            };

            let commit_nodes: Vec<CommitNode> = commits
                .iter()
                .map(|c| (*c).clone())
                .collect();

            self.file_histories.insert(
                path.clone(),
                FileHistory {
                    path: path.clone(),
                    commits: commit_nodes,
                    authors,
                    primary_owner,
                    last_touched,
                    staleness_days,
                    total_changes: total_commits,
                },
            );
        }
    }

    /// Detect files that are always changed together (hidden coupling).
    /// `min_co_changes` is the minimum number of times two files must
    /// appear in the same commit to be considered "coupled."
    pub fn detect_co_changes(&mut self, min_co_changes: usize) {
        let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();

        for commit in &self.commits {
            let files = &commit.files_changed;
            for i in 0..files.len() {
                for j in (i + 1)..files.len() {
                    let mut a = files[i].clone();
                    let mut b = files[j].clone();
                    // Canonical ordering
                    if a > b {
                        std::mem::swap(&mut a, &mut b);
                    }
                    *pair_counts.entry((a, b)).or_insert(0) += 1;
                }
            }
        }

        self.co_changes = pair_counts
            .into_iter()
            .filter(|(_, count)| *count >= min_co_changes)
            .map(|((file_a, file_b), count)| {
                // Coupling score: co-change count / max individual change count
                let a_changes = self
                    .file_histories
                    .get(&file_a)
                    .map(|h| h.total_changes)
                    .unwrap_or(1);
                let b_changes = self
                    .file_histories
                    .get(&file_b)
                    .map(|h| h.total_changes)
                    .unwrap_or(1);
                let max_changes = a_changes.max(b_changes).max(1);
                let coupling_score = count as f32 / max_changes as f32;

                CoChangePair {
                    file_a,
                    file_b,
                    co_change_count: count,
                    coupling_score: coupling_score.min(1.0),
                }
            })
            .collect();

        // Sort by coupling score descending
        self.co_changes
            .sort_by(|a, b| b.coupling_score.partial_cmp(&a.coupling_score).unwrap_or(std::cmp::Ordering::Equal));
    }

    // ────────────────────────────────────────────────────────────
    //  QUERY METHODS
    // ────────────────────────────────────────────────────────────

    /// Get the full timeline for a specific file.
    pub fn file_timeline(&self, path: &str) -> Option<&FileHistory> {
        // Try exact match first
        if let Some(h) = self.file_histories.get(path) {
            return Some(h);
        }
        // Try partial match (e.g., "engine.rs" matches "core/src/engine.rs")
        for (key, history) in &self.file_histories {
            if key.ends_with(path) || key.contains(path) {
                return Some(history);
            }
        }
        None
    }

    /// Get ownership information for all files.
    pub fn ownership_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "📊 File Ownership Report");
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        // Group files by primary owner
        let mut by_owner: HashMap<String, Vec<&FileHistory>> = HashMap::new();
        for history in self.file_histories.values() {
            let owner = history
                .primary_owner
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            by_owner.entry(owner).or_default().push(history);
        }

        // Sort owners by number of owned files
        let mut owners: Vec<_> = by_owner.into_iter().collect();
        owners.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (owner, files) in &owners {
            let _ = writeln!(&mut out, "\n👤 {} ({} files)", owner, files.len());
            // Show top 10 files per owner
            for file in files.iter().take(10) {
                let stale_indicator = if file.staleness_days > 30 {
                    format!(" ⚠️ stale {}d", file.staleness_days)
                } else {
                    String::new()
                };
                let _ = writeln!(
                    &mut out,
                    "   {} ({} commits, {:.0}% ownership{})",
                    file.path,
                    file.total_changes,
                    file.authors.first().map(|a| a.percentage).unwrap_or(0.0),
                    stale_indicator
                );
            }
            if files.len() > 10 {
                let _ = writeln!(&mut out, "   ... and {} more", files.len() - 10);
            }
        }

        out
    }

    pub fn project_history_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "📚 Project History Report");
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        if self.commits.is_empty() {
            let _ = writeln!(&mut out, "\nNo git history has been indexed yet.");
            return out;
        }

        let first_commit = self.commits.iter().min_by_key(|commit| commit.timestamp);
        let latest_commit = self.commits.iter().max_by_key(|commit| commit.timestamp);
        let recent_window = latest_commit
            .map(|commit| commit.timestamp - 90 * 86_400)
            .unwrap_or(0);
        let recent_commits = self
            .commits
            .iter()
            .filter(|commit| commit.timestamp >= recent_window)
            .count();

        let _ = writeln!(
            &mut out,
            "\nTimeline: {} total commits, {} developers, {} tracked files",
            self.commits.len(),
            self.developers.len(),
            self.file_histories.len()
        );
        if let Some(first) = first_commit {
            let date = chrono::DateTime::from_timestamp(first.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let _ = writeln!(
                &mut out,
                "First indexed commit: {} — {} ({})",
                date,
                first.summary,
                first.author
            );
        }
        if let Some(latest) = latest_commit {
            let date = chrono::DateTime::from_timestamp(latest.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let _ = writeln!(
                &mut out,
                "Latest indexed commit: {} — {} ({})",
                date,
                latest.summary,
                latest.author
            );
        }
        let _ = writeln!(
            &mut out,
            "Recent momentum: {} commits in roughly the last 90 days",
            recent_commits
        );

        let mut top_devs = self.developers.values().collect::<Vec<_>>();
        top_devs.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        if !top_devs.is_empty() {
            let _ = writeln!(&mut out, "\nTop contributors:");
            for dev in top_devs.into_iter().take(5) {
                let _ = writeln!(
                    &mut out,
                    "- {}: {} commits",
                    dev.name,
                    dev.commit_count
                );
            }
        }

        let mut hotspots = self.file_histories.values().collect::<Vec<_>>();
        hotspots.sort_by(|a, b| b.total_changes.cmp(&a.total_changes));
        if !hotspots.is_empty() {
            let _ = writeln!(&mut out, "\nChange hotspots:");
            for file in hotspots.into_iter().take(5) {
                let _ = writeln!(
                    &mut out,
                    "- {} ({} commits, owner: {}, stale: {}d)",
                    file.path,
                    file.total_changes,
                    file.primary_owner.as_deref().unwrap_or("unknown"),
                    file.staleness_days
                );
            }
        }

        if !self.co_changes.is_empty() {
            let _ = writeln!(&mut out, "\nStrong coupling pairs:");
            for pair in self.co_changes.iter().take(5) {
                let _ = writeln!(
                    &mut out,
                    "- {} <-> {} ({} co-changes, {:.0}% coupling)",
                    pair.file_a,
                    pair.file_b,
                    pair.co_change_count,
                    pair.coupling_score * 100.0
                );
            }
        }

        out
    }

    pub fn hotspot_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "🔥 Change Hotspots");
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        if self.file_histories.is_empty() {
            let _ = writeln!(&mut out, "\nNo file history has been indexed yet.");
            return out;
        }

        let mut hotspots = self.file_histories.values().collect::<Vec<_>>();
        hotspots.sort_by(|a, b| {
            b.total_changes
                .cmp(&a.total_changes)
                .then_with(|| a.staleness_days.cmp(&b.staleness_days))
        });

        let _ = writeln!(&mut out, "\nMost changed files:");
        for file in hotspots.iter().take(10) {
            let author_count = file.authors.len();
            let _ = writeln!(
                &mut out,
                "- {} (commits={}, owner={}, authors={}, stale={}d)",
                file.path,
                file.total_changes,
                file.primary_owner.as_deref().unwrap_or("unknown"),
                author_count,
                file.staleness_days
            );
        }

        if !self.co_changes.is_empty() {
            let _ = writeln!(&mut out, "\nFiles that move together:");
            for pair in self.co_changes.iter().take(5) {
                let _ = writeln!(
                    &mut out,
                    "- {} <-> {} ({} co-changes, {:.0}% coupling)",
                    pair.file_a,
                    pair.file_b,
                    pair.co_change_count,
                    pair.coupling_score * 100.0
                );
            }
        }

        out
    }

    /// Get the coupling report (files that secretly change together).
    pub fn coupling_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "🔗 Hidden Coupling Report");
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        if self.co_changes.is_empty() {
            let _ = writeln!(&mut out, "\nNo significant coupling detected yet.");
            let _ = writeln!(
                &mut out,
                "This improves with more git history. Run :index after more commits."
            );
            return out;
        }

        for (i, pair) in self.co_changes.iter().take(20).enumerate() {
            let urgency = if pair.coupling_score > 0.7 {
                "🔴"
            } else if pair.coupling_score > 0.4 {
                "🟡"
            } else {
                "🟢"
            };
            let _ = writeln!(
                &mut out,
                "\n{}. {} {} ↔ {}",
                i + 1,
                urgency,
                pair.file_a,
                pair.file_b
            );
            let _ = writeln!(
                &mut out,
                "   Co-changed {} times (coupling: {:.0}%)",
                pair.co_change_count,
                pair.coupling_score * 100.0
            );
        }

        if self.co_changes.len() > 20 {
            let _ = writeln!(
                &mut out,
                "\n... and {} more pairs",
                self.co_changes.len() - 20
            );
        }

        out
    }

    /// The killer demo: `astra why <file>`
    /// Returns a full structured story about a file.
    pub fn why_report(&self, path: &str) -> Option<String> {
        let history = self.file_timeline(path)?;

        let mut out = String::new();
        let _ = writeln!(&mut out, "📖 Story of: {}", history.path);
        let _ = writeln!(
            &mut out,
            "════════════════════════════════════════"
        );

        // Ownership
        let _ = writeln!(&mut out, "\n👥 Authors ({} total):", history.authors.len());
        for author in &history.authors {
            let is_primary = history
                .primary_owner
                .as_ref()
                .map(|o| o == &author.name)
                .unwrap_or(false);
            let badge = if is_primary { " 👑 primary" } else { "" };
            let _ = writeln!(
                &mut out,
                "   {} — {} commits ({:.0}%){badge}",
                author.name, author.commit_count, author.percentage
            );
        }

        // Staleness
        let _ = writeln!(&mut out, "\n⏱️ Last touched: {} days ago", history.staleness_days);
        if history.staleness_days > 60 {
            let _ = writeln!(&mut out, "   ⚠️ This file is becoming stale. Consider a review.");
        }

        // Commit timeline (last 10)
        let _ = writeln!(&mut out, "\n📅 Recent History (last 10 changes):");
        for commit in history.commits.iter().rev().take(10) {
            let date = chrono::DateTime::from_timestamp(commit.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let _ = writeln!(
                &mut out,
                "   {} — {} (by {})",
                date, commit.summary, commit.author
            );
        }

        // Hidden coupling
        let coupled: Vec<&CoChangePair> = self
            .co_changes
            .iter()
            .filter(|p| {
                p.file_a.contains(path)
                    || p.file_b.contains(path)
                    || history.path == p.file_a
                    || history.path == p.file_b
            })
            .collect();

        if !coupled.is_empty() {
            let _ = writeln!(&mut out, "\n🔗 Hidden Coupling:");
            for pair in coupled.iter().take(5) {
                let other = if pair.file_a == history.path || pair.file_a.contains(path) {
                    &pair.file_b
                } else {
                    &pair.file_a
                };
                let _ = writeln!(
                    &mut out,
                    "   Always changes with {} ({} times, {:.0}% coupling)",
                    other, pair.co_change_count, pair.coupling_score * 100.0
                );
            }
        }

        Some(out)
    }

    /// Save the temporal graph to disk.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let data = serde_json::to_string(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Load a temporal graph from disk.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let graph: Self = serde_json::from_str(&data)?;
        Ok(graph)
    }
}
