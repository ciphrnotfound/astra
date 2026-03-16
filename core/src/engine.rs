use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use crate::git::GitRepo;
use crate::health;
use crate::index::{is_indexable_path, CodeIndex};
use crate::memory::{MemoryEvent, MemoryStore};
use crate::migrate;
use crate::migrate::detect::Language;
use crate::migrate::orchestrate::MigrationConfig;
use crate::migration;
use crate::model::{CodexModel, SearchProvider};
use crate::parser::{parse_rust_file, ParsedSymbolKind};
use crate::persona::Persona;
use crate::scaffold;
use crate::teams::TeamManager;
use crate::time_travel;
use crate::ts_migrate;

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
}

impl CodexEngine {
    pub fn new() -> Self {
        let root = PathBuf::from(".");
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            search: None,
            memory: MemoryStore::load(memory_path),
            git,
            persona,
        }
    }

    pub fn with_root(root: PathBuf) -> Self {
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            search: None,
            memory: MemoryStore::load(memory_path),
            git,
            persona,
        }
    }

    pub fn with_model(root: PathBuf, model: Box<dyn CodexModel + Send + Sync>) -> Self {
        let memory_path = resolve_memory_path(&root);
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: Some(model),
            search: None,
            memory: MemoryStore::load(memory_path),
            git,
            persona,
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

        if normalized.to_ascii_lowercase().starts_with("migrate ") {
            let tokens: Vec<&str> = normalized.split_whitespace().collect();
            if tokens.len() >= 8 {
                let mut from_idx = None;
                let mut to_idx = None;
                let mut out_idx = None;
                for (i, t) in tokens.iter().enumerate() {
                    match *t {
                        "from" if i + 1 < tokens.len() => from_idx = Some(i),
                        "to" if i + 1 < tokens.len() => to_idx = Some(i),
                        "output" if i + 1 < tokens.len() => out_idx = Some(i),
                        _ => {}
                    }
                }
                if let (Some(fi), Some(ti), Some(oi)) = (from_idx, to_idx, out_idx) {
                    let src = tokens[1];
                    let from_lang = tokens[fi + 1];
                    let to_lang = tokens[ti + 1];
                    let out_dir = tokens[oi + 1];
                    let use_ai = tokens.iter().any(|t| *t == "--ai");
                    let use_clean = tokens.iter().any(|t| *t == "--clean");
                    let mut cmd = format!(
                        ":migrate {} {} {} {}",
                        src, from_lang, to_lang, out_dir
                    );
                    if use_ai {
                        cmd.push_str(" --ai");
                    }
                    if use_clean {
                        cmd.push_str(" --clean");
                    }
                    return self.handle_input(&cmd);
                }
            }
        }

        if let Some(cmd) = self.intent_for(trimmed) {
            return self.handle_input(cmd);
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
            return Ok(message);
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
                knowledge,
            };

            let model_ref: Option<&(dyn CodexModel + Send + Sync)> =
                self.model.as_ref().map(|m| m.as_ref());
            let result = migrate::run_migration(&config, model_ref)?;

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
                self.memory.search(query, 10)
            } else {
                self.memory.recent(10)
            };

            if results.is_empty() {
                return Ok("\u{1f5d1}\u{fe0f} No memories found yet. Try `:learn <fact>`!".to_string());
            }

            let mut out = String::from("\u{1f9e0} **Astra's Memory Bank**\n\n");
            for m in results {
                let date = chrono::NaiveDateTime::from_timestamp_opt(m.timestamp as i64, 0)
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
                        self.memory.add(
                            "source-doc",
                            format!("File: {}\nContents:\n{}", filename, contents)
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
        if let Some(model) = &self.model {
            let stats = self.index.stats();
            let by_lang = self.index.files_by_language();
            let mut matches = self.memory.search(question, 10); // Search memory more aggressively
            let recent = self.memory.recent(5);
            
            // --- NEW: Always prioritize project documentation (README/Vision) in context ---
            let docs = self.memory.events_of_kind("source-doc");
            for doc in docs.iter().rev().take(3) {
                if !matches.iter().any(|m| m.content == doc.content) {
                    matches.push((*doc).clone());
                }
            }
            
            // --- NEW: Add Git history context if relevant ---
            if question.to_lowercase().contains("change") || question.to_lowercase().contains("commit") || question.to_lowercase().contains("history") {
                let git_events = self.memory.events_of_kind("git-commit");
                for evt in git_events.iter().rev().take(3) {
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
            
            // Inject Memory context for "Proactive Triggering"
            if !matches.is_empty() {
                let _ = writeln!(&mut prompt, "\n### IMPORTANT PROJECT MEMORY (PRIORITIZE THIS):");
                for entry in matches {
                    let _ = writeln!(&mut prompt, "- [{}] {}", entry.kind, entry.content);
                }
                let _ = writeln!(&mut prompt, "\nIf the memory contains personal info (like the user's name) or specific project goals, use them directly in your response.");
            }
            
            let _ = writeln!(&mut prompt, "\n### Project Stats:");
            let _ = writeln!(&mut prompt, "- Root: {:?}", self.root);
            let _ = writeln!(&mut prompt, "- Indexed: {} files, {} lines.", stats.file_count, stats.total_lines);

            let _ = writeln!(&mut prompt, "\nUser question/task: {}", question);

            let answer = model.complete(&prompt)?;

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
        } else {
            if let Some(answer) = self.memory_answer(question) {
                self.memory
                    .add("qa-memory", format!("Q: {}\nA: {}", question, answer));
                Ok(answer)
            } else {
                Ok(
                    "No language model is configured. Try :summary or :memory <query>."
                        .to_string(),
                )
            }
        }
    }

    fn intent_for(&self, trimmed: &str) -> Option<&'static str> {
        if trimmed.starts_with(':') || trimmed.starts_with("? ") {
            return None;
        }
        let lower = trimmed.to_ascii_lowercase();

        if lower.contains("what do you remember")
            || (lower.contains("remember") && lower.contains("what"))
        {
            return Some(":memory");
        }

        if lower.contains("what do you know")
            || lower.contains("project summary")
            || lower.contains("project info")
            || lower.contains("project information")
        {
            return Some(":summary");
        }

        if (lower.contains("how many") && lower.contains("file"))
            || lower.contains("files by language")
            || lower.contains("files-by-lang")
        {
            return Some(":files-by-lang");
        }

        if lower.contains("git repo") || lower.contains("git repository") {
            return Some(":summary");
        }

        if (lower.contains("how many") && lower.contains("commit"))
            || lower.contains("commit count")
        {
            return Some(":git-commit-count");
        }

        if lower.contains("last commit")
            || lower.contains("recent commit")
            || lower.contains("most recent commit")
            || lower.contains("when did i make")
            || lower.contains("when did i commit")
            || lower.contains("when was my last commit")
        {
            return Some(":git-last-commit");
        }

        if lower.contains("health check") || lower == "health" {
            return Some(":health");
        }

        if lower.contains("graph") || lower.contains("semantic graph") {
            return Some(":graph");
        }

        None
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
}

fn resolve_memory_path(root: &Path) -> PathBuf {
    let preferred = root.join(".astra").join("memory.json");
    if preferred.exists() {
        return preferred;
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
