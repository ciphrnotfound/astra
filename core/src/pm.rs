//! Astra's Product-Manager brain.
//!
//! Turns a vague, half-formed idea ("I want a login thing") into a crisp,
//! well-scoped specification that an AI coding agent can execute reliably.
//! It identifies the missing information, drafts clarifying questions, and
//! emits a structured spec with goals, scope, acceptance criteria, and an
//! AI-agent-ready prompt.

use serde::{Deserialize, Serialize};

use crate::model::CodexModel;

/// A drafted specification produced from a rough idea.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Spec {
    pub title: String,
    pub problem: String,
    pub goals: Vec<String>,
    pub non_goals: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub open_questions: Vec<String>,
    pub suggested_stack: Vec<String>,
    /// A ready-to-paste prompt for an AI coding agent (Claude Code, Cursor, etc.)
    pub agent_prompt: String,
}

impl Spec {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# 📋 {}\n\n", self.title));
        out.push_str(&format!("## Problem\n{}\n\n", self.problem));

        out.push_str("## Goals\n");
        for g in &self.goals {
            out.push_str(&format!("- {}\n", g));
        }
        out.push('\n');

        if !self.non_goals.is_empty() {
            out.push_str("## Non-Goals (out of scope)\n");
            for ng in &self.non_goals {
                out.push_str(&format!("- {}\n", ng));
            }
            out.push('\n');
        }

        out.push_str("## Acceptance Criteria\n");
        for (i, ac) in self.acceptance_criteria.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, ac));
        }
        out.push('\n');

        if !self.suggested_stack.is_empty() {
            out.push_str(&format!("## Suggested Stack\n{}\n\n", self.suggested_stack.join(", ")));
        }

        if !self.open_questions.is_empty() {
            out.push_str("## ❓ Open Questions (answer these to sharpen the spec)\n");
            for q in &self.open_questions {
                out.push_str(&format!("- {}\n", q));
            }
            out.push('\n');
        }

        out.push_str("## 🤖 AI Agent Prompt (paste into Claude Code / Cursor)\n");
        out.push_str("```\n");
        out.push_str(self.agent_prompt.trim());
        out.push_str("\n```\n");
        out
    }
}

pub struct ProductManager<'a> {
    model: &'a dyn CodexModel,
}

impl<'a> ProductManager<'a> {
    pub fn new(model: &'a dyn CodexModel) -> Self {
        Self { model }
    }

    /// Draft a full specification from a rough idea + project context.
    pub fn draft_spec(&self, idea: &str, context: &str) -> anyhow::Result<Spec> {
        let prompt = format!(
            r#"You are Astra acting as a sharp, experienced product manager.
A developer gave you a rough idea. Turn it into a crisp, actionable spec that
an AI coding agent can execute without ambiguity.

## PROJECT CONTEXT
{}

## THE DEVELOPER'S ROUGH IDEA
{}

## YOUR JOB
Produce a complete specification. Where the idea is vague, make sensible,
explicit decisions (and note assumptions). Then write a ready-to-paste prompt
that another AI coding agent could execute to build this.

Respond in EXACTLY this JSON format (no markdown fences, no prose outside JSON):
{{
  "title": "Short feature/project title",
  "problem": "1-2 sentences: what problem does this solve and for whom",
  "goals": ["concrete goal 1", "concrete goal 2"],
  "non_goals": ["explicitly out of scope item 1"],
  "acceptance_criteria": ["verifiable criterion 1", "verifiable criterion 2"],
  "open_questions": ["question that would meaningfully change the design"],
  "suggested_stack": ["library or tool 1", "library or tool 2"],
  "agent_prompt": "A detailed, self-contained prompt instructing an AI coding agent exactly what to build, including file structure, key functions, and constraints. Write it as if briefing a capable engineer."
}}"#,
            context, idea
        );

        let response = self.model.complete(&prompt)?;
        let json_str = extract_json(&response);

        let spec: Spec = serde_json::from_str(&json_str).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse spec: {}\n\nRaw: {}",
                e,
                &json_str[..json_str.len().min(500)]
            )
        })?;
        Ok(spec)
    }

    /// Generate clarifying questions only — a quick interview before committing to a spec.
    pub fn clarifying_questions(&self, idea: &str, context: &str) -> anyhow::Result<Vec<String>> {
        let prompt = format!(
            r#"You are Astra, a product manager. A developer wants to build:

"{}"

## PROJECT CONTEXT
{}

List the 3-5 MOST IMPORTANT clarifying questions whose answers would change the
design or scope. Skip trivial questions. Output one question per line, no numbering,
no preamble."#,
            idea, context
        );
        let response = self.model.complete(&prompt)?;
        let questions: Vec<String> = response
            .lines()
            .map(|l| l.trim().trim_start_matches(['-', '*', '•', ' ']).to_string())
            .filter(|l| l.len() > 5 && l.contains('?'))
            .take(5)
            .collect();
        Ok(questions)
    }

    /// Critique a draft goal and suggest a sharper version.
    pub fn sharpen_goal(&self, goal: &str, context: &str) -> anyhow::Result<String> {
        let prompt = format!(
            r#"You are Astra, a product manager helping a developer sharpen a goal
before handing it to an AI coding agent.

## PROJECT CONTEXT
{}

## DRAFT GOAL
{}

Rewrite this as a sharp, unambiguous, scoped goal an AI agent can execute.
Then list 2-3 concrete acceptance criteria. Keep it tight. Format:

SHARPENED GOAL: <one clear sentence>

ACCEPTANCE CRITERIA:
- <criterion>
- <criterion>"#,
            context, goal
        );
        self.model.complete(&prompt)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_json(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(start) = trimmed.find('{') {
        let substr = &trimmed[start..];
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
