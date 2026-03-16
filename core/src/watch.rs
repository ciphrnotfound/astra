// ─── Watch Mode ─────────────────────────────────────────────────────
// Monitors the project directory for file changes and re-indexes
// modified files in real time, printing alerts when issues are detected.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::index::{is_indexable_path, CodeIndex};

/// Directories to skip during watching (same as indexing)
const SKIP_DIRS: &[&str] = &[
    "target", "node_modules", ".git", ".astra", ".codex", ".forge",
    "dist", "build", "__pycache__", ".venv", "venv",
];

/// Result of analyzing a single file change.
pub struct WatchAlert {
    pub path: PathBuf,
    pub kind: WatchAlertKind,
    pub message: String,
}

pub enum WatchAlertKind {
    FileModified,
    FileCreated,
    FileDeleted,
    Warning,
}

impl std::fmt::Display for WatchAlert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self.kind {
            WatchAlertKind::FileModified => "\u{1f4dd}",
            WatchAlertKind::FileCreated => "\u{2728}",
            WatchAlertKind::FileDeleted => "\u{1f5d1}\u{fe0f}",
            WatchAlertKind::Warning => "\u{26a0}\u{fe0f}",
        };
        write!(f, "{} {} — {}", icon, self.path.display(), self.message)
    }
}

/// Start watching a directory for file changes.
/// Returns a receiver that emits `WatchAlert` items.
/// The watcher runs on a background thread.
pub fn start_watcher(
    root: &Path,
) -> Result<(RecommendedWatcher, mpsc::Receiver<WatchAlert>)> {
    let (tx, rx) = mpsc::channel::<WatchAlert>();
    let root_owned = root.to_path_buf();

    let event_tx = tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |result: std::result::Result<Event, notify::Error>| {
            if let Ok(event) = result {
                for path in &event.paths {
                    // Skip non-indexable paths and skip directories
                    if should_skip_path(path, &root_owned) {
                        continue;
                    }

                    let alert = match event.kind {
                        EventKind::Create(_) => WatchAlert {
                            path: path.clone(),
                            kind: WatchAlertKind::FileCreated,
                            message: "New file created".to_string(),
                        },
                        EventKind::Modify(_) => WatchAlert {
                            path: path.clone(),
                            kind: WatchAlertKind::FileModified,
                            message: "File modified — re-indexing".to_string(),
                        },
                        EventKind::Remove(_) => WatchAlert {
                            path: path.clone(),
                            kind: WatchAlertKind::FileDeleted,
                            message: "File deleted".to_string(),
                        },
                        _ => continue,
                    };
                    let _ = event_tx.send(alert);
                }
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok((watcher, rx))
}

/// Process a file change event by re-indexing the changed file.
pub fn handle_file_change(index: &mut CodeIndex, alert: &WatchAlert) -> Vec<String> {
    let mut warnings = Vec::new();

    match alert.kind {
        WatchAlertKind::FileCreated | WatchAlertKind::FileModified => {
            if let Ok(contents) = std::fs::read_to_string(&alert.path) {
                let line_count = contents.lines().count();
                index.add_file(alert.path.clone(), &contents);

                // Warn about suspiciously large files
                if line_count > 500 {
                    warnings.push(format!(
                        "\u{26a0}\u{fe0f} {} has {} lines — consider splitting it",
                        alert.path.display(),
                        line_count
                    ));
                }

                // Warn about files with no functions
                let deps = index.find_dependencies(&alert.path);
                if deps.is_empty() && line_count > 50 {
                    warnings.push(format!(
                        "\u{26a0}\u{fe0f} {} has no detected imports — might be orphaned",
                        alert.path.display()
                    ));
                }
            }
        }
        WatchAlertKind::FileDeleted => {
            // Could track deleted files to warn about broken imports
            warnings.push(format!(
                "\u{26a0}\u{fe0f} {} was deleted — check for broken imports",
                alert.path.display()
            ));
        }
        _ => {}
    }

    warnings
}

fn should_skip_path(path: &Path, _root: &Path) -> bool {
    // Skip directories
    if path.is_dir() {
        return true;
    }

    // Skip paths containing skip directories
    for component in path.components() {
        if let Some(name) = component.as_os_str().to_str() {
            if SKIP_DIRS.contains(&name) {
                return true;
            }
        }
    }

    // Skip non-source files
    !is_indexable_path(path)
}

/// Install a pre-commit git hook that runs `astra --health`.
pub fn install_git_hook(root: &Path) -> Result<String> {
    let hooks_dir = root.join(".git").join("hooks");
    if !hooks_dir.exists() {
        return Err(anyhow::anyhow!(
            "No .git/hooks directory found. Is this a git repository?"
        ));
    }

    let hook_path = hooks_dir.join("pre-commit");
    let hook_script = if cfg!(windows) {
        r#"#!/bin/sh
# Astra pre-commit hook — checks codebase health before committing
echo "🔍 Astra: Running pre-commit health check..."
astra-cli --health
if [ $? -ne 0 ]; then
    echo "❌ Astra health check failed. Fix issues before committing."
    exit 1
fi
echo "✅ Astra health check passed."
"#
    } else {
        r#"#!/bin/sh
# Astra pre-commit hook — checks codebase health before committing
echo "🔍 Astra: Running pre-commit health check..."
astra-cli --health
if [ $? -ne 0 ]; then
    echo "❌ Astra health check failed. Fix issues before committing."
    exit 1
fi
echo "✅ Astra health check passed."
"#
    };

    std::fs::write(&hook_path, hook_script)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    Ok(format!(
        "✅ Pre-commit hook installed at {}",
        hook_path.display()
    ))
}
