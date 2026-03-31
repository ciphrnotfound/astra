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
use crate::memory::{MemoryEntry, MemoryEvent, MemoryStore};
use crate::migrate;
use crate::migrate::detect::Language;
use crate::migrate::orchestrate::MigrationConfig;
use crate::migration;
use crate::model::{CodexModel, SearchProvider};
use crate::parser::{parse_rust_file, ParsedSymbolKind};
use crate::persona::Persona;
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

pub struct CodexEngine {
    root: PathBuf,
    index: CodeIndex,
    model: Option<Box<dyn CodexModel + Send + Sync>>,
    search: Option<Box<dyn SearchProvider + Send + Sync>>,
    memory: MemoryStore,
    git: Option<GitRepo>,
    persona: Persona,
    agent_mode: bool,
    auto_approve: bool,
    temporal_graph: TemporalGraph,
    concept_store: ConceptStore,
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

        register_global_project(&root);

        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            search: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
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

        register_global_project(&root);

        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            search: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
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

        register_global_project(&root);

        Self {
            root,
            index: CodeIndex::new(),
            model: Some(model),
            search: None,
            memory,
            git,
            persona,
            agent_mode: false,
            auto_approve: false,
            temporal_graph,
            concept_store,
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

pub fn handle_input(&mut self, input: &str) -> Result<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok("Say something about your codebase to get started.".to_string());
        }

        let mut normalized = trimmed.to_string();
        if normalized.starts_with('›') {
            normalized = normalized.trim_start_matches('›').trim_start().to_string();
        }

        self.record_git_commit();
        self.record_worktree_snapshot();
 
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
                self.temporal_graph.enrich_from_git(git);
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

            return Ok(message);
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
            let matches = self.memory.search(query, 20);
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
            return Ok(summary);
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

        if let Some(rest) = trimmed.strip_prefix(":fix ") {
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
                // Check if this is a personal/identity fact and save to global memory
                let lower_fact = fact.to_ascii_lowercase();
                if lower_fact.starts_with("my name is ") || lower_fact.starts_with("i am ") {
                    self.memory.add_global("fact", fact.to_string());
                } else {
                    self.memory.add("fact", fact.to_string());
                }
                return Ok(format!(
                    "\u{1f9e0} **Memory Updated**\n\nI've stored this fact: \"{}\".\nI will remember this during future queries and migrations.",
                    fact
                ));
            }
        }

        // ── :memory [query] — list or search stored facts ───────────
        if trimmed == ":memory" || trimmed.starts_with(":memory ") {
            let results = if let Some(query) = trimmed.strip_prefix(":memory ") {
                self.memory.search(query, 10)
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
            return self.answer_question(rest);
        }

        if !trimmed.starts_with(':') && !trimmed.starts_with("? ") {
            // ── Auto-fact extraction: silently learn personal facts from conversation ──
            self.auto_extract_facts(trimmed);
            return self.answer_question(trimmed);
        }

        Ok(format!(
            "astra did not understand this yet:\n  {}\n\nTry one of:\n  :index\n  :summary\n  :memory\n  :web <query>       — search the web and remember\n  :learn <language>  — research a language's best practices\n  :migrate <src> <from> <to> <out> [--ai]\n  ? <question>",
            trimmed
        ))
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
                        let mut truncated_contents = contents.clone();
                        if truncated_contents.len() > 1500 {
                            truncated_contents.truncate(1500);
                            truncated_contents.push_str("\n... [Truncated for LLM Context]");
                        }
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
        Ok(())
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
        let matches = self.memory.search(question, 6);
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
        if out.trim().is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn answer_question(&mut self, question: &str) -> Result<String> {
        if is_name_question(question) {
            if let Some(name) = self.memory.user_name() {
                return Ok(format!("Your name is {}.", name));
            }
            return Ok("I don't know your name yet! Use `:learn my name is <your name>` to teach me.".to_string());
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
            let stats = self.index.stats();
            let by_lang = self.index.files_by_language();
            let mut matches = self
                .memory
                .search(question, 6)
                .into_iter()
                .filter(|entry| include_memory_entry_in_answer(entry, question))
                .take(3)
                .collect::<Vec<_>>();
            let recent = self.memory.recent(3);

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
            
            // --- Always prioritize project documentation (README/Vision) in context ---
            if stats.file_count > 0 {
                let docs = self.memory.events_of_kind("source-doc");
                for doc in docs.iter().rev().take(1) {
                    if !matches.iter().any(|m| m.content == doc.content) {
                        let mut doc_clone = (*doc).clone();
                        if doc_clone.content.len() > 800 {
                            doc_clone.content.truncate(800);
                            doc_clone.content.push_str("\n... [Truncated]");
                        }
                        matches.push(doc_clone);
                    }
                }
            }
            
            // --- Add Git history context only if directly relevant ---
            if question.to_lowercase().contains("commit") || question.to_lowercase().contains("history") {
                let git_events = self.memory.events_of_kind("git-commit");
                for evt in git_events.iter().rev().take(1) {
                    matches.push((*evt).clone());
                }
            }

            let mut prompt = String::new();
            let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
            let _ = writeln!(
                &mut prompt,
                "\nYou are Astra, a local codebase assistant. Project root: {:?}.",
                self.root
            );

            // --- Inject user identity from global memory as hard facts ---
            if let Some(name) = self.memory.user_name() {
                let _ = writeln!(&mut prompt, "\n### USER IDENTITY (ABSOLUTE FACTS - NEVER CONTRADICT):");
                let _ = writeln!(&mut prompt, "- The user's name is: {}", name);
            }
            // Inject any other global preferences/facts
            if let Some(lang) = self.memory.user_preference("language") {
                let _ = writeln!(&mut prompt, "- User's preferred language: {}", lang);
            }
            if let Some(reasoning) = self.memory.user_preference("reasoning") {
                let _ = writeln!(&mut prompt, "- Reasoning depth preference: {}", reasoning);
            }
            
            // Inject Memory context for "Proactive Triggering"
            if !matches.is_empty() {
                let _ = writeln!(&mut prompt, "\n### MEMORY BANK (USE THESE FACTS TO ANSWER - DO NOT IGNORE):");
                let _ = writeln!(&mut prompt, "The following are facts you have learned about the user and their project.");
                let _ = writeln!(&mut prompt, "If the user asks about ANY of these facts, you MUST use them in your answer.");
                let _ = writeln!(&mut prompt, "Do NOT say 'I don't have that information' if it appears below.");
                for entry in matches {
                    let _ = writeln!(&mut prompt, "- [{}] {}", entry.kind, entry.content);
                }
            }
            
            let _ = writeln!(&mut prompt, "\n### Project Stats:");
            let _ = writeln!(&mut prompt, "- Root: {:?}", self.root);
            let _ = writeln!(&mut prompt, "- Indexed: {} files, {} lines.", stats.file_count, stats.total_lines);
            if !by_lang.is_empty() {
                let mut top = by_lang.into_iter().collect::<Vec<_>>();
                top.sort_by(|a, b| b.1.cmp(&a.1));
                let top_str = top
                    .into_iter()
                    .take(4)
                    .map(|(lang, count)| format!("{}={}", lang, count))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(&mut prompt, "- Top languages: {}", top_str);
            }
            if !recent.is_empty() {
                let recent_kinds = recent
                    .iter()
                    .map(|e| e.kind.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(&mut prompt, "- Recent memory kinds: {}", recent_kinds);
            }
            let _ = writeln!(&mut prompt, "- Never reference README/framework details unless explicitly confirmed by indexed files or current tool output.");
            let _ = writeln!(&mut prompt, "- Do not infer active tasks unless the user explicitly mentions a task in this conversation turn or a verified active task record is provided.");

            // --- Autonomous Tool Calling: inject installed workflows ---
            let wf_manager = WorkflowManager::new(&self.root);
            let installed_tools = wf_manager.list_workflows_with_metadata().unwrap_or_default();
            if !installed_tools.is_empty() {
                let _ = writeln!(&mut prompt, "\n### YOUR INSTALLED TOOLS (you can trigger these):");
                for (tool_name, description) in &installed_tools {
                    let _ = writeln!(&mut prompt, "- {}: {}", tool_name, description);
                }
                let _ = writeln!(&mut prompt, "\nIMPORTANT RULES FOR TOOL USE:");
                let _ = writeln!(&mut prompt, "- ONLY use a workflow tool if the user's request DIRECTLY MATCHES the tool's description above.");
                let _ = writeln!(&mut prompt, "- If a tool matches and required args are present, return ONLY `:workflow run <tool_name> [args...]`.");
                let _ = writeln!(&mut prompt, "- If required args are missing, do NOT guess; ask a concise follow-up question instead.");
                let _ = writeln!(&mut prompt, "- For general code understanding, architecture, or questions, do NOT use workflow commands. Reply normally using indexed context.");
            }
            let _ = writeln!(&mut prompt, "\n### ACTIONS YOU CAN TAKE:");
            let _ = writeln!(&mut prompt, "- To migrate code: return exactly `:migrate <source_dir> <from_lang> <to_lang> <output_dir> [--ai]` (e.g. `:migrate core/src rs py out --ai`)");
            let _ = writeln!(&mut prompt, "- To automate something new: return exactly `:workflow generate <description>`");
            let _ = writeln!(&mut prompt, "- Reasoning protocol: infer intent, check grounding, and choose either command execution or conversational response.");
            let _ = writeln!(&mut prompt, "- For coding tasks: provide concise implementation reasoning, verify assumptions against project context, and avoid invented APIs or file paths.");
            let _ = writeln!(&mut prompt, "- If the user explicitly asks to execute an action and arguments are sufficient, output ONLY the command starting with `:`.");

            let _ = writeln!(&mut prompt, "\nUser question/task: {}", question);

            let answer = model.complete(&prompt)?;

            // --- Autonomous Interception: scan lines for command ---
            for line in answer.lines() {
                let text = line.trim().trim_matches('`');
                if text.starts_with(":workflow run ")
                    || text.starts_with(":workflow generate ")
                    || text.starts_with(":workflow list")
                    || text.starts_with(":migrate ")
                    || text.starts_with(":task ")
                {
                    self.memory.add("autonomous-action", format!("Astra autonomously triggered: {}", text));
                    return self.handle_input(text);
                }
            }

            // If the answer seems uncertain and we have web search, augment it
            let lower_answer = answer.to_ascii_lowercase();
            let seems_uncertain = lower_answer.contains("i don't have")
                || lower_answer.contains("i'm not sure")
                || lower_answer.contains("i cannot")
                || lower_answer.contains("without more context")
                || lower_answer.contains("i don't know");

            if seems_uncertain {
                if let Some(search) = &self.search {
                    if let Ok(results) = search.search(question) {
                        self.memory.add("web-search", format!("Auto-search for: {}\n{}", question, &results[..results.len().min(2000)]));
                        let augmented_prompt = format!(
                            "{}\n\nI also found these web search results that may help:\n{}\n\nNow answer the user's question using ALL available context: {}",
                            prompt, &results[..results.len().min(3000)], question
                        );
                        if let Ok(better_answer) = model.complete(&augmented_prompt) {
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
        let lower = trimmed.to_lowercase();
        // Greetings/social should NOT be intercepted by intent rules.
        if lower == "hey astra" || lower == "hello astra" || lower == "astra" || lower == "hey" || lower == "hi" {
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
        {
            return Some(":memory".to_string());
        }

        if lower.contains("what do you know")
            || lower.contains("project summary")
            || lower.contains("project info")
            || lower.contains("project information")
        {
            return Some(":summary".to_string());
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
        {
            return Some(":files-by-lang".to_string());
        }

        if lower.contains("git repo") || lower.contains("git repository") {
            return Some(":summary".to_string());
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

        if lower.contains("health check") || lower == "health" {
            return Some(":health".to_string());
        }

        if lower.contains("graph") || lower.contains("semantic graph") {
            return Some(":graph".to_string());
        }

        if lower.contains("index codebase")
            || lower.contains("index project")
            || lower.contains("reindex")
            || lower == "index"
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

        if lower.contains("predict") && (lower.contains("refactor") || lower.contains("debt") || lower.contains("drift")) {
            return Some(":predict".to_string());
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
            return self.get_next_task(None);
        }
        let model = match &self.model {
            Some(m) => m,
            None => return self.get_next_task(Some(goal)),
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
        self.memory
            .add("task-execution", format!("TASK: {}\nRESULT: {}", goal, result));
        Ok(result)
    }

    pub fn report_task_result(&mut self, task_id: &str, success: bool, details: &str) -> Result<String> {
        crate::orchestrator::process_task_result(self, task_id, success, details)
    }

    /// Builds a comprehensive grounding context for external AI integrations (like MCP)
    /// to ensure the agent doesn't hallucinate facts about the user or project.
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
            let truncated = if contents.len() > 500 { format!("{}...", &contents[..500]) } else { contents.clone() };
            let _ = writeln!(&mut context, "\nProject Overview from README.md:\n---\n{}---", truncated.trim());
        }

        let stats = self.index.stats();
        let _ = writeln!(&mut context, "\nCodebase Stats: {} files indexed, {} total lines.", stats.file_count, stats.total_lines);

        let team_mgr = TeamManager::new(&self.root);
        if let Ok(state) = team_mgr.load_state() {
            if !state.team_name.is_empty() {
                let user = load_global_config()
                    .user
                    .unwrap_or_else(|| std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "local_dev".to_string()));
                let normalize = |v: &str| v.trim().trim_start_matches('@').to_ascii_lowercase();
                let user_norm = normalize(&user);
                let is_member = state.members.keys().any(|name| normalize(name) == user_norm);
                let _ = writeln!(
                    &mut context,
                    "Team State: team='{}', user='{}', member={}",
                    state.team_name, user, is_member
                );
                if is_member {
                    let my_open = state
                        .tasks
                        .values()
                        .filter(|t| normalize(&t.assignee) == user_norm)
                        .filter(|t| t.status != crate::teams::TaskStatus::Done)
                        .map(|t| format!("[{}] {}", t.id, t.description))
                        .collect::<Vec<_>>();
                    if !my_open.is_empty() {
                        let _ = writeln!(&mut context, "My Open Team Tasks: {}", my_open.join(" | "));
                    }
                }
            }
        }

        context
    }

    /// Automatically extract personal facts from casual user messages and store them globally.
    fn auto_extract_facts(&mut self, input: &str) {
        let lower = input.trim().to_ascii_lowercase();
        
        // Don't extract from questions
        if lower.ends_with('?') || lower.len() < 8 {
            return;
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
                    let existing = self.memory.search(&fact, 3);
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
        "here",
    ];
    keywords.iter().any(|k| lower.contains(k))
}

fn is_name_question(question: &str) -> bool {
    let lower = question.to_ascii_lowercase();
    lower.contains("what's my name")
        || lower.contains("whats my name")
        || lower.contains("what is my name")
        || lower.contains("who am i")
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

#[cfg(test)]
mod tests {
    use super::CodexEngine;
    use crate::memory::MemoryEntry;
    use std::path::PathBuf;

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
    fn detects_project_context_question() {
        assert!(super::is_project_context_question("what's the agenda for this project tonight?"));
        assert!(!super::is_project_context_question("how do I write a Rust iterator?"));
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
        };
        assert!(!super::include_memory_entry_in_answer(
            &entry,
            "what's on the agenda"
        ));
    }
}
