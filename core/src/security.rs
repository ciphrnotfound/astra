use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

use crate::index::CodeIndex;
use crate::model::CodexModel;

#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub file: PathBuf,
    pub line_number: usize,
    pub severity: &'static str,
    pub description: String,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct SecurityReport {
    pub issues: Vec<SecurityIssue>,
    pub files_scanned: usize,
    pub ai_analysis: Option<String>,
}

pub fn run_security_scan(
    root: &Path,
    index: &CodeIndex,
    model: Option<&(dyn CodexModel + Send + Sync)>,
) -> SecurityReport {
    let mut issues = Vec::new();
    let mut files_scanned = 0usize;
    let patterns: [(&str, &'static str, &'static str); 8] = [
        ("api_key=", "High", "Hardcoded API key"),
        ("api_key =", "High", "Hardcoded API key"),
        ("password=", "High", "Hardcoded password"),
        ("password =", "High", "Hardcoded password"),
        ("secret=", "High", "Hardcoded secret"),
        ("secret =", "High", "Hardcoded secret"),
        ("http://", "Medium", "Plain HTTP reference"),
        ("select * from", "Medium", "Raw SQL pattern"),
    ];

    for rel_path in index.files().keys() {
        let abs_path = if rel_path.is_absolute() {
            rel_path.clone()
        } else {
            root.join(rel_path)
        };
        let contents = match fs::read_to_string(&abs_path) {
            Ok(v) => v,
            Err(_) => continue,
        };
        files_scanned += 1;
        for (line_idx, line) in contents.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            for (needle, severity, description) in &patterns {
                if lower.contains(needle) {
                    issues.push(SecurityIssue {
                        file: rel_path.clone(),
                        line_number: line_idx + 1,
                        severity,
                        description: (*description).to_string(),
                        snippet: line.trim().to_string(),
                    });
                }
            }
        }
    }

    let ai_analysis = if issues.is_empty() {
        None
    } else {
        model.and_then(|m| {
            let mut prompt = String::from(
                "Review these potential security findings and summarize practical remediation priorities:\n\n",
            );
            for (idx, issue) in issues.iter().take(15).enumerate() {
                let _ = writeln!(
                    &mut prompt,
                    "{}. [{}] {}:{} — {}\n   {}",
                    idx + 1,
                    issue.severity,
                    issue.file.display(),
                    issue.line_number,
                    issue.description,
                    issue.snippet
                );
            }
            m.complete(&prompt).ok()
        })
    };

    SecurityReport {
        issues,
        files_scanned,
        ai_analysis,
    }
}

impl SecurityReport {
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "🛡️ Security Scan");
        let _ = writeln!(&mut out, "Scanned {} files", self.files_scanned);
        if self.issues.is_empty() {
            let _ = writeln!(&mut out, "✅ No obvious pattern-level issues found.");
            return out;
        }
        let high = self.issues.iter().filter(|i| i.severity == "High").count();
        let medium = self
            .issues
            .iter()
            .filter(|i| i.severity == "Medium")
            .count();
        let _ = writeln!(
            &mut out,
            "Found {} issues ({} High, {} Medium)",
            self.issues.len(),
            high,
            medium
        );
        for issue in self.issues.iter().take(30) {
            let _ = writeln!(
                &mut out,
                "- [{}] {}:{} {}",
                issue.severity,
                issue.file.display(),
                issue.line_number,
                issue.description
            );
        }
        if let Some(analysis) = &self.ai_analysis {
            let _ = writeln!(&mut out, "\nAI Analysis:\n{}", analysis);
        }
        out
    }
}
