use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use crate::agent::{self, AgentConfig};
use crate::config::load_global_config;
use crate::git::GitRepo;
use crate::health;
use crate::index::{is_indexable_path, CodeIndex};
use crate::memory::{compact_for_context, MemoryEntry, MemoryEvent, MemoryStore};
use crate::migrate;
use crate::migrate::detect::Language;
use crate::migrate::orchestrate::MigrationConfig;
use crate::migration;
use crate::model::{CodexModel, SearchProvider, EmbeddingProvider};
use crate::parser::{parse_rust_file, ParsedSymbolKind};
use crate::persona::Persona;
use crate::rag::{VectorStore, chunk_file, format_chunks_for_prompt};
use crate::semantic_graph::TemporalGraph;
use crate::semantic_memory::ConceptStore;
use crate::scaffold;
use crate::teams::TeamManager;
use crate::time_travel;
use crate::ts_migrate;
use crate::workflow::WorkflowManager;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".idea",
    ".vscode",
    ".astra",
    ".forge",
    ".codex",
    "vendor",
    "bin",
    "obj",
];

const HELP_TEXT: &str = "\
◈ ASTRA — Command Reference

🧠 PLAN & EXECUTE
  :task <goal>        Decompose a goal into subtasks and autonomously execute it
  :plan               Show the current plan dashboard
  :plan resume        Continue a paused plan
  :plan clear         Discard the current plan

📋 PRODUCT MANAGER
  :spec <idea>        Turn a rough idea into a full spec + AI-agent prompt
  :ask <idea>         Get the key clarifying questions to scope an idea
  :sharpen <goal>     Rewrite a vague goal into a sharp, scoped one

🤝 COWORK
  :cowork init        Connect Astra to Codex, Claude Code, and Cursor via MCP
  :delegate <worker> <goal>   Queue work for codex, claude, cursor, or any
  :dispatch <worker> <goal>   Launch an installed worker CLI on the job
  :jobs               Show the shared cross-editor job board

🛡️  REVIEW (don't ship rubbish)
  :review             Review your changed files & give a ship verdict
  :review all         Review the entire codebase
  :review --install-gate   Block commits with critical issues (git hook)

🚀 SHIP (git + GitHub)
  :commit             Stage all changes & commit with an AI-drafted message
  :ship               Commit + push to the remote branch
  :pr [base]          Open a pull request (drafts title + body). Default base: main
  :release <tag>      Cut a GitHub release with AI-drafted notes
  :standup            Quick standup: tree state, recent work, next step

🔍 UNDERSTAND
  :search <query>     RAG search over your code (semantic or keyword)
  :inspect <path>     Grounded summary of a file or directory
  :index              Index the codebase + build the semantic/temporal graph
  :audit              Re-index everything, then compute grounded project health
  :health             Codebase health dashboard
  :summary            Project summary
  :history            Project evolution, contributors, and major hotspots
  :hotspots           Most changed files and risky coupling areas
  :why <path>         Why does this file exist / its history
  :owners             Who owns what (bus-factor)
  :predict            Predictive refactoring / debt forecast

🔧 TRANSFORM
  :migrate <src> <from> <to> <out> [--ai]   Cross-language migration
  :fix <path> <bug>   Legacy single-file fix (verified against build)
  :fix-bug <report>   Capture, triage, and delegate a bug report
  :issues             List tracked bug reports
  :vibe <name>        Switch persona

Type any natural-language question to chat. :quit to exit.";

pub struct CodexEngine {
    root: PathBuf,
    index: CodeIndex,
    model: Option<Box<dyn CodexModel + Send + Sync>>,
    search: Option<Box<dyn SearchProvider + Send + Sync>>,
    embedder: Option<Box<dyn EmbeddingProvider + Send + Sync>>,
    memory: MemoryStore,
    git: Option<GitRepo>,
    persona: Persona,
    agent_mode: bool,
    auto_approve: bool,
    temporal_graph: TemporalGraph,
    concept_store: ConceptStore,
    vector_store: VectorStore,
}

impl CodexEngine {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn memory(&self) -> &MemoryStore {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut MemoryStore {
        &mut self.memory
    }

    pub fn index(&self) -> &CodeIndex {
        &self.index
    }

    pub fn set_agent_mode(&mut self, enabled: bool) {
        self.agent_mode = enabled;
    }

    pub fn set_auto_approve(&mut self, enabled: bool) {
        self.auto_approve = enabled;
    }

    pub fn new() -> Self {
        let root = PathBuf::from(".");
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        let mut memory = MemoryStore::load(memory_path);
        memory.attach_global(MemoryStore::load_global());
        
        // Load Global Brain Storage
        let global_brain = crate::config::get_global_brain_path(&root);
        let temporal_graph = TemporalGraph::load(&global_brain.join("temporal_graph.json"))
            .unwrap_or_else(|_| TemporalGraph::new());
        let concept_store = ConceptStore::load(&global_brain.join("concepts.json"))
            .unwrap_or_else(|_| ConceptStore::new());
        let index = CodeIndex::load(&global_brain.join("index.json"))
            .unwrap_or_else(|_| CodeIndex::new());
        let vector_store = VectorStore::load(&global_brain.join("vectors.json"))
            .unwrap_or_else(|_| VectorStore::new());

        register_global_project(&root);

        Self {
            root,
            index,
            model: None,
            search: None,
            embedder: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
            vector_store,
        }
    }

    pub fn with_root(root: PathBuf) -> Self {
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        let mut memory = MemoryStore::load(memory_path);
        memory.attach_global(MemoryStore::load_global());

        // Load Global Brain Storage
        let global_brain = crate::config::get_global_brain_path(&root);
        let temporal_graph = TemporalGraph::load(&global_brain.join("temporal_graph.json"))
            .unwrap_or_else(|_| TemporalGraph::new());
        let concept_store = ConceptStore::load(&global_brain.join("concepts.json"))
            .unwrap_or_else(|_| ConceptStore::new());
        let index = CodeIndex::load(&global_brain.join("index.json"))
            .unwrap_or_else(|_| CodeIndex::new());
        let vector_store = VectorStore::load(&global_brain.join("vectors.json"))
            .unwrap_or_else(|_| VectorStore::new());

        register_global_project(&root);

        Self {
            root,
            index,
            model: None,
            search: None,
            embedder: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
            vector_store,
        }
    }

    pub fn with_model(root: PathBuf, model: Box<dyn CodexModel + Send + Sync>) -> Self {
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        let mut memory = MemoryStore::load(memory_path);
        memory.attach_global(MemoryStore::load_global());

        // Load Global Brain Storage
        let global_brain = crate::config::get_global_brain_path(&root);
        let temporal_graph = TemporalGraph::load(&global_brain.join("temporal_graph.json"))
            .unwrap_or_else(|_| TemporalGraph::new());
        let concept_store = ConceptStore::load(&global_brain.join("concepts.json"))
            .unwrap_or_else(|_| ConceptStore::new());
        let index = CodeIndex::load(&global_brain.join("index.json"))
            .unwrap_or_else(|_| CodeIndex::new());
        let vector_store = VectorStore::load(&global_brain.join("vectors.json"))
            .unwrap_or_else(|_| VectorStore::new());

        register_global_project(&root);

        Self {
            root,
            index,
            model: Some(model),
            search: None,
            embedder: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
            vector_store,
        }
    }

    pub fn set_persona(&mut self, persona: Persona) {
        self.persona = persona;
    }

    pub fn set_model(&mut self, model: Box<dyn CodexModel + Send + Sync>) {
        self.model = Some(model);
    }

    pub fn index_mut(&mut self) -> &mut CodeIndex {
        &mut self.index
    }

    pub fn has_search(&self) -> bool {
        self.search.is_some()
    }

    pub fn set_search(&mut self, search: Box<dyn SearchProvider + Send + Sync>) {
        self.search = Some(search);
    }

    pub fn set_embedder(&mut self, embedder: Box<dyn EmbeddingProvider + Send + Sync>) {
        self.embedder = Some(embedder);
    }

    fn get_query_embedding(&self, text: &str) -> Option<Vec<f32>> {
        self.embedder.as_ref().and_then(|e| e.get_embedding(text).ok())
    }

    pub fn research_language(&mut self, lang: Language) -> Result<String> {
        let query = format!("idiomatic {} 2024 2025 syntax patterns and standard libraries", lang);
        if let Some(search) = &self.search {
            let results = search.search(&query)?;
            if let Some(model) = &self.model {
                let prompt = format!(
                    "You are an expert software architect. Based on these search results, \
                    summarize the LATEST (2024-2025) idiomatic syntax, project structure, and patterns for {}.\n\n\
                    Search Results:\n{}",
                    lang, results
                );
                let summary = model.complete(&prompt)?;
                let kind = format!("best-practices:{}", lang.to_string().to_lowercase());
                self.memory.add(&kind, summary.clone());
                return Ok(summary);
            }
        }
        Ok("Web search not available; using default AI knowledge.".to_string())
    }

    /// Public command boundary. Every successful command is recorded as a
    /// tiny, keyed memory snapshot so the next short follow-up can refer to
    /// what Astra actually found without replaying the full transcript.
    pub fn handle_input(&mut self, input: &str) -> Result<String> {
        let is_command = input.trim().starts_with(':') || self.intent_for(input).is_some();
        let response = self.handle_input_inner(input)?;
        if is_command {
            self.remember_last_command(input, &response);
        }
        Ok(response)
    }

    fn handle_input_inner(&mut self, input: &str) -> Result<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok("Say something about your codebase to get started.".to_string());
        }

        // Resolve confirmations before every other intent. A bare "yup" is
        // meaningful only when Astra has a concrete pending action.
        if let Some(pending) = self.memory.conversation_state("pending_action") {
            if is_confirmation(trimmed) {
                self.memory.clear_conversation_state("pending_action");
                self.memory.clear_conversation_state("pending_action_subject");
                if pending == "abandon_active_task" {
                    return self.abandon_active_task();
                }
            } else if is_confirmation_rejection(trimmed) {
                self.memory.clear_conversation_state("pending_action");
                self.memory.clear_conversation_state("pending_action_subject");
                return Ok("Okay - I left the active task untouched.".to_string());
            }
        }

        if is_abandon_active_task_request(trimmed) {
            let task_path = self.root.join(".astra").join("active_task.json");
            if let Ok(task_content) = std::fs::read_to_string(&task_path) {
                let title = serde_json::from_str::<serde_json::Value>(&task_content)
                    .ok()
                    .and_then(|task| {
                        task.get("title")
                            .and_then(|value| value.as_str())
                            .map(str::to_owned)
                    })
                    .unwrap_or_else(|| "the current task".to_string());
                self.memory
                    .remember_conversation_state("pending_action", "abandon_active_task");
                self.memory
                    .remember_conversation_state("pending_action_subject", &title);
                return Ok(format!(
                    "Got it - drop the active task **{}**. Say `yes` to confirm. I won't delete any project files.",
                    title
                ));
            }
            return Ok(
                "There isn't an active task to drop. Your project files are untouched.".to_string(),
            );
        }

        // ── Fast-path: instant greetings (skip ALL heavy processing) ──
        {
            let lower = trimmed.to_ascii_lowercase();
            if is_social_message(trimmed) {
                let user = crate::config::load_global_config()
                    .user
                    .unwrap_or_else(|| "there".to_string());
                if is_social_wellbeing_question(trimmed) {
                    return Ok(format!(
                        "Hey, {} - I'm good and locked in. What are we working on?",
                        user
                    ));
                }
                return Ok(format!("Hey, {} - what's up?", user));
            }

            if lower.contains("why are you stuck") || lower.contains("why're you stuck") {
                return Ok("I'm not stuck. If I implied I was busy or blocked on a project without evidence, that was wrong. I only have a task in progress when the saved task state says so.".to_string());
            }

            // Fast-path: "what are we doing" / "what's the agenda" — check active task instantly
            if lower.contains("what are we doing") || lower.contains("what were we doing")
                || lower.contains("on the agenda") || lower.contains("whats on the agenda")
                || lower.contains("what's on the agenda") || lower.contains("what was i doing")
                || lower.contains("what were we working on") || lower.contains("what are we working on")
                || lower.contains("previous task") || lower.contains("current task")
                || lower.contains("active task") || lower.contains("check the task")
            {
                let task_path = self.root.join(".astra").join("active_task.json");
                if let Ok(task_content) = std::fs::read_to_string(&task_path) {
                    if let Ok(task) = serde_json::from_str::<serde_json::Value>(&task_content) {
                        let title = task.get("title").and_then(|t| t.as_str()).unwrap_or("unknown");
                        let phase = task.get("phase").and_then(|t| t.as_str()).unwrap_or("unknown");
                        let desc = task.get("description").and_then(|t| t.as_str()).unwrap_or("");
                        return Ok(format!(
                            "📌 **Active Task:** {}\n   Status: {}\n   {}\n\nWant to continue this, or start something new?",
                            title, phase, desc
                        ));
                    }
                }
                return Ok("No active task found. Tell me what you'd like to work on!".to_string());
            }

            // Resume autonomous work only from an explicit execution request.
            // A conversational acknowledgement such as "sure" or "okay" is
            // never sufficient authorization to launch tools or modify files.
            if is_explicit_continue_request(trimmed) {
                let task_path = self.root.join(".astra").join("active_task.json");
                if let Ok(task_content) = std::fs::read_to_string(&task_path) {
                    if let Ok(task) = serde_json::from_str::<serde_json::Value>(&task_content) {
                        if let Some(title) = task.get("title").and_then(|t| t.as_str()) {
                            println!(" ⚙️ Launching agent to continue: {}", title);
                            return self.execute_task(title);
                        }
                    }
                }
                return Ok("No active task found. Use `:task <goal>` to create one.".to_string());
            }
        }

        let mut normalized = trimmed.to_string();
        if normalized.starts_with('›') {
            normalized = normalized.trim_start_matches('›').trim_start().to_string();
        }

        if normalized == ":cowork init" || normalized == ":cowork setup" {
            let executable = std::env::current_exe()?;
            let report = crate::coworker::install_editor_bridges(&self.root, &executable)?;
            self.memory.remember_project_fact(
                "coworker",
                "Astra MCP is configured for Codex, Claude Code, and Cursor",
            );
            return Ok(report.render());
        }

        if let Some(scope) = normalized.strip_prefix(":inspect ") {
            return self.inspect_scope(scope.trim());
        }

        if normalized == ":jobs" || normalized == ":cowork jobs" {
            let jobs = crate::coworker::CoworkStore::new(&self.root).list(30)?;
            if jobs.is_empty() {
                return Ok("The cowork job board is empty. Use `:delegate <worker> <goal>`.".to_string());
            }
            let mut output = String::from("Astra cowork jobs:\n");
            for job in jobs {
                let worker = job
                    .claimed_by
                    .as_deref()
                    .or(job.preferred_worker.as_deref())
                    .unwrap_or("any");
                let _ = writeln!(
                    &mut output,
                    "- {} [{:?}] {} — {}",
                    job.id, job.status, worker, job.goal
                );
            }
            return Ok(output.trim_end().to_string());
        }

        if let Some(rest) = normalized.strip_prefix(":delegate ") {
            let Some((worker, goal)) = parse_delegate_args(rest) else {
                return Ok("Usage: `:delegate <codex|claude|cursor|any> <goal>`.".to_string());
            };
            let store = crate::coworker::CoworkStore::new(&self.root);
            let job = store.create_job(goal, Some(worker), Vec::new())?;
            self.memory.add(
                "cowork-job",
                format!("Created {} for {}: {}", job.id, worker, job.goal),
            );
            return Ok(format!(
                "Queued **{}** for **{}** as `{}`. The worker can claim it through Astra MCP with `astra_claim_job`.",
                job.goal, worker, job.id
            ));
        }

        // Resolve the numbered action menu before social/general Q&A routing.
        // This keeps inputs such as “yeah, do all that” and “1,2,3” attached
        // to the menu Astra just displayed instead of sending them to the LLM.
        if self.memory.conversation_state("last_option_menu").is_some() {
            if let Some(selected) = parse_option_selection(trimmed) {
                self.memory.clear_conversation_state("last_option_menu");
                return self.execute_option_selection(&selected);
            }
        }

        if let Some(focused_path) = self.memory.conversation_state("focused_path") {
            if !is_focused_scope_followup(&trimmed.to_ascii_lowercase(), &focused_path) {
                if is_command_followup(trimmed) {
                    if let Some(answer) = self.command_followup_answer(trimmed) {
                        return Ok(answer);
                    }
                }
            }
        } else if is_command_followup(trimmed) {
            if let Some(answer) = self.command_followup_answer(trimmed) {
                return Ok(answer);
            }
        }

        if let Some(rest) = normalized.strip_prefix(":dispatch ") {
            let (worker, goal) = rest
                .split_once(char::is_whitespace)
                .map(|(worker, goal)| (worker.trim(), goal.trim()))
                .unwrap_or(("", ""));
            if worker.is_empty() || goal.is_empty() {
                return Ok("Usage: `:dispatch <codex|claude|cursor> <goal>`.".to_string());
            }
            let store = crate::coworker::CoworkStore::new(&self.root);
            let job = store.create_job(goal, Some(worker), Vec::new())?;
            let job = store.claim(&job.id, worker)?;
            let prompt = format!(
                "{}\n\n{}",
                job.worker_prompt(),
                self.build_cowork_context(goal, 2_500)
            );
            let run = match crate::coworker::dispatch_worker(&self.root, worker, &prompt) {
                Ok(run) => run,
                Err(error) => {
                    let _ = store.report(
                        &job.id,
                        Some(worker),
                        crate::coworker::CoworkJobStatus::Failed,
                        &error.to_string(),
                        Vec::new(),
                        Vec::new(),
                    );
                    return Err(error);
                }
            };
            self.memory.add(
                "cowork-result",
                format!(
                    "Dispatched {} to {} (exit {:?}, success={}): {}",
                    job.id, worker, run.exit_code, run.success, run.output
                ),
            );
            let latest_status = store
                .get(&job.id)?
                .map(|latest| format!("{:?}", latest.status))
                .unwrap_or_else(|| "unknown".to_string());
            return Ok(format!(
                "{} finished its process for `{}` (exit {:?}; Astra job status: {}).\n\n{}",
                worker, job.id, run.exit_code, latest_status, run.output
            ));
        }

        self.record_git_commit();
        self.record_worktree_snapshot();

        // Keep Git-backed memory current before command routing. This makes
        // ownership/history answers work end to end even before the user runs
        // :index manually, while incremental refresh remains cheap afterward.
        self.refresh_git_memory();
 
        if let Some(cmd) = self.parse_natural_migrate_request(&normalized) {
            return self.handle_input(&cmd);
        }

        // Detect high-level planning goals BEFORE intent_for so broad objectives can be delegated.
        if !trimmed.starts_with(':') {
            let lower = trimmed.to_ascii_lowercase();
            if lower.starts_with("i want to")
                || lower.starts_with("task:")
                || lower.starts_with("build:")
                || lower.starts_with("goal:")
            {
                return self.get_next_task(Some(trimmed));
            }
        }

        if let Some(cmd) = self.intent_for(trimmed) {
            return self.handle_input(&cmd);
        }

        if trimmed == ":index" {
            self.build_index()?;
            let stats = self.index.stats();
            let languages = self.index.files_by_language();
            let mut message = format!(
                "Indexed {} files with a total of {} lines.",
                stats.file_count, stats.total_lines
            );

            // Ingest historical Git Memory
            if let Some(git) = &self.git {
                if let Ok(commits) = git.recent_commits(100) {
                    let commit_count = commits.len();
                    for commit in commits {
                        self.memory.add_event(
                            "git",
                            format!(
                                "Historical commit {}: {} by {}",
                                commit.id, commit.summary, commit.author
                            ),
                            MemoryEvent::GitCommit {
                                id: commit.id.clone(),
                                summary: commit.summary.clone(),
                                author: commit.author.clone(),
                                date: commit.time.to_string(), // Store timestamp as an approximation of date for historical ingestion
                            },
                        );
                    }
                    message.push_str(&format!(" Ingested {} historical Git commits into memory.", commit_count));
                }
            }

            self.memory.add_event(
                "index",
                format!("root: {:?}, {}", self.root, message),
                MemoryEvent::IndexSnapshot {
                    file_count: stats.file_count,
                    total_lines: stats.total_lines,
                    languages,
                },
            );

            // Enrich with Temporal Graph (zero API calls)
            if let Some(git) = &self.git {
                if self.temporal_graph.enriched {
                    // Incremental: only process commits since last watermark
                    self.temporal_graph.enrich_incremental(git);
                } else {
                    // First run: full build
                    self.temporal_graph.enrich_from_git(git);
                }
                let dev_count = self.temporal_graph.developers.len();
                let co_change_count = self.temporal_graph.co_changes.len();
                message.push_str(&format!(
                    " Temporal graph: {} developers, {} co-change pairs detected.",
                    dev_count, co_change_count
                ));

                // Detect procedural patterns + set up watches (zero API calls)
                self.concept_store.detect_patterns(&self.temporal_graph);
                self.concept_store.check_watches(&self.temporal_graph);
                let pattern_count = self.concept_store.patterns.len();
                let watch_count = self.concept_store.watches.len();
                if pattern_count > 0 || watch_count > 0 {
                    message.push_str(&format!(
                        " Semantic memory: {} patterns, {} watches.",
                        pattern_count, watch_count
                    ));
                }

                // Save temporal graph
                let global_brain = crate::config::get_global_brain_path(&self.root);
                let _ = self.temporal_graph.save(&global_brain.join("temporal_graph.json"));
                let _ = self.concept_store.save(&global_brain.join("concepts.json"));
            }

            // Report RAG stats
            let chunk_count = self.vector_store.len();
            let embedded_count = self.vector_store.embedded_count();
            if chunk_count > 0 {
                if embedded_count > 0 {
                    message.push_str(&format!(
                        " RAG: {} chunks indexed, {} embedded (semantic search active).",
                        chunk_count, embedded_count
                    ));
                } else {
                    message.push_str(&format!(
                        " RAG: {} chunks indexed (keyword search active — add Gemini key for semantic search).",
                        chunk_count
                    ));
                }
            }

            return Ok(message);
        }

        if trimmed == ":help" || trimmed == ":commands" {
            return Ok(HELP_TEXT.to_string());
        }

        // ── Cross-Project Intelligence ─────────────────────────────
        if let Some(query) = trimmed.strip_prefix(":global search ") {
            return self.search_global_knowledge(query.trim());
        }

        // ── Semantic Graph + Memory Commands ─────────────────────────
        if let Some(rest) = trimmed.strip_prefix(":why ") {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            match self.temporal_graph.why_report(rest.trim()) {
                Some(report) => return Ok(report),
                None => return Ok(format!("No history found for '{}'. Try a different path.", rest.trim())),
            }
        }

        if trimmed == ":owners" {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            return Ok(self.temporal_graph.ownership_report());
        }

        if let Some(scope) = trimmed.strip_prefix(":owners ") {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            return Ok(self
                .temporal_graph
                .ownership_report_for(scope.trim())
                .unwrap_or_else(|| {
                    format!(
                        "I couldn't find Git ownership evidence matching `{}`. Try `:owners` for the repository-wide report or name a folder/file.",
                        scope.trim()
                    )
                }));
        }

        if trimmed == ":history" {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            return Ok(self.temporal_graph.project_history_report());
        }

        if trimmed == ":hotspots" {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            return Ok(self.temporal_graph.hotspot_report());
        }

        if trimmed == ":coupling" {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            return Ok(self.temporal_graph.coupling_report());
        }

        if trimmed == ":concepts" {
            return Ok(self.concept_store.concepts_report());
        }

        if trimmed == ":watches" {
            return Ok(self.concept_store.watches_report());
        }

        if trimmed == ":analyze" {
            if !self.temporal_graph.enriched {
                return Ok("Temporal graph not built yet. Run :index first.".to_string());
            }
            if let Some(model) = &self.model {
                match self.concept_store.derive_concepts(&self.memory, &self.temporal_graph, model.as_ref()) {
                    Ok(count) => {
                        let global_brain = crate::config::get_global_brain_path(&self.root);
                        let _ = self.concept_store.save(&global_brain.join("concepts.json"));
                        return Ok(format!(
                            "🧠 Derived {} new semantic concepts.\n\n{}",
                            count,
                            self.concept_store.concepts_report()
                        ));
                    }
                    Err(e) => return Ok(format!("Concept extraction failed: {}", e)),
                }
            } else {
                return Ok("No language model configured. Cannot run :analyze.".to_string());
            }
        }

        if normalized.starts_with(":task") {
            let goal = if normalized.len() > 5 {
                Some(normalized[6..].trim())
            } else {
                None
            };
            if let Some(g) = goal {
                return self.execute_task(g);
            }
            return self.get_next_task(None);
        }

        // ── Code Review / ship-gate commands ──────────────────────────
        if trimmed == ":review" || trimmed == ":review all" || trimmed.starts_with(":review ") {
            // Determine scope
            let rest = trimmed.strip_prefix(":review").unwrap_or("").trim();

            if rest == "--install-gate" {
                return Ok(self.install_review_gate());
            }

            // :review ack <n> — acknowledge a finding so it stops alarming
            if let Some(num) = rest.strip_prefix("ack ") {
                match num.trim().parse::<usize>() {
                    Ok(n) => return Ok(crate::review::acknowledge(&self.root, n)),
                    Err(_) => return Ok("Usage: :review ack <number>   (the # shown next to a finding)".to_string()),
                }
            }
            if rest == "acked" || rest == "ack" {
                let mem = crate::review::ReviewMemory::load(&self.root);
                if mem.acknowledged.is_empty() {
                    return Ok("No acknowledged findings yet. Use `:review ack <#>` to acknowledge one.".to_string());
                }
                let mut out = String::from("🧠 Acknowledged findings (hidden from the ship-verdict):\n");
                for (i, fp) in mem.acknowledged.iter().enumerate() {
                    out.push_str(&format!("  {}. {}\n", i + 1, fp));
                }
                return Ok(out);
            }

            let (files, scope_label): (Vec<String>, String) = if rest == "all" {
                // Whole codebase: every indexed file
                if self.index.stats().file_count == 0 {
                    self.build_index()?;
                }
                let files = self
                    .index
                    .files()
                    .keys()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                (files, "whole codebase".to_string())
            } else if !rest.is_empty() {
                // :review <path> — scope to a single file or path prefix.
                let target = self.root.join(rest);
                if target.is_file() {
                    (vec![rest.to_string()], format!("file {}", rest))
                } else {
                    // Treat as a path filter over changed/indexed files.
                    let pool: Vec<String> = match &self.git {
                        Some(git) => git.changed_files(),
                        None => self
                            .index
                            .files()
                            .keys()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect(),
                    };
                    let matched: Vec<String> = pool
                        .into_iter()
                        .filter(|p| p.replace('\\', "/").contains(&rest.replace('\\', "/")))
                        .collect();
                    if matched.is_empty() {
                        return Ok(format!("No file matching '{}' found to review. Try a path like `:review core/src/config.rs`.", rest));
                    }
                    (matched, format!("path filter '{}'", rest))
                }
            } else {
                // Default: git-changed files (what they're about to ship)
                match &self.git {
                    Some(git) => {
                        let changed = git.changed_files();
                        if changed.is_empty() {
                            return Ok("✅ No uncommitted changes to review. Use `:review all` to scan the whole codebase.".to_string());
                        }
                        (changed, "changed files".to_string())
                    }
                    None => {
                        // No git — review whole codebase
                        if self.index.stats().file_count == 0 {
                            self.build_index()?;
                        }
                        let files = self
                            .index
                            .files()
                            .keys()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        (files, "whole codebase (no git)".to_string())
                    }
                }
            };

            println!("🔍 Reviewing {} ({} file(s))... this may take a moment.\n", scope_label, files.len());

            let report = crate::review::review_files(
                &self.root,
                &files,
                &scope_label,
                self.model.as_deref().map(|m| m as &(dyn CodexModel + Send + Sync)),
            );

            self.memory.add_event(
                "review",
                format!(
                    "Review ({}): {} findings, verdict {:?}",
                    scope_label,
                    report.findings.len(),
                    report.verdict
                ),
                crate::memory::MemoryEvent::IndexSnapshot {
                    file_count: report.files_scanned,
                    total_lines: report.findings.len(),
                    languages: std::collections::HashMap::new(),
                },
            );

            return Ok(report.render());
        }

        // ── Product-Manager brain commands ────────────────────────────
        if let Some(idea) = trimmed.strip_prefix(":spec ") {
            let idea = idea.trim();
            if idea.is_empty() {
                return Ok("Usage: :spec <rough idea>   e.g. :spec a login system with google oauth".to_string());
            }
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:spec` needs an LLM.".to_string()),
            };
            let context = self.pm_context_snapshot();
            let pm = crate::pm::ProductManager::new(model);
            match pm.draft_spec(idea, &context) {
                Ok(spec) => {
                    self.memory.add("pm-spec", format!("Drafted spec: {}", spec.title));
                    return Ok(spec.render());
                }
                Err(e) => return Ok(format!("❌ Spec drafting failed: {}", e)),
            }
        }

        if let Some(idea) = trimmed.strip_prefix(":ask ") {
            let idea = idea.trim();
            if idea.is_empty() {
                return Ok("Usage: :ask <idea>   — Astra asks the key questions to scope your idea".to_string());
            }
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:ask` needs an LLM.".to_string()),
            };
            let context = self.pm_context_snapshot();
            let pm = crate::pm::ProductManager::new(model);
            match pm.clarifying_questions(idea, &context) {
                Ok(qs) if !qs.is_empty() => {
                    let mut out = format!("🤔 **Before we build \"{}\", answer these:**\n\n", idea);
                    for (i, q) in qs.iter().enumerate() {
                        out.push_str(&format!("{}. {}\n", i + 1, q));
                    }
                    out.push_str("\n_Then run `:spec <idea>` to get the full spec._");
                    return Ok(out);
                }
                Ok(_) => return Ok("The idea seems clear enough — run `:spec <idea>` to draft the full spec.".to_string()),
                Err(e) => return Ok(format!("❌ Failed: {}", e)),
            }
        }

        if let Some(goal) = trimmed.strip_prefix(":sharpen ") {
            let goal = goal.trim();
            if goal.is_empty() {
                return Ok("Usage: :sharpen <draft goal>".to_string());
            }
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:sharpen` needs an LLM.".to_string()),
            };
            let context = self.pm_context_snapshot();
            let pm = crate::pm::ProductManager::new(model);
            match pm.sharpen_goal(goal, &context) {
                Ok(sharp) => return Ok(format!("✨ **Sharpened**\n\n{}", sharp)),
                Err(e) => return Ok(format!("❌ Failed: {}", e)),
            }
        }

        // ── DevOps / Project-Manager commands ─────────────────────────
        if trimmed == ":commit" {
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:commit` needs an LLM to draft the message.".to_string()),
            };
            match crate::devops::commit_all(&self.root, model) {
                Ok(res) => {
                    self.memory.add("devops", format!("committed: {}", res.message.lines().next().unwrap_or("")));
                    return Ok(format!("✅ **Committed**\n\n```\n{}\n```\n\n{}", res.message, res.log.trim()));
                }
                Err(e) => return Ok(format!("❌ Commit failed: {}", e)),
            }
        }

        if trimmed == ":ship" || trimmed == ":ship --confirm" || trimmed == ":ship -y" {
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:ship` needs an LLM to draft the commit.".to_string()),
            };
            let confirmed = trimmed.ends_with("--confirm") || trimmed.ends_with("-y");

            if !confirmed {
                // Step 1: preview only — never touches git or the remote.
                match crate::devops::preview(&self.root, model) {
                    Ok(p) => {
                        let mut out = String::from("📦 **Ship preview** — nothing has been committed or pushed yet.\n\n");
                        out.push_str(&format!("Branch: `{}`{}\n\n", p.branch,
                            if p.has_remote { "" } else { "  (no remote configured — would commit locally only)" }));
                        out.push_str(&format!("Files ({}):\n", p.files.len()));
                        for f in p.files.iter().take(20) {
                            out.push_str(&format!("  {}\n", f));
                        }
                        if p.files.len() > 20 {
                            out.push_str(&format!("  …and {} more\n", p.files.len() - 20));
                        }
                        out.push_str(&format!("\nProposed commit message:\n```\n{}\n```\n", p.message));
                        out.push_str("\n👉 Run `:ship --confirm` to commit & push, or `:commit` to commit locally only.");
                        return Ok(out);
                    }
                    Err(e) => return Ok(format!("❌ Nothing to ship: {}", e)),
                }
            }

            // Step 2: confirmed — actually commit + push.
            match crate::devops::ship(&self.root, model) {
                Ok(res) => {
                    self.memory.add("devops", format!("shipped to {}: {}", res.branch, res.commit_message.lines().next().unwrap_or("")));
                    let push_status = if res.pushed {
                        format!("🚀 Pushed to `origin/{}`.", res.branch)
                    } else {
                        format!("⚠️  {}", res.push_log)
                    };
                    return Ok(format!(
                        "✅ **Shipped**\n\n```\n{}\n```\n\n{}",
                        res.commit_message, push_status
                    ));
                }
                Err(e) => return Ok(format!("❌ Ship failed: {}", e)),
            }
        }

        if trimmed == ":pr" || trimmed.starts_with(":pr ") {
            let base = trimmed.strip_prefix(":pr ").map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("main");
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:pr` needs an LLM to draft the PR.".to_string()),
            };
            match crate::devops::open_pr(&self.root, model, base) {
                Ok(res) => {
                    self.memory.add("devops", format!("opened PR: {}", res.title));
                    return Ok(format!(
                        "✅ **Pull Request Opened**\n\n**{}**\n\n{}\n\n🔗 {}",
                        res.title, res.body, res.url
                    ));
                }
                Err(e) => return Ok(format!("❌ PR failed: {}", e)),
            }
        }

        if let Some(tag) = trimmed.strip_prefix(":release ") {
            let tag = tag.trim();
            if tag.is_empty() {
                return Ok("Usage: :release <tag>   e.g. :release v0.2.0".to_string());
            }
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:release` needs an LLM to draft notes.".to_string()),
            };
            match crate::devops::cut_release(&self.root, model, tag) {
                Ok(res) => {
                    self.memory.add("devops", format!("cut release {}", res.tag));
                    return Ok(format!(
                        "🎉 **Release {} created**\n\n{}\n\n🔗 {}",
                        res.tag, res.notes, res.url
                    ));
                }
                Err(e) => return Ok(format!("❌ Release failed: {}", e)),
            }
        }

        if trimmed == ":standup" {
            let model = match &self.model {
                Some(m) => m.as_ref(),
                None => return Ok("No language model configured. `:standup` needs an LLM.".to_string()),
            };
            match crate::devops::standup_report(&self.root, model) {
                Ok(report) => return Ok(format!("☕ **Astra Standup**\n\n{}", report)),
                Err(e) => return Ok(format!("❌ Standup failed: {}", e)),
            }
        }

        // ── Plan management commands ──────────────────────────────────
        if trimmed == ":plan status" || trimmed == ":plan" {
            return match crate::planner::Plan::load(&self.root) {
                Some(plan) => {
                    let planner_ref: Option<()> = None; // status-only, no model needed
                    let _ = planner_ref;
                    Ok(plan.render_dashboard())
                }
                None => Ok("No active plan. Use `:task <goal>` to create one.".to_string()),
            };
        }

        if trimmed == ":plan clear" {
            crate::planner::Plan::clear(&self.root);
            return Ok("🗑️  Active plan cleared.".to_string());
        }

        if trimmed == ":plan resume" {
            return match crate::planner::Plan::load(&self.root) {
                Some(plan) if !plan.is_complete() => self.resume_plan(plan),
                Some(_) => Ok("Plan is already complete. Use `:plan clear` and `:task <goal>` to start fresh.".to_string()),
                None => Ok("No active plan. Use `:task <goal>` to create one.".to_string()),
            };
        }

        if let Some(rest) = trimmed.strip_prefix(":plan ") {
            let instruction = rest.trim();
            if instruction.is_empty() {
                return Ok("Usage: :plan <instruction>".to_string());
            }
            return self.execute_plan(instruction);
        }

        if let Some(rest) = trimmed.strip_prefix(":edit ") {
            let instruction = rest.trim();
            if instruction.is_empty() {
                return Ok("Usage: :edit <filename> <instructions>".to_string());
            }
            return self.handle_edit(instruction);
        }

        if trimmed == ":team status" {
            return self.team_status_summary();
        }

        if trimmed == ":memory compact" {
            let removed = self.memory.compact_noise();
            return Ok(format!(
                "🧠 Memory compacted. Removed {} noisy entries (qa/web/autonomous chatter).",
                removed
            ));
        }

        if let Some(rest) = trimmed.strip_prefix(":memory ") {
            let query = rest.trim();
            let q_vec = self.get_query_embedding(query);
            let matches = self.memory.search(query, q_vec.as_deref(), 20);
            if matches.is_empty() {
                return Ok("No memory matches found.".to_string());
            }
            let mut out = String::new();
            for entry in matches {
                let _ = writeln!(
                    &mut out,
                    "- [{}] {} (ts: {})",
                    entry.kind, entry.content, entry.timestamp
                );
            }
            return Ok(out);
        }

        if trimmed == ":memory" {
            let recent = self.memory.recent(5);
            if recent.is_empty() {
                return Ok("Memory is empty.".to_string());
            }
            let mut out = String::new();
            for entry in recent {
                let _ = writeln!(
                    &mut out,
                    "- [{}] {} (ts: {})",
                    entry.kind, entry.content, entry.timestamp
                );
            }
            return Ok(out);
        }

        if trimmed == ":files-by-lang" {
            let by_lang = self.index.files_by_language();
            if by_lang.is_empty() {
                return Ok("No files indexed yet. Run :index first.".to_string());
            }
            let mut out = String::new();
            for (lang, count) in by_lang {
                let _ = writeln!(&mut out, "{}: {} files", lang, count);
            }
            return Ok(out);
        }

        // :deps <symbol> — find all files that depend on a symbol
        if let Some(rest) = trimmed.strip_prefix(":deps ") {
            let symbol = rest.trim();
            if symbol.is_empty() {
                return Ok("Usage: :deps <symbol_name>".to_string());
            }
            let dependents = self.index.find_dependents(symbol);
            if dependents.is_empty() {
                return Ok(format!("No files found that depend on '{}'.", symbol));
            }
            let mut out = format!("\u{1f50d} Files that depend on '{}':\n", symbol);
            for dep in dependents {
                let _ = writeln!(&mut out, "  \u{2022} {}", dep.display());
            }
            return Ok(out);
        }

        // :imports <filepath> — show what a file imports
        if let Some(rest) = trimmed.strip_prefix(":imports ") {
            let file_path = rest.trim();
            if file_path.is_empty() {
                return Ok("Usage: :imports <file_path>".to_string());
            }
            let path = std::path::PathBuf::from(file_path);
            let deps = self.index.find_dependencies(&path);
            if deps.is_empty() {
                return Ok(format!("No imports tracked for '{}'.", file_path));
            }
            let mut out = format!("\u{1f4e6} Imports for '{}':\n", file_path);
            for dep in deps {
                let _ = writeln!(&mut out, "  \u{2022} {}", dep);
            }
            return Ok(out);
        }


        // ── :search <query> — RAG search over indexed code ───────────
        if let Some(rest) = trimmed.strip_prefix(":search ") {
            let query = rest.trim();
            if query.is_empty() {
                return Ok("Usage: :search <query>\nExample: :search where is user authentication handled".to_string());
            }
            if self.vector_store.is_empty() {
                return Ok("No chunks indexed yet. Run :index first, then search.".to_string());
            }
            let q_vec = self.get_query_embedding(query);
            let results = self.vector_store.search(query, q_vec.as_deref(), 5);
            if results.is_empty() {
                return Ok(format!("No code chunks matched '{}'.", query));
            }
            let mode = if self.vector_store.embedded_count() > 0 { "semantic" } else { "keyword" };
            let mut out = format!(
                "🔍 **RAG Search** ({} mode) — top {} results for: \"{}\"\n\n",
                mode,
                results.len(),
                query
            );
            for (i, chunk) in results.iter().enumerate() {
                out.push_str(&format!(
                    "**{}. {}** (lines {}-{})\n```{}\n{}\n```\n\n",
                    i + 1,
                    chunk.path.display(),
                    chunk.start_line,
                    chunk.end_line,
                    chunk.language,
                    chunk.content.trim()
                ));
            }

            // If we have a model, also synthesize an answer from the chunks
            if let Some(model) = &self.model {
                let context = format_chunks_for_prompt(&results, 6000);
                let prompt = format!(
                    "You are Astra, a codebase assistant. The user searched for: \"{}\"\n\n\
                    Here are the most relevant code chunks from the codebase:\n\n{}\n\
                    Based only on the code above, answer the user's question concisely. \
                    Reference specific file paths and line numbers where relevant.",
                    query, context
                );
                if let Ok(answer) = model.complete(&prompt) {
                    out.push_str(&format!("---\n**Astra's Analysis:**\n{}\n", answer));
                }
            }

            self.memory.add("rag-search", format!("query: {} ({} results)", query, results.len()));
            return Ok(out);
        }

        if let Some(rest) = trimmed.strip_prefix(":web ") {
            let query = rest.trim();
            if query.is_empty() {
                return Ok("Usage: :web <query>".to_string());
            }
            if let Some(search) = &self.search {
                let results = search.search(query)?;
                // Store raw search results in memory permanently
                self.memory.add(
                    "web-search",
                    format!("Query: {}\nResults:\n{}", query, &results[..results.len().min(2000)]),
                );

                if let Some(model) = &self.model {
                    let prompt = format!(
                        "You are a helpful assistant. Summarize the following web search results \
                        into a clear, actionable answer.\n\nQuery: {}\n\nSearch Results:\n{}",
                        query, results
                    );
                    let answer = model.complete(&prompt)?;
                    // Store the AI summary permanently
                    self.memory.add(
                        "web-knowledge",
                        format!("Q: {}\nA: {}", query, answer),
                    );
                    return Ok(format!(
                        "\u{1f310} **Web Search Results**\n\n{}\n\n_This knowledge has been saved to memory._",
                        answer
                    ));
                } else {
                    return Ok(format!(
                        "\u{1f310} Raw search results (no LLM to summarize):\n\n{}",
                        &results[..results.len().min(3000)]
                    ));
                }
            } else {
                return Ok("Web search is not configured. Set TAVILY_API_KEY in your .env file.".to_string());
            }
        }


        if trimmed == ":summary" {
            let stats = self.index.stats();
            let has_git = self.git.is_some();
            let recent = self.memory.recent(5);
            let symbol_count = self.index.total_symbol_count();
            let symbols_by_lang = self.index.symbols_by_language();
            let graph_stats = self.index.graph_stats();

            let mut summary = String::new();
            let _ = writeln!(&mut summary, "Project root: {:?}", self.root);
            let _ = writeln!(
                &mut summary,
                "Indexed files: {}",
                stats.file_count
            );
            let _ = writeln!(&mut summary, "Total lines: {}", stats.total_lines);
            if symbol_count > 0 {
                let _ = writeln!(&mut summary, "Symbols detected: {}", symbol_count);
            }
            if graph_stats.node_count > 0 {
                let _ = writeln!(
                    &mut summary,
                    "Semantic graph: {} nodes ({} files, {} symbols), {} edges",
                    graph_stats.node_count,
                    graph_stats.file_nodes,
                    graph_stats.symbol_nodes,
                    graph_stats.edge_count
                );
            }
            let _ = writeln!(
                &mut summary,
                "Git repository detected: {}",
                if has_git { "yes" } else { "no" }
            );
            if !symbols_by_lang.is_empty() {
                let _ = writeln!(&mut summary, "Symbols by language:");
                for (lang, count) in symbols_by_lang {
                    let _ = writeln!(&mut summary, "- {}: {}", lang, count);
                }
            }
            if !recent.is_empty() {
                let _ = writeln!(&mut summary, "Recent memory:");
                for entry in recent {
                    let _ = writeln!(&mut summary, "- [{}] {}", entry.kind, entry.content);
                }
            }

            if let Some(model) = &self.model {
                let mut prompt = String::new();
                let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
                let _ = writeln!(
                    &mut prompt,
                    "You are Astra. Summarize this project information for the user:\n{}",
                    summary
                );
                let answer = model.complete(&prompt)?;
                self.memory
                    .add("summary", answer.clone());
                return Ok(answer);
            }

            self.memory.add("summary", summary.clone());
            self.remember_project_snapshot();
            return Ok(summary);
        }
        if trimmed == ":onboard" {
            if self.index.stats().file_count == 0 {
                self.build_index()?;
            }
            if !self.temporal_graph.enriched {
                if let Some(git) = &self.git {
                    self.temporal_graph.enrich_from_git(git);
                    self.concept_store.detect_patterns(&self.temporal_graph);
                    self.concept_store.check_watches(&self.temporal_graph);
                }
            }
            return Ok(self.render_onboarding());
        }
        if trimmed == ":graph" {
            if self.index.stats().file_count == 0 {
                self.build_index()?;
            }
            let dot = self.index.graph_dot();
            let output_dir = self.root.join(".astra");
            fs::create_dir_all(&output_dir)?;
            let output_path = output_dir.join("graph.dot");
            fs::write(&output_path, dot)?;
            let message = format!("Wrote semantic graph to {:?}", output_path);
            self.memory.add("graph", message.clone());
            return Ok(message);
        }

        if trimmed == ":git-commit-count" {
            if let Some(git) = &self.git {
                let count = git.total_commit_count();
                let message = format!("Total commits: {}", count);
                self.memory.add("git", message.clone());
                return Ok(message);
            } else {
                return Ok("No git repository detected for this root.".to_string());
            }
        }

        if trimmed == ":git-last-commit" {
            if let Some(git) = &self.git {
                let info = git.last_commit_info()?;
                let message = format!(
                    "Last commit: {} by {} at {} — {}",
                    info.id, info.author, info.date, info.summary
                );
                self.memory.add("git", message.clone());
                return Ok(message);
            } else {
                return Ok("No git repository detected for this root.".to_string());
            }
        }

        if trimmed == ":health" {
            if self.index.stats().file_count == 0 {
                self.build_index()?;
            }

            let team_mgr = TeamManager::new(&self.root);
            let report = health::compute_health(
                &self.root,
                &self.index,
                &self.memory,
                Some(&team_mgr),
            );

            self.memory.add_event(
                "health",
                format!(
                    "Health check: quality={} test={} drift={} security={} git={} team={}",
                    report.scores.code_quality,
                    report.scores.test_health,
                    report.scores.cross_lang_drift,
                    report.scores.security_surface,
                    report.scores.git_health,
                    report.scores.team_velocity,
                ),
                MemoryEvent::HealthSnapshot {
                    scores: report.scores.clone(),
                },
            );

            return Ok(report.render());
        }

        if trimmed == ":audit" {
            self.build_index()?;
            let report = self.handle_input(":health")?;
            return Ok(format!(
                "Fresh full-project audit completed from {} indexed files and {} lines.\n\n{}",
                self.index.stats().file_count,
                self.index.stats().total_lines,
                report
            ));
        }

        if let Some(desc) = trimmed.strip_prefix(":bisect ") {
            if let Some(git) = &self.git {
                if let Some(model) = &self.model {
                    match time_travel::run_semantic_bisect(git, model.as_ref(), desc, 20) {
                        Ok(result) => {
                            let mut out = String::new();
                            let _ = writeln!(&mut out, "🐛 **Time Travel Debugging Complete** 🐛");
                            let _ = writeln!(&mut out, "Analyzed recent {} commits.", result.analyzed_count);
                            let _ = writeln!(&mut out, "\n**Suspect Commit Found!**");
                            let _ = writeln!(&mut out, "Commit: {} ({})", result.suspect_commit_id, result.suspect_commit_summary);
                            let _ = writeln!(&mut out, "Author: {}", result.suspect_author);
                            let _ = writeln!(&mut out, "\n**AI Explanation:**\n{}", result.explanation);
                            return Ok(out);
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    return Ok("Semantic bisect requires an LLM to be configured (use Groq).".to_string());
                }
            } else {
                return Ok("No git repository detected. Semantic bisect requires git.".to_string());
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":vibe ") {
            let vibe_name = rest.trim();
            let persona = Persona::from_vibe(vibe_name);
            let display_name = persona.name.clone();
            let _ = persona.save(&self.root);
            self.set_persona(persona);
            return Ok(format!("Vibe changed! You are now talking to {}.", display_name));
        }

        if let Some(rest) = trimmed.strip_prefix(":git-history ") {
            if let Some(git) = &self.git {
                let rel = PathBuf::from(rest);
                let commits = git.recent_commits_for_path(&rel, 5)?;
                if commits.is_empty() {
                    return Ok(format!("No recent commits found touching {}", rest));
                }
                let mut out = String::new();
                for c in &commits {
                    let _ = writeln!(
                        &mut out,
                        "{} {} by {} at {}",
                        c.id, c.summary, c.author, c.time
                    );
                }
                self.memory.add(
                    "git",
                    format!("history for {} queried ({} commits)", rest, commits.len()),
                );
                return Ok(out);
            } else {
                return Ok("No git repository detected for this root.".to_string());
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":rust-symbols ") {
            let path = PathBuf::from(rest);
            let contents = fs::read_to_string(&path)?;
            let symbols = parse_rust_file(&path, &contents)?;
            if symbols.is_empty() {
                return Ok(format!("No Rust symbols found in {:?}", path));
            }
            let mut out = String::new();
            let _ = writeln!(&mut out, "Rust symbols in {:?}:", path);
            for sym in symbols {
                let kind = match sym.kind {
                    ParsedSymbolKind::Struct => "struct",
                    ParsedSymbolKind::Enum => "enum",
                    ParsedSymbolKind::Function => "fn",
                    ParsedSymbolKind::Class => "class",
                    ParsedSymbolKind::Interface => "interface",
                    ParsedSymbolKind::Type => "type",
                    ParsedSymbolKind::Constant => "const",
                };
                let _ = writeln!(&mut out, "  {} {}", kind, sym.name);
            }
            return Ok(out);
        }

        if trimmed == ":migrations" {
            let migrations = migration::list_migrations();
            if migrations.is_empty() {
                return Ok("No migrations are registered.".to_string());
            }
            let mut out = String::new();
            for m in migrations {
                let _ = writeln!(
                    &mut out,
                    "{}: {} -> {}",
                    m.id, m.from_stack, m.to_stack
                );
            }
            self.memory
                .add("migration-list", format!("listed {} migrations", migrations.len()));
            return Ok(out);
        }

        if let Some(rest) = trimmed.strip_prefix(":plan-migration ") {
            if let Some(m) = migration::find_migration(rest) {
                let mut out = String::new();
                let _ = writeln!(
                    &mut out,
                    "Migration {}: {} -> {}",
                    m.id, m.from_stack, m.to_stack
                );
                let _ = writeln!(&mut out, "Description: {}", m.description);
                let _ = writeln!(&mut out, "High-level steps:");
                for (i, step) in m.steps.iter().enumerate() {
                    let _ = writeln!(&mut out, "{}. {}", i + 1, step);
                }
                self.memory
                    .add("migration-plan", format!("planned migration {}", m.id));
                return Ok(out);
            } else {
                return Ok(format!("Unknown migration id: {}", rest));
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":scaffold ") {
            let plan = scaffold::plan_scaffold(rest);
            let mut out = String::new();
            let _ = writeln!(&mut out, "Scaffold plan for stack `{}`:", plan.stack);
            if !plan.commands.is_empty() {
                let _ = writeln!(&mut out, "Suggested shell commands:");
                for cmd in &plan.commands {
                    let _ = writeln!(&mut out, "  {}", cmd);
                }
            }
            if !plan.notes.is_empty() {
                let _ = writeln!(&mut out, "Notes:");
                for note in &plan.notes {
                    let _ = writeln!(&mut out, "- {}", note);
                }
            }
            self.memory
                .add("scaffold-plan", format!("stack: {}", plan.stack));
            return Ok(out);
        }

        if let Some(rest) = trimmed.strip_prefix(":migrate-ts-ai ") {
            let ts_path = PathBuf::from(rest);
            let ts_code = fs::read_to_string(&ts_path)?;
            if let Some(model) = &self.model {
                let mut prompt = String::new();
                let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
                let _ = writeln!(
                    &mut prompt,
                    "You are an expert code translator. Translate the following TypeScript code into equivalent Rust."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Preserve function names and signatures as closely as possible."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Use idiomatic Rust, but do not add comments or explanations."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Output only Rust code, with no markdown fences or extra text."
                );
                let _ = writeln!(&mut prompt, "Do not omit any logic or types.");
                let _ = writeln!(&mut prompt, "Avoid TODO stubs or placeholder code.");
                let _ = writeln!(&mut prompt);
                let _ = writeln!(&mut prompt, "TypeScript code:");
                let _ = writeln!(&mut prompt, "```ts");
                let _ = writeln!(&mut prompt, "{}", ts_code);
                let _ = writeln!(&mut prompt, "```");

                let rust_code_raw = model.complete(&prompt)?;
                let rust_code = Self::strip_markdown_fences(&rust_code_raw);
                self.memory.add(
                    "migration-ai",
                    format!("from: {:?}\n{}", ts_path, rust_code),
                );
                return Ok(rust_code);
            } else {
                return Ok("No language model is configured.".to_string());
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":clean ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() < 2 {
                return Ok("Usage: :clean <path> <language>".to_string());
            }
            let path = PathBuf::from(parts[0]);
            let lang = match Language::from_str_loose(parts[1]) {
                Some(l) => l,
                None => return Ok(format!("Unknown language: {}", parts[1])),
            };

            let contents = fs::read_to_string(&path)?;
            if let Some(model) = &self.model {
                let cleaner = migrate::clean::CleanupEngine::new(model.as_ref());
                let (cleaned, smells) = cleaner.clean(&contents, lang)?;
                
                fs::write(&path, &cleaned)?;

                let mut out = String::new();
                let _ = writeln!(&mut out, "✓ Cleaned up {:?}", path);
                if !smells.is_empty() {
                    let _ = writeln!(&mut out, "Smells fixed:");
                    for s in &smells {
                        let _ = writeln!(&mut out, "  - {}", s.name);
                    }
                }
                
                self.memory.add("cleanup", format!("cleaned {:?}, {} smells found", path, smells.len()));
                return Ok(out);
            } else {
                return Ok("No language model is configured.".to_string());
            }
        }

        if let Some(report) = trimmed.strip_prefix(":fix-bug ") {
            return self.intake_bug_report(report.trim());
        }

        if trimmed == ":issues" || trimmed == ":issue list" {
            let issues = crate::issues::IssueStore::new(&self.root).list(20)?;
            if issues.is_empty() {
                return Ok("No tracked bug reports yet. Use `:fix-bug <what went wrong>` to start one.".to_string());
            }
            let mut out = String::from("Astra issue ledger:\n");
            for issue in issues {
                let _ = writeln!(&mut out, "- {} [{:?}] {}", issue.id, issue.status, issue.report);
            }
            return Ok(out.trim_end().to_string());
        }

        if let Some(id) = trimmed.strip_prefix(":issue ") {
            if id.trim() == "list" {
                return self.handle_input(":issues");
            }
            let store = crate::issues::IssueStore::new(&self.root);
            let Some(issue) = store.get(id.trim())? else {
                return Ok(format!("I couldn't find issue `{}`.", id.trim()));
            };
            return Ok(render_issue(&issue));
        }

        if let Some(rest) = trimmed.strip_prefix(":fix ") {
            let first_token = rest.split_whitespace().next().unwrap_or("");
            let legacy_path = self.root.join(first_token);
            if !first_token.is_empty() && !legacy_path.exists() {
                return self.intake_bug_report(rest.trim());
            }
            let mut parts = rest.split_whitespace();
            let path_part = match parts.next() {
                Some(p) => p,
                None => {
                    return Ok("Usage: :fix <path> <bug or error description>".to_string());
                }
            };
            let desc = parts.collect::<Vec<_>>().join(" ");
            if desc.is_empty() {
                return Ok("Usage: :fix <path> <bug or error description>".to_string());
            }

            let path = PathBuf::from(path_part);
            let contents = fs::read_to_string(&path)?;

            if let Some(model) = &self.model {
                let mut prompt = String::new();
                let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
                let _ = writeln!(
                    &mut prompt,
                    "You are an expert software engineer. A bug has been reported in this file."
                );
                let _ = writeln!(&mut prompt, "Bug or error description: {}", desc);
                let _ = writeln!(
                    &mut prompt,
                    "Below is the current file contents. Return a fixed version of the file that compiles and resolves the bug."
                );
                let _ = writeln!(&mut prompt, "Rules:");
                let _ = writeln!(
                    &mut prompt,
                    "- Keep the same overall structure and public API."
                );
                let _ = writeln!(
                    &mut prompt,
                    "- Focus only on changes needed to fix the described bug."
                );
                let _ = writeln!(
                    &mut prompt,
                    "- Output only the full fixed file contents in the same language, with no explanations and no markdown fences."
                );
                let _ = writeln!(
                    &mut prompt,
                    "- Preserve existing formatting and style where possible."
                );
                let _ = writeln!(&mut prompt);
                let _ = writeln!(&mut prompt, "File path: {:?}", path);
                let _ = writeln!(&mut prompt, "Current file contents:");
                let _ = writeln!(&mut prompt, "{}", contents);

                let fixed_raw = model.complete(&prompt)?;
                let fixed = Self::strip_markdown_fences(&fixed_raw);

                let backup_path = path.with_extension("astra.bak");
                fs::write(&backup_path, contents.as_bytes())?;
                fs::write(&path, fixed.as_bytes())?;

                let mut cmd = Command::new("cargo");
                cmd.arg("check").current_dir(&self.root);
                let status = cmd.status();

                if let Ok(s) = status {
                    if s.success() {
                        self.memory.add(
                            "fix",
                            format!("applied fix for {:?}: {}", path, desc),
                        );
                        return Ok(format!(
                            "Applied AI-generated fix to {:?} (backup at {:?}).",
                            path, backup_path
                        ));
                    }
                }

                fs::write(&path, contents.as_bytes())?;

                return Ok(format!(
                    "AI-generated fix for {:?} did not pass cargo check. Original file was restored; backup kept at {:?}.",
                    path, backup_path
                ));
            } else {
                return Ok(
                    "No language model is configured. Configure Groq or another model to use :fix."
                        .to_string(),
                );
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":migrate-ts-ai-to-file ") {
            let mut parts = rest.split_whitespace();
            let ts_part = match parts.next() {
                Some(p) => p,
                None => {
                    return Ok(
                        "Usage: :migrate-ts-ai-to-file <ts-path> <rust-path>".to_string()
                    )
                }
            };
            let rust_part = match parts.next() {
                Some(p) => p,
                None => {
                    return Ok(
                        "Usage: :migrate-ts-ai-to-file <ts-path> <rust-path>".to_string()
                    )
                }
            };

            let ts_path = PathBuf::from(ts_part);
            let rust_path = PathBuf::from(rust_part);
            let ts_code = fs::read_to_string(&ts_path)?;

            if let Some(model) = &self.model {
                let mut prompt = String::new();
                let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
                let _ = writeln!(
                    &mut prompt,
                    "You are an expert code translator. Translate the following TypeScript code into equivalent Rust."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Preserve function names and signatures as closely as possible."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Use idiomatic Rust, but do not add comments or explanations."
                );
                let _ = writeln!(
                    &mut prompt,
                    "Output only Rust code, with no markdown fences or extra text."
                );
                let _ = writeln!(&mut prompt, "Do not omit any logic or types.");
                let _ = writeln!(&mut prompt, "Avoid TODO stubs or placeholder code.");
                let _ = writeln!(&mut prompt);
                let _ = writeln!(&mut prompt, "TypeScript code:");
                let _ = writeln!(&mut prompt, "```ts");
                let _ = writeln!(&mut prompt, "{}", ts_code);
                let _ = writeln!(&mut prompt, "```");

                let rust_code_raw = model.complete(&prompt)?;
                let rust_code = Self::strip_markdown_fences(&rust_code_raw);

                if let Some(parent) = rust_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&rust_path, rust_code.as_bytes())?;

                self.memory.add(
                    "migration-ai-file",
                    format!("from: {:?} to: {:?}", ts_path, rust_path),
                );

                return Ok(format!("Wrote migrated Rust code to {:?}", rust_path));
            } else {
                return Ok("No language model is configured.".to_string());
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":migrate-ts-file ") {
            let ts_path = PathBuf::from(rest);
            let code = ts_migrate::translate_ts_file(&ts_path)?;
            self.memory
                .add("migration-generated", format!("from: {:?}", ts_path));
            return Ok(code);
        }

        if let Some(rest) = trimmed.strip_prefix(":migrate ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() < 4 {
                return Ok(
                    "Usage: :migrate <source-dir> <from-lang> <to-lang> <output-dir> [--ai]"
                        .to_string(),
                );
            }
            let source_dir = PathBuf::from(parts[0]);
            let from_lang = match Language::from_str_loose(parts[1]) {
                Some(l) => l,
                None => return Ok(format!("Unknown source language: {}", parts[1])),
            };
            let to_lang = match Language::from_str_loose(parts[2]) {
                Some(l) => l,
                None => return Ok(format!("Unknown target language: {}", parts[2])),
            };
            let output_dir = PathBuf::from(parts[3]);
            let use_ai = parts.iter().any(|s| *s == "--ai");
            let use_clean = parts.iter().any(|s| *s == "--clean");

            // Phase 6: Pre-migration research
            let kind = format!("best-practices:{}", to_lang.to_string().to_lowercase());
            let knowledge = if let Some(entry) = self.memory.latest_event(&kind) {
                Some(entry.content.clone())
            } else if self.search.is_some() && use_ai {
                println!("Researching latest {} syntax and best practices...", to_lang);
                self.research_language(to_lang).ok()
            } else {
                None
            };

            let config = MigrationConfig {
                source_dir,
                output_dir,
                from_lang,
                to_lang,
                use_ai,
                use_clean,
                use_fix: false,
                knowledge,
            };

            let model_ref: Option<&(dyn CodexModel + Send + Sync)> =
                self.model.as_ref().map(|m| m.as_ref());
            let search_ref: Option<&(dyn SearchProvider + Send + Sync)> =
                self.search.as_ref().map(|s| s.as_ref());
            let result = migrate::run_migration(&config, model_ref, search_ref)?;

            self.memory.add_event(
                "migration",
                format!(
                    "{} → {}: {} files migrated",
                    config.from_lang,
                    config.to_lang,
                    result.migrated.len()
                ),
                MemoryEvent::MigrationRun {
                    from: config.from_lang.to_string(),
                    to: config.to_lang.to_string(),
                    file_count: result.migrated.len(),
                },
            );

            let mut out = String::new();
            let _ = writeln!(&mut out, "{}", result.plan_text);
            out.push('\n');
            out.push_str(&result.scaffold_log);
            out.push_str(&result.summary());
            return Ok(out);
        }

        // ── :workflow generate|list|run — custom dynamic workflows ──
        if trimmed == ":workflow list" {
            let manager = WorkflowManager::new(&self.root);
            match manager.list_workflows() {
                Ok(wfs) => {
                    if wfs.is_empty() {
                        return Ok("No custom workflows found in .astra/workflows/".to_string());
                    }
                    let mut out = String::from("🛠️ **Available Workflows**\n");
                    for w in wfs {
                        out.push_str(&format!("  - {}\n", w));
                    }
                    return Ok(out);
                }
                Err(e) => return Ok(format!("Error listing workflows: {}", e)),
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":workflow generate ") {
            let desc = rest.trim();
            if let Some(model) = &self.model {
                let manager = WorkflowManager::new(&self.root);
                match manager.generate_workflow(desc, model.as_ref()) {
                    Ok(msg) => {
                        self.memory.add("workflow", format!("Generated new workflow: {}", desc));
                        return Ok(msg);
                    }
                    Err(e) => return Ok(format!("Failed to generate workflow: {}", e)),
                }
            } else {
                return Ok("No LLM configured. Cannot generate workflow.".to_string());
            }
        }

        if let Some(rest) = trimmed.strip_prefix(":workflow run ") {
            let mut parts = rest.split_whitespace();
            let name = match parts.next() {
                Some(n) => n,
                None => return Ok("Usage: :workflow run <name> [args...]".to_string()),
            };
            let args: Vec<String> = parts.map(|s| s.to_string()).collect();
            let manager = WorkflowManager::new(&self.root);
            match manager.execute_workflow(name, args) {
                Ok(msg) => return Ok(msg),
                Err(e) => return Ok(format!("Failed to execute workflow: {}", e)),
            }
        }

        // ── :learn <language|fact> — research a language or store a fact ──
        if let Some(rest) = trimmed.strip_prefix(":learn ") {
            let fact = rest.trim();
            if let Some(lang) = Language::from_str_loose(fact) {
                let result = self.research_language(lang)?;
                return Ok(format!(
                    "\u{1f4da} **Learned {} Best Practices**\n\n{}\n\n_Stored permanently in memory for future migrations._",
                    lang, &result[..result.len().min(2000)]
                ));
            } else {
                self.auto_extract_facts(fact);
                let lower_fact = fact.to_ascii_lowercase();
                if lower_fact.starts_with("project ")
                    || lower_fact.starts_with("this project ")
                    || lower_fact.starts_with("our project ")
                {
                    self.memory.remember_project_fact("note", fact);
                } else if lower_fact.starts_with("we prefer ")
                    || lower_fact.starts_with("our style ")
                    || lower_fact.starts_with("we use ")
                {
                    self.memory.remember_style_fact("preference", fact);
                }
                self.memory.add("fact", fact.to_string());
                return Ok(format!(
                    "\u{1f9e0} **Memory Updated**\n\nI've stored this fact: \"{}\".\nI will remember this during future queries and migrations.",
                    fact
                ));
            }
        }

        // ── :memory [query] — list or search stored facts ───────────
        if trimmed == ":memory" || trimmed.starts_with(":memory ") {
            let results = if let Some(query) = trimmed.strip_prefix(":memory ") {
                let q_vec = self.get_query_embedding(query);
                self.memory.search(query, q_vec.as_deref(), 10)
            } else {
                self.memory.recent(10)
            };

            if results.is_empty() {
                return Ok("\u{1f5d1}\u{fe0f} No memories found yet. Try `:learn <fact>`!".to_string());
            }

            let mut out = String::from("\u{1f9e0} **Astra's Memory Bank**\n\n");
            for m in results {
                let date = chrono::DateTime::from_timestamp(m.timestamp as i64, 0)
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                let _ = writeln!(&mut out, "\u{2022} **[{}]** ({}) — {}", m.kind.to_uppercase(), date, m.content);
            }
            return Ok(out);
        }

        if trimmed == ":profile" {
            return Ok(self.memory.profile_report());
        }

        if trimmed == ":project-memory" {
            return Ok(self.memory.project_report());
        }

        // ── :predict — predictive refactoring analysis ──────────────
        if trimmed == ":predict" {
            self.build_index()?;
            let predictions = crate::predict::analyze(
                self.git.as_ref(),
                &self.index,
                &self.root,
            );
            return Ok(crate::predict::format_report(&predictions));
        }

        // ── :hook — install git pre-commit hook ─────────────────────
        if trimmed == ":hook" {
            match crate::watch::install_git_hook(&self.root) {
                Ok(msg) => return Ok(msg),
                Err(e) => return Ok(format!("\u{274c} Failed to install hook: {}", e)),
            }
        }

        // ── :watch — explain watch mode ─────────────────────────────
        if trimmed == ":watch" {
            return Ok(
                "\u{1f440} **Watch Mode**\n\n\
                To start watch mode, run Astra with the `--watch` flag:\n\
                ```\n\
                astra-cli --watch --env .env\n\
                ```\n\n\
                In watch mode, Astra monitors your project directory for file changes.\n\
                When you save a file, it:\n\
                  \u{2022} Re-indexes the changed file\n\
                  \u{2022} Warns about files growing too large\n\
                  \u{2022} Flags orphaned files with no imports\n\
                  \u{2022} Alerts you if a deletion breaks imports\n\n\
                _Watch mode runs until you press Ctrl+C._"
                    .to_string(),
            );
        }

        if let Some(rest) = trimmed.strip_prefix("? ") {
            let lower = rest.to_ascii_lowercase();
            if lower.contains("how many") && lower.contains("commit") {
                if let Some(git) = &self.git {
                    let count = git.total_commit_count();
                    return Ok(format!("Total commits: {}", count));
                } else {
                    return Ok("No git repository detected for this root.".to_string());
                }
            }
            if lower.contains("last commit")
                || lower.contains("most recent commit")
                || lower.contains("when did i make")
                || lower.contains("when did i commit")
                || lower.contains("when was my last commit")
            {
                if let Some(git) = &self.git {
                    let info = git.last_commit_info()?;
                    let message = format!(
                        "Last commit: {} by {} at {} — {}",
                        info.id, info.author, info.date, info.summary
                    );
                    return Ok(message);
                } else {
                    return Ok("No git repository detected for this root.".to_string());
                }
            }
            if lower.contains("health check") || lower == "health" {
                return self.handle_input(":health");
            }
            if lower.contains("graph") || lower.contains("semantic graph") {
                return self.handle_input(":graph");
            }
            let answer = self.answer_question(rest)?;
            self.memory.add("qa", format!("Q: {}\nA: {}", rest, answer));
            return Ok(answer);
        }

        if !trimmed.starts_with(':') && !trimmed.starts_with("? ") {
            // ── Auto-fact extraction: silently learn personal facts from conversation ──
            self.auto_extract_facts(trimmed);
            self.memory
                .remember_conversation_state("last_user_message", trimmed);
            let answer = self.answer_question(trimmed)?;
            self.memory
                .add("qa", format!("Q: {}\nA: {}", trimmed, answer));
            return Ok(answer);
        }

        Ok(format!(
            "astra did not understand this yet:\n  {}\n\nTry one of:\n  :index\n  :summary\n  :memory\n  :web <query>       — search the web and remember\n  :learn <language>  — research a language's best practices\n  :migrate <src> <from> <to> <out> [--ai]\n  ? <question>",
            trimmed
        ))
    }

    /// Archive task-control state while leaving every project source file
    /// untouched. The archive makes an accidental confirmation recoverable.
    fn abandon_active_task(&mut self) -> Result<String> {
        let astra_dir = self.root.join(".astra");
        let task_path = astra_dir.join("active_task.json");
        if !task_path.exists() {
            return Ok("There isn't an active task to drop. Your project files are untouched.".to_string());
        }

        let title = std::fs::read_to_string(&task_path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|task| {
                task.get("title")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "the current task".to_string());

        let archive_dir = astra_dir.join("abandoned");
        std::fs::create_dir_all(&archive_dir)?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let archive_name = format!("active_task_{}.json", stamp);
        std::fs::rename(&task_path, archive_dir.join(&archive_name))?;

        if let Some(mut plan) = crate::planner::Plan::load(&self.root) {
            plan.phase = crate::planner::PlanPhase::Abandoned("dropped by user".to_string());
            plan.touch();
            plan.save(&self.root)?;
        }

        Ok(format!(
            "Dropped **{}**. I didn't delete any project files; the old task state is recoverable at `.astra/abandoned/{}`.",
            title, archive_name
        ))
    }

    fn execute_option_selection(&mut self, selected: &[u8]) -> Result<String> {
        let mut sections = Vec::new();
        for option in selected {
            let (label, command) = match option {
                1 => ("Review", ":review"),
                2 => ("Health", ":health"),
                3 => ("Ship preview", ":ship"),
                _ => continue,
            };
            let result = self.handle_input(command)?;
            sections.push(format!("### {}\n{}", label, result));
        }
        if sections.is_empty() {
            Ok("Choose one of the displayed options: 1, 2, or 3.".to_string())
        } else {
            Ok(format!(
                "Completed the selected checks.\n\n{}",
                sections.join("\n\n")
            ))
        }
    }

    fn intake_bug_report(&mut self, report: &str) -> Result<String> {
        if report.trim().is_empty() {
            return Ok("Usage: `:fix-bug <what went wrong>`".to_string());
        }
        if self.index.stats().file_count == 0 {
            self.build_index()?;
        }

        let store = crate::issues::IssueStore::new(&self.root);
        let mut issue = store.create(report)?;
        if let Some(git) = &self.git {
            issue.head_commit = git.get_head_commit().ok();
            issue.branch = git.current_branch();
            issue.changed_files = git.changed_files().into_iter().take(40).collect();
        }

        issue.likely_files = self
            .rank_relevant_files(report, 8)
            .into_iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();

        for path in issue.likely_files.iter().take(5) {
            if let Some(history) = self.temporal_graph.file_timeline(path) {
                for commit in history.commits.iter().rev().take(2) {
                    issue.git_evidence.push(format!(
                        "{} — {} (by {})",
                        commit.id, commit.summary, commit.author
                    ));
                }
            }
        }
        issue.status = crate::issues::IssueStatus::Triaged;
        issue.reproduction = Some(
            "Not reproduced yet. The assigned worker must create a failing regression test or an explicit replay command before changing production code.".to_string(),
        );

        let acceptance = vec![
            format!("Create a reproducible failing test or replay command for issue {}", issue.id),
            "Identify and explain the root cause before implementing the fix".to_string(),
            "Run the focused regression test and relevant project checks after the fix".to_string(),
            "Report changed files, verification evidence, and any remaining risk".to_string(),
        ];
        let job = crate::coworker::CoworkStore::new(&self.root).create_job(
            &format!("Resolve {}: {}", issue.id, issue.report),
            Some("any"),
            acceptance,
        )?;
        issue.cowork_job_id = Some(job.id.clone());
        store.save(&issue)?;
        self.memory.add("issue", issue.compact_summary());

        Ok(format!(
            "Created **{}** and triaged it from the current Git state.\n\n{}\n\nReproduction gate: {}\nWorker job: `{}`\n\nNothing was changed yet. A worker must reproduce the failure before editing code.",
            issue.id,
            render_issue(&issue),
            issue.reproduction.as_deref().unwrap_or("pending"),
            job.id
        ))
    }

    fn remember_last_command(&mut self, input: &str, response: &str) {
        let command = if input.trim().starts_with(':') {
            input.trim().to_string()
        } else {
            self.intent_for(input).unwrap_or_else(|| input.trim().to_string())
        };
        self.memory
            .remember_conversation_state("last_command", &command);
        self.memory.remember_conversation_state(
            "last_command_result",
            &truncate_for_context(response, 2_000),
        );
    }

    fn command_followup_answer(&self, question: &str) -> Option<String> {
        let command = self.memory.conversation_state("last_command")?;
        let result = self.memory.conversation_state("last_command_result")?;
        if command.to_ascii_lowercase().starts_with(":owners") && is_owner_followup(question) {
            let owners = result
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("👤 ") {
                        return Some(
                            rest.split_once(" (")
                                .map(|(name, files)| format!("{} {}", name.trim(), files.trim_end_matches(')')))
                                .unwrap_or_else(|| rest.to_string()),
                        );
                    }
                    let owner = line.split_once("primary owner:")?.1.split('(').next()?.trim();
                    (!owner.is_empty()).then_some(owner.to_string())
                })
                .take(5)
                .collect::<Vec<_>>();
            if !owners.is_empty() {
                return Some(format!(
                    "From the Git ownership report: {}. This is based on the authors and commit history Astra indexed; the ⚠️ markers mean those files are stale, so ownership does not necessarily mean recent activity.",
                    owners.join(", ")
                ));
            }
        }

        Some(format!(
            "I checked `{}` and found this: {}",
            command,
            truncate_for_context(&result, 1_000).trim()
        ))
    }

    fn inspect_scope(&mut self, requested: &str) -> Result<String> {
        let requested = requested.trim().trim_matches(['"', '\'', '`']);
        if requested.is_empty() {
            return Ok("Usage: `:inspect <project path>`.".to_string());
        }
        let root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
        let target = root.join(requested);
        let target = match target.canonicalize() {
            Ok(path) if path.starts_with(&root) => path,
            _ => {
                return Ok(format!(
                    "I couldn't find a safe project path matching `{}`.",
                    requested
                ))
            }
        };
        let relative = target
            .strip_prefix(&root)
            .unwrap_or(&target)
            .to_string_lossy()
            .replace('\\', "/");
        self.memory
            .remember_conversation_state("focused_path", &relative);

        if target.is_file() {
            let metadata = fs::metadata(&target)?;
            let mut output = format!(
                "Inspected `{}` from disk.\n- Type: file\n- Size: {} bytes",
                relative,
                metadata.len()
            );
            let target_key = normalized_fs_path(&target);
            if let Some((_, summary)) = self
                .index
                .files()
                .iter()
                .find(|(path, _)| {
                    normalized_fs_path(&resolve_index_path(&root, path)) == target_key
                })
            {
                let _ = write!(
                    &mut output,
                    "\n- Language: {}\n- Lines: {}\n- Functions: {}",
                    summary.language, summary.line_count, summary.approx_fn_count
                );
            }
            return Ok(output);
        }

        if self.index.stats().file_count == 0 {
            self.build_index()?;
        }
        let target_key = normalized_fs_path(&target);
        let mut indexed = self
            .index
            .files()
            .iter()
            .filter(|(path, _)| {
                let path_key = normalized_fs_path(&resolve_index_path(&root, path));
                path_key == target_key || path_key.starts_with(&(target_key.clone() + "/"))
            })
            .collect::<Vec<_>>();
        indexed.sort_by(|left, right| right.1.line_count.cmp(&left.1.line_count));
        let file_count = indexed.len();
        let total_lines = indexed
            .iter()
            .map(|(_, summary)| summary.line_count)
            .sum::<usize>();
        let mut languages = std::collections::HashMap::<String, usize>::new();
        for (_, summary) in &indexed {
            *languages.entry(summary.language.clone()).or_insert(0) += summary.line_count;
        }
        let mut languages = languages.into_iter().collect::<Vec<_>>();
        languages.sort_by(|left, right| right.1.cmp(&left.1));

        let mut children = fs::read_dir(&target)?
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                (!SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.')).then_some(name)
            })
            .collect::<Vec<_>>();
        children.sort();
        children.truncate(24);

        let mut output = format!(
            "Inspected `{}` from the index and filesystem.\n- Indexed scope: {} files, {} lines",
            relative, file_count, total_lines
        );
        if !languages.is_empty() {
            let stack = languages
                .into_iter()
                .take(5)
                .map(|(language, lines)| format!("{} ({} lines)", language, lines))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = write!(&mut output, "\n- Main languages: {}", stack);
        }
        if !children.is_empty() {
            let _ = write!(&mut output, "\n- Top-level contents: {}", children.join(", "));
        }

        let package_path = target.join("package.json");
        if let Ok(content) = fs::read_to_string(&package_path) {
            if let Ok(package) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = package.get("name").and_then(|value| value.as_str()).unwrap_or("unknown");
                let scripts = package
                    .get("scripts")
                    .and_then(|value| value.as_object())
                    .map(|scripts| scripts.keys().cloned().collect::<Vec<_>>().join(", "))
                    .unwrap_or_default();
                let dependency_count = package
                    .get("dependencies")
                    .and_then(|value| value.as_object())
                    .map(|dependencies| dependencies.len())
                    .unwrap_or(0);
                let _ = write!(
                    &mut output,
                    "\n- Package: {} ({} runtime dependencies; scripts: {})",
                    name,
                    dependency_count,
                    if scripts.is_empty() { "none" } else { &scripts }
                );
            }
        }

        let component_names = indexed
            .iter()
            .filter_map(|(path, _)| path.file_stem().and_then(|name| name.to_str()))
            .filter(|name| {
                matches!(
                    name.to_ascii_lowercase().as_str(),
                    "hero" | "pricing" | "testimonials" | "finalcta" | "navbar" | "socialproof"
                )
            })
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let source_mentions_astra = indexed.iter().take(80).any(|(path, _)| {
            fs::read_to_string(resolve_index_path(&root, path))
                .map(|content| content.to_ascii_lowercase().contains("astra"))
                .unwrap_or(false)
        });
        if component_names.len() >= 2 {
            let brand = if source_mentions_astra { "an Astra" } else { "a" };
            let _ = write!(
                &mut output,
                "\n- Evidence-based purpose: this is {} marketing/landing frontend (components include {}).",
                brand,
                component_names.join(", ")
            );
        }

        if !indexed.is_empty() {
            let _ = writeln!(&mut output, "\n- Largest indexed files:");
            for (path, summary) in indexed.into_iter().take(6) {
                let path_key = normalized_fs_path(&resolve_index_path(&root, path));
                let root_key = normalized_fs_path(&root);
                let display = path_key
                    .strip_prefix(&(root_key + "/"))
                    .unwrap_or(&path_key);
                let _ = writeln!(&mut output, "  - {} ({} lines)", display, summary.line_count);
            }
        }
        Ok(output.trim_end().to_string())
    }

    fn build_index(&mut self) -> Result<()> {
        let mut index = CodeIndex::new();
        let mut stack = vec![self.root.clone()];

        while let Some(path) = stack.pop() {
            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if SKIP_DIRS.contains(&name) {
                        continue;
                    }
                }
                let entries = match fs::read_dir(&path) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                for entry in entries {
                    if let Ok(entry) = entry {
                        stack.push(entry.path());
                    }
                }
            } else if is_indexable_path(&path) {
                let contents = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                
                // --- NEW: Index high-priority docs into memory for LLM context ---
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    let lower = filename.to_lowercase();
                    if lower.contains("readme") || lower.contains("vision") || lower == "cargo.toml" {
                        let truncated_contents = truncate_for_context(&contents, 1500);
                        self.memory.add(
                            "source-doc",
                            format!("File: {}\nContents:\n{}", filename, truncated_contents)
                        );
                    }
                }
                // --- End NEW ---

                index.add_file(path, &contents);
            }
        }

        self.index = index;
        // Second pass: resolve cross-file import edges
        self.index.resolve_imports();
        self.remember_project_snapshot();

        let global_brain = crate::config::get_global_brain_path(&self.root);
        let index_path = global_brain.join("index.json");
        let _ = self.index.save(&index_path);

        // RAG: chunk every indexed file and build the vector store
        self.build_vector_store();
        let _ = self.vector_store.save(&global_brain.join("vectors.json"));

        Ok(())
    }

    fn build_vector_store(&mut self) {
        let file_entries: Vec<(std::path::PathBuf, String)> = self.index.indexed_paths();
        let mut new_store = VectorStore::new();

        for (path, language) in &file_entries {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let chunks = chunk_file(path, &content, language);
            let chunk_ids: Vec<String> = chunks.iter().map(|c| c.id.clone()).collect();
            new_store.upsert_chunks(chunks);
            self.index.set_chunk_ids(path, chunk_ids);
        }

        // Embed chunks if an embedding model is configured
        if self.embedder.is_some() {
            let chunk_ids: Vec<String> = new_store.chunks.iter().map(|c| c.id.clone()).collect();
            let chunk_texts: Vec<String> = new_store.chunks.iter().map(|c| c.content.clone()).collect();
            for (id, text) in chunk_ids.iter().zip(chunk_texts.iter()) {
                if let Some(embedder) = &self.embedder {
                    if let Ok(emb) = embedder.get_embedding(text) {
                        new_store.set_embedding(id, emb);
                    }
                }
            }
        }

        self.vector_store = new_store;
    }

    fn strip_markdown_fences(text: &str) -> String {
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

    fn memory_answer(&self, question: &str) -> Option<String> {
        let q_vec = self.get_query_embedding(question);
        let matches = self.memory.search(question, q_vec.as_deref(), 6);
        let stats = self.index.stats();
        let by_lang = self.index.files_by_language();
        if matches.is_empty() && stats.file_count == 0 {
            return None;
        }
        let mut out = String::new();
        let mut wrote_header = false;
        if stats.file_count > 0 {
            let _ = writeln!(
                &mut out,
                "Project snapshot: {} files, {} lines.",
                stats.file_count, stats.total_lines
            );
            wrote_header = true;
        }
        if !by_lang.is_empty() {
            if wrote_header {
                out.push('\n');
            }
            let _ = writeln!(&mut out, "Files by language:");
            for (lang, count) in by_lang {
                let _ = writeln!(&mut out, "- {}: {}", lang, count);
            }
            wrote_header = true;
        }
        if !matches.is_empty() {
            if wrote_header {
                out.push('\n');
            }
            let _ = writeln!(&mut out, "Memory matches:");
            for entry in matches {
                let _ = writeln!(
                    &mut out,
                    "- [{}] {} (ts: {})",
                    entry.kind, entry.content, entry.timestamp
                );
            }
            wrote_header = true;
        }
        if let Some(last_health) = self
            .memory
            .latest_event("health")
            .and_then(|entry| entry.event.as_ref())
        {
            if let MemoryEvent::HealthSnapshot { scores } = last_health {
                if wrote_header {
                    out.push('\n');
                }
                let _ = writeln!(
                    &mut out,
                    "Last health scores: quality={} test={} drift={} security={} git={} team={}",
                    scores.code_quality,
                    scores.test_health,
                    scores.cross_lang_drift,
                    scores.security_surface,
                    scores.git_health,
                    scores.team_velocity
                );
                wrote_header = true;
            }
        }
        if let Some(last_commit) = self
            .memory
            .latest_event("git-commit")
            .and_then(|entry| entry.event.as_ref())
        {
            if wrote_header {
                out.push('\n');
            }
            if let MemoryEvent::GitCommit {
                id,
                summary,
                author,
                date,
            } = last_commit
            {
                let _ = writeln!(
                    &mut out,
                    "Last commit: {} by {} at {} — {}",
                    id, author, date, summary
                );
            }
        }
        let user_profile = self.memory.profile_report();
        if user_profile != "No user profile facts stored yet." {
            if wrote_header {
                out.push('\n');
            }
            let _ = writeln!(&mut out, "{}", user_profile);
            wrote_header = true;
        }
        let project_profile = self.memory.project_report();
        if project_profile != "No project memory facts stored yet." {
            if wrote_header {
                out.push('\n');
            }
            let _ = writeln!(&mut out, "{}", project_profile);
        }
        if out.trim().is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn recent_conversation_context(&self, limit: usize) -> Vec<String> {
        self.memory
            .events_of_kind("qa")
            .iter()
            .rev()
            .take(limit)
            .rev()
            .map(|entry| entry.content.clone())
            .collect()
    }

    fn rank_relevant_files(&self, question: &str, limit: usize) -> Vec<PathBuf> {
        let question_lower = question.to_ascii_lowercase();
        let mentioned_file = self
            .extract_file_path_from_question(question)
            .map(|value| value.to_ascii_lowercase());
        let query_terms = extract_query_terms(question);

        let mut scored: Vec<(usize, PathBuf)> = self
            .index
            .files()
            .iter()
            .filter_map(|(path, summary)| {
                let path_lower = path.to_string_lossy().to_ascii_lowercase();
                let stem_lower = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let mut score = 0usize;

                if let Some(file) = &mentioned_file {
                    if path_lower.ends_with(file) || path_lower.contains(file) {
                        score += 100;
                    }
                }

                if !stem_lower.is_empty() && question_lower.contains(&stem_lower) {
                    score += 12;
                }

                for term in &query_terms {
                    if term.len() < 3 {
                        continue;
                    }
                    if path_lower.contains(term) {
                        score += 4;
                    }
                    if stem_lower == *term {
                        score += 8;
                    }
                    if summary
                        .symbols
                        .iter()
                        .any(|symbol| symbol.name.eq_ignore_ascii_case(term))
                    {
                        score += 12;
                    } else if summary.symbols.iter().any(|symbol| {
                        symbol.name.to_ascii_lowercase().contains(term)
                    }) {
                        score += 5;
                    }
                }

                if score > 0 {
                    Some((score, path.clone()))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, path)| path)
            .collect()
    }

    fn collect_relevant_file_hints(&self, question: &str, limit: usize) -> Vec<String> {
        self.rank_relevant_files(question, limit)
            .into_iter()
            .filter_map(|path| {
                self.index.files().get(&path).map(|summary| {
                    let examples = summary
                        .symbols
                        .iter()
                        .take(4)
                        .map(|symbol| symbol.name.clone())
                        .collect::<Vec<_>>();
                    let example_text = if examples.is_empty() {
                        String::new()
                    } else {
                        format!(" | symbols: {}", examples.join(", "))
                    };
                    format!(
                        "{} [{}] {} lines, {} functions{}",
                        path.display(),
                        summary.language,
                        summary.line_count,
                        summary.approx_fn_count,
                        example_text
                    )
                })
            })
            .collect()
    }

    fn collect_relevant_memories(
        &self,
        question: &str,
        query_embedding: Option<&[f32]>,
    ) -> Vec<MemoryEntry> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();

        for entry in self
            .memory
            .context(question, query_embedding, 6, 1_200)
            .into_iter()
            .filter(|entry| include_memory_entry_in_answer(entry, question))
        {
            let key = format!("{}::{}", entry.kind, entry.content);
            if seen.insert(key) {
                out.push(entry);
            }
            if out.len() >= 6 {
                break;
            }
        }

        let lower = question.to_ascii_lowercase();
        if out.len() < 6
            && (lower.contains("readme")
                || lower.contains("vision")
                || lower.contains("roadmap")
                || lower.contains("docs")
                || lower.contains("product"))
        {
            for doc in self.memory.events_of_kind("source-doc").iter().rev().take(2) {
                let mut cloned = (*doc).clone();
                if cloned.content.len() > 900 {
                    cloned.content.truncate(900);
                    cloned.content.push_str("\n... [Truncated]");
                }
                let key = format!("{}::{}", cloned.kind, cloned.content);
                if seen.insert(key) {
                    out.push(cloned);
                }
            }
        }

        out
    }

    fn collect_relevant_chunks(
        &self,
        question: &str,
        query_embedding: Option<&[f32]>,
        limit: usize,
    ) -> Vec<crate::rag::Chunk> {
        let mut selected = Vec::new();
        let mut seen_ids = HashSet::new();

        for chunk in self.vector_store.search(question, query_embedding, limit) {
            if seen_ids.insert(chunk.id.clone()) {
                selected.push(chunk.clone());
            }
        }

        let mentioned_file = self
            .extract_file_path_from_question(question)
            .map(|value| value.to_ascii_lowercase());
        if let Some(file) = mentioned_file {
            for chunk in self
                .vector_store
                .chunks
                .iter()
                .filter(|chunk| {
                    let path_lower = chunk.path.to_string_lossy().to_ascii_lowercase();
                    path_lower.ends_with(&file) || path_lower.contains(&file)
                })
                .take(2)
            {
                if seen_ids.insert(chunk.id.clone()) {
                    selected.insert(0, chunk.clone());
                }
            }
        }

        for path in self.rank_relevant_files(question, 3) {
            for chunk in self
                .vector_store
                .chunks
                .iter()
                .filter(|chunk| chunk.path == path)
                .take(2)
            {
                if seen_ids.insert(chunk.id.clone()) {
                    selected.push(chunk.clone());
                }
            }
        }

        selected.truncate(limit);
        selected
    }

    fn answer_question(&mut self, question: &str) -> Result<String> {
        if is_assistant_identity_question(question) {
            return Ok("I’m Astra — your dev + codebase companion for this repo. What are we doing today?".to_string());
        }

        if is_what_should_we_do_question(question) {
            let changed = self
                .git
                .as_ref()
                .map(|g| g.changed_files())
                .unwrap_or_default();
            if !changed.is_empty() {
                self.memory.remember_conversation_state(
                    "last_option_menu",
                    "1=review,2=health,3=ship-preview",
                );
                return Ok("We’ve got uncommitted changes. Want me to (1) review the diff, (2) run a quick health check, or (3) help you ship it clean?".to_string());
            }
            return Ok("Depends what you’re trying to do. Are we building a feature, fixing a bug, or just exploring the repo?".to_string());
        }

        if is_name_question(question) {
            if let Some(name) = self.memory.user_name() {
                return Ok(format!(
                    "I’ve got your name saved as “{}”. Is that still what you want me to call you?",
                    name
                ));
            }
            return Ok(
                "I don’t actually know your name yet. Tell me “my name is <name>” and I’ll remember it."
                    .to_string(),
            );
        }

        if is_language_question(question) {
            let stats = self.index.stats();
            if stats.file_count == 0 {
                return Ok(
                    "I don’t have indexed project context yet, so I can’t answer language breakdown stuff without guessing. Run `:index` first, then ask again."
                        .to_string(),
                );
            }
            let lines_by_lang = self.index.lines_by_language();
            let files_by_lang = self.index.files_by_language();

            if let Some(lang) = extract_language_mention(question) {
                let lines = *lines_by_lang.get(&lang).unwrap_or(&0);
                let files = *files_by_lang.get(&lang).unwrap_or(&0);
                if stats.total_lines > 0 {
                    let pct = (lines as f64 / stats.total_lines as f64) * 100.0;
                    return Ok(format!(
                        "{}: {} lines across {} files (~{:.1}% of indexed lines).",
                        lang, lines, files, pct
                    ));
                }
                return Ok(format!("{}: {} lines across {} files.", lang, lines, files));
            }

            if let Some((top_lang, top_lines)) = top_language_by_lines(&lines_by_lang) {
                if stats.total_lines > 0 {
                    let pct = (top_lines as f64 / stats.total_lines as f64) * 100.0;
                    return Ok(format!(
                        "Dominant language (by lines): {} — {} lines (~{:.1}% of indexed lines).",
                        top_lang, top_lines, pct
                    ));
                }
                return Ok(format!(
                    "Dominant language (by lines): {} — {} lines.",
                    top_lang, top_lines
                ));
            }
        }

        // ── Direct personal fact lookup — gather ALL identity facts ──
        if is_personal_question(question) {
            let mut all_facts: Vec<String> = Vec::new();
            
            // Gather from global memory
            if let Some(global) = &self.memory.global {
                for entry in &global.entries {
                    if matches!(entry.kind.as_str(), "fact" | "user-identity" | "user-preference") {
                        all_facts.push(format!("[{}] {}", entry.kind, entry.content));
                    }
                }
            }
            // Also gather from local memory
            for entry in &self.memory.entries {
                if entry.kind == "fact" || entry.kind == "user-identity" || entry.kind == "user-preference" {
                    all_facts.push(format!("[{}] {}", entry.kind, entry.content));
                }
            }

            if !all_facts.is_empty() {
                // Use LLM to synthesize a natural answer if available
                if let Some(model) = &self.model {
                    let facts_text = all_facts.join("\n");
                    let prompt = format!(
                        "You are Astra, a helpful assistant. The user has asked a personal question.\n\
                        \n\
                        USER QUESTION: {}\n\
                        \n\
                        HERE ARE ALL THE FACTS YOU KNOW ABOUT THIS USER (TREAT AS GROUND TRUTH):\n\
                        {}\n\
                        \n\
                        RULES:\n\
                        - Answer the user's question using ONLY the facts above.\n\
                        - Be direct and conversational. Do NOT say you don't have the information if it's in the facts.\n\
                        - If the answer isn't in the facts, say what you DO know about them.\n\
                        - Keep your response to 1-3 sentences max.",
                        question, facts_text
                    );
                    match model.complete(&prompt) {
                        Ok(answer) => return Ok(answer),
                        Err(e) => {
                            eprintln!("LLM Fact Synthesis Failed: {}", e);
                            // fall through to raw facts
                        }
                    }
                }

                // Fallback: show raw facts if no LLM
                let mut reply = String::from("Here's what I remember about you:\n");
                for fact in &all_facts {
                    reply.push_str(&format!("• {}\n", fact));
                }
                return Ok(reply);
            }
        }
        if let Some(model) = &self.model {
            let lower_question = question.to_ascii_lowercase();
            let risk_related = lower_question.contains("risk")
                || lower_question.contains("fragile")
                || lower_question.contains("hotspot")
                || lower_question.contains("coupling")
                || lower_question.contains("bus factor")
                || lower_question.contains("ownership")
                || lower_question.contains("who owns")
                || lower_question.contains("stale")
                || lower_question.contains("history")
                || lower_question.contains("incident");
            if risk_related
                && self.temporal_graph.enriched
                && self.concept_store.concepts.is_empty()
            {
                if let Ok(_) = self
                    .concept_store
                    .derive_concepts(&self.memory, &self.temporal_graph, model.as_ref())
                {
                    let global_brain = crate::config::get_global_brain_path(&self.root);
                    let _ = self
                        .concept_store
                        .save(&global_brain.join("concepts.json"));
                }
            }
            let stats = self.index.stats();
            let by_lang = self.index.files_by_language();
            let lines_by_lang = self.index.lines_by_language();
            let q_vec = self.get_query_embedding(question);
            let mut matches = self.collect_relevant_memories(question, q_vec.as_deref());

            if stats.file_count == 0 {
                if is_project_context_question(question) {
                    return Ok(
                        "I don’t have indexed project context yet, so I can’t reliably answer project-specific questions without guessing. Run `:index` first, then ask again."
                            .to_string(),
                    );
                }
                let low_context_prompt = format!(
                    "You are Astra, a concise and respectful engineering assistant.\n\
                     The project is not indexed yet (0 files), so do not claim repo-specific facts.\n\
                     Rules:\n\
                     - Answer the user's question directly.\n\
                     - No sarcasm, no catchphrases, no personality filler.\n\
                     - If the user asks for a repo feature in normal conversation, answer naturally, but do not invent missing project facts.\n\
                     - If the question requires project context, say to run :index.\n\
                     - Never mention README/framework/tooling unless explicitly provided in this question.\n\n\
                     User question: {}",
                    question
                );
                let answer = model.complete(&low_context_prompt)?;
                self.memory
                    .add("qa", format!("Q: {}\nA: {}", question, answer));
                return Ok(answer);
            }

            let relevant_files = self.collect_relevant_file_hints(question, 4);
            let social_question = is_social_message(question);
            let architecture_context = if !social_question
                && (is_architecture_question(question) || is_project_context_question(question))
            {
                self.architecture_context_summary(question)
            } else {
                None
            };
            let team_context = if !social_question && is_team_question(question) {
                self.team_context_summary()
            } else {
                None
            };
            let conversation_feature_request = !social_question
                && (
                    is_architecture_question(question)
                        || is_team_question(question)
                        || lower_question.contains("history")
                        || lower_question.contains("hotspot")
                        || lower_question.contains("who owns")
                        || lower_question.contains("onboard")
                        || lower_question.contains("where should i start")
                        || lower_question.contains("summary")
                        || lower_question.contains("what changed")
                );
            let include_recent_qa = !social_question
                && (
                    lower_question.contains("continue")
                        || lower_question.contains("resume")
                        || lower_question.contains("again")
                        || lower_question.contains("earlier")
                        || lower_question.contains("before")
                        || lower_question.contains("previous")
                        || lower_question.contains("last time")
                        || lower_question.contains("we were")
                        || lower_question.contains("we are")
                );
            let include_turn_context = include_recent_qa || needs_recent_turn_context(question);
            let (last_turn_snippet, last_two_snippet) = if include_turn_context {
                let recent_turns = self.recent_conversation_context(2);
                let last_turn_snippet = recent_turns
                    .last()
                    .map(|turn| truncate_for_context(turn, 500));
                let last_two_snippet = if recent_turns.len() > 1 {
                    Some(truncate_for_context(&recent_turns.join("\n\n"), 900))
                } else {
                    None
                };
                (last_turn_snippet, last_two_snippet)
            } else {
                (None, None)
            };
            let include_profile_report = is_name_question(question) || is_personal_question(question);
            let include_project_report = !social_question
                && (
                    lower_question.contains("project memory")
                        || lower_question.contains("what do you know about this project")
                        || lower_question.contains("remember about this project")
                        || lower_question.contains("codebase")
                        || lower_question.contains("architecture")
                        || lower_question.contains("ownership")
                        || lower_question.contains("team")
                        || lower_question.contains("history")
                );
            // MemoryStore::search only returns entries with lexical or semantic
            // relevance, so useful project continuity should be available for
            // every substantive question—not only questions containing words
            // such as "remember" or "history".
            if risk_related && !self.concept_store.concepts.is_empty() {
                let mut concepts = self.concept_store.concepts.clone();
                concepts.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for concept in concepts.into_iter().take(5) {
                    matches.push(MemoryEntry {
                        kind: "semantic-concept".to_string(),
                        content: format!(
                            "{} (category: {:?}, confidence: {:.0}%)",
                            concept.description,
                            concept.category,
                            concept.confidence * 100.0
                        ),
                        timestamp: concept.last_updated,
                        event: None,
                        embedding: None,
                    });
                }
            }
            if risk_related && !self.concept_store.patterns.is_empty() {
                let mut patterns = self.concept_store.patterns.clone();
                patterns.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for pattern in patterns.into_iter().take(3) {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    matches.push(MemoryEntry {
                        kind: "procedural-pattern".to_string(),
                        content: format!(
                            "{} (confidence: {:.0}%, evidence: {})",
                            pattern.description,
                            pattern.confidence * 100.0,
                            pattern.evidence_count
                        ),
                        timestamp: ts,
                        event: None,
                        embedding: None,
                    });
                }
            }
            if risk_related && !self.concept_store.watches.is_empty() {
                let mut watches = self.concept_store.watches.clone();
                watches.sort_by(|a, b| {
                    let pa = match a.priority {
                        crate::semantic_memory::WatchPriority::Critical => 0,
                        crate::semantic_memory::WatchPriority::High => 1,
                        crate::semantic_memory::WatchPriority::Medium => 2,
                        crate::semantic_memory::WatchPriority::Low => 3,
                    };
                    let pb = match b.priority {
                        crate::semantic_memory::WatchPriority::Critical => 0,
                        crate::semantic_memory::WatchPriority::High => 1,
                        crate::semantic_memory::WatchPriority::Medium => 2,
                        crate::semantic_memory::WatchPriority::Low => 3,
                    };
                    pa.cmp(&pb)
                });
                for watch in watches.into_iter().take(3) {
                    matches.push(MemoryEntry {
                        kind: "prospective-watch".to_string(),
                        content: format!(
                            "{} [priority: {:?}]",
                            watch.description,
                            watch.priority
                        ),
                        timestamp: watch.created_at,
                        event: None,
                        embedding: None,
                    });
                }
            }

            // --- Add live Git history when the question is about what changed/shipped ---
            let ql = question.to_lowercase();
            let asks_about_history = [
                "commit", "history", "ship", "shipped", "push", "pushed", "release",
                "did we", "did you", "what changed", "recent", "recently", "latest",
                "last change", "just do", "just did", "worked on", "merge",
            ]
            .iter()
            .any(|kw| ql.contains(kw));
            if asks_about_history {
                if let Some(git) = &self.git {
                    if let Ok(commits) = git.recent_commits(8) {
                        let mut log = String::from("Recent git commits (authoritative — this is what was actually shipped, newest first):\n");
                        for c in &commits {
                            let _ = writeln!(&mut log, "- {} — {} (by {})", &c.id[..c.id.len().min(8)], c.summary, c.author);
                        }
                        matches.push(MemoryEntry {
                            kind: "git-log".to_string(),
                            content: log,
                            timestamp: 0,
                            event: None,
                            embedding: None,
                        });
                    }
                    // Also surface the exact working-tree state so "what did we ship" is precise.
                    let changed = git.changed_files();
                    if !changed.is_empty() {
                        matches.push(MemoryEntry {
                            kind: "git-status".to_string(),
                            content: format!(
                                "Uncommitted changes still in the working tree ({} file(s)): {}",
                                changed.len(),
                                changed.iter().take(12).cloned().collect::<Vec<_>>().join(", ")
                            ),
                            timestamp: 0,
                            event: None,
                            embedding: None,
                        });
                    }
                }
            }

            // --- Autonomous Tool Interception (Semantic Enrichment) ---
            let semantic_context = self.try_semantic_enrichment(question);
            for entry in semantic_context {
                if !matches
                    .iter()
                    .any(|existing| existing.kind == entry.kind && existing.content == entry.content)
                {
                    matches.push(entry);
                }
            }

            // The model receives only a compact slice of verified memory, even
            // when multiple subsystems contribute candidates on this turn.
            matches = compact_for_context(&matches, 4, 1_200);
            let include_memory_matches = !social_question && !matches.is_empty();

            // --- RAG: inject relevant code chunks ---
            let relevant_chunks = if !self.vector_store.is_empty() {
                self.collect_relevant_chunks(question, q_vec.as_deref(), 4)
            } else {
                Vec::new()
            };
            let weak_grounding = !social_question
                && is_project_context_question(question)
                && matches.is_empty()
                && relevant_chunks.is_empty()
                && architecture_context.is_none()
                && team_context.is_none();
            let rag_context = if relevant_chunks.is_empty() {
                None
            } else {
                let refs = relevant_chunks.iter().collect::<Vec<_>>();
                Some(format_chunks_for_prompt(&refs, 3_600))
            };

            let mut system_prompt = String::new();
            let _ = writeln!(&mut system_prompt, "{}", self.persona.system_prompt());
            let _ = writeln!(
                &mut system_prompt,
                "You are Astra, a grounded local codebase companion and pair engineer."
            );
            let _ = writeln!(
                &mut system_prompt,
                "Answer naturally and intelligently, but never pretend you saw code, ran commands, or know facts that are not in the provided context."
            );
            let _ = writeln!(
                &mut system_prompt,
                "Prefer retrieved code and verified memory over generic advice."
            );
            let _ = writeln!(
                &mut system_prompt,
                "When you infer rather than know, say so plainly."
            );
            let _ = writeln!(
                &mut system_prompt,
                "If code evidence is provided, cite the file paths and line ranges from that evidence when it helps."
            );
            let _ = writeln!(
                &mut system_prompt,
                "In chat mode you cannot run commands or edit files, so never claim that you already did."
            );
            let _ = writeln!(
                &mut system_prompt,
                "Never invent your own activity or emotional work state: do not say you were reviewing, working on, busy with, or stuck on a project unless verified task state explicitly establishes it."
            );
            let _ = writeln!(
                &mut system_prompt,
                "If the user asks you to implement or change code, explain the next step and say you can start."
            );
            let _ = writeln!(
                &mut system_prompt,
                "For repo-specific questions, use only grounded evidence from the provided context sections. Never fill missing repo details with generic guesses."
            );
            let _ = writeln!(
                &mut system_prompt,
                "If the user asks for something that exists as a CLI feature in normal conversation, answer directly from the grounded context instead of redirecting them to a command unless the data is missing."
            );

            let mut user_prompt = String::new();
            let _ = writeln!(&mut user_prompt, "### USER QUESTION");
            let _ = writeln!(&mut user_prompt, "{}", question);
            let _ = writeln!(&mut user_prompt);
            if !social_question {
                let convo_state = self.memory.conversation_state_report();
                if let Some(convo_state) = &convo_state {
                    let _ = writeln!(&mut user_prompt, "### CONVERSATION STATE (FOR CONTINUITY)");
                    let _ = writeln!(
                        &mut user_prompt,
                        "Use this silently for continuity. Do not recap it unless the user asks.\n{}",
                        convo_state
                    );
                    let _ = writeln!(&mut user_prompt);
                }
                let style = self.memory.style_report();
                if let Some(style) = &style {
                    let _ = writeln!(&mut user_prompt, "### STYLE PREFERENCES");
                    let _ = writeln!(
                        &mut user_prompt,
                        "These are user preferences. Follow them unless they conflict with safety/grounding.\n{}",
                        style
                    );
                    let _ = writeln!(&mut user_prompt);
                }
                if let Some(snippet) = &last_two_snippet {
                    let _ = writeln!(&mut user_prompt, "### LAST 2 TURNS (FOR CONTINUITY)");
                    let _ = writeln!(
                        &mut user_prompt,
                        "Use this silently to keep continuity. Do not recap it unless the user asks.\n{}",
                        snippet
                    );
                    let _ = writeln!(&mut user_prompt);
                } else if let Some(snippet) = &last_turn_snippet {
                    let _ = writeln!(&mut user_prompt, "### LAST TURN (FOR CONTINUITY)");
                    let _ = writeln!(
                        &mut user_prompt,
                        "Use this silently to keep continuity. Do not recap it unless the user asks.\n{}",
                        snippet
                    );
                    let _ = writeln!(&mut user_prompt);
                }
            }
            let _ = writeln!(&mut user_prompt, "### VERIFIED PROJECT CONTEXT");
            let _ = writeln!(&mut user_prompt, "- Root: {:?}", self.root);
            let _ = writeln!(
                &mut user_prompt,
                "- Indexed: {} files, {} lines.",
                stats.file_count,
                stats.total_lines
            );

            if let Ok(entries) = std::fs::read_dir(&self.root) {
                let mut dirs: Vec<String> = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        dirs.push(name);
                    }
                }
                dirs.sort();
                if !dirs.is_empty() {
                    let _ = writeln!(
                        &mut user_prompt,
                        "- Top-level directories: {}",
                        dirs.join(", ")
                    );
                }
            }
            if !lines_by_lang.is_empty() {
                let mut top = lines_by_lang.into_iter().collect::<Vec<_>>();
                top.sort_by(|a, b| b.1.cmp(&a.1));
                let top_str = top
                    .into_iter()
                    .take(4)
                    .map(|(lang, lines)| format!("{}={} lines", lang, lines))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(&mut user_prompt, "- Top languages (by lines): {}", top_str);
            } else if !by_lang.is_empty() {
                let mut top = by_lang.into_iter().collect::<Vec<_>>();
                top.sort_by(|a, b| b.1.cmp(&a.1));
                let top_str = top
                    .into_iter()
                    .take(4)
                    .map(|(lang, count)| format!("{}={} files", lang, count))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(&mut user_prompt, "- Top languages (by files): {}", top_str);
            }

            if is_task_context_question(question) {
                let task_path = self.root.join(".astra").join("active_task.json");
                if let Ok(task_content) = std::fs::read_to_string(&task_path) {
                    if let Ok(task) = serde_json::from_str::<serde_json::Value>(&task_content) {
                        let title = task
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown");
                        let phase = task
                            .get("phase")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown");
                        let _ = writeln!(&mut user_prompt, "- Active task: {} ({})", title, phase);
                    }
                }
            }
            if !relevant_files.is_empty() {
                let _ = writeln!(&mut user_prompt, "\n### LIKELY RELEVANT FILES");
                for hint in relevant_files {
                    let _ = writeln!(&mut user_prompt, "- {}", hint);
                }
            }

            if let Some(architecture_context) = &architecture_context {
                let _ = writeln!(&mut user_prompt, "\n### ARCHITECTURE SNAPSHOT");
                let _ = writeln!(&mut user_prompt, "{}", architecture_context);
            }

            if let Some(team_context) = &team_context {
                let _ = writeln!(&mut user_prompt, "\n### TEAM CONTEXT");
                let _ = writeln!(&mut user_prompt, "{}", team_context);
            }

            if conversation_feature_request {
                let _ = writeln!(&mut user_prompt, "\n### CONVERSATION MODE");
                let _ = writeln!(
                    &mut user_prompt,
                    "The user is asking for companion features in normal conversation. Answer normally using the grounded context below instead of referring them to a command unless required data is missing."
                );
            }

            if weak_grounding {
                let _ = writeln!(&mut user_prompt, "\n### GROUNDING WARNING");
                let _ = writeln!(
                    &mut user_prompt,
                    "Grounded evidence for this specific repo question is thin. Do not guess missing project facts. Say what is confirmed, then call out what is still unknown."
                );
            }

            let profile_report = self.memory.profile_report();
            if include_profile_report && profile_report != "No user profile facts stored yet." {
                let _ = writeln!(&mut user_prompt, "\n### USER PROFILE");
                let _ = writeln!(&mut user_prompt, "{}", profile_report);
            }
            let project_report = self.memory.project_report();
            if include_project_report && project_report != "No project memory facts stored yet." {
                let _ = writeln!(&mut user_prompt, "\n### PROJECT MEMORY");
                let _ = writeln!(&mut user_prompt, "{}", project_report);
            }

            if include_memory_matches {
                let _ = writeln!(&mut user_prompt, "\n### RELEVANT MEMORY");
                for entry in &matches {
                    let _ = writeln!(&mut user_prompt, "- [{}] {}", entry.kind, entry.content);
                }
            }

            if let Some(ref rag) = rag_context {
                let _ = writeln!(
                    &mut user_prompt,
                    "\n### RELEVANT CODE (GROUND TRUTH FROM THE CODEBASE)"
                );
                let _ = writeln!(&mut user_prompt, "{}", rag);
            }

            let _ = writeln!(&mut user_prompt, "\n### RESPONSE INSTRUCTIONS");
            let _ = writeln!(
                &mut user_prompt,
                "- Start with the direct answer, not a preamble."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- For casual chat, reply like a real teammate in 1-2 natural sentences."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- Do not mention dates, timestamps, stored memory, or old conversation history unless the user asks for them."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- Use the retrieved code and memory when relevant."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- Use architecture and team context when it directly helps answer the question."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- If this is a normal-conversation version of a repo feature, answer it directly instead of telling the user to run a command."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- If the context is incomplete, say what is missing instead of guessing."
            );
            let _ = writeln!(
                &mut user_prompt,
                "- If the user is asking for implementation work, explain the best next step and mention that you can start."
            );

            println!(" 💭 Thinking...");
            let answer = model.complete_chat(&system_prompt, &user_prompt)?;

            // --- Autonomous Interception: scan lines for command ---
            let explicit_execution_request = {
                let lower_question = question.to_ascii_lowercase();
                lower_question.contains("run ")
                    || lower_question.contains("start ")
                    || lower_question.contains("build ")
                    || lower_question.contains("implement ")
                    || lower_question.contains("fix ")
                    || lower_question.contains("migrate ")
                    || lower_question.contains("automate ")
            };
            if explicit_execution_request {
                for line in answer.lines() {
                    let text = line.trim().trim_matches('`');
                    if text.starts_with(":workflow run ")
                        || text.starts_with(":workflow generate ")
                        || text.starts_with(":workflow list")
                        || text.starts_with(":migrate ")
                        || text.starts_with(":task ")
                    {
                        self.memory
                            .add("autonomous-action", format!("Astra autonomously triggered: {}", text));
                        return self.handle_input(text);
                    }
                }
            }

            // If the answer seems uncertain and we have web search, augment it
            let lower_answer = answer.to_ascii_lowercase();
            // Only trigger web search fallback for very specific "I have no info" patterns
            // Avoid false positives that cause a slow double-call
            let seems_uncertain = (lower_answer.contains("i don't have") && lower_answer.contains("information"))
                || (lower_answer.contains("i'm not sure") && lower_answer.len() < 200)
                || (lower_answer.contains("i don't know") && lower_answer.len() < 200);

            if seems_uncertain && should_auto_search_web(question) {
                if let Some(search) = &self.search {
                    println!("\n ◈ Astra ❯ Searching the web to find the answer... ⏳");
                    if let Ok(results) = search.search(question) {
                        self.memory.add("web-search", format!("Auto-search for: {}\n{}", question, &results[..results.len().min(2000)]));
                        let augmented_user_prompt = format!(
                            "{}\n\n### WEB RESULTS\n{}\n\nUse these only if they actually help answer the question better.",
                            user_prompt,
                            &results[..results.len().min(3000)]
                        );
                        if let Ok(better_answer) =
                            model.complete_chat(&system_prompt, &augmented_user_prompt)
                        {
                            self.memory.add("web-knowledge", format!("Q: {}\nA: {}", question, better_answer));
                            self.memory.add("qa", format!("Q: {}\nA: {}", question, better_answer));
                            return Ok(format!("{}\n\n_🌐 Answer augmented with web search results._", better_answer));
                        }
                    }
                }
            }

            self.memory
                .add("qa", format!("Q: {}\nA: {}", question, answer));
            Ok(answer)
        } else if let Some(answer) = self.memory_answer(question) {
            self.memory
                .add("qa-memory", format!("Q: {}\nA: {}", question, answer));
            Ok(answer)
        } else {
            Ok(
                "No language model is configured. Try :summary or :memory <query>.".to_string(),
            )
        }
    }

    fn intent_for(&self, trimmed: &str) -> Option<String> {
        // Greetings/social should NOT be intercepted by intent rules.
        if is_social_message(trimmed) {
            return None;
        }

        if trimmed.starts_with(':') || trimmed.starts_with("? ") {
            return None;
        }
        
        if trimmed.starts_with('?') {
             return Some(format!(":memory {}", &trimmed[1..].trim()));
        }
        if let Some(cmd) = self.parse_natural_migrate_request(trimmed) {
            return Some(cmd);
        }

        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("fully index")
            || lower.contains("index everything")
            || lower.contains("index the entire")
            || lower.contains("reindex everything")
            || lower.contains("scan the entire folder")
        {
            return Some(":index".to_string());
        }

        if let Some(focused_path) = self.memory.conversation_state("focused_path") {
            if is_focused_scope_followup(&lower, &focused_path) {
                return Some(format!(":inspect {}", focused_path));
            }
        }

        if let Some(path) = self.find_mentioned_top_level_path(trimmed) {
            let scope_request = lower.contains("audit")
                || lower.contains("inspect")
                || lower.contains("check")
                || lower.contains("look at")
                || lower.contains("observe")
                || lower.contains("what is")
                || lower.contains("about")
                || lower.contains(" first")
                || (lower.split_whitespace().count() <= 8 && lower.starts_with("the "));
            if scope_request {
                return Some(format!(":inspect {}", path));
            }
        }

        if (lower.contains("connect") || lower.contains("set up") || lower.contains("setup"))
            && lower.contains("astra")
            && (lower.contains("codex")
                || lower.contains("claude")
                || lower.contains("cursor")
                || lower.contains("mcp"))
        {
            return Some(":cowork init".to_string());
        }

        if let Some(command) = parse_cowork_delegate_request(trimmed) {
            return Some(command);
        }

        if (lower.starts_with("fix ") || lower.starts_with("debug "))
            && !trimmed.ends_with('?')
        {
            let report = trimmed
                .split_once(char::is_whitespace)
                .map(|(_, rest)| rest.trim())
                .unwrap_or("");
            if !report.is_empty() {
                return Some(format!(":fix-bug {}", report));
            }
        }

        if (
            lower.contains("migrate")
                || lower.contains("refactor")
                || lower.contains("implement")
                || lower.contains("fix ")
                || lower.contains("write code")
                || lower.contains("build feature")
                || lower.contains("add feature")
                || lower.contains("debug")
                || lower.contains("create folder")
                || lower.contains("create directory")
                || lower.contains("create a folder")
                || lower.contains("create a directory")
                || lower.contains("add file")
                || lower.contains("create file")
                || lower.contains("write to file")
                || lower.contains("then run")
        )
            && !trimmed.ends_with('?')
        {
            return Some(format!(":task {}", trimmed));
        }

        if lower.contains("what do you remember")
            || (lower.contains("remember") && lower.contains("what"))
            || lower.contains("memory bank")
            || lower.contains("show your memory")
            || lower.contains("what have you learned")
        {
            return Some(":memory".to_string());
        }

        if lower.contains("my profile")
            || lower.contains("what do you know about me")
            || lower.contains("remember about me")
        {
            return Some(":profile".to_string());
        }

        if lower.contains("project memory")
            || lower.contains("what do you know about this project")
            || lower.contains("remember about this project")
        {
            return Some(":project-memory".to_string());
        }

        if lower.contains("what do you know")
            || lower.contains("project summary")
            || lower.contains("project info")
            || lower.contains("project information")
            || lower.contains("overview of the project")
            || lower.contains("summarize this codebase")
            || lower.contains("summarize the project")
        {
            return Some(":summary".to_string());
        }

        if lower.contains("project history")
            || lower.contains("full history")
            || lower.contains("history of this project")
            || lower.contains("history of the project")
            || lower.contains("how did this project evolve")
            || lower.contains("how has this project changed")
            || lower.contains("project evolution")
        {
            return Some(":history".to_string());
        }

        if lower == "astra team status"
            || lower == "team status"
            || (lower.contains("team") && lower.contains("status"))
        {
            return Some(":team status".to_string());
        }

        if lower.starts_with("learn ")
            || lower.contains("learn rust")
            || lower.contains("learn go")
            || lower.contains("learn python")
            || lower.contains("learn typescript")
            || lower.contains("teach me")
        {
            let topic = trimmed
                .split_once("learn ")
                .map(|(_, rest)| rest.trim())
                .or_else(|| trimmed.split_once("teach me ").map(|(_, rest)| rest.trim()))
                .unwrap_or(trimmed);
            return Some(format!(":learn {}", topic));
        }

        if (lower.contains("how many") && lower.contains("file"))
            || lower.contains("files by language")
            || lower.contains("files-by-lang")
            || lower.contains("file breakdown by language")
            || lower.contains("language breakdown")
        {
            return Some(":files-by-lang".to_string());
        }

        if lower.contains("git repo") || lower.contains("git repository") {
            return Some(":summary".to_string());
        }

        if lower.contains("search the web for")
            || lower.starts_with("search web for ")
            || lower.starts_with("web search ")
            || lower.starts_with("google ")
        {
            let patterns = [
                "search the web for ",
                "search web for ",
                "web search ",
                "google ",
            ];
            for pat in &patterns {
                if let Some(pos) = lower.find(pat) {
                    let start = pos + pat.len();
                    if start <= trimmed.len() {
                        let query = trimmed[start..].trim();
                        if !query.is_empty() {
                            return Some(format!(":web {}", query));
                        }
                    }
                    break;
                }
            }
        }

        if (lower.contains("how many") && lower.contains("commit"))
            || lower.contains("commit count")
        {
            return Some(":git-commit-count".to_string());
        }

        if lower.contains("last commit")
            || lower.contains("recent commit")
            || lower.contains("most recent commit")
            || lower.contains("when did i make")
            || lower.contains("when did i commit")
            || lower.contains("when was my last commit")
        {
            return Some(":git-last-commit".to_string());
        }

        if lower.contains("audit")
            || (lower.contains("how good") && lower.contains("project"))
            || (lower.contains("observe") && lower.contains("project"))
            || (lower.contains("how well are we doing")
                && (lower.contains("project")
                    || lower.contains("repo")
                    || lower.contains("codebase")
                    || lower.contains("we doing")))
        {
            return Some(":audit".to_string());
        }

        if lower.contains("who built this project")
            || lower.contains("who built the project")
            || lower.contains("who made this project")
            || lower.contains("project authors")
            || lower.contains("top contributors")
        {
            return Some(":history".to_string());
        }

        if lower.contains("health check")
            || lower == "health"
            || lower.contains("codebase health")
            || lower.contains("health of the project")
            || lower.contains("project health")
        {
            return Some(":health".to_string());
        }

        if lower.contains("graph")
            || lower.contains("semantic graph")
            || lower.contains("dependency graph")
            || lower.contains("architecture graph")
            || lower.contains("map of the project")
        {
            return Some(":graph".to_string());
        }

        if lower.contains("where should i start")
            || lower.contains("onboard me")
            || lower.contains("onboarding")
            || lower.contains("understand this codebase")
        {
            return Some(":onboard".to_string());
        }

        if (lower.contains("risk")
            || lower.contains("risky")
            || lower.contains("fragile")
            || lower.contains("hotspot"))
            && self.temporal_graph.enriched
        {
            return Some(":analyze".to_string());
        }

        if lower.contains("hotspots")
            || lower.contains("most changed files")
            || lower.contains("change hotspots")
            || lower.contains("churn hotspots")
            || lower.contains("hot files")
        {
            return Some(":hotspots".to_string());
        }

        if lower.contains("semantic concepts")
            || lower.contains("semantic memory")
            || lower.contains("concepts summary")
        {
            return Some(":concepts".to_string());
        }

        if lower.contains("watches")
            || lower.contains("prospective alerts")
            || lower.contains("watch list")
        {
            return Some(":watches".to_string());
        }

        if lower.contains("who owns")
            || lower.contains("ownership report")
            || lower.contains("file owners")
        {
            if let Some((_, scope)) = lower.split_once("who owns") {
                let scope = scope
                    .trim()
                    .trim_matches(|character: char| character == '?' || character == '.')
                    .trim();
                if !scope.is_empty() {
                    return Some(format!(":owners {}", scope));
                }
            }
            return Some(":owners".to_string());
        }

        if lower.contains("coupling")
            || lower.contains("hidden dependency")
            || lower.contains("changes together")
        {
            return Some(":coupling".to_string());
        }

        if lower.starts_with("why ") && lower.contains('.') {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            for t in tokens {
                if t.contains('.') && (t.contains('/') || t.contains('\\')) {
                    return Some(format!(":why {}", t));
                }
            }
        }

        if lower.contains("index codebase")
            || lower.contains("index project")
            || lower.contains("reindex")
            || lower == "index"
            || lower.contains("scan the codebase")
            || lower.contains("scan this codebase")
            || lower.contains("analyze the project")
            || lower.contains("analyze this project")
        {
            return Some(":index".to_string());
        }

        if lower.contains("list workflows")
            || lower.contains("show workflows")
            || lower.contains("workflow list")
        {
            return Some(":workflow list".to_string());
        }

        if let Some(start) = lower.find("run workflow ") {
            let args = trimmed[start + "run workflow ".len()..].trim();
            if !args.is_empty() {
                return Some(format!(":workflow run {}", args));
            }
        }

        if lower.starts_with("generate workflow ") || lower.starts_with("create workflow ") {
            let description = trimmed
                .split_once(' ')
                .and_then(|(_, rest)| rest.split_once(' '))
                .map(|(_, rest)| rest.trim())
                .unwrap_or(trimmed)
                .trim();
            if !description.is_empty() {
                return Some(format!(":workflow generate {}", description));
            }
        }

        if lower.contains("install hook") || lower.contains("setup hook") || lower == "hook" {
            return Some(":hook".to_string());
        }

        if lower.contains("watch mode") || lower == "watch" {
            return Some(":watch".to_string());
        }

        if lower.contains("predict")
            && (lower.contains("refactor")
                || lower.contains("debt")
                || lower.contains("drift")
                || lower.contains("future problems")
                || lower.contains("future issues")
                || lower.contains("upcoming issues"))
        {
            return Some(":predict".to_string());
        }

        if (lower.contains("find the commit") || lower.contains("which commit"))
            && (lower.contains("introduced") || lower.contains("caused") || lower.contains("bug"))
        {
            return Some(format!(":bisect {}", trimmed));
        }

        if lower.contains("time travel debug") || lower.contains("time-travel debug") {
            return Some(format!(":bisect {}", trimmed));
        }

        if lower.contains("do multiple things")
            || lower.contains("do several things")
            || lower.contains("multi step")
            || lower.contains("multi-step")
            || lower.contains("plan these steps")
            || lower.contains("plan this out")
            || lower.contains("create a todo")
            || lower.contains("make a todo")
        {
            return Some(format!(":plan {}", trimmed));
        }

        if lower.contains("list migrations") || lower.contains("show migrations") {
            return Some(":migrations".to_string());
        }


        None
    }

    fn team_status_summary(&self) -> Result<String> {
        let team_mgr = TeamManager::new(&self.root);
        let state = team_mgr.load_state()?;
        if state.team_name.is_empty() {
            return Ok("No team is initialized for this repository.".to_string());
        }
        let user = load_global_config()
            .user
            .unwrap_or_else(|| std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "local_dev".to_string()));
        let normalize = |v: &str| v.trim().trim_start_matches('@').to_ascii_lowercase();
        let user_norm = normalize(&user);
        let is_member = state.members.keys().any(|name| normalize(name) == user_norm);
        let mut out = String::new();
        let _ = writeln!(&mut out, "👥 Team: {}", state.team_name);
        let _ = writeln!(&mut out, "👤 User: {} (member: {})", user, is_member);
        let my_open = state
            .tasks
            .values()
            .filter(|t| normalize(&t.assignee) == user_norm)
            .filter(|t| t.status != crate::teams::TaskStatus::Done)
            .collect::<Vec<_>>();
        let _ = writeln!(&mut out, "📌 Open tasks: {}", my_open.len());
        for task in my_open.iter().take(5) {
            let _ = writeln!(&mut out, "   - [{}] {}", task.id, task.description);
        }
        if let Some(active) = state
            .sessions
            .iter()
            .find(|s| normalize(&s.developer) == user_norm && s.end_time.is_none())
        {
            let _ = writeln!(&mut out, "⏱️ Active session: {}", active.task_id);
        }
        Ok(out)
    }

    fn team_context_summary(&self) -> Option<String> {
        let team_mgr = TeamManager::new(&self.root);
        let state = team_mgr.load_state().ok()?;
        if state.team_name.is_empty() {
            return Some("No team state is initialized for this repository yet.".to_string());
        }

        let user = load_global_config()
            .user
            .unwrap_or_else(|| std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "local_dev".to_string()));
        let normalize = |v: &str| v.trim().trim_start_matches('@').to_ascii_lowercase();
        let user_norm = normalize(&user);
        let is_member = state.members.keys().any(|name| normalize(name) == user_norm);
        let active_sessions = state.sessions.iter().filter(|session| session.end_time.is_none()).collect::<Vec<_>>();
        let pending = state.tasks.values().filter(|task| task.status == crate::teams::TaskStatus::Pending).count();
        let in_progress = state.tasks.values().filter(|task| task.status == crate::teams::TaskStatus::InProgress).count();
        let done = state.tasks.values().filter(|task| task.status == crate::teams::TaskStatus::Done).count();

        let mut out = String::new();
        let _ = writeln!(
            &mut out,
            "Team: {} | members={} | tasks: pending={}, in_progress={}, done={} | active_sessions={}",
            state.team_name,
            state.members.len(),
            pending,
            in_progress,
            done,
            active_sessions.len()
        );
        let _ = writeln!(&mut out, "Current user: {} | member={}", user, is_member);

        let mut members = state.members.values().collect::<Vec<_>>();
        members.sort_by(|a, b| a.name.cmp(&b.name));
        if !members.is_empty() {
            let preview = members
                .iter()
                .take(5)
                .map(|member| format!("{} ({:?})", member.name, member.role))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(&mut out, "Members: {}", preview);
        }

        let my_open = state
            .tasks
            .values()
            .filter(|task| normalize(&task.assignee) == user_norm)
            .filter(|task| task.status != crate::teams::TaskStatus::Done)
            .collect::<Vec<_>>();
        if !my_open.is_empty() {
            let _ = writeln!(&mut out, "My open tasks:");
            for task in my_open.iter().take(5) {
                let _ = writeln!(&mut out, "- [{}] {} ({:?})", task.id, task.description, task.status);
            }
        }

        if !active_sessions.is_empty() {
            let _ = writeln!(&mut out, "Active sessions:");
            for session in active_sessions.iter().take(5) {
                let _ = writeln!(
                    &mut out,
                    "- {} on {} (files_touched={}, prompts={})",
                    session.developer,
                    session.task_id,
                    session.files_touched.len(),
                    session.prompts_asked.len()
                );
            }
        }

        Some(out.trim_end().to_string())
    }

    fn architecture_context_summary(&self, question: &str) -> Option<String> {
        let stats = self.index.stats();
        if stats.file_count == 0 {
            return None;
        }

        let mut out = String::new();
        let _ = writeln!(
            &mut out,
            "Indexed files: {} | total lines: {}",
            stats.file_count,
            stats.total_lines
        );

        let by_lang = self.index.files_by_language();
        if !by_lang.is_empty() {
            let mut top = by_lang.into_iter().collect::<Vec<_>>();
            top.sort_by(|a, b| b.1.cmp(&a.1));
            let top_str = top
                .into_iter()
                .take(5)
                .map(|(lang, count)| format!("{}={}", lang, count))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(&mut out, "Top languages: {}", top_str);
        }

        if let Ok(entries) = std::fs::read_dir(&self.root) {
            let mut dirs = entries
                .flatten()
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        return None;
                    }
                    entry.file_type()
                        .ok()
                        .filter(|file_type| file_type.is_dir())
                        .map(|_| name)
                })
                .collect::<Vec<_>>();
            dirs.sort();
            if !dirs.is_empty() {
                let _ = writeln!(&mut out, "Top-level directories: {}", dirs.join(", "));
            }
        }

        let ranked_files = self.rank_relevant_files(question, 4);
        if !ranked_files.is_empty() {
            let _ = writeln!(&mut out, "Relevant files:");
            for path in &ranked_files {
                let path_str = path.to_string_lossy();
                if let Some(summary) = self.index.files().get(path) {
                    let _ = writeln!(
                        &mut out,
                        "- {} [{}] lines={} functions={}",
                        path_str,
                        summary.language,
                        summary.line_count,
                        summary.approx_fn_count
                    );
                } else {
                    let _ = writeln!(&mut out, "- {}", path_str);
                }
            }
        }

        if self.temporal_graph.enriched {
            let mut ownership_lines = Vec::new();
            for path in &ranked_files {
                let key = path.to_string_lossy().to_string();
                if let Some(history) = self.temporal_graph.file_histories.get(&key) {
                    ownership_lines.push(format!(
                        "{} -> owner={} commits={} stale={}d",
                        history.path,
                        history.primary_owner.as_deref().unwrap_or("unknown"),
                        history.total_changes,
                        history.staleness_days
                    ));
                }
            }

            if ownership_lines.is_empty() {
                let mut hotspots = self
                    .temporal_graph
                    .file_histories
                    .values()
                    .collect::<Vec<_>>();
                hotspots.sort_by(|a, b| b.total_changes.cmp(&a.total_changes));
                for history in hotspots.into_iter().take(3) {
                    ownership_lines.push(format!(
                        "{} -> owner={} commits={} stale={}d",
                        history.path,
                        history.primary_owner.as_deref().unwrap_or("unknown"),
                        history.total_changes,
                        history.staleness_days
                    ));
                }
            }

            if !ownership_lines.is_empty() {
                let _ = writeln!(&mut out, "Ownership / hotspots:");
                for line in ownership_lines {
                    let _ = writeln!(&mut out, "- {}", line);
                }
            }

            if (question.to_ascii_lowercase().contains("coupling")
                || question.to_ascii_lowercase().contains("architecture")
                || question.to_ascii_lowercase().contains("dependency")
                || question.to_ascii_lowercase().contains("dependencies"))
                && !self.temporal_graph.co_changes.is_empty()
            {
                let _ = writeln!(&mut out, "Notable co-change pairs:");
                for pair in self.temporal_graph.co_changes.iter().take(3) {
                    let _ = writeln!(
                        &mut out,
                        "- {} <-> {} (co_changes={}, coupling={:.2})",
                        pair.file_a,
                        pair.file_b,
                        pair.co_change_count,
                        pair.coupling_score
                    );
                }
            }
        }

        Some(out.trim_end().to_string())
    }

    fn render_onboarding(&self) -> String {
        let mut out = String::new();
        let stats = self.index.stats();
        let _ = writeln!(
            &mut out,
            "Onboarding guide for project at {:?}",
            self.root
        );
        let _ = writeln!(
            &mut out,
            "Files: {}  Lines: {}",
            stats.file_count,
            stats.total_lines
        );

        let mut coupling_score: HashMap<String, f32> = HashMap::new();
        for pair in &self.temporal_graph.co_changes {
            *coupling_score.entry(pair.file_a.clone()).or_insert(0.0) += pair.coupling_score;
            *coupling_score.entry(pair.file_b.clone()).or_insert(0.0) += pair.coupling_score;
        }

        let mut start_here = Vec::new();
        for history in self.temporal_graph.file_histories.values() {
            let base = history.total_changes as f32;
            let coupling = coupling_score
                .get(&history.path)
                .copied()
                .unwrap_or(0.0);
            let owner_boost = if history.authors.len() == 1 && history.total_changes >= 5 {
                15.0
            } else {
                0.0
            };
            let staleness_penalty = if history.staleness_days > 365 {
                10.0
            } else if history.staleness_days > 180 {
                5.0
            } else {
                0.0
            };
            let score = base + coupling * 10.0 + owner_boost - staleness_penalty;
            start_here.push((score, history));
        }
        start_here.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let _ = writeln!(&mut out, "\nStart here:");
        if start_here.is_empty() {
            let _ = writeln!(
                &mut out,
                "- No git history available yet. Begin with core modules and README files."
            );
        } else {
            for (_, history) in start_here.iter().take(3) {
                let owner = history
                    .primary_owner
                    .as_deref()
                    .unwrap_or("unknown");
                let _ = writeln!(
                    &mut out,
                    "- {} (owner: {}, commits: {}, stale: {} days)",
                    history.path,
                    owner,
                    history.total_changes,
                    history.staleness_days
                );
            }
        }

        let mut avoid_paths = Vec::new();
        for path in self.index.files().keys() {
            let s = path.to_string_lossy().to_lowercase();
            if s.contains("legacy")
                || s.contains("experimental")
                || s.contains("playground")
                || s.contains("scratch")
                || s.contains("tmp")
            {
                avoid_paths.push(path.clone());
            }
        }
        let _ = writeln!(&mut out, "\nAvoid for now:");
        if avoid_paths.is_empty() {
            let _ = writeln!(
                &mut out,
                "- No obvious legacy or experimental folders detected."
            );
        } else {
            for path in avoid_paths.iter().take(5) {
                let _ = writeln!(&mut out, "- {}", path.display());
            }
        }

        let mut devs: Vec<_> = self.temporal_graph.developers.values().collect();
        devs.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        let _ = writeln!(&mut out, "\nPeople to talk to:");
        if devs.is_empty() {
            let _ = writeln!(
                &mut out,
                "- No git authors detected. Configure git user and make a few commits."
            );
        } else {
            for dev in devs.iter().take(3) {
                let _ = writeln!(
                    &mut out,
                    "- {} ({} commits)",
                    dev.name,
                    dev.commit_count
                );
            }
        }

        let mut surprises = Vec::new();
        for pattern in &self.concept_store.patterns {
            surprises.push(format!(
                "Pattern: {} (confidence: {:.0}%)",
                pattern.description,
                pattern.confidence * 100.0
            ));
        }
        for watch in &self.concept_store.watches {
            surprises.push(format!(
                "Watch: {} [priority: {:?}]",
                watch.description,
                watch.priority
            ));
        }
        if let Some(pair) = self.temporal_graph.co_changes.first() {
            surprises.push(format!(
                "Hidden coupling: {} ↔ {} ({} changes, {:.0}% coupling)",
                pair.file_a,
                pair.file_b,
                pair.co_change_count,
                pair.coupling_score * 100.0
            ));
        }

        let _ = writeln!(&mut out, "\nThings that will surprise you:");
        if surprises.is_empty() {
            let _ = writeln!(
                &mut out,
                "- Run :index to build history and patterns, then ask again."
            );
        } else {
            for s in surprises.iter().take(5) {
                let _ = writeln!(&mut out, "- {}", s);
            }
        }

        if let Some(last_health) = self
            .memory
            .latest_event("health")
            .and_then(|entry| entry.event.as_ref())
        {
            if let MemoryEvent::HealthSnapshot { scores } = last_health {
                let _ = writeln!(&mut out, "\nLatest health snapshot:");
                let _ = writeln!(
                    &mut out,
                    "- Code quality: {}  Tests: {}  Drift: {}  Security: {}  Git: {}  Team: {}",
                    scores.code_quality,
                    scores.test_health,
                    scores.cross_lang_drift,
                    scores.security_surface,
                    scores.git_health,
                    scores.team_velocity
                );
            }
        }

        out
    }

    fn handle_edit(&mut self, instruction: &str) -> Result<String> {
        let parts: Vec<&str> = instruction.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return Ok("Usage: :edit <filename> <instructions>".to_string());
        }
        let target_file = self.root.join(parts[0]);
        let prompt_instruction = parts[1];

        if !target_file.exists() {
            return Ok(format!("❌ File not found: {}", target_file.display()));
        }

        let content = std::fs::read_to_string(&target_file)?;

        if let Some(model) = &self.model {
            println!(" ⚙️ Preparing to autonomous edit target file...");
            let mut sys_prompt = String::new();
            let _ = writeln!(&mut sys_prompt, "You are Astra, an autonomous coding agent.");
            let _ = writeln!(&mut sys_prompt, "Your task is to completely rewrite the provided file based on the user's instructions.");
            let _ = writeln!(&mut sys_prompt, "You must output ONLY the final raw code for the file. NO MARKDOWN formatting, NO backticks, NO explanations, NO greetings.");
            let _ = writeln!(&mut sys_prompt, "The exact output you generate will be written directly to the file.");

            let prompt = format!(
                "{}\n\nINSTRUCTIONS:\n{}\n\nFILE CONTENT:\n{}",
                sys_prompt, prompt_instruction, content
            );

            println!(" ⚙️ Waiting for LLM to rewrite {}...", parts[0]);
            let mut rewritten = model.complete(&prompt)?;

            // Strip markdown backticks if the model ignores the prompt
            if rewritten.starts_with("```") {
                if let Some(end) = rewritten.find('\n') {
                    rewritten = rewritten[end + 1..].to_string();
                }
                if rewritten.ends_with("```") {
                    rewritten = rewritten[..rewritten.len() - 3].to_string();
                }
            }

            std::fs::write(&target_file, &rewritten)?;

            if let Some(git) = &self.git {
                let git_root = git.root_path();
                let _ = std::process::Command::new("git")
                    .current_dir(git_root)
                    .args(&["add", &target_file.to_string_lossy()])
                    .output();
                let _ = std::process::Command::new("git")
                    .current_dir(git_root)
                    .args(&["commit", "-m", &format!("(astra): auto-edited {}", parts[0])])
                    .output();
            }

            Ok(format!("✅ Successfully rewritten {} automatically.", parts[0]))
        } else {
            Ok("No language model configured.".to_string())
        }
    }

    fn parse_natural_migrate_request(&self, input: &str) -> Option<String> {
        let normalized = input
            .replace("->", " to ")
            .replace("=>", " to ")
            .replace("→", " to ");
        let tokens: Vec<String> = normalized
            .split_whitespace()
            .map(Self::clean_intent_token)
            .filter(|t| !t.is_empty())
            .collect();
        if tokens.len() < 4 {
            return None;
        }
        let action_idx = tokens.iter().position(|t| Self::is_migration_action(t))?;
        let from_idx = tokens.iter().position(|t| t == "from");
        let to_idx = tokens.iter().position(|t| t == "to");
        let output_idx = tokens.iter().position(|t| Self::is_output_keyword(t));

        let from_lang = tokens
            .iter()
            .position(|t| t == "from")
            .and_then(|i| tokens.get(i + 1))
            .and_then(|v| Language::from_str_loose(v))
            .or_else(|| {
                let langs: Vec<(usize, Language)> = tokens
                    .iter()
                    .enumerate()
                    .filter_map(|(i, t)| Language::from_str_loose(t).map(|l| (i, l)))
                    .collect();
                langs.first().map(|(_, l)| *l)
            })?;
        let to_lang = tokens
            .iter()
            .position(|t| t == "to")
            .and_then(|i| tokens.get(i + 1))
            .and_then(|v| Language::from_str_loose(v))
            .or_else(|| {
                let langs: Vec<(usize, Language)> = tokens
                    .iter()
                    .enumerate()
                    .filter_map(|(i, t)| Language::from_str_loose(t).map(|l| (i, l)))
                    .collect();
                langs.get(1).map(|(_, l)| *l)
            })?;

        let source = tokens
            .iter()
            .position(|t| Self::is_source_keyword(t))
            .and_then(|i| tokens.get(i + 1).cloned())
            .or_else(|| {
                let upper_bound = from_idx
                    .or(to_idx)
                    .or(output_idx)
                    .unwrap_or(tokens.len());
                if action_idx + 1 >= upper_bound {
                    return None;
                }
                let span = tokens[action_idx + 1..upper_bound]
                    .iter()
                    .filter(|t| !Self::is_noise_token(t) && !Self::is_output_keyword(t))
                    .cloned()
                    .collect::<Vec<_>>();
                if span.is_empty() {
                    None
                } else {
                    Some(span.join("_"))
                }
            })?;

        let output = tokens
            .iter()
            .position(|t| Self::is_output_keyword(t))
            .and_then(|i| tokens.get(i + 1))?
            .clone();

        let lower = normalized.to_ascii_lowercase();
        let use_ai = tokens.iter().any(|t| t == "--ai" || t == "ai")
            || lower.contains(" with ai")
            || lower.contains(" using ai");
        let use_clean = tokens.iter().any(|t| t == "--clean");

        let mut cmd = format!(
            ":migrate {} {} {} {}",
            source,
            Self::language_cli_token(from_lang),
            Self::language_cli_token(to_lang),
            output
        );
        if use_ai {
            cmd.push_str(" --ai");
        }
        if use_clean {
            cmd.push_str(" --clean");
        }
        Some(cmd)
    }

    fn clean_intent_token(token: &str) -> String {
        token
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '\\' && c != '_' && c != '-' && c != '.')
            .to_ascii_lowercase()
    }

    fn is_migration_action(token: &str) -> bool {
        matches!(
            token,
            "migrate" | "convert" | "translate" | "port" | "rewrite" | "move"
        )
    }

    fn is_source_keyword(token: &str) -> bool {
        matches!(token, "source" | "src" | "path" | "input")
    }

    fn is_output_keyword(token: &str) -> bool {
        matches!(token, "output" | "out" | "into" | "destination" | "dest" | "in")
    }

    fn is_noise_token(token: &str) -> bool {
        matches!(token, "the" | "this" | "that" | "a" | "an" | "my")
    }

    fn language_cli_token(lang: Language) -> &'static str {
        match lang {
            Language::TypeScript => "ts",
            Language::JavaScript => "js",
            Language::Python => "py",
            Language::Go => "go",
            Language::Rust => "rs",
            Language::Java => "java",
            Language::React => "react",
            Language::NextJs => "nextjs",
            Language::Vue => "vue",
            Language::Svelte => "svelte",
            Language::Cpp => "cpp",
            Language::Assembly => "asm",
        }
    }

    fn record_worktree_snapshot(&mut self) {
        let git = match &self.git {
            Some(g) => g,
            None => return,
        };
        let files = git.changed_files();
        let changed_files = files.len();
        let last = self
            .memory
            .latest_event("worktree")
            .and_then(|entry| match entry.event {
                Some(MemoryEvent::WorktreeSnapshot { changed_files, .. }) => Some(changed_files),
                _ => None,
            });
        if last == Some(changed_files) {
            return;
        }
        self.memory.add_event(
            "worktree",
            format!("uncommitted files: {}", changed_files),
            MemoryEvent::WorktreeSnapshot {
                changed_files,
                files,
            },
        );
    }

    fn refresh_git_memory(&mut self) {
        let Some(git) = &self.git else {
            return;
        };
        if self.temporal_graph.enriched {
            self.temporal_graph.enrich_incremental(git);
        } else {
            self.temporal_graph.enrich_from_git(git);
        }
        let global_brain = crate::config::get_global_brain_path(&self.root);
        let _ = self
            .temporal_graph
            .save(&global_brain.join("temporal_graph.json"));
    }

    fn record_git_commit(&mut self) {
        let git = match &self.git {
            Some(g) => g,
            None => return,
        };
        let head = match git.get_head_commit() {
            Ok(h) => h,
            Err(_) => return,
        };
        let last = self
            .memory
            .latest_event("git-commit")
            .and_then(|entry| match &entry.event {
                Some(MemoryEvent::GitCommit { id, .. }) => Some(id.as_str()),
                _ => None,
            });
        if last == Some(head.as_str()) {
            return;
        }
        if let Ok(info) = git.last_commit_info() {
            let content = format!("{} by {} — {}", info.id, info.author, info.summary);
            self.memory.add_event(
                "git-commit",
                content,
                MemoryEvent::GitCommit {
                    id: info.id.clone(),
                    summary: info.summary.clone(),
                    author: info.author.clone(),
                    date: info.date.clone(),
                },
            );
        }
    }

    // --- Phase 2: Orchestrator Delegation ---
    pub fn get_next_task(&mut self, goal: Option<&str>) -> Result<String> {
        crate::orchestrator::generate_next_task(self, goal)
    }

    fn execute_task(&mut self, goal: &str) -> Result<String> {
        if goal.trim().is_empty() {
            // No goal — show existing plan or ask for one
            if let Some(plan) = crate::planner::Plan::load(&self.root) {
                if !plan.is_complete() {
                    return self.resume_plan(plan);
                }
            }
            return self.get_next_task(None);
        }

        let model_available = self.model.is_some();
        if !model_available {
            return self.get_next_task(Some(goal));
        }

        // Check if there's already an active plan for this goal
        if let Some(existing) = crate::planner::Plan::load(&self.root) {
            if existing.goal.trim().eq_ignore_ascii_case(goal.trim()) && !existing.is_complete() {
                println!("\n📋 Resuming existing plan for: {}", goal);
                return self.resume_plan(existing);
            }
        }

        // Fresh start: decompose goal → plan → execute
        self.run_planned_task(goal)
    }

    /// Decompose a goal into a plan and execute it subtask by subtask.
    fn run_planned_task(&mut self, goal: &str) -> Result<String> {
        use crate::planner::{Planner, build_context_snapshot};

        let model = match &self.model {
            Some(m) => m.as_ref() as *const dyn CodexModel,
            None => return Ok("No language model configured. Cannot plan.".to_string()),
        };
        // SAFETY: we hold &mut self and only use model for LLM calls; no aliasing.
        let model = unsafe { &*model };

        // Build context snapshot for the planner
        let stats = self.index.stats();
        let by_lang: Vec<(String, usize)> = {
            let mut v: Vec<_> = self.index.files_by_language().into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            v
        };
        let memory_facts: Vec<String> = self.memory.recent(6)
            .iter()
            .map(|e| format!("[{}] {}", e.kind, truncate_for_context(&e.content, 120)))
            .collect();
        let context = build_context_snapshot(
            &self.root,
            stats.file_count,
            stats.total_lines,
            &by_lang,
            &memory_facts,
        );

        println!("\n🧠 Astra is decomposing your goal into a plan...");
        println!("   (drafting → self-critiquing → refining)\n");
        let planner = Planner::new(model, &self.root);

        let plan = match planner.decompose_deep(goal, &context) {
            Ok(p) => p,
            Err(e) => {
                // Decomposition failed — fall back to single-shot agent
                eprintln!("⚠️  Planning failed ({}), falling back to direct execution.", e);
                return self.run_agent_for_goal(goal);
            }
        };

        plan.save(&self.root)?;
        println!("{}", plan.render_dashboard());

        self.resume_plan(plan)
    }

    /// Execute pending subtasks one by one, reflecting after each.
    fn resume_plan(&mut self, mut plan: crate::planner::Plan) -> Result<String> {
        use crate::planner::{Planner, SubtaskStatus};

        let model_ptr = match &self.model {
            Some(m) => m.as_ref() as *const dyn CodexModel,
            None => return Ok("No language model configured.".to_string()),
        };
        let model = unsafe { &*model_ptr };
        let planner = Planner::new(model, &self.root);

        let mut output = String::new();

        loop {
            // Find the next pending task
            let task_idx = match plan.subtasks.iter().position(|t| t.status == SubtaskStatus::Pending) {
                Some(i) => i,
                None => break,
            };

            {
                let task = &mut plan.subtasks[task_idx];
                task.status = SubtaskStatus::InProgress;
                task.started_at = Some(now_secs());
            }
            plan.save(&self.root)?;

            let task = plan.subtasks[task_idx].clone();
            println!(
                "\n⚙️  [{}/{}] Executing: {}\n    {}\n",
                task.id, plan.total(), task.title, task.description
            );

            // Build a focused goal for the agent
            let subtask_goal = format!(
                "## OVERALL GOAL\n{}\n\n## YOUR CURRENT SUBTASK (#{} of {})\n{}\n\n## DESCRIPTION\n{}\n\n## ACCEPTANCE CRITERION\n{}\n\n## ALREADY DONE\n{}",
                plan.goal,
                task.id,
                plan.total(),
                task.title,
                task.description,
                task.acceptance,
                plan.subtasks[..task_idx]
                    .iter()
                    .filter(|t| t.status == SubtaskStatus::Done)
                    .map(|t| format!("✅ {}: {}", t.title, t.result_summary.as_deref().unwrap_or("done")))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let execution_log = match self.run_agent_for_goal(&subtask_goal) {
                Ok(log) => log,
                Err(e) => format!("AGENT ERROR: {}", e),
            };

            // Reflect on what happened
            println!("\n🪞 Reflecting on subtask outcome...\n");
            let reflection = planner.reflect_on_subtask(&plan, task_idx, &execution_log);

            match reflection {
                Ok(r) => {
                    // Collect data needed after mutable borrow ends
                    let (task_id, task_title) = {
                        let task = &mut plan.subtasks[task_idx];
                        task.finished_at = Some(now_secs());
                        task.result_summary = Some(r.result_summary.clone());
                        task.files_touched = r.files_touched.clone();

                        match &r.verdict {
                            crate::planner::ReflectionVerdict::Done => {
                                task.status = SubtaskStatus::Done;
                                println!("✅ Subtask done: {}", r.result_summary);
                            }
                            crate::planner::ReflectionVerdict::Retry => {
                                task.status = SubtaskStatus::Pending;
                                println!("🔄 Retrying: {}", r.reason);
                            }
                            crate::planner::ReflectionVerdict::Skip => {
                                task.status = SubtaskStatus::Skipped(r.reason.clone());
                                println!("⏭️  Skipped: {}", r.reason);
                            }
                            crate::planner::ReflectionVerdict::Blocked => {
                                task.status = SubtaskStatus::Blocked(r.blocker.clone());
                                println!("🚧 Blocked: {}", r.blocker);
                            }
                        }
                        (task.id, task.title.clone())
                    };
                    // mutable borrow of plan.subtasks[task_idx] ends here

                    if r.verdict == crate::planner::ReflectionVerdict::Blocked {
                        output.push_str(&format!("\n🚧 **Blocked on subtask #{}: {}**\n   {}\n", task_id, task_title, r.blocker));
                        plan.save(&self.root)?;
                        break;
                    }

                    // Apply replan if needed
                    if r.replan_needed && !r.replan_additions.is_empty() {
                        println!("📋 Replanning: adding {} new subtask(s).", r.replan_additions.len());
                        planner.apply_replan(&mut plan, task_idx, r.replan_additions);
                    }

                    output.push_str(&format!("[{}] {}: {}\n", task_id, task_title, r.result_summary));
                }
                Err(e) => {
                    // Reflection failed — mark done and continue
                    let (task_id, task_title) = {
                        let task = &mut plan.subtasks[task_idx];
                        task.status = SubtaskStatus::Done;
                        task.result_summary = Some(format!("Completed (reflection failed: {})", e));
                        task.finished_at = Some(now_secs());
                        (task.id, task.title.clone())
                    };
                    output.push_str(&format!("[{}] {}: done (reflection failed)\n", task_id, task_title));
                }
            }

            plan.touch();
            plan.save(&self.root)?;
            println!("{}", plan.render_dashboard());
        }

        if plan.is_complete() {
            plan.phase = crate::planner::PlanPhase::Done;
            plan.save(&self.root)?;
            self.memory.add(
                "task-execution",
                format!("GOAL: {}\nCOMPLETE. Subtasks: {}", plan.goal, plan.total()),
            );
            Ok(format!(
                "{}\n\n🎉 **Goal complete:** {}\n\n{}",
                plan.render_dashboard(),
                plan.goal,
                output
            ))
        } else {
            Ok(format!(
                "{}\n\n⏸️  Plan paused. Run `:task` to continue.\n\n{}",
                plan.render_dashboard(),
                output
            ))
        }
    }

    /// Direct single-shot agent execution (no planning layer).
    fn run_agent_for_goal(&mut self, goal: &str) -> Result<String> {
        let model = match &self.model {
            Some(m) => m,
            None => return Ok("No model configured.".to_string()),
        };
        let config = AgentConfig {
            auto_approve: self.auto_approve,
            max_iterations: 24,
        };
        let system_context = self.build_grounding_context();
        let result = agent::run_agent_loop(
            model.as_ref(),
            goal,
            &self.root,
            &config,
            &system_context,
            self.search.as_deref().map(|s| s as &dyn SearchProvider),
        )?;
        self.memory.add("task-execution", format!("TASK: {}\nRESULT: {}", goal, result));
        Ok(result)
    }

    fn execute_plan(&mut self, instruction: &str) -> Result<String> {
        let model = match &self.model {
            Some(m) => m,
            None => {
                return Ok("No language model configured. Cannot run :plan.".to_string());
            }
        };

        let prompt = format!(
            "You are Astra's CLI planner. Convert the user's instruction into 2-6 concrete CLI steps.\n\
            Allowed commands:\n\
            - :web <query>\n\
            - :index\n\
            - :summary\n\
            - :health\n\
            - :predict\n\
            - :onboard\n\
            - :analyze\n\
            - :task <goal>\n\
            Rules:\n\
            - Always choose commands that make sense given the instruction.\n\
            - Use :web for any research or web search (phrases like 'search the web', 'web search', 'google').\n\
            - Use :index if the user talks about scanning or analyzing the codebase.\n\
            - Use :health if the user wants a health check or overall codebase health.\n\
            - Use :predict if the user mentions refactoring debt, drift, or future problems.\n\
            - Use :onboard when the user asks where to start or how to understand the codebase.\n\
            - Use :analyze when the user asks about risk, hotspots, or semantic memory.\n\
            - Use :task when the user wants Astra to autonomously work towards a goal.\n\
            - Do NOT generate :plan or any shell commands.\n\
            - Format each step on its own line exactly as: STEP|<command>|<short description>\n\
            - <command> must start with ':' and contain no extra text.\n\
            Example mappings:\n\
            - 'search the web for latest rust async patterns' -> STEP|:web latest rust async patterns|Research async patterns\n\
            - 'scan the codebase and run a health check' -> STEP|:index|Index codebase\n\
              STEP|:health|Run health dashboard\n\
            User instruction:\n{}",
            instruction
        );

        let response = model.complete(&prompt)?;
        let mut steps: Vec<(String, String)> = Vec::new();

        for line in response.lines() {
            let line = line.trim();
            if !line.starts_with("STEP|") {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() < 3 {
                continue;
            }
            let command = parts[1].trim().to_string();
            let description = parts[2].trim().to_string();
            if command.is_empty() || !command.starts_with(':') {
                continue;
            }
            if command.starts_with(":plan") {
                continue;
            }
            steps.push((command, description));
        }

        let lower_instruction = instruction.to_ascii_lowercase();

        let has_web_step = steps.iter().any(|(cmd, _)| cmd.starts_with(":web "));
        if (lower_instruction.contains("search the web for")
            || lower_instruction.starts_with("search web for ")
            || lower_instruction.starts_with("web search ")
            || lower_instruction.starts_with("google "))
            && !has_web_step
        {
            let patterns = [
                "search the web for ",
                "search web for ",
                "web search ",
                "google ",
            ];
            for pat in &patterns {
                if let Some(pos) = lower_instruction.find(pat) {
                    let start = pos + pat.len();
                    if start <= instruction.len() {
                        let query = instruction[start..].trim();
                        if !query.is_empty() {
                            steps.insert(
                                0,
                                (format!(":web {}", query), format!("Web search: {}", query)),
                            );
                        }
                    }
                    break;
                }
            }
        }

        let has_index_step = steps.iter().any(|(cmd, _)| cmd == ":index");
        if (lower_instruction.contains("index codebase")
            || lower_instruction.contains("index project")
            || lower_instruction.contains("reindex")
            || lower_instruction.contains("scan the codebase")
            || lower_instruction.contains("scan this codebase")
            || lower_instruction.contains("analyze the project")
            || lower_instruction.contains("analyze this project"))
            && !has_index_step
        {
            steps.push((":index".to_string(), "Index the codebase".to_string()));
        }

        let has_health_step = steps.iter().any(|(cmd, _)| cmd == ":health");
        if (lower_instruction.contains("health check")
            || lower_instruction.contains("codebase health")
            || lower_instruction.contains("health of the project")
            || lower_instruction.contains("project health"))
            && !has_health_step
        {
            steps.push((
                ":health".to_string(),
                "Run codebase health dashboard".to_string(),
            ));
        }

        let has_predict_step = steps.iter().any(|(cmd, _)| cmd == ":predict");
        if (lower_instruction.contains("predict")
            && (lower_instruction.contains("refactor")
                || lower_instruction.contains("debt")
                || lower_instruction.contains("drift")
                || lower_instruction.contains("future problems")
                || lower_instruction.contains("future issues")
                || lower_instruction.contains("upcoming issues")))
            && !has_predict_step
        {
            steps.push((
                ":predict".to_string(),
                "Run predictive refactoring analysis".to_string(),
            ));
        }

        if steps.is_empty() {
            return Ok("Planner could not extract any executable steps from that instruction.".to_string());
        }

        let mut out = String::new();
        let total = steps.len();
        let _ = writeln!(&mut out, "Plan overview ({} steps):", total);
        let _ = writeln!(&mut out, "----------------------------------------");

        for (i, (cmd, desc)) in steps.iter().enumerate() {
            let step_no = i + 1;
            let _ = writeln!(&mut out, "Step {}/{}:", step_no, total);
            let _ = writeln!(&mut out, "  Description: {}", desc);
            let _ = writeln!(&mut out, "  Command    : {}", cmd);
            let result = self.handle_input(cmd)?;
            let snippet: String = if result.len() > 400 {
                result.chars().take(400).collect()
            } else {
                result
            };
            let one_line = snippet.replace('\n', " ");
            let _ = writeln!(&mut out, "  Result     : {}", one_line);
            if step_no < total {
                let _ = writeln!(&mut out, "----------------------------------------");
            }
        }

        Ok(out)
    }

    pub fn report_task_result(&mut self, task_id: &str, success: bool, details: &str) -> Result<String> {
        crate::orchestrator::process_task_result(self, task_id, success, details)
    }

    /// Builds a comprehensive grounding context for external AI integrations (like MCP)
    /// to ensure the agent doesn't hallucinate facts about the user or project.
    /// Install a git pre-commit hook that runs Astra's review gate and
    /// blocks the commit if critical issues are found.
    fn install_review_gate(&self) -> String {
        let hooks_dir = self.root.join(".git").join("hooks");
        if !hooks_dir.exists() {
            return "❌ No .git/hooks directory found. Is this a git repository?".to_string();
        }
        let hook_path = hooks_dir.join("pre-commit");

        // Cross-platform-ish shell hook. Runs `astra :review` and blocks on ⛔ BLOCK.
        let hook = "#!/bin/sh\n\
# Astra ship-gate — blocks commits with critical issues.\n\
echo \"🔍 Astra is reviewing your changes before commit...\"\n\
REVIEW_OUTPUT=$(astra \":review\" 2>/dev/null)\n\
echo \"$REVIEW_OUTPUT\"\n\
if echo \"$REVIEW_OUTPUT\" | grep -q \"⛔ BLOCK\"; then\n\
  echo \"\"\n\
  echo \"⛔ Commit blocked by Astra: critical issues found. Fix them or run 'git commit --no-verify' to bypass.\"\n\
  exit 1\n\
fi\n\
exit 0\n";

        if let Err(e) = std::fs::write(&hook_path, hook) {
            return format!("❌ Failed to write pre-commit hook: {}", e);
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&hook_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&hook_path, perms);
            }
        }

        format!(
            "✅ Ship-gate installed at {}\n\nFrom now on, every `git commit` runs Astra's review and blocks if critical issues are found.\nBypass once with: git commit --no-verify",
            hook_path.display()
        )
    }

    /// Compact project snapshot used by the PM and planner brains.
    fn pm_context_snapshot(&self) -> String {
        let stats = self.index.stats();
        let by_lang: Vec<(String, usize)> = {
            let mut v: Vec<_> = self.index.files_by_language().into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            v
        };
        let memory_facts: Vec<String> = self
            .memory
            .recent(6)
            .iter()
            .map(|e| format!("[{}] {}", e.kind, truncate_for_context(&e.content, 120)))
            .collect();
        crate::planner::build_context_snapshot(
            &self.root,
            stats.file_count,
            stats.total_lines,
            &by_lang,
            &memory_facts,
        )
    }

    pub fn build_grounding_context(&self) -> String {
        let mut context = String::from("### GROUNDING CONTEXT (TREAT AS FACT)\n");

        let cursorrules_path = self.root.join(".cursorrules");
        if let Ok(contents) = fs::read_to_string(&cursorrules_path) {
            let _ = writeln!(&mut context, "User/Team Context from .cursorrules:\n---\n{}---", contents.trim());
        } else {
            let user_name = self.memory.user_name().unwrap_or_else(|| "the user".to_string());
            let _ = writeln!(&mut context, "User Identity: The user is {}", user_name);
        }

        let readme_path = self.root.join("README.md");
        if let Ok(contents) = fs::read_to_string(&readme_path) {
            let truncated = truncate_for_context(&contents, 500);
            let _ = writeln!(&mut context, "\nProject Overview from README.md:\n---\n{}---", truncated.trim());
        }

        let stats = self.index.stats();
        let _ = writeln!(&mut context, "\nCodebase Stats: {} files indexed, {} total lines.", stats.file_count, stats.total_lines);

        if let Some(architecture) = self.architecture_context_summary("codebase architecture overview") {
            let _ = writeln!(&mut context, "\nArchitecture Snapshot:\n{}", architecture);
        }

        if self.temporal_graph.enriched {
            let history = self.temporal_graph.project_history_report();
            let _ = writeln!(
                &mut context,
                "\nProject History Snapshot:\n{}",
                history.chars().take(1600).collect::<String>()
            );
        }

        if let Some(team_context) = self.team_context_summary() {
            let _ = writeln!(&mut context, "\nTeam Snapshot:\n{}", team_context);
        }

        context
    }

    /// Token-bounded context intended for external MCP workers. Editors already
    /// have filesystem access, so Astra sends paths, state, and durable decisions
    /// rather than copying large source files or chat transcripts.
    pub fn build_cowork_context(&self, query: &str, max_chars: usize) -> String {
        let max_chars = max_chars.clamp(800, 8_000);
        let stats = self.index.stats();
        let mut context = String::from("ASTRA PROJECT CONTEXT\n");
        let _ = writeln!(&mut context, "Root: {}", self.root.display());
        let _ = writeln!(
            &mut context,
            "Index: {} files, {} lines",
            stats.file_count, stats.total_lines
        );

        let relevant_files = self.rank_relevant_files(query, 8);
        if !relevant_files.is_empty() {
            let _ = writeln!(&mut context, "Likely relevant files:");
            for path in relevant_files {
                let _ = writeln!(&mut context, "- {}", path.display());
            }
        }

        if let Some(git) = &self.git {
            let changed = git.changed_files();
            if !changed.is_empty() {
                let _ = writeln!(&mut context, "Uncommitted files (preserve unrelated changes):");
                for path in changed.into_iter().take(20) {
                    let _ = writeln!(&mut context, "- {}", path);
                }
            }
        }

        let memories = self.memory.context(query, None, 4, 1_000);
        if !memories.is_empty() {
            let _ = writeln!(&mut context, "Relevant durable memory:");
            for memory in memories {
                let _ = writeln!(&mut context, "- [{}] {}", memory.kind, memory.content);
            }
        }

        let issues = crate::issues::IssueStore::new(&self.root)
            .list(6)
            .unwrap_or_default();
        if !issues.is_empty() {
            let _ = writeln!(&mut context, "Tracked issues (use astra_issue_status for full evidence):");
            for issue in issues.iter().take(4) {
                let _ = writeln!(&mut context, "- {}", issue.compact_summary());
            }
        }

        let jobs = crate::coworker::CoworkStore::new(&self.root)
            .list(8)
            .unwrap_or_default();
        let active = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    crate::coworker::CoworkJobStatus::Queued
                        | crate::coworker::CoworkJobStatus::Claimed
                        | crate::coworker::CoworkJobStatus::Blocked
                )
            })
            .collect::<Vec<_>>();
        if !active.is_empty() {
            let _ = writeln!(&mut context, "Active cowork jobs:");
            for job in active.into_iter().take(5) {
                let _ = writeln!(
                    &mut context,
                    "- {} [{:?}] {}",
                    job.id, job.status, job.goal
                );
            }
        }

        truncate_for_context(&context, max_chars)
    }

    fn find_mentioned_top_level_path(&self, input: &str) -> Option<String> {
        let lower = input.trim().to_ascii_lowercase();
        let tokens = lower
            .split_whitespace()
            .map(|token| {
                token.trim_matches(|character: char| {
                    !character.is_alphanumeric() && character != '-' && character != '_'
                })
            })
            .collect::<Vec<_>>();
        let mut directories = fs::read_dir(&self.root)
            .ok()?
            .flatten()
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()))
            .collect::<Vec<_>>();
        directories.sort();

        for name in &directories {
            let name_lower = name.to_ascii_lowercase();
            if tokens.iter().any(|token| **token == name_lower) {
                return Some(name.clone());
            }
        }
        directories.sort_by_key(|name| std::cmp::Reverse(name.len()));
        for name in directories {
            let alias = name.to_ascii_lowercase().replace(['-', '_'], " ");
            if alias.split_whitespace().count() > 1 && lower.contains(&alias) {
                return Some(name);
            }
        }
        None
    }

    /// Automatically extract personal facts from casual user messages and store them globally.
    fn auto_extract_facts(&mut self, input: &str) {
        let lower = input.trim().to_ascii_lowercase();
        
        // Don't extract from questions
        if lower.ends_with('?') || lower.len() < 8 {
            return;
        }

        if let Some(style) = extract_value_after_any(input, &["i want astra to ", "astra should ", "make astra "]) {
            if style.len() >= 3 {
                self.memory.remember_style_fact("assistant_style", &style);
            }
        }
        if !lower.ends_with('?') {
            let trimmed = input.trim();
            if trimmed.len() >= 8 {
                if lower.starts_with("it should")
                    || lower.starts_with("it needs")
                    || lower.starts_with("it must")
                    || lower.contains("should be")
                    || lower.contains("needs to")
                {
                    let style = if trimmed.len() > 200 {
                        let mut t = trimmed.chars().take(200).collect::<String>();
                        t.push_str("... [TRUNCATED]");
                        t
                    } else {
                        trimmed.to_string()
                    };
                    self.memory.remember_style_fact("assistant_style", &style);
                }
            }
        }

        // Patterns that indicate a personal fact — we search for these ANYWHERE in the input
        let identity_patterns = [
            "my name is ",
            "i'm called ",
            "call me ",
            "i am ",
            "i'm ",
            "my age is ",
            "i like ",
            "i love ",
            "i prefer ",
            "i use ",
            "i live in ",
            "i work at ",
            "i work as ",
            "my favorite ",
            "my favourite ",
            "i speak ",
            "i'm from ",
            "i am from ",
            "turning ",
            "years old",
            "my birthday ",
        ];

        let mut extracted: Vec<String> = Vec::new();

        if let Some(value) = extract_value_after_any(input, &["my name is ", "i'm called ", "call me "]) {
            self.memory.remember_user_identity("name", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i live in ", "i'm from ", "i am from "]) {
            self.memory.remember_user_identity("location", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i work at "]) {
            self.memory.remember_user_identity("company", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i work as "]) {
            self.memory.remember_user_identity("role", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i speak "]) {
            self.memory.remember_user_preference("language", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i prefer "]) {
            self.memory.remember_user_preference("general", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i use "]) {
            self.memory.remember_user_preference("tooling", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["i like ", "i love "]) {
            self.memory.remember_user_preference("likes", &value);
        }
        if let Some(value) = extract_value_after_any(input, &["my favorite ", "my favourite "]) {
            self.memory.remember_user_preference("favorite", &value);
        }

        for pattern in &identity_patterns {
            if let Some(pos) = lower.find(pattern) {
                // Extract a reasonable chunk: from pattern start to end of sentence or 80 chars
                let start = pos;
                let remaining = &input.trim()[start..];
                // Find end: next period, comma that ends a clause, "and" boundary, or end
                let end = remaining.len().min(80);
                let fact = remaining[..end].trim().to_string();
                
                if fact.len() >= 6 {
                    // Check for duplicates
                    let fact_lower = fact.to_ascii_lowercase();
                    let q_vec = self.get_query_embedding(&fact);
                    let existing = self.memory.search(&fact, q_vec.as_deref(), 3);
                    let already_known = existing.iter().any(|e| {
                        e.kind == "fact" && e.content.to_ascii_lowercase().contains(&fact_lower)
                    });
                    if !already_known && !extracted.iter().any(|e| e.to_ascii_lowercase() == fact_lower) {
                        extracted.push(fact);
                    }
                }
            }
        }

        // Store all extracted facts globally
        for fact in extracted {
            self.memory.add_global("fact", fact);
        }
    }

    fn remember_project_snapshot(&mut self) {
        let stats = self.index.stats();
        if stats.file_count == 0 {
            return;
        }
        self.memory
            .remember_project_fact("root", &self.root.to_string_lossy());
        self.memory
            .remember_project_fact("file_count", &stats.file_count.to_string());
        self.memory
            .remember_project_fact("total_lines", &stats.total_lines.to_string());
        self.memory
            .remember_project_fact("symbol_count", &self.index.total_symbol_count().to_string());
        self.memory.remember_project_fact(
            "git_enabled",
            if self.git.is_some() { "yes" } else { "no" },
        );
        let mut top = self.index.files_by_language().into_iter().collect::<Vec<_>>();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        let top_languages = top
            .into_iter()
            .take(5)
            .map(|(lang, count)| format!("{}={}", lang, count))
            .collect::<Vec<_>>()
            .join(", ");
        if !top_languages.is_empty() {
            self.memory
                .remember_project_fact("top_languages", &top_languages);
        }
    }

    // ────────────────────────────────────────────────────────────────
    //  CONVERSATIONAL TOOL INTERCEPTION
    // ────────────────────────────────────────────────────────────────

    /// Before the LLM generates a response, check if the question
    /// matches a semantic tool. If it does, execute the tool first
    /// and return the results as MemoryEntry context for the LLM.
    fn try_semantic_enrichment(&self, question: &str) -> Vec<MemoryEntry> {
        let mut extra = Vec::new();
        let lower = question.to_ascii_lowercase();

        if !self.temporal_graph.enriched {
            return extra;
        }

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Ownership questions: "who owns engine.rs?", "owner of config.rs"
        if lower.contains("who owns")
            || lower.contains("owner of")
            || lower.contains("who maintains")
            || lower.contains("who wrote")
        {
            if let Some(path) = self.extract_file_path_from_question(question) {
                if let Some(history) = self.temporal_graph.file_timeline(&path) {
                    let authors_str: Vec<String> = history
                        .authors
                        .iter()
                        .take(3)
                        .map(|a| format!("{} ({} commits, {:.0}%)", a.name, a.commit_count, a.percentage))
                        .collect();
                    extra.push(MemoryEntry {
                        kind: "tool-result".to_string(),
                        content: format!(
                            "[Ownership data for {}] Primary owner: {}. Contributors: {}. Total commits: {}. Last touched: {} days ago.",
                            history.path,
                            history.primary_owner.as_deref().unwrap_or("unknown"),
                            authors_str.join(", "),
                            history.total_changes,
                            history.staleness_days
                        ),
                        timestamp: now_ts,
                        event: None,
                        embedding: None,
                    });
                }
            } else {
                // No specific file mentioned — give a summary
                let report = self.temporal_graph.ownership_report();
                if report.len() > 50 {
                    extra.push(MemoryEntry {
                        kind: "tool-result".to_string(),
                        content: format!("[Ownership summary] {}", report.chars().take(1500).collect::<String>()),
                        timestamp: now_ts,
                        event: None,
                        embedding: None,
                    });
                }
            }
        }

        // Coupling questions: "what files are coupled?", "hidden dependencies"
        if lower.contains("coupling")
            || lower.contains("coupled")
            || lower.contains("change together")
            || lower.contains("hidden dependenc")
        {
            let report = self.temporal_graph.coupling_report();
            if report.len() > 50 {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!("[Coupling analysis] {}", report.chars().take(1500).collect::<String>()),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        // Project evolution questions: "full history", "how did this project evolve?"
        if lower.contains("project history")
            || lower.contains("full history")
            || lower.contains("history of the project")
            || lower.contains("history of this project")
            || lower.contains("project evolution")
            || lower.contains("how did this project evolve")
            || lower.contains("how has this project changed")
        {
            let report = self.temporal_graph.project_history_report();
            if report.len() > 50 {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!(
                        "[Project history] {}",
                        report.chars().take(1800).collect::<String>()
                    ),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        if lower.contains("team status")
            || (lower.contains("team") && lower.contains("working on"))
            || (lower.contains("team") && lower.contains("doing"))
            || lower.contains("who is working on what")
        {
            if let Ok(report) = self.team_status_summary() {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!("[Team status] {}", report.chars().take(1500).collect::<String>()),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        if lower.contains("where should i start")
            || lower.contains("onboard me")
            || lower.contains("onboarding")
            || lower.contains("understand this codebase")
            || lower.contains("how should i get started")
        {
            let report = self.render_onboarding();
            if report.len() > 50 {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!("[Onboarding] {}", report.chars().take(1800).collect::<String>()),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        if lower.contains("project summary")
            || lower.contains("overview of the project")
            || lower.contains("summarize this codebase")
            || lower.contains("summarize the project")
            || lower.contains("what does this project do")
        {
            if let Some(summary) = self.architecture_context_summary(question) {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!("[Project summary] {}", summary.chars().take(1600).collect::<String>()),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        // Timeline / history questions: "history of engine.rs", "what happened to config.rs"
        if lower.contains("timeline")
            || lower.contains("history of")
            || lower.contains("what happened to")
            || lower.contains("story of")
        {
            if let Some(path) = self.extract_file_path_from_question(question) {
                if let Some(report) = self.temporal_graph.why_report(&path) {
                    extra.push(MemoryEntry {
                        kind: "tool-result".to_string(),
                        content: format!("[File timeline] {}", report.chars().take(2000).collect::<String>()),
                        timestamp: now_ts,
                        event: None,
                        embedding: None,
                    });
                }
            }
        }

        if lower.contains("hotspots")
            || lower.contains("most changed files")
            || lower.contains("change hotspots")
            || lower.contains("churn hotspots")
            || lower.contains("hot files")
        {
            let report = self.temporal_graph.hotspot_report();
            if report.len() > 50 {
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!(
                        "[Hotspots] {}",
                        report.chars().take(1600).collect::<String>()
                    ),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        // Staleness / risk questions: "what files are stale?", "bus factor"
        if lower.contains("stale") || lower.contains("bus factor") || lower.contains("at risk") {
            let mut stale_files: Vec<_> = self
                .temporal_graph
                .file_histories
                .values()
                .filter(|h| h.staleness_days > 30)
                .collect();
            stale_files.sort_by(|a, b| b.staleness_days.cmp(&a.staleness_days));

            if !stale_files.is_empty() {
                let summary: Vec<String> = stale_files
                    .iter()
                    .take(10)
                    .map(|h| {
                        format!(
                            "{} (stale {} days, owner: {}, {} commits)",
                            h.path,
                            h.staleness_days,
                            h.primary_owner.as_deref().unwrap_or("unknown"),
                            h.total_changes
                        )
                    })
                    .collect();
                extra.push(MemoryEntry {
                    kind: "tool-result".to_string(),
                    content: format!("[Staleness report] {} stale files (>30 days): {}", stale_files.len(), summary.join("; ")),
                    timestamp: now_ts,
                    event: None,
                    embedding: None,
                });
            }
        }

        extra
    }

    /// Extract a file path from a natural language question.
    /// Looks for tokens containing '.' that look like file paths.
    fn extract_file_path_from_question(&self, question: &str) -> Option<String> {
        let file_extensions = [
            ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".go", ".java",
            ".toml", ".json", ".yaml", ".yml", ".md", ".css", ".html",
        ];
        for token in question.split_whitespace() {
            let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '\\' && c != '_' && c != '-');
            if file_extensions.iter().any(|ext| clean.ends_with(ext)) {
                return Some(clean.to_string());
            }
        }
        None
    }

    // ────────────────────────────────────────────────────────────────
    //  CROSS-PROJECT INTELLIGENCE
    // ────────────────────────────────────────────────────────────────

    /// Search memory and concepts across ALL registered projects.
    fn search_global_knowledge(&self, query: &str) -> Result<String> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let registry_path = home.join(".astra").join("registry.json");
        let registry: serde_json::Value = match fs::read_to_string(&registry_path) {
            Ok(c) => serde_json::from_str(&c).unwrap_or(serde_json::json!({})),
            Err(_) => return Ok("No global registry found. Run :index in at least one project first.".to_string()),
        };

        let obj = match registry.as_object() {
            Some(o) => o,
            None => return Ok("Registry is empty.".to_string()),
        };

        let query_lower = query.to_ascii_lowercase();
        let mut results: Vec<String> = Vec::new();

        for (project_id, project_path) in obj {
            let brain_dir = home.join(".astra").join("brain").join(project_id);
            let project_label = project_path.as_str().unwrap_or(project_id);

            // Search episodic memory
            let memory_path = brain_dir.join("episodic_memory.json");
            if memory_path.exists() {
                if let Ok(data) = fs::read_to_string(&memory_path) {
                    if let Ok(entries) = serde_json::from_str::<Vec<MemoryEntry>>(&data) {
                        for entry in entries.iter().rev().take(200) {
                            if entry.content.to_ascii_lowercase().contains(&query_lower) {
                                results.push(format!(
                                    "[{}] ({}) {}",
                                    project_id,
                                    entry.kind,
                                    entry.content.chars().take(200).collect::<String>()
                                ));
                                if results.len() >= 15 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Search concepts
            let concepts_path = brain_dir.join("concepts.json");
            if concepts_path.exists() {
                if let Ok(concepts) = crate::semantic_memory::ConceptStore::load(&concepts_path) {
                    for c in &concepts.concepts {
                        if c.description.to_ascii_lowercase().contains(&query_lower) {
                            results.push(format!(
                                "[{}] Concept: {} (confidence: {:.0}%)",
                                project_id,
                                c.description,
                                c.confidence * 100.0
                            ));
                        }
                    }
                    for p in &concepts.patterns {
                        if p.description.to_ascii_lowercase().contains(&query_lower) {
                            results.push(format!(
                                "[{}] Pattern: {}",
                                project_id, p.description
                            ));
                        }
                    }
                }
            }

            if results.len() >= 15 {
                break;
            }
        }

        if results.is_empty() {
            Ok(format!(
                "No matches for '{}' across {} registered projects.",
                query,
                obj.len()
            ))
        } else {
            let mut out = format!(
                "Cross-project search for '{}' ({} results across {} projects):\n\n",
                query,
                results.len(),
                obj.len()
            );
            for r in &results {
                out.push_str(&format!("  {}\n", r));
            }
            Ok(out)
        }
    }
}


fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn resolve_memory_path(root: &Path) -> PathBuf {
    let global_brain = crate::config::get_global_brain_path(root);
    let preferred = global_brain.join("episodic_memory.json");
    if preferred.exists() {
        return preferred;
    }
    
    // Fallback exactly to local .astra if it has history there
    let local = root.join(".astra").join("memory.json");
    if local.exists() {
        return local;
    }
    
    let previous = root.join(".forge").join("memory.json");
    if previous.exists() {
        return previous;
    }
    let legacy = root.join(".codex").join("memory.json");
    if legacy.exists() {
        return legacy;
    }
    preferred
}

fn register_global_project(root: &Path) {
    use std::fs;
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    
    let registry_path = home.join(".astra").join("registry.json");
    let mut registry: serde_json::Value = match fs::read_to_string(&registry_path) {
        Ok(c) => serde_json::from_str(&c).unwrap_or(serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    
    if let Some(obj) = registry.as_object_mut() {
        let abs_path = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let id = crate::config::get_global_project_id(root);
        obj.insert(id, serde_json::json!(abs_path.to_string_lossy().to_string()));
    }
    
    if let Some(parent) = registry_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(registry_path, serde_json::to_string_pretty(&registry).unwrap_or_default());
}

fn is_project_context_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    let keywords = [
        "project",
        "repo",
        "repository",
        "codebase",
        "agenda",
        "tonight",
        "our app",
        "our service",
        "this app",
        "this project",
        "this folder",
        "this directory",
        "in this code",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

fn is_architecture_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    let keywords = [
        "architecture",
        "structure",
        "organized",
        "organised",
        "overview",
        "module",
        "modules",
        "component",
        "components",
        "service",
        "services",
        "dependency",
        "dependencies",
        "coupling",
        "flow",
        "where is",
        "where are",
        "how does",
        "how do",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

fn is_team_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    let keywords = [
        "team",
        "developer",
        "developers",
        "member",
        "members",
        "task",
        "tasks",
        "assignee",
        "assigned",
        "session",
        "sessions",
        "ownership",
        "owner",
        "who owns",
        "bus factor",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

fn is_social_message(question: &str) -> bool {
    let lower = question
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase();
    let exact = matches!(
        lower.as_str(),
        "hey"
            | "hi"
            | "hello"
            | "hey astra"
            | "hi astra"
            | "hello astra"
            | "astra"
            | "sup"
            | "yo"
            | "what's up"
            | "whats up"
            | "how are you"
            | "how are you astra"
            | "how r you"
            | "how you doing"
            | "how's it going"
            | "hows it going"
    ) || matches!(
        lower.as_str(),
        "hii" | "hiii" | "hiiii" | "heyy" | "heyyy" | "helloo" | "hellooo"
    );
    if exact {
        return true;
    }

    let word_count = lower.split_whitespace().count();
    if word_count > 9 {
        return false;
    }
    let has_social_phrase = [
        "what's up",
        "whats up",
        "how are you",
        "how are yoi",
        "how are u",
        "how r you",
        "how you doing",
        "how is it",
        "how are things",
        "how's it going",
        "hows it going",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
        || lower.starts_with("how are yo");
    let first_word = lower.split_whitespace().next().unwrap_or("");
    let starts_as_greeting = is_stretched_word(first_word, "hey", 'y')
        || is_stretched_word(first_word, "hi", 'i')
        || is_stretched_word(first_word, "hello", 'o')
        || matches!(first_word, "yo" | "sup");

    has_social_phrase || (starts_as_greeting && !contains_work_intent(&lower))
}

fn is_stretched_word(word: &str, base: &str, stretch: char) -> bool {
    word == base
        || word
            .strip_prefix(base)
            .map(|tail| !tail.is_empty() && tail.len() <= 5 && tail.chars().all(|c| c == stretch))
            .unwrap_or(false)
}

fn is_social_wellbeing_question(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    [
        "what's up",
        "whats up",
        "how are you",
        "how are yoi",
        "how are u",
        "how r you",
        "how you doing",
        "how is it",
        "how are things",
        "how's it going",
        "hows it going",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
        || lower.starts_with("how are yo")
}

fn is_focused_scope_followup(lower: &str, _focused_path: &str) -> bool {
    lower == "yes that"
        || lower == "yeah that"
        || lower == "yep that"
        || lower.contains("what is it about")
        || lower.contains("what's it about")
        || lower.contains("that you just checked")
        || lower.contains("check it again")
        || lower.contains("check again")
        || lower.contains("inspect it")
}

fn is_task_context_question(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    [
        "active task",
        "current task",
        "previous task",
        "what are we working on",
        "what were we working on",
        "continue working",
        "resume the task",
        "resume the plan",
        "task status",
        "plan status",
        "on the agenda",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

fn is_command_followup(question: &str) -> bool {
    let lower = question
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase();
    let words = lower.split_whitespace().count();
    words <= 12
        && (matches!(
            lower.as_str(),
            "who"
                | "so who"
                | "and who"
                | "what did you find"
                | "what did it find"
                | "what did you see"
                | "tell me more"
                | "explain that"
                | "summarize that"
                | "what does that mean"
                | "why did we make it"
                | "why did we do it"
        ) || lower.starts_with("who owns "))
}

fn is_owner_followup(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    matches!(lower.as_str(), "who" | "so who" | "and who")
        || lower.contains("who owns")
        || lower.contains("ownership")
}

fn render_issue(issue: &crate::issues::IssueRecord) -> String {
    let mut out = format!(
        "Issue {} [{:?}]\nReport: {}",
        issue.id, issue.status, issue.report
    );
    if let Some(head) = &issue.head_commit {
        let _ = writeln!(&mut out, "\nGit HEAD: {}{}", head, issue.branch.as_ref().map(|branch| format!(" on {}", branch)).unwrap_or_default());
    }
    if !issue.changed_files.is_empty() {
        let _ = writeln!(&mut out, "Changed files: {}", issue.changed_files.iter().take(12).cloned().collect::<Vec<_>>().join(", "));
    }
    if !issue.likely_files.is_empty() {
        let _ = writeln!(&mut out, "Likely files: {}", issue.likely_files.iter().take(8).cloned().collect::<Vec<_>>().join(", "));
    }
    if !issue.git_evidence.is_empty() {
        let _ = writeln!(&mut out, "Git evidence:");
        for evidence in issue.git_evidence.iter().take(8) {
            let _ = writeln!(&mut out, "- {}", evidence);
        }
    }
    if let Some(reproduction) = &issue.reproduction {
        let _ = writeln!(&mut out, "Reproduction: {}", reproduction);
    }
    if let Some(job) = &issue.cowork_job_id {
        let _ = writeln!(&mut out, "Cowork job: {}", job);
    }
    out
}

fn contains_work_intent(lower: &str) -> bool {
    [
        "fix ", "build ", "implement ", "change ", "edit ", "delete ", "remove ",
        "run ", "test ", "review ", "audit ", "commit ", "ship ", "deploy ", "create ",
    ]
    .iter()
    .any(|verb| lower.contains(verb))
}

fn parse_cowork_delegate_request(input: &str) -> Option<String> {
    let lower = input.trim().to_ascii_lowercase();
    for worker in ["codex", "claude", "cursor"] {
        let dispatch_prefix = format!("dispatch {} to ", worker);
        if lower.starts_with(&dispatch_prefix) {
            let goal = input.trim()[dispatch_prefix.len()..].trim();
            if !goal.is_empty() {
                return Some(format!(":dispatch {} {}", worker, goal));
            }
        }
        for prefix in [
            format!("tell {} to ", worker),
            format!("ask {} to ", worker),
            format!("have {} ", worker),
            format!("delegate to {} ", worker),
            format!("send to {}: ", worker),
        ] {
            if lower.starts_with(&prefix) {
                let goal = input.trim()[prefix.len()..].trim();
                if !goal.is_empty() {
                    return Some(format!(":delegate {} {}", worker, goal));
                }
            }
        }
    }
    None
}

fn parse_delegate_args(input: &str) -> Option<(&str, &str)> {
    let (mut worker, mut goal) = input.trim()
        .split_once(char::is_whitespace)
        .map(|(worker, goal)| (worker.trim(), goal.trim()))
        .unwrap_or(("any", input.trim()));

    // Be forgiving when an LLM emits “:delegate to codex …” instead of
    // “:delegate codex …”, but never persist “to” as a worker name.
    if worker.eq_ignore_ascii_case("to") {
        let (candidate, remaining) = goal.split_once(char::is_whitespace)?;
        worker = candidate.trim();
        goal = remaining.trim();
    }

    let valid_worker = matches!(worker.to_ascii_lowercase().as_str(), "codex" | "claude" | "cursor" | "any");
    if valid_worker && !goal.is_empty() {
        Some((worker, goal))
    } else {
        None
    }
}

fn is_abandon_active_task_request(input: &str) -> bool {
    let lower = input.trim().to_ascii_lowercase();
    [
        "ditch that",
        "drop that",
        "ditch the task",
        "drop the task",
        "abandon the task",
        "abandon that",
        "cancel the task",
        "forget that task",
        "stop working on that",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

fn is_confirmation(input: &str) -> bool {
    let lower = input
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "yes"
            | "yep"
            | "yup"
            | "yeah"
            | "confirm"
            | "yes please"
            | "do it"
            | "go ahead"
            | "abandon it"
            | "yes abandon it"
    )
}

fn is_confirmation_rejection(input: &str) -> bool {
    let lower = input
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "no" | "nope" | "cancel" | "never mind" | "nevermind" | "nvm" | "don't" | "dont"
    )
}

/// Automatic web fallback is intentionally narrow. Local/project questions
/// must never leave the machine merely because the model sounds uncertain.
fn should_auto_search_web(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    [
        "search the web",
        "search web",
        "look it up online",
        "look this up online",
        "google ",
        "latest release",
        "latest version",
        "current version",
        "current documentation",
        "official documentation",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

/// Starting an autonomous task requires a clear imperative. Generic agreement
/// belongs to conversation and must never be treated as execution consent.
fn is_explicit_continue_request(input: &str) -> bool {
    let lower = input.trim().to_ascii_lowercase();
    if lower.contains("don't")
        || lower.contains("dont ")
        || lower.contains("do not")
        || lower.contains("not continue")
        || lower.contains("not resume")
    {
        return false;
    }

    matches!(
        lower.as_str(),
        "continue"
            | "resume"
            | "keep going"
            | "keep working"
            | "go ahead"
            | "do it"
            | "start working"
            | "get to work"
            | "proceed"
    ) || lower.starts_with("continue ")
        || lower.starts_with("resume ")
        || lower.starts_with("proceed with ")
        || lower.contains("continue working")
        || lower.contains("resume the task")
        || lower.contains("resume the plan")
        || lower.contains("pick up where we left off")
        || lower.contains("go ahead and ")
        || lower.contains("execute the plan")
        || lower.contains("run the plan")
}

fn is_name_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    lower.contains("what's my name")
        || lower.contains("whats my name")
        || lower.contains("what is my name")
        || lower.contains("who am i")
}

fn is_assistant_identity_question(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    lower == "who are you"
        || lower == "what are you"
        || lower == "what is your name"
        || lower == "whats your name"
        || lower == "what's your name"
}

fn is_what_should_we_do_question(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    lower == "what should we do"
        || lower == "what do we do"
        || lower == "what should i do"
        || lower == "what now"
        || lower == "now what"
}

fn is_language_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    lower.contains("major language")
        || lower.contains("dominant language")
        || lower.contains("language is dominant")
        || lower.contains("what language is dominant")
        || lower.contains("what language is used")
        || lower.contains("major language used")
        || lower.contains("what about rust")
        || lower.contains("what about typescript")
        || lower.contains("what about javascript")
        || lower.contains("what about python")
        || lower.contains("what about go")
        || lower.contains("what about java")
}

fn extract_language_mention(question: &str) -> Option<String> {
    let lower = question.to_ascii_lowercase();
    let candidates = [
        ("rust", "rust"),
        ("typescript", "typescript"),
        ("javascript", "javascript"),
        ("python", "python"),
        ("go", "go"),
        ("java", "java"),
    ];
    for (needle, lang) in candidates {
        if lower.contains(needle) {
            return Some(lang.to_string());
        }
    }
    None
}

fn top_language_by_lines(lines_by_lang: &std::collections::HashMap<String, usize>) -> Option<(String, usize)> {
    lines_by_lang
        .iter()
        .max_by_key(|(_, lines)| **lines)
        .map(|(lang, lines)| (lang.clone(), *lines))
}

fn is_personal_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    let personal_keywords = [
        // Age
        "how old",
        "my age",
        "age am i",
        "years old",
        // Birthday
        "my birthday",
        "when was i born",
        "born on",
        // Location
        "where do i live",
        "where am i from",
        "where i live",
        // Preferences
        "what do i like",
        "what do i prefer",
        "what do i use",
        "what language do i",
        "what's my favorite",
        "whats my favorite",
        "what is my favorite",
        "what is my favourite",
        // General recall
        "tell me about myself",
        "what do you know about me",
        "what do you remember about me",
        "do you know me",
        "do you remember",
        "about me",
        "about this user",
        "about the user",
        "based on ur memory",
        "based on your memory",
        "from your memory",
        "from memory",
        "remember about",
        "know about me",
        "my info",
        "my profile",
    ];
    personal_keywords.iter().any(|k| lower.contains(k))
}

fn extract_value_after_any(input: &str, prefixes: &[&str]) -> Option<String> {
    let lower = input.to_ascii_lowercase();
    for prefix in prefixes {
        if let Some(pos) = lower.find(prefix) {
            let start = pos + prefix.len();
            if start <= input.len() {
                let rest = input[start..].trim();
                let value = trim_fact_value(rest);
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn trim_fact_value(value: &str) -> String {
    let mut end = value.len();
    for separator in [".", ",", ";", " and ", " but "] {
        if let Some(idx) = value.find(separator) {
            end = end.min(idx);
        }
    }
    value[..end]
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

/// Only carry prior turns when the user is clearly continuing a thought. This
/// avoids paying a large context cost on unrelated questions in the same chat.
fn needs_recent_turn_context(question: &str) -> bool {
    let lower = question.trim().to_ascii_lowercase();
    let short_follow_up = lower.split_whitespace().count() <= 12;
    let brief_acknowledgement = matches!(
        lower.trim_matches(|character: char| character.is_ascii_punctuation()),
        "yes" | "yep" | "yup" | "yeah" | "sure" | "okay" | "ok" | "got it"
    );
    short_follow_up
        && (brief_acknowledgement
            || lower.starts_with("and ")
            || lower.starts_with("also ")
            || lower.starts_with("what about")
            || lower.starts_with("no ")
            || lower.starts_with("but ")
            || lower.contains("that")
            || lower.contains("you said")
            || lower.contains("i meant")
            || lower.chars().any(|character| character.is_ascii_digit())
            || lower.contains("the same")
            || lower.contains("the above")
            || lower.contains("as well"))
}

fn parse_option_selection(input: &str) -> Option<Vec<u8>> {
    let lower = input.trim().to_ascii_lowercase();
    let all_requested = (lower.contains("all") || lower.contains("everything"))
        && ["do", "run", "execute", "yes", "yeah", "yup", "sure"]
            .iter()
            .any(|word| lower.split_whitespace().any(|token| token == *word));

    let mut selected = if all_requested {
        vec![1, 2, 3]
    } else {
        let mut values = lower
            .split(|character: char| !character.is_ascii_digit())
            .filter(|token| !token.is_empty())
            .filter_map(|token| token.parse::<u8>().ok())
            .collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        values
    };

    if selected.is_empty() || selected.iter().any(|value| !matches!(value, 1..=3)) {
        return None;
    }
    selected.sort_unstable();
    Some(selected)
}

fn truncate_for_context(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut compact = value.chars().take(limit.saturating_sub(16)).collect::<String>();
    compact.push_str("... [truncated]");
    compact
}

fn normalized_fs_path(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    value
        .strip_prefix("//?/")
        .unwrap_or(&value)
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn resolve_index_path(root: &Path, indexed_path: &Path) -> PathBuf {
    let candidate = if indexed_path.is_absolute() {
        indexed_path.to_path_buf()
    } else {
        root.join(indexed_path)
    };
    candidate.canonicalize().unwrap_or(candidate)
}

fn include_memory_entry_in_answer(entry: &MemoryEntry, question: &str) -> bool {
    let kind = entry.kind.as_str();
    if matches!(
        kind,
        "qa"
            | "qa-memory"
            | "web-search"
            | "web-knowledge"
            | "autonomous-action"
            | "task-execution"
            | "orchestrator-delegated"
            | "orchestrator-result"
    ) {
        return false;
    }

    if kind == "command" {
        let lower_q = question.to_ascii_lowercase();
        return lower_q.contains("command") || lower_q.contains("run");
    }

    true
}

fn extract_query_terms(question: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "is", "are", "to", "for", "of", "in", "on", "at", "how", "why",
        "what", "where", "when", "does", "do", "did", "this", "that", "these", "those",
        "with", "and", "from", "about", "into", "your", "our",
    ];
    let mut seen = HashSet::new();
    question
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| {
                !c.is_alphanumeric() && c != '_' && c != '-' && c != '.' && c != '/'
            })
            .to_ascii_lowercase()
        })
        .filter(|token| token.len() >= 3)
        .filter(|token| !stop_words.contains(&token.as_str()))
        .filter(|token| seen.insert(token.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::CodexEngine;
    use crate::index::CodeIndex;
    use crate::memory::{MemoryEntry, MemoryStore};
    use crate::model::CodexModel;
    use crate::rag::{Chunk, VectorStore};
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct RecordingModel {
        prompt: Arc<Mutex<String>>,
        response: String,
    }

    impl CodexModel for RecordingModel {
        fn complete(&self, prompt: &str) -> Result<String> {
            *self.prompt.lock().unwrap() = prompt.to_string();
            Ok(self.response.clone())
        }
    }

    fn fresh_engine() -> CodexEngine {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("astra_engine_test_{}_{}", std::process::id(), unique));
        let _ = fs::create_dir_all(&root);
        let mut engine = CodexEngine::with_root(root);
        engine.memory = MemoryStore::default();
        engine.index = CodeIndex::new();
        engine.vector_store = VectorStore::new();
        engine
    }

    #[test]
    fn parses_loose_natural_migration_request() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.parse_natural_migrate_request(
            "convert auth service from rust to go into out_go with ai",
        );
        assert_eq!(cmd.as_deref(), Some(":migrate auth_service rs go out_go --ai"));
    }

    #[test]
    fn parses_compact_migration_request_with_symbols() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.parse_natural_migrate_request(
            "migrate core/src from rs -> py output out_py",
        );
        assert_eq!(cmd.as_deref(), Some(":migrate core/src rs py out_py"));
    }

    #[test]
    fn routes_coding_intent_to_task_command() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.intent_for("write code to build feature for oauth login");
        assert_eq!(
            cmd.as_deref(),
            Some(":task write code to build feature for oauth login")
        );
    }

    #[test]
    fn routes_folder_file_and_run_request_to_task_command() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let prompt = "Create a folder sandbox/auth_service and add sandbox/auth_service/main.rs with a minimal Rust HTTP server, then run cargo check";
        let cmd = engine.intent_for(prompt);
        assert_eq!(cmd.as_deref(), Some(":task Create a folder sandbox/auth_service and add sandbox/auth_service/main.rs with a minimal Rust HTTP server, then run cargo check"));
    }

    #[test]
    fn routes_workflow_listing_intent() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.intent_for("show workflows");
        assert_eq!(cmd.as_deref(), Some(":workflow list"));
    }

    #[test]
    fn routes_learning_intent_to_learn_command() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.intent_for("teach me rust async patterns");
        assert_eq!(cmd.as_deref(), Some(":learn rust async patterns"));
    }

    #[test]
    fn routes_team_status_intent_to_command() {
        let engine = CodexEngine::with_root(PathBuf::from("."));
        let cmd = engine.intent_for("astra team status");
        assert_eq!(cmd.as_deref(), Some(":team status"));
    }

    #[test]
    fn routes_natural_cross_editor_requests_without_starting_astra_agent() {
        let engine = fresh_engine();
        assert_eq!(
            engine.intent_for("tell codex to implement OAuth login"),
            Some(":delegate codex implement OAuth login".to_string())
        );
        assert_eq!(
            engine.intent_for("ask claude to review the checkout flow"),
            Some(":delegate claude review the checkout flow".to_string())
        );
        assert_eq!(
            engine.intent_for("connect Astra to Cursor and Codex with MCP"),
            Some(":cowork init".to_string())
        );
    }

    #[test]
    fn bug_report_creates_triaged_issue_with_reproduction_gate() {
        let mut engine = fresh_engine();
        engine
            .index
            .add_file(PathBuf::from("src/auth.rs"), "pub fn login() { panic!(\"boom\") }\n");

        assert_eq!(
            engine.intent_for("fix login crashes when the session expires"),
            Some(":fix-bug login crashes when the session expires".to_string())
        );
        let response = engine
            .handle_input("fix login crashes when the session expires")
            .unwrap();
        assert!(response.contains("Created **astra-issue-"));
        assert!(response.contains("Reproduction gate"));
        assert!(response.contains("Nothing was changed yet"));
        let issues = crate::issues::IssueStore::new(&engine.root).list(10).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].status, crate::issues::IssueStatus::Triaged);
        assert!(issues[0].cowork_job_id.is_some());
    }

    #[test]
    fn delegate_parser_handles_llm_to_word_without_persisting_it_as_worker() {
        assert_eq!(
            super::parse_delegate_args("to codex improve the landing page"),
            Some(("codex", "improve the landing page"))
        );
        assert_eq!(
            super::parse_delegate_args("codex improve the landing page"),
            Some(("codex", "improve the landing page"))
        );
        assert_eq!(super::parse_delegate_args("to codex"), None);
        assert_eq!(super::parse_delegate_args("to definitely-not-a-worker ship it"), None);
    }

    #[test]
    fn cowork_context_is_small_and_contains_shared_state() {
        let mut engine = fresh_engine();
        engine
            .index
            .add_file(PathBuf::from("src/auth.rs"), "pub fn login() {}\n");
        engine
            .memory
            .remember_project_fact("auth", "OAuth uses PKCE and short-lived sessions");
        let store = crate::coworker::CoworkStore::new(&engine.root);
        store
            .create_job("Implement OAuth callback", Some("codex"), Vec::new())
            .unwrap();

        let context = engine.build_cowork_context("OAuth auth callback", 1_200);
        assert!(context.contains("src/auth.rs"));
        assert!(context.contains("OAuth uses PKCE"));
        assert!(context.contains("Active cowork jobs"));
        assert!(context.chars().count() <= 1_200);
    }

    #[test]
    fn detects_project_context_question() {
        assert!(super::is_project_context_question("what's the agenda for this project tonight?"));
        assert!(!super::is_project_context_question("how do I write a Rust iterator?"));
        assert!(!super::is_project_context_question("is there a 4?"));
    }

    #[test]
    fn conversational_agreement_never_resumes_autonomous_work() {
        assert!(!super::is_explicit_continue_request("sure you doo"));
        assert!(!super::is_explicit_continue_request("okay"));
        assert!(!super::is_explicit_continue_request("sounds good"));
        assert!(!super::is_explicit_continue_request("don't continue"));
        assert!(super::is_explicit_continue_request("continue working on the calculator"));
        assert!(super::is_explicit_continue_request("go ahead and run the plan"));
    }

    #[test]
    fn agreement_with_an_active_task_stays_conversational() {
        let mut engine = fresh_engine();
        engine
            .index
            .add_file(PathBuf::from("src/main.rs"), "fn main() {}\n");
        let task_dir = engine.root.join(".astra");
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(
            task_dir.join("active_task.json"),
            r#"{"title":"dangerous old task","phase":"executing"}"#,
        )
        .unwrap();
        let prompt = Arc::new(Mutex::new(String::new()));
        engine.model = Some(Box::new(RecordingModel {
            prompt: prompt.clone(),
            response: "I hear you 😄".to_string(),
        }));

        let answer = engine.handle_input("sure you doo").unwrap();

        assert_eq!(answer, "I hear you 😄");
        assert!(prompt.lock().unwrap().contains("sure you doo"));
    }

    #[test]
    fn numbered_follow_up_receives_the_previous_option_list() {
        let mut engine = fresh_engine();
        engine
            .index
            .add_file(PathBuf::from("src/main.rs"), "fn main() {}\n");
        engine.memory.add(
            "qa",
            "Q: what should we do\nA: Pick (1) review, (2) health, or (3) ship.".to_string(),
        );
        let prompt = Arc::new(Mutex::new(String::new()));
        engine.model = Some(Box::new(RecordingModel {
            prompt: prompt.clone(),
            response: "😂 Fair question—option four is take a break.".to_string(),
        }));

        let answer = engine.answer_question("is there a 4 😏").unwrap();
        let captured = prompt.lock().unwrap().clone();

        assert!(answer.contains("option four"));
        assert!(captured.contains("Pick (1) review, (2) health, or (3) ship."));
    }

    #[test]
    fn action_menu_selections_parse_without_llm_routing() {
        assert_eq!(
            super::parse_option_selection("yeah do all that"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            super::parse_option_selection("1,2,3"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(super::parse_option_selection("1 and 3"), Some(vec![1, 3]));
        assert_eq!(super::parse_option_selection("is there a 4?"), None);
        assert_eq!(super::parse_option_selection("sure"), None);
    }

    #[test]
    fn stretched_greetings_stay_in_fast_social_mode() {
        assert!(super::is_social_message("hii"));
        assert!(super::is_social_message("heyyy"));
        assert!(super::is_social_message("heyy broski"));
        assert!(super::is_social_message("how are yoi"));
        assert!(super::is_social_message("hey whats up broski"));
        assert!(super::is_social_message("how is it my guy"));
        assert!(!super::is_social_message("hey audit the project"));
    }

    #[test]
    fn social_transcript_never_invents_project_activity() {
        let mut engine = fresh_engine();
        let first = engine.handle_input("hey whats up broski").unwrap();
        let second = engine.handle_input("how is it my guy").unwrap();
        let third = engine.handle_input("heyy broski").unwrap();
        let fourth = engine.handle_input("how are yoi").unwrap();

        for answer in [first, second, third, fourth] {
            let lower = answer.to_ascii_lowercase();
            assert!(!lower.contains("calculator"));
            assert!(!lower.contains("reviewing"));
            assert!(!lower.contains("stuck"));
        }
    }

    #[test]
    fn audit_request_routes_to_grounded_health_dashboard() {
        let engine = fresh_engine();
        assert_eq!(
            engine.intent_for("audit the project how well are we doing"),
            Some(":audit".to_string())
        );
        assert_eq!(
            engine.intent_for("how good is project can you observe it for me"),
            Some(":audit".to_string())
        );
        assert_eq!(
            engine.intent_for("audit it please"),
            Some(":audit".to_string())
        );
        assert_eq!(
            engine.intent_for("fully index and search through everything"),
            Some(":index".to_string())
        );
    }

    #[test]
    fn scoped_ownership_questions_preserve_the_git_scope() {
        let engine = fresh_engine();
        assert_eq!(
            engine.intent_for("astra who owns the next js landing page"),
            Some(":owners the next js landing page".to_string())
        );
        assert_eq!(
            engine.intent_for("who built this project"),
            Some(":history".to_string())
        );
    }

    #[test]
    fn scoped_ownership_command_answers_with_commit_message_and_followup() {
        let mut engine = fresh_engine();
        engine.temporal_graph.enriched = true;
        engine.temporal_graph.file_histories.insert(
            "astra-landing-next/app/page.tsx".to_string(),
            crate::semantic_graph::FileHistory {
                path: "astra-landing-next/app/page.tsx".to_string(),
                commits: vec![crate::semantic_graph::CommitNode {
                    id: "abc12345".to_string(),
                    summary: "Build Next.js landing page hero".to_string(),
                    author: "Jeremy".to_string(),
                    timestamp: 1_700_000_000,
                    files_changed: vec![],
                }],
                authors: vec![crate::semantic_graph::AuthorContribution {
                    name: "Jeremy".to_string(),
                    commit_count: 1,
                    percentage: 100.0,
                    first_commit: 1_700_000_000,
                    last_commit: 1_700_000_000,
                }],
                primary_owner: Some("Jeremy".to_string()),
                last_touched: 1_700_000_000,
                staleness_days: 0,
                total_changes: 1,
            },
        );

        let report = engine
            .handle_input("astra who owns the next js landing page")
            .unwrap();
        assert!(report.contains("Jeremy"));
        assert!(report.contains("Build Next.js landing page hero"));

        let followup = engine.handle_input("so who").unwrap();
        assert!(followup.contains("Jeremy"));
        assert!(followup.contains("ownership report"));
    }

    #[test]
    fn command_result_is_available_to_short_followup() {
        let mut engine = fresh_engine();
        engine.memory.remember_conversation_state("last_command", ":owners");
        engine.memory.remember_conversation_state(
            "last_command_result",
            "📊 File Ownership Report\n👤 Astra (522 files)\n👤 Jeremy (84 files)",
        );
        let followup = engine.command_followup_answer("so who").unwrap();
        assert!(followup.contains("ownership report"));
        assert!(followup.contains("Astra"));
    }

    #[test]
    fn local_conversation_does_not_trigger_automatic_web_search() {
        assert!(!super::should_auto_search_web("lets ditch that completely"));
        assert!(!super::should_auto_search_web(
            "audit the project how well are we doing"
        ));
        assert!(super::should_auto_search_web(
            "search the web for the latest Rust release"
        ));
    }

    #[test]
    fn confirmation_archives_active_task_without_touching_project_files() {
        let mut engine = fresh_engine();
        let astra_dir = engine.root.join(".astra");
        fs::create_dir_all(&astra_dir).unwrap();
        fs::write(
            astra_dir.join("active_task.json"),
            r#"{"title":"calculator project","phase":"executing"}"#,
        )
        .unwrap();
        fs::write(engine.root.join("keep_me.txt"), "project data").unwrap();

        let request = engine.handle_input("lets ditch that completely").unwrap();
        assert!(request.contains("Say `yes` to confirm"));
        assert!(astra_dir.join("active_task.json").exists());
        assert_eq!(
            engine.memory.conversation_state("pending_action").as_deref(),
            Some("abandon_active_task")
        );

        let result = engine.handle_input("yup").unwrap();
        assert!(result.contains("Dropped **calculator project**"));
        assert!(!astra_dir.join("active_task.json").exists());
        assert_eq!(
            fs::read_to_string(engine.root.join("keep_me.txt")).unwrap(),
            "project data"
        );
        assert!(fs::read_dir(astra_dir.join("abandoned"))
            .unwrap()
            .next()
            .is_some());
    }

    #[test]
    fn directory_focus_is_inspected_and_survives_pronoun_followups() {
        let mut engine = fresh_engine();
        let landing = engine.root.join("astra-landing");
        fs::create_dir_all(landing.join("src").join("components")).unwrap();
        fs::write(
            landing.join("package.json"),
            r#"{"name":"astra-landing","dependencies":{"react":"19"},"scripts":{"build":"react-scripts build"}}"#,
        )
        .unwrap();
        for component in ["Hero", "Pricing", "Testimonials"] {
            let content = format!("export const {} = () => <div>Astra</div>;", component);
            let path = landing
                .join("src")
                .join("components")
                .join(format!("{}.tsx", component));
            fs::write(&path, &content).unwrap();
            engine.index.add_file(
                PathBuf::from("astra-landing")
                    .join("src")
                    .join("components")
                    .join(format!("{}.tsx", component)),
                &content,
            );
        }

        let first = engine.handle_input("the astra-landing first").unwrap();
        assert!(first.contains("Astra marketing/landing frontend"));
        assert!(!first.to_ascii_lowercase().contains("calculator"));
        let followup = engine.handle_input("what is it about").unwrap();
        assert!(followup.contains("Astra marketing/landing frontend"));
    }

    #[test]
    fn stale_task_is_not_injected_into_unrelated_project_questions() {
        let mut engine = fresh_engine();
        let astra_dir = engine.root.join(".astra");
        fs::create_dir_all(&astra_dir).unwrap();
        fs::write(
            astra_dir.join("active_task.json"),
            r#"{"title":"calculator project","phase":"InProgress"}"#,
        )
        .unwrap();
        let prompt = Arc::new(Mutex::new(String::new()));
        engine.model = Some(Box::new(RecordingModel {
            prompt: prompt.clone(),
            response: "grounded answer".to_string(),
        }));

        engine
            .answer_question("what is astra-landing about")
            .unwrap();
        assert!(!prompt.lock().unwrap().contains("Active task: calculator"));
    }

    #[test]
    fn detects_name_question() {
        assert!(super::is_name_question("what's my name"));
        assert!(super::is_name_question("who am i"));
        assert!(!super::is_name_question("what's the project name"));
    }

    #[test]
    fn excludes_qa_memory_from_answer_context() {
        let entry = MemoryEntry {
            kind: "qa".to_string(),
            content: "old conversation".to_string(),
            timestamp: 0,
            event: None,
            embedding: None,
        };
        assert!(!super::include_memory_entry_in_answer(
            &entry,
            "what's on the agenda"
        ));
    }

    #[test]
    fn ranks_direct_file_mentions_first() {
        let mut engine = fresh_engine();
        engine
            .index
            .add_file(PathBuf::from("src/auth.rs"), "pub fn login_user() {}\n");
        engine
            .index
            .add_file(PathBuf::from("src/payments.rs"), "pub fn charge_card() {}\n");

        let ranked = engine.rank_relevant_files("explain src/auth.rs", 2);
        assert_eq!(ranked.first(), Some(&PathBuf::from("src/auth.rs")));
    }

    #[test]
    fn collects_relevant_chunks_from_file_mentions() {
        let mut engine = fresh_engine();
        engine.vector_store.chunks = vec![
            Chunk {
                id: "src/auth.rs::0".to_string(),
                path: PathBuf::from("src/auth.rs"),
                start_line: 1,
                end_line: 4,
                content: "pub fn login_user() {}\n".to_string(),
                language: "rust".to_string(),
                embedding: None,
            },
            Chunk {
                id: "src/payments.rs::0".to_string(),
                path: PathBuf::from("src/payments.rs"),
                start_line: 1,
                end_line: 4,
                content: "pub fn charge_card() {}\n".to_string(),
                language: "rust".to_string(),
                embedding: None,
            },
        ];

        let chunks = engine.collect_relevant_chunks("what happens in auth.rs", None, 3);
        assert_eq!(chunks.first().map(|chunk| chunk.path.clone()), Some(PathBuf::from("src/auth.rs")));
    }

    #[test]
    fn relevant_memories_skip_noisy_conversation_entries() {
        let mut engine = fresh_engine();
        engine.memory.add("qa", "Q: hi\nA: hello".to_string());
        engine
            .memory
            .remember_project_fact("goal", "make astra feel like a real pair engineer");

        let memories = engine.collect_relevant_memories("what is the project goal", None);
        assert!(memories.iter().any(|entry| entry.kind == "project-fact"));
        assert!(memories.iter().all(|entry| entry.kind != "qa"));
    }

    #[test]
    fn only_follow_up_questions_receive_previous_turn_context() {
        assert!(super::needs_recent_turn_context("what about that approach?"));
        assert!(super::needs_recent_turn_context("and test it too"));
        assert!(super::needs_recent_turn_context("is there a 4 😏"));
        assert!(super::needs_recent_turn_context("no, you said there were three options"));
        assert!(!super::needs_recent_turn_context("how does Rust async work?"));
        assert!(!super::needs_recent_turn_context("explain the authentication module"));
    }

    #[test]
    fn context_truncation_respects_the_requested_budget() {
        let compact = super::truncate_for_context(&"x".repeat(1_000), 100);
        assert!(compact.chars().count() <= 100);
        assert!(compact.ends_with("... [truncated]"));
    }
}
