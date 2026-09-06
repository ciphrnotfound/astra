//! Astra's DevOps / project-manager layer.
//!
//! Shells out to `git` and the GitHub CLI (`gh`) to let Astra act as a
//! release manager: staging changes, drafting commit messages from the
//! real diff, opening pull requests, and cutting releases — all with an
//! LLM writing the human-facing prose.

use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::model::CodexModel;

const MAX_DIFF_CHARS: usize = 12_000;

/// Outcome of a shell command.
struct CmdOut {
    success: bool,
    stdout: String,
    stderr: String,
}

fn run(root: &Path, program: &str, args: &[&str]) -> Result<CmdOut> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| anyhow!("failed to run `{} {}`: {}", program, args.join(" "), e))?;
    Ok(CmdOut {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// Check whether a CLI tool is available on PATH.
pub fn tool_available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Repo state inspection
// ---------------------------------------------------------------------------

pub struct RepoState {
    pub branch: String,
    pub staged_diff: String,
    pub unstaged_diff: String,
    pub status_short: String,
    pub has_changes: bool,
    pub has_remote: bool,
}

pub fn inspect(root: &Path) -> Result<RepoState> {
    let branch = run(root, "git", &["rev-parse", "--abbrev-ref", "HEAD"])?
        .stdout
        .trim()
        .to_string();
    let status_short = run(root, "git", &["status", "--short"])?.stdout;
    let staged_diff = run(root, "git", &["diff", "--cached"])?.stdout;
    let unstaged_diff = run(root, "git", &["diff"])?.stdout;
    let has_remote = run(root, "git", &["remote"])?
        .stdout
        .trim()
        .lines()
        .next()
        .is_some();

    let has_changes = !status_short.trim().is_empty();

    Ok(RepoState {
        branch,
        staged_diff,
        unstaged_diff,
        status_short,
        has_changes,
        has_remote,
    })
}

fn truncate_diff(diff: &str) -> &str {
    if diff.len() <= MAX_DIFF_CHARS {
        diff
    } else {
        &diff[..MAX_DIFF_CHARS]
    }
}

// ---------------------------------------------------------------------------
// Commit message drafting
// ---------------------------------------------------------------------------

/// Draft a conventional-commit message from the staged (or full) diff.
pub fn draft_commit_message(
    model: &dyn CodexModel,
    diff: &str,
    status_short: &str,
) -> Result<String> {
    let prompt = format!(
        r#"You are Astra, a senior engineer writing a git commit message.

## CHANGED FILES
{}

## DIFF
{}

## YOUR JOB
Write a clean Conventional Commit message for these changes.

Rules:
- First line: `<type>(<scope>): <summary>` — type is feat/fix/chore/refactor/docs/test/perf. Max 72 chars.
- Blank line, then 1-4 bullet points explaining WHAT changed and WHY (not how).
- Be specific and accurate to the diff. Do not invent changes.
- Output ONLY the commit message. No markdown fences, no preamble."#,
        status_short.trim(),
        truncate_diff(diff)
    );
    let msg = model.complete(&prompt)?;
    Ok(clean_commit_message(&msg))
}

/// Extract a real Conventional Commit message from a possibly-chatty LLM reply.
///
/// Small models often wrap the message in preamble ("Here's a clean commit
/// message:"), backticks, and trailing explanation. We keep only the actual
/// commit: the first `type(scope): summary` line and any bullet body that
/// immediately follows, dropping everything else.
fn clean_commit_message(raw: &str) -> String {
    let stripped = strip_fences(raw);
    let types = [
        "feat", "fix", "chore", "refactor", "docs", "test", "perf", "build", "ci", "style", "revert",
    ];

    let is_subject = |line: &str| -> bool {
        let l = line.trim().trim_start_matches('`').trim();
        types.iter().any(|t| {
            l.starts_with(&format!("{}:", t)) || l.starts_with(&format!("{}(", t))
        })
    };

    let lines: Vec<&str> = stripped.lines().collect();

    // Find the first line that looks like a conventional-commit subject.
    let start = lines.iter().position(|l| is_subject(l));

    let Some(start) = start else {
        // No conventional subject found — fall back to the first non-empty,
        // non-preamble line, capped at 72 chars.
        let first = stripped
            .lines()
            .map(|l| l.trim().trim_matches('`').trim())
            .find(|l| !l.is_empty() && !l.to_ascii_lowercase().contains("commit message"))
            .unwrap_or("chore: update");
        return truncate_subject(first);
    };

    // Subject line, cleaned of surrounding backticks.
    let subject = truncate_subject(lines[start].trim().trim_matches('`').trim());

    // Collect a bullet body that immediately follows (blank line then `- ...`),
    // stopping at the first line that reads like prose explanation.
    let mut body: Vec<String> = Vec::new();
    for line in lines.iter().skip(start + 1) {
        let t = line.trim();
        if t.is_empty() {
            if body.is_empty() {
                continue; // allow the blank line between subject and body
            } else {
                break; // blank line after body ends it
            }
        }
        if t.starts_with('-') || t.starts_with('*') {
            body.push(format!("- {}", t.trim_start_matches(['-', '*', ' '])));
        } else {
            // Prose / explanation ("This commit message follows…") — stop.
            break;
        }
    }

    if body.is_empty() {
        subject
    } else {
        format!("{}\n\n{}", subject, body.join("\n"))
    }
}

fn truncate_subject(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() <= 72 {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(72).collect();
        truncated
    }
}

// ---------------------------------------------------------------------------
// Preview — draft what WOULD be shipped, without changing anything
// ---------------------------------------------------------------------------

pub struct ShipPreview {
    pub message: String,
    pub branch: String,
    pub files: Vec<String>,
    pub has_remote: bool,
}

/// Draft the commit message and list what would be pushed — no git writes.
pub fn preview(root: &Path, model: &dyn CodexModel) -> Result<ShipPreview> {
    let state = inspect(root)?;
    if !state.has_changes {
        return Err(anyhow!("Nothing to ship — working tree is clean."));
    }
    // Diff = staged + unstaged, so the preview reflects everything `:ship` will stage.
    let combined = format!("{}\n{}", state.staged_diff, state.unstaged_diff);
    let message = draft_commit_message(model, &combined, &state.status_short)?;
    let files: Vec<String> = state
        .status_short
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(ShipPreview {
        message,
        branch: state.branch,
        files,
        has_remote: state.has_remote,
    })
}

// ---------------------------------------------------------------------------
// :commit — stage + commit
// ---------------------------------------------------------------------------

pub struct CommitResult {
    pub message: String,
    pub log: String,
}

/// Stage all changes and commit with an AI-drafted message.
pub fn commit_all(root: &Path, model: &dyn CodexModel) -> Result<CommitResult> {
    let state = inspect(root)?;
    if !state.has_changes {
        return Err(anyhow!("Nothing to commit — working tree is clean."));
    }

    // Stage everything
    let add = run(root, "git", &["add", "-A"])?;
    if !add.success {
        return Err(anyhow!("git add failed: {}", add.stderr));
    }

    // Re-read the now-staged diff
    let staged = run(root, "git", &["diff", "--cached"])?.stdout;
    let message = draft_commit_message(model, &staged, &state.status_short)?;

    // Commit with the drafted message
    let commit = run(root, "git", &["commit", "-m", &message])?;
    if !commit.success {
        return Err(anyhow!("git commit failed: {}", commit.stderr));
    }

    Ok(CommitResult {
        message,
        log: commit.stdout,
    })
}

// ---------------------------------------------------------------------------
// :ship — commit + push
// ---------------------------------------------------------------------------

pub struct ShipResult {
    pub commit_message: String,
    pub branch: String,
    pub pushed: bool,
    pub push_log: String,
}

/// Commit all changes and push to the remote tracking branch.
pub fn ship(root: &Path, model: &dyn CodexModel) -> Result<ShipResult> {
    let commit = commit_all(root, model)?;
    let state = inspect(root)?;

    if !state.has_remote {
        return Ok(ShipResult {
            commit_message: commit.message,
            branch: state.branch,
            pushed: false,
            push_log: "No git remote configured — committed locally only.".to_string(),
        });
    }

    // Push, setting upstream if needed
    let push = run(
        root,
        "git",
        &["push", "--set-upstream", "origin", &state.branch],
    )?;

    Ok(ShipResult {
        commit_message: commit.message,
        branch: state.branch,
        pushed: push.success,
        push_log: if push.success {
            push.stdout + &push.stderr
        } else {
            format!("push failed: {}", push.stderr)
        },
    })
}

// ---------------------------------------------------------------------------
// :pr — open a pull request via gh
// ---------------------------------------------------------------------------

pub struct PrResult {
    pub title: String,
    pub body: String,
    pub url: String,
}

/// Draft a PR title + body from the branch diff against the base branch.
fn draft_pr(
    model: &dyn CodexModel,
    branch: &str,
    base: &str,
    diff: &str,
    commits: &str,
) -> Result<(String, String)> {
    let prompt = format!(
        r#"You are Astra, writing a GitHub pull request for branch `{}` into `{}`.

## COMMITS ON THIS BRANCH
{}

## DIFF
{}

## YOUR JOB
Write a pull request title and body.

Respond in EXACTLY this format (no markdown fences):
TITLE: <concise PR title, max 72 chars>
BODY:
## Summary
<2-3 sentences on what this PR does and why>

## Changes
- <bullet per meaningful change>

## Testing
<how this was or should be verified>"#,
        branch,
        base,
        commits.trim(),
        truncate_diff(diff)
    );
    let response = model.complete(&prompt)?;
    parse_pr_response(&response)
}

fn parse_pr_response(response: &str) -> Result<(String, String)> {
    let cleaned = strip_fences(response);
    let mut title = String::new();
    let mut body = String::new();
    let mut in_body = false;

    for line in cleaned.lines() {
        if let Some(rest) = line.strip_prefix("TITLE:") {
            title = rest.trim().to_string();
        } else if line.trim_start().starts_with("BODY:") {
            in_body = true;
        } else if in_body {
            body.push_str(line);
            body.push('\n');
        }
    }

    if title.is_empty() {
        // Fallback: first non-empty line is the title
        title = cleaned.lines().find(|l| !l.trim().is_empty()).unwrap_or("Update").trim().to_string();
    }
    if body.trim().is_empty() {
        body = cleaned.to_string();
    }
    Ok((title, body.trim().to_string()))
}

/// Open a pull request for the current branch using the GitHub CLI.
pub fn open_pr(root: &Path, model: &dyn CodexModel, base: &str) -> Result<PrResult> {
    if !tool_available("gh") {
        return Err(anyhow!(
            "GitHub CLI (`gh`) is not installed. Install it from https://cli.github.com/ to open PRs."
        ));
    }

    let state = inspect(root)?;
    if state.branch == base {
        return Err(anyhow!(
            "You are on `{}`. Create a feature branch before opening a PR (e.g. `git checkout -b my-feature`).",
            base
        ));
    }

    // Make sure the branch is pushed first
    if state.has_remote {
        let _ = run(root, "git", &["push", "--set-upstream", "origin", &state.branch]);
    }

    let diff = run(root, "git", &["diff", &format!("{}...HEAD", base)])?.stdout;
    let commits = run(
        root,
        "git",
        &["log", &format!("{}..HEAD", base), "--pretty=format:- %s"],
    )?
    .stdout;

    let (title, body) = draft_pr(model, &state.branch, base, &diff, &commits)?;

    let pr = run(
        root,
        "gh",
        &[
            "pr", "create", "--base", base, "--head", &state.branch, "--title", &title, "--body",
            &body,
        ],
    )?;

    if !pr.success {
        return Err(anyhow!("gh pr create failed: {}", pr.stderr));
    }

    let url = pr.stdout.trim().lines().last().unwrap_or("").to_string();
    Ok(PrResult { title, body, url })
}

// ---------------------------------------------------------------------------
// :release — cut a GitHub release with AI-drafted notes
// ---------------------------------------------------------------------------

pub struct ReleaseResult {
    pub tag: String,
    pub notes: String,
    pub url: String,
}

fn draft_release_notes(model: &dyn CodexModel, tag: &str, commits: &str) -> Result<String> {
    let prompt = format!(
        r#"You are Astra, writing release notes for version `{}`.

## COMMITS SINCE LAST RELEASE
{}

## YOUR JOB
Write clean, user-facing release notes in markdown. Group changes under
### Features / ### Fixes / ### Other headings. Be concise. Skip noise like
"merge" or "wip" commits. Output ONLY the markdown notes."#,
        tag,
        commits.trim()
    );
    Ok(strip_fences(&model.complete(&prompt)?))
}

/// Cut a GitHub release at the given tag using AI-drafted notes.
pub fn cut_release(root: &Path, model: &dyn CodexModel, tag: &str) -> Result<ReleaseResult> {
    if !tool_available("gh") {
        return Err(anyhow!(
            "GitHub CLI (`gh`) is not installed. Install it from https://cli.github.com/ to cut releases."
        ));
    }

    // Find commits since the last tag (or all if none)
    let last_tag = run(root, "git", &["describe", "--tags", "--abbrev=0"])?
        .stdout
        .trim()
        .to_string();
    let range = if last_tag.is_empty() {
        "HEAD".to_string()
    } else {
        format!("{}..HEAD", last_tag)
    };
    let commits = run(root, "git", &["log", &range, "--pretty=format:- %s"])?.stdout;

    let notes = draft_release_notes(model, tag, &commits)?;

    let release = run(
        root,
        "gh",
        &["release", "create", tag, "--title", tag, "--notes", &notes],
    )?;

    if !release.success {
        return Err(anyhow!("gh release create failed: {}", release.stderr));
    }

    let url = release.stdout.trim().lines().last().unwrap_or("").to_string();
    Ok(ReleaseResult {
        tag: tag.to_string(),
        notes,
        url,
    })
}

// ---------------------------------------------------------------------------
// Status report — the PM's "where are we" view
// ---------------------------------------------------------------------------

pub fn standup_report(root: &Path, model: &dyn CodexModel) -> Result<String> {
    let state = inspect(root)?;
    let recent = run(root, "git", &["log", "-10", "--pretty=format:- %s (%cr)"])?.stdout;

    let prompt = format!(
        r#"You are Astra, a technical project manager giving a quick standup summary.

## CURRENT BRANCH
{}

## UNCOMMITTED CHANGES
{}

## RECENT COMMITS
{}

## YOUR JOB
Give a short, friendly standup-style summary:
- What's the current state of the working tree?
- What was recently done (from commits)?
- One concrete suggestion for the next step.
Keep it to 4-6 lines. Be specific and useful."#,
        state.branch,
        if state.status_short.trim().is_empty() {
            "(clean)".to_string()
        } else {
            state.status_short.clone()
        },
        recent.trim()
    );
    model.complete(&prompt)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn strip_fences(text: &str) -> String {
    let mut s = text.trim().to_string();
    if s.starts_with("```") {
        if let Some(pos) = s.find('\n') {
            s = s[pos + 1..].to_string();
        }
    }
    if s.ends_with("```") {
        if let Some(pos) = s.rfind("```") {
            s = s[..pos].to_string();
        }
    }
    s.trim().to_string()
}
