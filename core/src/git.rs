use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use git2::{Commit, Oid, Repository};

pub struct GitRepo {
    root: PathBuf,
    repo: Repository,
}

pub struct CommitSummary {
    pub id: String,
    pub summary: String,
    pub author: String,
    pub time: i64,
}

pub struct CommitInfo {
    pub id: String,
    pub summary: String,
    pub author: String,
    pub date: String,
}

impl GitRepo {
    pub fn discover(root: &Path) -> Result<Self> {
        let repo = Repository::discover(root)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| anyhow!("repository has no workdir"))?;
        Ok(Self {
            root: workdir.to_path_buf(),
            repo,
        })
    }

    pub fn recent_commit_count(&self, limit: usize) -> usize {
        let mut revwalk = match self.repo.revwalk() {
            Ok(rw) => rw,
            Err(_) => return 0,
        };
        if revwalk.push_head().is_err() {
            return 0;
        }
        revwalk.take(limit).count()
    }

    pub fn total_commit_count(&self) -> usize {
        let mut revwalk = match self.repo.revwalk() {
            Ok(rw) => rw,
            Err(_) => return 0,
        };
        if revwalk.push_head().is_err() {
            return 0;
        }
        revwalk.count()
    }

    pub fn uncommitted_file_count(&self) -> usize {
        let statuses = match self.repo.statuses(None) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        statuses.len()
    }

    pub fn changed_files(&self) -> Vec<String> {
        let statuses = match self.repo.statuses(None) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let mut files = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }
        files.sort();
        files.dedup();
        files
    }

    /// Get `git log --stat` for the most recent N commits.
    pub fn recent_log(&self, limit: usize) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(&["log", "--stat", "--format=", &format!("-{}", limit)])
            .output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("git log failed"))
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    pub fn get_head_commit(&self) -> Result<String> {
        let head = self.repo.head();
        if head.is_err() {
            return Err(anyhow!("Repository has no HEAD"));
        }

        let output = Command::new("git")
            .current_dir(&self.root)
            .args(&["rev-parse", "HEAD"])
            .output()?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(hash)
        } else {
            Err(anyhow::anyhow!("Failed to get HEAD commit"))
        }
    }

    pub fn get_diff_stats(&self, from_commit: &str) -> Result<(usize, usize)> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(&["diff", "--numstat", from_commit])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to run git diff"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut added = 0;
        let mut deleted = 0;

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(a) = parts[0].parse::<usize>() {
                    added += a;
                }
                if let Ok(d) = parts[1].parse::<usize>() {
                    deleted += d;
                }
            }
        }

        Ok((added, deleted))
    }

    pub fn get_commits_by_author(&self, author_name: &str, limit: usize) -> Result<String> {
        let author_arg = format!("--author={}", author_name);
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(&["log", &author_arg, "-p", &format!("-n{}", limit)])
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(anyhow::anyhow!(
                "Failed to get commits for author {}",
                author_name
            ))
        }
    }

    pub fn last_commit_info(&self) -> Result<CommitInfo> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(&[
                "log",
                "-1",
                "--format=%H|%an|%ad|%s",
                "--date=iso-strict",
            ])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get last commit"));
        }

        let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut parts = line.splitn(4, '|');
        let id = parts.next().unwrap_or("").to_string();
        let author = parts.next().unwrap_or("").to_string();
        let date = parts.next().unwrap_or("").to_string();
        let summary = parts.next().unwrap_or("").to_string();

        Ok(CommitInfo {
            id,
            summary,
            author,
            date,
        })
    }

    pub fn recent_commits(&self, limit: usize) -> Result<Vec<CommitSummary>> {
        self.collect_commits(Some(limit))
    }

    pub fn all_commits(&self) -> Result<Vec<CommitSummary>> {
        self.collect_commits(None)
    }

    fn collect_commits(&self, limit: Option<usize>) -> Result<Vec<CommitSummary>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = match oid_result {
                Ok(oid) => oid,
                Err(_) => continue,
            };
            let commit = match self.repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };
            commits.push(self.summarize_commit(&commit));
            if let Some(limit) = limit {
                if commits.len() >= limit {
                    break;
                }
            }
        }

        Ok(commits)
    }

    pub fn recent_commits_for_path(
        &self,
        rel_path: &Path,
        limit: usize,
    ) -> Result<Vec<CommitSummary>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = match oid_result {
                Ok(oid) => oid,
                Err(_) => continue,
            };
            let commit = match self.repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if Self::commit_touches_path(&commit, rel_path, &self.repo)? {
                commits.push(self.summarize_commit(&commit));
                if commits.len() >= limit {
                    break;
                }
            }
        }

        Ok(commits)
    }

    fn commit_touches_path(commit: &Commit<'_>, rel_path: &Path, repo: &Repository) -> Result<bool> {
        let tree = commit.tree()?;
        let parent = match commit.parents().next() {
            Some(p) => p,
            None => return Ok(false),
        };
        let parent_tree = parent.tree()?;

        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

        let mut touched = false;
        let rel_str = rel_path.to_string_lossy();

        diff.foreach(
            &mut |delta, _| {
                if let Some(new_file) = delta.new_file().path() {
                    if new_file.to_string_lossy().contains(&*rel_str) {
                        touched = true;
                    }
                }
                if let Some(old_file) = delta.old_file().path() {
                    if old_file.to_string_lossy().contains(&*rel_str) {
                        touched = true;
                    }
                }
                if touched {
                    false
                } else {
                    true
                }
            },
            None,
            None,
            None,
        )?;

        Ok(touched)
    }

    fn summarize_commit(&self, commit: &Commit<'_>) -> CommitSummary {
        let id = short_oid(commit.id());
        let summary = commit
            .summary()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<no summary>".to_string());
        let author = commit
            .author()
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let time = commit.time().seconds();

        CommitSummary {
            id,
            summary,
            author,
            time,
        }
    }
}

fn short_oid(oid: Oid) -> String {
    let s = oid.to_string();
    s.chars().take(8).collect()
}
