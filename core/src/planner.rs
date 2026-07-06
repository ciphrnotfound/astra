//! Astra's System 2 planning brain.
//!
//! Turns a high-level goal into an ordered list of concrete subtasks,
//! tracks execution progress, reflects after each subtask, and replans
//! when the situation changes. The plan is persisted to .astra/plan.json
//! so work can resume across terminal sessions.

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::CodexModel;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SubtaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked(String),   // blocked by a specific reason
    Skipped(String),   // skipped with reason
}

impl std::fmt::Display for SubtaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtaskStatus::Pending      => write!(f, "⬜ pending"),
            SubtaskStatus::InProgress   => write!(f, "🔄 in-progress"),
            SubtaskStatus::Done         => write!(f, "✅ done"),
            SubtaskStatus::Blocked(r)   => write!(f, "🚧 blocked: {}", r),
            SubtaskStatus::Skipped(r)   => write!(f, "⏭️  skipped: {}", r),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subtask {
    pub id: usize,
    pub title: String,
    pub description: String,
    /// What done looks like — used by the reflection pass to verify completion.
    pub acceptance: String,
    pub status: SubtaskStatus,
    pub result_summary: Option<String>,
    pub files_touched: Vec<String>,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
}

impl Subtask {
    pub fn duration_secs(&self) -> Option<u64> {
        match (self.started_at, self.finished_at) {
            (Some(s), Some(e)) => Some(e.saturating_sub(s)),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlanPhase {
    Planning,
    Executing,
    Reflecting,
    Done,
    Abandoned(String),
}

impl std::fmt::Display for PlanPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanPhase::Planning        => write!(f, "🗺️  planning"),
            PlanPhase::Executing       => write!(f, "⚙️  executing"),
            PlanPhase::Reflecting      => write!(f, "🪞 reflecting"),
            PlanPhase::Done            => write!(f, "🎉 done"),
            PlanPhase::Abandoned(r)    => write!(f, "❌ abandoned: {}", r),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    pub goal: String,
    pub phase: PlanPhase,
    pub subtasks: Vec<Subtask>,
    pub context_snapshot: String,  // codebase summary at plan creation time
    pub risk_notes: Vec<String>,   // things that could go wrong
    pub created_at: u64,
    pub updated_at: u64,
    pub replan_count: u32,
}

impl Plan {
    pub fn new(goal: &str, context_snapshot: &str) -> Self {
        let now = now_secs();
        Self {
            goal: goal.to_string(),
            phase: PlanPhase::Planning,
            subtasks: Vec::new(),
            context_snapshot: context_snapshot.to_string(),
            risk_notes: Vec::new(),
            created_at: now,
            updated_at: now,
            replan_count: 0,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = now_secs();
    }

    pub fn next_pending(&self) -> Option<&Subtask> {
        self.subtasks.iter().find(|t| t.status == SubtaskStatus::Pending)
    }

    pub fn next_pending_mut(&mut self) -> Option<&mut Subtask> {
        self.subtasks.iter_mut().find(|t| t.status == SubtaskStatus::Pending)
    }

    pub fn current_task_mut(&mut self) -> Option<&mut Subtask> {
        self.subtasks.iter_mut().find(|t| t.status == SubtaskStatus::InProgress)
    }

    pub fn done_count(&self) -> usize {
        self.subtasks.iter().filter(|t| t.status == SubtaskStatus::Done).count()
    }

    pub fn total(&self) -> usize {
        self.subtasks.len()
    }

    pub fn is_complete(&self) -> bool {
        !self.subtasks.is_empty()
            && self.subtasks.iter().all(|t| {
                matches!(t.status, SubtaskStatus::Done | SubtaskStatus::Skipped(_))
            })
    }

    /// Render the plan as a compact dashboard string.
    pub fn render_dashboard(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            &mut out,
            "\n╔═══════════════════════════════════════════════════════╗"
        );
        let _ = writeln!(&mut out, "║  🧠 ASTRA PLAN: {:<38} ║", truncate(&self.goal, 38));
        let _ = writeln!(
            &mut out,
            "╠═══════════════════════════════════════════════════════╣"
        );
        let _ = writeln!(
            &mut out,
            "║  Phase: {:<46} ║",
            format!("{}", self.phase)
        );
        let _ = writeln!(
            &mut out,
            "║  Progress: {}/{} subtasks done{:<24} ║",
            self.done_count(),
            self.total(),
            ""
        );
        if self.replan_count > 0 {
            let _ = writeln!(
                &mut out,
                "║  Replanned: {} time(s){:<33} ║",
                self.replan_count, ""
            );
        }
        let _ = writeln!(
            &mut out,
            "╠═══════════════════════════════════════════════════════╣"
        );
        for task in &self.subtasks {
            let marker = match &task.status {
                SubtaskStatus::Done         => "✅",
                SubtaskStatus::InProgress   => "🔄",
                SubtaskStatus::Blocked(_)   => "🚧",
                SubtaskStatus::Skipped(_)   => "⏭️",
                SubtaskStatus::Pending      => "⬜",
            };
            let _ = writeln!(
                &mut out,
                "║  {}  {}. {:<43} ║",
                marker,
                task.id,
                truncate(&task.title, 43)
            );
            if let Some(ref summary) = task.result_summary {
                let _ = writeln!(
                    &mut out,
                    "║      ↳ {:<49} ║",
                    truncate(summary, 49)
                );
            }
        }
        if !self.risk_notes.is_empty() {
            let _ = writeln!(
                &mut out,
                "╠═══════════════════════════════════════════════════════╣"
            );
            let _ = writeln!(&mut out, "║  ⚠️  RISKS:{:<45} ║", "");
            for risk in &self.risk_notes {
                let _ = writeln!(&mut out, "║     • {:<50} ║", truncate(risk, 50));
            }
        }
        let _ = writeln!(
            &mut out,
            "╚═══════════════════════════════════════════════════════╝"
        );
        out
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let dir = root.join(".astra");
        fs::create_dir_all(&dir)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(dir.join("plan.json"), data)?;
        Ok(())
    }

    pub fn load(root: &Path) -> Option<Self> {
        let path = root.join(".astra").join("plan.json");
        if !path.exists() {
            return None;
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
    }

    pub fn clear(root: &Path) {
        let path = root.join(".astra").join("plan.json");
        let _ = fs::remove_file(path);
    }
}

// ---------------------------------------------------------------------------
// Planner
// ---------------------------------------------------------------------------

pub struct Planner<'a> {
    model: &'a dyn CodexModel,
    root: PathBuf,
}

impl<'a> Planner<'a> {
    pub fn new(model: &'a dyn CodexModel, root: &Path) -> Self {
        Self {
            model,
            root: root.to_path_buf(),
        }
    }

    /// Decompose a goal into an ordered subtask list. Returns a populated Plan.
    pub fn decompose(&self, goal: &str, context: &str) -> Result<Plan> {
        let prompt = format!(
            r#"You are Astra, a senior engineering project manager and technical architect.

## PROJECT CONTEXT
{}

## GOAL
{}

## YOUR JOB
Break this goal into a precise, ordered list of subtasks that can be executed autonomously.

Rules:
- Each subtask must be independently executable (no "it depends" tasks)
- Each subtask must have a crystal-clear acceptance criterion — what does DONE look like?
- Order tasks by dependency: foundational work first, integration last
- Identify risks that could derail the plan
- Be specific about files, modules, or systems involved
- Max 10 subtasks. If the goal is simple, use fewer.
- Do NOT include "write tests" as a separate subtask unless explicitly requested

Respond in this EXACT JSON format (no markdown, no prose):
{{
  "subtasks": [
    {{
      "title": "Short imperative title (max 60 chars)",
      "description": "What exactly needs to be done. Be specific about files/functions/changes.",
      "acceptance": "Concrete, verifiable definition of done."
    }}
  ],
  "risks": [
    "Risk description 1",
    "Risk description 2"
  ]
}}
"#,
            context, goal
        );

        let response = self.model.complete(&prompt)?;
        let json_str = extract_json(&response);

        #[derive(Deserialize)]
        struct DecomposeResponse {
            subtasks: Vec<SubtaskDef>,
            #[serde(default)]
            risks: Vec<String>,
        }
        #[derive(Deserialize)]
        struct SubtaskDef {
            title: String,
            description: String,
            acceptance: String,
        }

        let parsed: DecomposeResponse = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse plan from LLM: {}\n\nRaw: {}", e, &json_str[..json_str.len().min(500)]))?;

        let mut plan = Plan::new(goal, context);
        plan.subtasks = parsed
            .subtasks
            .into_iter()
            .enumerate()
            .map(|(i, def)| Subtask {
                id: i + 1,
                title: def.title,
                description: def.description,
                acceptance: def.acceptance,
                status: SubtaskStatus::Pending,
                result_summary: None,
                files_touched: Vec::new(),
                started_at: None,
                finished_at: None,
            })
            .collect();
        plan.risk_notes = parsed.risks;
        plan.phase = PlanPhase::Executing;
        plan.touch();

        Ok(plan)
    }

    /// Deep planning: decompose, then critique the plan and refine it once.
    /// This catches missing steps, wrong ordering, and unscoped tasks before
    /// any execution happens.
    pub fn decompose_deep(&self, goal: &str, context: &str) -> Result<Plan> {
        let mut plan = self.decompose(goal, context)?;

        // Render the draft plan for critique
        let draft = plan
            .subtasks
            .iter()
            .map(|t| format!("{}. {} — {}\n   acceptance: {}", t.id, t.title, t.description, t.acceptance))
            .collect::<Vec<_>>()
            .join("\n");

        let critique_prompt = format!(
            r#"You are Astra's plan critic — a skeptical staff engineer reviewing a plan
before any work begins.

## GOAL
{}

## PROJECT CONTEXT
{}

## DRAFT PLAN
{}

## YOUR JOB
Find the flaws. Consider:
- Missing steps (setup, dependencies, wiring, integration)
- Wrong ordering (something depends on a later step)
- Vague or unverifiable subtasks
- Scope creep or redundant steps

If the plan is already excellent, respond with exactly: PLAN_OK

Otherwise, respond with a CORRECTED full plan in this EXACT JSON format:
{{
  "subtasks": [
    {{"title": "...", "description": "...", "acceptance": "..."}}
  ],
  "risks": ["..."]
}}"#,
            goal, context, draft
        );

        let response = self.model.complete(&critique_prompt)?;
        if response.trim().contains("PLAN_OK") {
            return Ok(plan);
        }

        // Try to parse the refined plan; if it fails, keep the original
        let json_str = extract_json(&response);

        #[derive(Deserialize)]
        struct DecomposeResponse {
            subtasks: Vec<SubtaskDef>,
            #[serde(default)]
            risks: Vec<String>,
        }
        #[derive(Deserialize)]
        struct SubtaskDef {
            title: String,
            description: String,
            acceptance: String,
        }

        if let Ok(parsed) = serde_json::from_str::<DecomposeResponse>(&json_str) {
            if !parsed.subtasks.is_empty() {
                plan.subtasks = parsed
                    .subtasks
                    .into_iter()
                    .enumerate()
                    .map(|(i, def)| Subtask {
                        id: i + 1,
                        title: def.title,
                        description: def.description,
                        acceptance: def.acceptance,
                        status: SubtaskStatus::Pending,
                        result_summary: None,
                        files_touched: Vec::new(),
                        started_at: None,
                        finished_at: None,
                    })
                    .collect();
                if !parsed.risks.is_empty() {
                    plan.risk_notes = parsed.risks;
                }
                plan.replan_count += 1; // count the refinement pass
                plan.touch();
            }
        }

        Ok(plan)
    }

    /// After a subtask attempt, reflect on what happened and update the subtask.
    /// Returns whether the subtask should be marked Done (true) or needs retry/skip (false).
    pub fn reflect_on_subtask(
        &self,
        plan: &Plan,
        subtask_idx: usize,
        execution_log: &str,
    ) -> Result<ReflectionResult> {
        let subtask = &plan.subtasks[subtask_idx];
        let remaining: Vec<&Subtask> = plan.subtasks[subtask_idx + 1..]
            .iter()
            .filter(|t| t.status == SubtaskStatus::Pending)
            .collect();

        let prompt = format!(
            r#"You are Astra's reflection engine. You just attempted a subtask and need to assess the outcome.

## ORIGINAL GOAL
{}

## SUBTASK ATTEMPTED
Title: {}
Description: {}
Acceptance criterion: {}

## EXECUTION LOG
{}

## REMAINING PLAN (subtasks not yet started)
{}

## YOUR JOB
Assess what happened and decide what to do next.

Respond in this EXACT JSON format:
{{
  "verdict": "done" | "retry" | "skip" | "blocked",
  "reason": "Why you chose this verdict. Be specific.",
  "result_summary": "One sentence summary of what was accomplished (or what failed).",
  "files_touched": ["list", "of", "modified", "file", "paths"],
  "replan_needed": true | false,
  "replan_additions": [
    {{
      "title": "New subtask title",
      "description": "What needs to happen",
      "acceptance": "Definition of done"
    }}
  ],
  "blocker": "If verdict is blocked, what is blocking progress. Otherwise empty string."
}}
"#,
            plan.goal,
            subtask.title,
            subtask.description,
            subtask.acceptance,
            execution_log,
            remaining.iter().map(|t| format!("- {}: {}", t.id, t.title)).collect::<Vec<_>>().join("\n"),
        );

        let response = self.model.complete(&prompt)?;
        let json_str = extract_json(&response);

        let result: ReflectionResult = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse reflection: {}\n\nRaw: {}", e, &json_str[..json_str.len().min(500)]))?;

        Ok(result)
    }

    /// Generate a plan status summary for the user.
    pub fn status_report(&self, plan: &Plan) -> String {
        let mut out = plan.render_dashboard();

        // Add a brief narrative
        let done = plan.done_count();
        let total = plan.total();

        if plan.is_complete() {
            out.push_str("\n🎉 All subtasks are complete! The goal has been achieved.\n");
        } else if done == 0 {
            out.push_str("\n⚙️  Execution has not started yet. Run `:task` to begin.\n");
        } else {
            let pct = (done * 100) / total.max(1);
            out.push_str(&format!(
                "\n📊 {done}/{total} subtasks done ({pct}%). Run `:task` to continue.\n"
            ));
        }
        out
    }

    /// Replan: add new subtasks discovered during reflection.
    pub fn apply_replan(
        &self,
        plan: &mut Plan,
        after_idx: usize,
        additions: Vec<NewSubtask>,
    ) {
        if additions.is_empty() {
            return;
        }
        let start_id = plan.subtasks.len() + 1;
        let new_tasks: Vec<Subtask> = additions
            .into_iter()
            .enumerate()
            .map(|(i, def)| Subtask {
                id: start_id + i,
                title: def.title,
                description: def.description,
                acceptance: def.acceptance,
                status: SubtaskStatus::Pending,
                result_summary: None,
                files_touched: Vec::new(),
                started_at: None,
                finished_at: None,
            })
            .collect();

        // Insert after current subtask index
        let insert_at = (after_idx + 1).min(plan.subtasks.len());
        for (i, task) in new_tasks.into_iter().enumerate() {
            plan.subtasks.insert(insert_at + i, task);
        }
        plan.replan_count += 1;
        plan.touch();
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReflectionResult {
    pub verdict: ReflectionVerdict,
    pub reason: String,
    pub result_summary: String,
    #[serde(default)]
    pub files_touched: Vec<String>,
    #[serde(default)]
    pub replan_needed: bool,
    #[serde(default)]
    pub replan_additions: Vec<NewSubtask>,
    #[serde(default)]
    pub blocker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReflectionVerdict {
    Done,
    Retry,
    Skip,
    Blocked,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewSubtask {
    pub title: String,
    pub description: String,
    pub acceptance: String,
}

// ---------------------------------------------------------------------------
// Engine integration helpers
// ---------------------------------------------------------------------------

/// Build the context snapshot string passed to the planner.
/// Summarizes what Astra knows about the project right now.
pub fn build_context_snapshot(
    root: &Path,
    file_count: usize,
    total_lines: usize,
    top_langs: &[(String, usize)],
    memory_facts: &[String],
) -> String {
    let mut ctx = String::new();
    let _ = writeln!(&mut ctx, "Project root: {}", root.display());
    let _ = writeln!(&mut ctx, "Indexed: {} files, {} lines", file_count, total_lines);
    if !top_langs.is_empty() {
        let langs = top_langs
            .iter()
            .take(4)
            .map(|(l, c)| format!("{} ({})", l, c))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(&mut ctx, "Top languages: {}", langs);
    }
    // Top-level directories
    if let Ok(entries) = fs::read_dir(root) {
        let dirs: Vec<String> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| !n.starts_with('.') && n != "node_modules" && n != "target")
            .collect();
        if !dirs.is_empty() {
            let _ = writeln!(&mut ctx, "Directories: {}", dirs.join(", "));
        }
    }
    if !memory_facts.is_empty() {
        let _ = writeln!(&mut ctx, "Known facts:");
        for fact in memory_facts.iter().take(6) {
            let _ = writeln!(&mut ctx, "  - {}", fact);
        }
    }
    ctx
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

/// Extract the first JSON object from an LLM response, stripping prose and fences.
fn extract_json(text: &str) -> String {
    let trimmed = text.trim();

    // Strip ```json ... ``` fence
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(start) = trimmed.find("```\n") {
        let after = &trimmed[start + 4..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }

    // Find first { ... } block
    if let Some(start) = trimmed.find('{') {
        let substr = &trimmed[start..];
        // Find balanced closing brace
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in substr.char_indices() {
            if escape { escape = false; continue; }
            if ch == '\\' && in_string { escape = true; continue; }
            if ch == '"' { in_string = !in_string; continue; }
            if in_string { continue; }
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return substr[..=i].to_string();
                    }
                }
                _ => {}
            }
        }
        return substr.to_string();
    }

    trimmed.to_string()
}
