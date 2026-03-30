use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use crate::memory::MemoryStore;

pub struct SessionTracker {
    start_time: Instant,
    project_root: PathBuf,
}

impl SessionTracker {
    pub fn new(project_root: &Path) -> Self {
        Self {
            start_time: Instant::now(),
            project_root: project_root.to_path_buf(),
        }
    }

    /// Calculates session duration and uses `git diff --stat` to track changed files.
    /// Saves the result directly to MemoryStore.
    pub fn finish_session(&self, memory: &mut MemoryStore) {
        let duration = self.start_time.elapsed();
        let seconds = duration.as_secs();
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        let secs = seconds % 60;

        let time_str = if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        };

        // Try to get git diff stats for the session
        let diff_stat = Command::new("git")
            .current_dir(&self.project_root)
            .args(["diff", "HEAD", "--stat"])
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "No git diff available.".to_string());

        let log_entry = format!(
            "Session Duration: {}\n\nChanges Made:\n{}",
            time_str, diff_stat
        );

        // Store as an accomplishment log in memory
        memory.add("accomplishment", log_entry.clone());

        println!("\n\u{1f4be} Session tracked. Worked for: {}", time_str);
    }
}
