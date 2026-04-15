use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::engine::CodexEngine;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskPhase {
    Planned,
    InProgress,
    Reviewing,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrchestratedTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub phase: TaskPhase,
    pub last_update: u64,
}

pub fn generate_next_task(engine: &mut CodexEngine, goal: Option<&str>) -> Result<String> {
    let root = engine.root().to_path_buf();
    let active = load_active_task(&root);

    // 1. If we have an active task and NO new explicit goal, continue the existing task
    if goal.is_none() {
        if let Some(task) = &active {
            if task.phase == TaskPhase::Reviewing {
                return generate_review_step(engine, task);
            }
            if task.phase == TaskPhase::InProgress {
                return Ok(json_plan(task, "Currently Executing... Continuing original plan."));
            }
        }
    }

    // 2. If there IS a new explicit goal, archive the old task and proceed
    if goal.is_some() {
        if let Some(old_task) = &active {
            engine.memory_mut().add(
                "task-archived",
                format!("Archived previous task '{}' (phase: {:?}) to make way for new goal.", old_task.title, old_task.phase),
            );
        }
    }

    // 2. No active task or done, look for a new goal
    let task_id = format!("astra-task-{}", now_secs());
    let inferred_goal = if goal.is_none() {
        engine.memory().recent(10).iter().rev()
            .find(|e| e.kind == "command" || e.kind == "chat")
            .map(|e| e.content.clone())
    } else {
        None
    };

    let title = goal.map(|g| g.to_string())
        .or(inferred_goal)
        .unwrap_or_else(|| "Resolve Hot Files & Health Debt".to_string());
    
    let description = if title.contains("Health") {
        "Astra identified technical debt in the codebase index. Implement these fixes to restore perfect health scores."
    } else {
        "Astra has analyzed this specific user goal and formulated an execution plan based on our previous conversation."
    };

    let new_task = OrchestratedTask {
        id: task_id.clone(),
        title: title.clone(),
        description: description.to_string(),
        phase: TaskPhase::InProgress,
        last_update: now_secs(),
    };

    save_active_task(&root, &new_task)?;

    // Save the delegation event in memory
    engine.memory_mut().add(
        "orchestrator-delegated",
        format!("Delegated task {} to IDE: {}", task_id, title)
    );

    Ok(json_plan(&new_task, description))
}

pub fn process_task_result(engine: &mut CodexEngine, task_id: &str, success: bool, details: &str) -> Result<String> {
    let root = engine.root().to_path_buf();
    let mut task = match load_active_task(&root) {
        Some(t) if t.id == task_id => t,
        _ => return Ok(format!("Task {} not found or not active.", task_id)),
    };

    if !success {
        task.phase = TaskPhase::Planned; // Reset or mark failed
        save_active_task(&root, &task)?;
        return Ok(format!("Astra logged the failure of {}. Details: {}", task_id, details));
    }

    // Move to Reviewing phase
    task.phase = TaskPhase::Reviewing;
    task.last_update = now_secs();
    save_active_task(&root, &task)?;

    let msg = format!("Task {} completed. Astra is now moving to the Review phase.", task_id);
    engine.memory_mut().add("orchestrator-result", msg.clone());

    Ok(msg)
}

// --- Internal Helpers ---

fn load_active_task(root: &Path) -> Option<OrchestratedTask> {
    let path = root.join(".astra").join("active_task.json");
    if !path.exists() { return None; }
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_active_task(root: &Path, task: &OrchestratedTask) -> Result<()> {
    let dir = root.join(".astra");
    if !dir.exists() { fs::create_dir_all(&dir)?; }
    let content = serde_json::to_string_pretty(task)?;
    fs::write(dir.join("active_task.json"), content)?;
    
    // Proactively "shout" the task to the IDE by updating the rules context
    let _ = inject_task_to_ide_context(root, task);
    
    Ok(())
}

fn inject_task_to_ide_context(root: &Path, task: &OrchestratedTask) -> Result<()> {
    // Build a self-contained task block that tells the IDE agent EXACTLY what to do
    let task_block = format!(
        r#"# ASTRA ACTIVE TASK
# ⚡ IMMEDIATE ACTION REQUIRED — DO NOT ASK FOR CLARIFICATION ⚡
#
# Your current mission assigned by Astra (the Tech Lead):
#   GOAL: {}
#   PHASE: {:?}
#   TASK ID: {}
#
# INSTRUCTIONS:
# 1. You are an AI agent working under Astra's orchestration.
# 2. Your ONLY job right now is to execute the goal above.
# 3. Analyze the relevant files, make the changes, and verify they compile.
# 4. Do NOT ask "what should I work on?" — the answer is above.
# 5. When done, report success by noting what you changed.
#
# DESCRIPTION: {}
#
# END ASTRA TASK
"#,
        task.title, task.phase, task.id, task.description
    );

    // Helper: strip old Astra blocks and prepend the new one
    let inject = |path: &std::path::Path| -> Result<()> {
        let existing = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let mut clean_lines = Vec::new();
        let mut skipping = false;
        for line in existing.lines() {
            if line.contains("# ASTRA ACTIVE TASK") {
                skipping = true;
                continue;
            }
            if line.contains("# END ASTRA TASK") {
                skipping = false;
                continue;
            }
            if !skipping {
                clean_lines.push(line);
            }
        }

        let mut final_content = task_block.clone();
        final_content.push('\n');
        final_content.push_str(&clean_lines.join("\n"));

        fs::write(path, final_content)?;
        Ok(())
    };

    // Always write to BOTH rule files
    inject(&root.join(".cursorrules"))?;
    inject(&root.join(".windsurfrules"))?;

    Ok(())
}

fn generate_review_step(_engine: &mut CodexEngine, task: &OrchestratedTask) -> Result<String> {
    let plan = json!({
        "task_id": task.id,
        "title": format!("REVIEW: {}", task.title),
        "priority": "critical",
        "context": "Astra is now analyzing your changes to see if they are production-ready.",
        "steps": [
            {
                "action": "analyze",
                "description": "Examine the diff and verify it matches the architectural requirements of the project.",
            },
            {
                "action": "verify",
                "description": "Run the build and all relevant tests. Confirm NO regressions were introduced.",
            }
        ]
    });
    
    Ok(serde_json::to_string_pretty(&plan).unwrap_or_default())
}

fn json_plan(task: &OrchestratedTask, context: &str) -> String {
    let plan = json!({
        "task_id": task.id,
        "title": task.title,
        "priority": "high",
        "context": context,
        "steps": [
            {
                "action": "analyze",
                "description": format!("Inspect the files related to '{}' to verify types and dependencies before editing.", task.title),
            },
            {
                "action": "execute",
                "description": "Write or refactor the required code. Strictly adhere to existing architectural patterns.",
            },
            {
                "action": "verify",
                "description": "Run the appropriate linter, compiler, or build commands.",
            }
        ]
    });
    serde_json::to_string_pretty(&plan).unwrap_or_default()
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
