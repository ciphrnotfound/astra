use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use crate::git::GitRepo;
use crate::model::CodexModel;

pub struct BisectResult {
    pub suspect_commit_id: String,
    pub suspect_commit_summary: String,
    pub suspect_author: String,
    pub explanation: String,
    pub analyzed_count: usize,
}

pub fn run_semantic_bisect(
    repo: &GitRepo,
    model: &(dyn CodexModel + Send + Sync),
    bug_description: &str,
    max_commits: usize,
) -> Result<BisectResult> {
    // We use git log to get the last N commits with their diff patches
    let output = Command::new("git")
        .current_dir(repo.root_path())
        .args(&["log", "-p", &format!("-n{}", max_commits)])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to run git log to fetch diffs"));
    }

    let log_output = String::from_utf8_lossy(&output.stdout);
    
    // Naively split log by "commit " to chunk into individual commits
    let mut commits = Vec::new();
    let mut current_commit = String::new();
    
    for line in log_output.lines() {
        if line.starts_with("commit ") && !current_commit.is_empty() {
            commits.push(current_commit.clone());
            current_commit.clear();
        }
        current_commit.push_str(line);
        current_commit.push('\n');
    }
    if !current_commit.is_empty() {
        commits.push(current_commit);
    }

    if commits.is_empty() {
        return Err(anyhow!("No commits found in git history to analyze."));
    }

    println!("codex ▸ Analyzing {} recent commits to find the bug...", commits.len());

    let mut analyzed_count = 0;

    for commit_text in commits {
        analyzed_count += 1;
        
        // Extract basic metadata for the result
        let mut id = "unknown".to_string();
        let mut author = "unknown".to_string();
        let mut summary = "unknown".to_string();
        
        let mut lines = commit_text.lines();
        if let Some(first) = lines.next() {
            id = first.replace("commit ", "").trim().to_string();
            // Shorten id
            id = id.chars().take(8).collect();
        }
        for line in &mut lines {
            if line.starts_with("Author: ") {
                author = line.replace("Author: ", "").trim().to_string();
            } else if line.is_empty() {
                // The next non-empty line is usually the summary
                if let Some(summary_line) = lines.next() {
                    summary = summary_line.trim().to_string();
                }
                break;
            }
        }

        print!("codex ▸ Semantically checking {} ({}) ... ", id, summary);
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let prompt = format!(
            "You are a time-travel debugging assistant. \
            The user is looking for the commit that introduced this specific bug or behavior:\n\n\
            <bug_description>\n{}\n</bug_description>\n\n\
            I am showing you the diff of a specific commit. Analyze the diff and determine if this commit is the LIKELY CAUSE of the bug.\n\
            1. If this commit is DEFINITELY NOT related, reply with the exact word 'NO'.\n\
            2. If this commit IS highly likely to be the cause, reply with the exact word 'YES' followed by a newline, and then a detailed explanation of WHY it caused the bug and what the developer was likely trying to do.\n\n\
            <commit_diff>\n{}\n</commit_diff>",
            bug_description,
            commit_text
        );

        let answer = model.complete(&prompt)?;
        
        if answer.trim().to_uppercase().starts_with("YES") {
            println!("FOUND IT!");
            let explanation = answer.replacen("YES", "", 1).replacen("yes", "", 1).trim().to_string();
            
            return Ok(BisectResult {
                suspect_commit_id: id,
                suspect_commit_summary: summary,
                suspect_author: author,
                explanation,
                analyzed_count,
            });
        } else {
            println!("nope.");
        }
    }

    Err(anyhow!("Could not find any commit matching that bug description in the last {} commits.", max_commits))
}
