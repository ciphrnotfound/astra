// TODO: use std::path::{Path, PathBuf};
// TODO: use std::process::Command;

// TODO: use anyhow::{anyhow, Result};
// TODO: use git2::{Commit, Oid, Repository};

export interface GitRepo {
  root: any;
  repo: any;
}


export interface CommitSummary {
  id: string;
  summary: string;
  author: string;
  time: number;
}


// impl GitRepo {
export function discover(root: any): any {
  // let repo = Repository::discover(root)?;
  // let workdir = repo
  // .workdir()
  // .ok_or_else(|| anyhow!("repository has no workdir"))?;
  // Ok(Self {
  // root: workdir.to_path_buf(),
  // repo,
  // })
}


export function recent_commit_count(&self: any, limit: number): number {
  // let mut revwalk = match self.repo.revwalk() {
  // Ok(rw) => rw,
  // Err(_) => return 0,
  // };
  // if revwalk.push_head().is_err() {
  // return 0;
  // }
  // revwalk.take(limit).count()
}


export function uncommitted_file_count(&self: any): number {
  // let statuses = match self.repo.statuses(None) {
  // Ok(s) => s,
  // Err(_) => return 0,
  // };
  // statuses.len()
}


export function root_path(&self: any): any {
  // &self.root
}


// /// Returns the current HEAD commit hash.
export function get_head_commit(&self: any): any {
  // let output = Command::new("git")
  // .current_dir(&self.root)
  // .args(&["rev-parse", "HEAD"])
  // .output()?;
  // 
  // if output.status.success() {
  // let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
  // Ok(hash)
  // } else {
  // Err(anyhow::anyhow!("Failed to get HEAD commit"))
  // }
}


// /// Returns (lines_added, lines_deleted) between the given commit and the current working tree.
export function get_diff_stats(&self: any, from_commit: string): any {
  // // git diff --numstat <commit>
  // let output = Command::new("git")
  // .current_dir(&self.root)
  // .args(&["diff", "--numstat", from_commit])
  // .output()?;
  // 
  // if !output.status.success() {
  // return Err(anyhow::anyhow!("Failed to run git diff"));
  // }
  // 
  // let output_str = String::from_utf8_lossy(&output.stdout);
  // let mut added = 0;
  // let mut deleted = 0;
  // 
  // for line in output_str.lines() {
  // let parts: Vec<&str> = line.split_whitespace().collect();
  // if parts.len() >= 2 {
  // if let Ok(a) = parts[0].parse::<usize>() {
  // added += a;
  // }
  // if let Ok(d) = parts[1].parse::<usize>() {
  // deleted += d;
  // }
  // }
  // }
  // 
  // Ok((added, deleted))
}


// /// Fetches the last N commits made by a specific author, including their diff patches.
// /// This is used for simulating teammate code reviews and learning their style.
export function get_commits_by_author(&self: any, author_name: string, limit: number): any {
  // let author_arg = format!("--author={}", author_name);
  // let output = Command::new("git")
  // .current_dir(&self.root)
  // .args(&["log", &author_arg, "-p", &format!("-n{}", limit)])
  // .output()?;
  // 
  // if output.status.success() {
  // Ok(String::from_utf8_lossy(&output.stdout).into_owned())
  // } else {
  // Err(anyhow::anyhow!("Failed to get commits for author {}", author_name))
  // }
}


// pub fn recent_commits_for_path(

// &self,
// rel_path: &Path,
// limit: usize,
// ) -> Result<Vec<CommitSummary>> {
// let mut revwalk = self.repo.revwalk()?;
// revwalk.push_head()?;
// let mut commits = Vec::new();

// for oid_result in revwalk {
// let oid = match oid_result {
// Ok(oid) => oid,
// Err(_) => continue,
// };
// let commit = match self.repo.find_commit(oid) {
// Ok(c) => c,
// Err(_) => continue,
// };
// if self.commit_touches_path(&commit, rel_path)? {
// commits.push(self.summarize_commit(&commit));
// if commits.len() >= limit {
// break;
// }
// }
// }

// Ok(commits)
// }

export function commit_touches_path(&self: any, commit: any, rel_path: any): any {
  // let tree = commit.tree()?;
  // let parent = match commit.parents().next() {
  // Some(p) => p,
  // None => return Ok(false),
  // };
  // let parent_tree = parent.tree()?;
  // 
  // let diff = self
  // .repo
  // .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
  // 
  // let mut touched = false;
  // let rel_str = rel_path.to_string_lossy();
  // 
  // diff.foreach(
  // &mut |delta, _| {
  // if let Some(new_file) = delta.new_file().path() {
  // if new_file.to_string_lossy().contains(&*rel_str) {
  // touched = true;
  // }
  // }
  // if let Some(old_file) = delta.old_file().path() {
  // if old_file.to_string_lossy().contains(&*rel_str) {
  // touched = true;
  // }
  // }
  // if touched {
  // false
  // } else {
  // true
  // }
  // },
  // None,
  // None,
  // None,
  // )?;
  // 
  // Ok(touched)
}


export function summarize_commit(&self: any, commit: any): any {
  // let id = short_oid(commit.id());
  // let summary = commit
  // .summary()
  // .map(|s| s.to_string())
  // .unwrap_or_else(|| "<no summary>".to_string());
  // let author = commit
  // .author()
  // .name()
  // .map(|s| s.to_string())
  // .unwrap_or_else(|| "<unknown>".to_string());
  // let time = commit.time().seconds();
  // 
  // CommitSummary {
  // id,
  // summary,
  // author,
  // time,
  // }
}

// }

export function short_oid(oid: any): string {
  // let s = oid.to_string();
  // s.chars().take(8).collect()
}

