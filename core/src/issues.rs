use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Reported,
    Triaged,
    Reproduced,
    InProgress,
    Verified,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRecord {
    pub id: String,
    pub report: String,
    pub status: IssueStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub head_commit: Option<String>,
    pub branch: Option<String>,
    pub changed_files: Vec<String>,
    pub likely_files: Vec<String>,
    pub git_evidence: Vec<String>,
    pub reproduction: Option<String>,
    pub root_cause: Option<String>,
    pub fix_summary: Option<String>,
    pub verification: Vec<String>,
    pub cowork_job_id: Option<String>,
}

impl IssueRecord {
    pub fn compact_summary(&self) -> String {
        let evidence = if self.git_evidence.is_empty() {
            "no Git evidence yet".to_string()
        } else {
            self.git_evidence
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        };
        format!(
            "{} [{:?}] {} — likely files: {} — {}",
            self.id,
            self.status,
            self.report,
            if self.likely_files.is_empty() {
                "none found".to_string()
            } else {
                self.likely_files
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            evidence
        )
    }
}

pub struct IssueStore {
    root: PathBuf,
}

impl IssueStore {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn create(&self, report: &str) -> Result<IssueRecord> {
        let report = report.trim();
        if report.is_empty() {
            return Err(anyhow!("A bug report needs a description."));
        }
        let now = now_millis();
        let issue = IssueRecord {
            id: format!("astra-issue-{}", now_nanos()),
            report: truncate(report, 4_000),
            status: IssueStatus::Reported,
            created_at: now,
            updated_at: now,
            head_commit: None,
            branch: None,
            changed_files: Vec::new(),
            likely_files: Vec::new(),
            git_evidence: Vec::new(),
            reproduction: None,
            root_cause: None,
            fix_summary: None,
            verification: Vec::new(),
            cowork_job_id: None,
        };
        self.save(&issue)?;
        Ok(issue)
    }

    pub fn save(&self, issue: &IssueRecord) -> Result<()> {
        let dir = self.dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", safe_id(&issue.id)?));
        fs::write(path, serde_json::to_string_pretty(issue)?)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<IssueRecord>> {
        let path = self.dir().join(format!("{}.json", safe_id(id)?));
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&fs::read_to_string(path)?)?))
    }

    pub fn list(&self, limit: usize) -> Result<Vec<IssueRecord>> {
        let dir = self.dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut issues = Vec::new();
        for entry in fs::read_dir(dir)?.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(issue) = serde_json::from_str::<IssueRecord>(&content) {
                    issues.push(issue);
                }
            }
        }
        issues.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        issues.truncate(limit.min(100));
        Ok(issues)
    }

    fn dir(&self) -> PathBuf {
        self.root.join(".astra").join("issues")
    }
}

fn safe_id(id: &str) -> Result<String> {
    let id = id.trim();
    if id.is_empty() || id.len() > 120 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(anyhow!("Invalid issue ID."));
    }
    Ok(id.to_string())
}

fn truncate(value: &str, limit: usize) -> String {
    let mut output = value.chars().take(limit).collect::<String>();
    if value.chars().count() > limit {
        output.push_str("... [truncated]");
    }
    output
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{IssueStatus, IssueStore};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn issue_records_are_persistent_and_bounded() {
        let root = std::env::temp_dir().join(format!(
            "astra_issue_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let store = IssueStore::new(&root);
        let issue = store.create(&"x".repeat(5_000)).unwrap();
        assert_eq!(issue.status, IssueStatus::Reported);
        assert!(issue.report.chars().count() <= 4_015);
        let loaded = store.get(&issue.id).unwrap().unwrap();
        assert_eq!(loaded.id, issue.id);
    }
}
