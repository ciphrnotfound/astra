// ─── Predictive Refactoring ─────────────────────────────────────────
// Analyzes git history and codebase patterns to proactively suggest
// refactoring actions before technical debt becomes a crisis.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::git::GitRepo;
use crate::index::CodeIndex;

/// A proactive suggestion from Astra's predictive analysis.
#[derive(Debug)]
pub struct Prediction {
    pub severity: Severity,
    pub category: PredictionCategory,
    pub message: String,
    pub affected_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug)]
pub enum PredictionCategory {
    HotFile,
    LargeFile,
    CrossLanguageDrift,
    OrphanedCode,
    DuplicatePattern,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "\u{1f4a1}"),
            Severity::Warning => write!(f, "\u{26a0}\u{fe0f}"),
            Severity::Critical => write!(f, "\u{1f6a8}"),
        }
    }
}

impl std::fmt::Display for Prediction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.severity, self.message)?;
        if !self.affected_files.is_empty() {
            write!(f, "\n    Files: ")?;
            for (i, file) in self.affected_files.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", file.display())?;
            }
        }
        Ok(())
    }
}

/// Analyze the codebase and git history to generate predictive suggestions.
pub fn analyze(
    git: Option<&GitRepo>,
    index: &CodeIndex,
    root: &Path,
) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    // 1. Hot files — files that change too frequently
    if let Some(git) = git {
        predictions.extend(detect_hot_files(git, root));
    }

    // 2. Large files — files growing beyond maintainable size
    predictions.extend(detect_large_files(index));

    // 3. Cross-language drift — same concept implemented differently
    predictions.extend(detect_cross_language_drift(index));

    // 4. Orphaned code — files with no imports/dependents
    predictions.extend(detect_orphaned_code(index));

    // Sort by severity (critical first)
    predictions.sort_by_key(|p| match p.severity {
        Severity::Critical => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    });

    predictions
}

/// Detect files that have been modified many times recently.
fn detect_hot_files(git: &GitRepo, _root: &Path) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    // Get recent commits and count file changes
    let mut file_change_count: HashMap<String, usize> = HashMap::new();
    if let Ok(log) = git.recent_log(50) {
        for line in log.lines() {
            let trimmed = line.trim();
            // Look for file paths in commit diffs
            if trimmed.ends_with(".rs")
                || trimmed.ends_with(".ts")
                || trimmed.ends_with(".py")
                || trimmed.ends_with(".go")
                || trimmed.ends_with(".java")
                || trimmed.ends_with(".js")
            {
                *file_change_count.entry(trimmed.to_string()).or_insert(0) += 1;
            }
        }
    }

    // Flag files changed more than 5 times in recent history
    let mut hot_files: Vec<(String, usize)> = file_change_count
        .into_iter()
        .filter(|(_, count)| *count >= 5)
        .collect();
    hot_files.sort_by(|a, b| b.1.cmp(&a.1));

    for (file, count) in hot_files.iter().take(5) {
        let severity = if *count >= 10 {
            Severity::Critical
        } else {
            Severity::Warning
        };

        predictions.push(Prediction {
            severity,
            category: PredictionCategory::HotFile,
            message: format!(
                "'{}' has been changed {} times recently — consider splitting or decoupling it",
                file, count
            ),
            affected_files: vec![PathBuf::from(file)],
        });
    }

    predictions
}

/// Detect files that have grown too large.
fn detect_large_files(index: &CodeIndex) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    let stats = index.all_file_stats();
    for (path, line_count, fn_count) in stats {
        if line_count > 1000 {
            predictions.push(Prediction {
                severity: Severity::Critical,
                category: PredictionCategory::LargeFile,
                message: format!(
                    "'{}' has {} lines and {} functions — urgently needs splitting",
                    path.display(), line_count, fn_count
                ),
                affected_files: vec![path],
            });
        } else if line_count > 500 {
            predictions.push(Prediction {
                severity: Severity::Warning,
                category: PredictionCategory::LargeFile,
                message: format!(
                    "'{}' has {} lines — approaching maintainability limit",
                    path.display(), line_count
                ),
                affected_files: vec![path],
            });
        }
    }

    predictions
}

/// Detect when the same concept exists in multiple languages but has drifted.
fn detect_cross_language_drift(index: &CodeIndex) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    // Group files by stem name across languages
    let mut name_groups: HashMap<String, Vec<(PathBuf, String)>> = HashMap::new();
    let by_lang = index.files_by_language();

    // Simple heuristic: files with the same stem in different languages
    for (path, line_count, _) in index.all_file_stats() {
        if line_count == 0 {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let lang = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string();
            name_groups
                .entry(stem.to_lowercase())
                .or_default()
                .push((path, lang));
        }
    }

    for (stem, files) in &name_groups {
        if files.len() >= 2 {
            let langs: Vec<&str> = files.iter().map(|(_, l)| l.as_str()).collect();
            let unique_langs: std::collections::HashSet<&str> = langs.iter().copied().collect();
            if unique_langs.len() >= 2 {
                predictions.push(Prediction {
                    severity: Severity::Warning,
                    category: PredictionCategory::CrossLanguageDrift,
                    message: format!(
                        "'{}' exists in {} languages ({}) — check for semantic drift",
                        stem,
                        unique_langs.len(),
                        unique_langs.into_iter().collect::<Vec<_>>().join(", ")
                    ),
                    affected_files: files.iter().map(|(p, _)| p.clone()).collect(),
                });
            }
        }
    }

    let _ = by_lang; // suppress unused warning

    predictions
}

/// Detect files that nothing imports (potential dead code).
fn detect_orphaned_code(index: &CodeIndex) -> Vec<Prediction> {
    let mut predictions = Vec::new();

    for (path, line_count, _) in index.all_file_stats() {
        if line_count < 20 {
            continue; // Skip tiny files
        }
        // Check if any other file imports this one
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if stem == "main" || stem == "lib" || stem == "mod" || stem == "index" {
            continue; // Entry points are never "orphaned"
        }

        let dependents = index.find_dependents(stem);
        if dependents.is_empty() {
            predictions.push(Prediction {
                severity: Severity::Info,
                category: PredictionCategory::OrphanedCode,
                message: format!(
                    "'{}' ({} lines) — no other file imports it. Possible dead code?",
                    path.display(), line_count
                ),
                affected_files: vec![path],
            });
        }
    }

    predictions
}

/// Format all predictions into a readable report.
pub fn format_report(predictions: &[Prediction]) -> String {
    if predictions.is_empty() {
        return "\u{2705} No predictive refactoring suggestions — your codebase looks healthy!"
            .to_string();
    }

    let critical = predictions.iter().filter(|p| matches!(p.severity, Severity::Critical)).count();
    let warnings = predictions.iter().filter(|p| matches!(p.severity, Severity::Warning)).count();
    let info = predictions.iter().filter(|p| matches!(p.severity, Severity::Info)).count();

    let mut out = String::new();
    out.push_str("\u{1f52e} **Predictive Refactoring Report**\n");
    out.push_str(&format!(
        "  {} critical \u{2022} {} warnings \u{2022} {} suggestions\n\n",
        critical, warnings, info
    ));

    for (i, pred) in predictions.iter().enumerate() {
        out.push_str(&format!("{}. {}\n\n", i + 1, pred));
    }

    out.push_str("_Run `:predict` again after making changes to re-analyze._\n");
    out
}
