use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;
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
use crate::model::CodexModel;
use crate::parser::{parse_rust_file, RustSymbolKind};
use crate::persona::Persona;
use crate::scaffold;
use crate::teams::TeamManager;
use crate::time_travel;
use crate::ts_migrate;

pub struct CodexEngine {
    root: PathBuf,
    index: CodeIndex,
    model: Option<Box<dyn CodexModel + Send + Sync>>,
    memory: MemoryStore,
    git: Option<GitRepo>,
    persona: Persona,
}

impl CodexEngine {
    pub fn new() -> Self {
        let root = PathBuf::from(".");
        let memory_path = root.join(".codex").join("memory.json");
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            memory: MemoryStore::load(memory_path),
            git,
            persona,
        }
    }

    pub fn with_root(root: PathBuf) -> Self {
        let memory_path = root.join(".codex").join("memory.json");
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: None,
            memory: MemoryStore::load(memory_path),
            git,
            persona,
        }
    }

    pub fn with_model(root: PathBuf, model: Box<dyn CodexModel + Send + Sync>) -> Self {
        let memory_path = root.join(".codex").join("memory.json");
        let git = GitRepo::discover(&root).ok();
        let persona = Persona::load(&root);
        Self {
            root,
            index: CodeIndex::new(),
            model: Some(model),
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

    pub fn handle_input(&mut self, input: &str) -> Result<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok("Say something about your codebase to get started.".to_string());
        }

        let mut normalized = trimmed.to_string();
        if normalized.starts_with('›') {
            normalized = normalized.trim_start_matches('›').trim_start().to_string();
        }

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
                    let mut cmd = format!(
                        ":migrate {} {} {} {}",
                        src, from_lang, to_lang, out_dir
                    );
                    if use_ai {
                        cmd.push_str(" --ai");
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
            let message = format!(
                "Indexed {} files with a total of {} lines.",
                stats.file_count, stats.total_lines
            );
            self.memory
                .add("index", format!("root: {:?}, {}", self.root, message));
            return Ok(message);
        }

        if trimmed == ":memory" {
            let recent = self.memory.recent(5);
            if recent.is_empty() {
                return Ok("Memory is empty.".to_string());
            }
            let mut out = String::new();
            for entry in recent {
                let _ = writeln!(&mut out, "- [{}] {}", entry.kind, entry.content);
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

        if trimmed == ":summary" {
            let stats = self.index.stats();
            let has_git = self.git.is_some();
            let recent = self.memory.recent(5);

            let mut summary = String::new();
            let _ = writeln!(&mut summary, "Project root: {:?}", self.root);
            let _ = writeln!(
                &mut summary,
                "Indexed files: {}",
                stats.file_count
            );
            let _ = writeln!(&mut summary, "Total lines: {}", stats.total_lines);
            let _ = writeln!(
                &mut summary,
                "Git repository detected: {}",
                if has_git { "yes" } else { "no" }
            );
            if !recent.is_empty() {
                let _ = writeln!(&mut summary, "Recent memory:");
                for entry in recent {
                    let _ = writeln!(&mut summary, "- [{}] {}", entry.kind, entry.content);
                }
            }

            if let Some(model) = &self.model {
                let mut prompt = String::new();
                let _ = writeln!(
                    &mut prompt,
                    "You are codex. Summarize this project information for the user:\n{}",
                    summary
                );
                let answer = model.complete(&prompt)?;
                self.memory
                    .add("summary", answer.clone());
                return Ok(answer);
            }

            return Ok(summary);
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
            self.set_persona(Persona::from_vibe(vibe_name));
            return Ok(format!("Vibe changed to '{}'!", vibe_name));
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
                    RustSymbolKind::Struct => "struct",
                    RustSymbolKind::Enum => "enum",
                    RustSymbolKind::Function => "fn",
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
                let _ = writeln!(&mut prompt);
                let _ = writeln!(&mut prompt, "File path: {:?}", path);
                let _ = writeln!(&mut prompt, "Current file contents:");
                let _ = writeln!(&mut prompt, "{}", contents);

                let fixed_raw = model.complete(&prompt)?;
                let fixed = Self::strip_markdown_fences(&fixed_raw);

                let backup_path = path.with_extension("codex.bak");
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
            let use_ai = parts.get(4).map(|s| *s == "--ai").unwrap_or(false);

            let config = MigrationConfig {
                source_dir,
                output_dir,
                from_lang,
                to_lang,
                use_ai,
            };

            let model_ref: Option<&(dyn CodexModel + Send + Sync)> =
                self.model.as_ref().map(|m| m.as_ref());
            let result = migrate::run_migration(&config, model_ref)?;

            self.memory.add(
                "migration",
                format!(
                    "{} → {}: {} files migrated",
                    config.from_lang,
                    config.to_lang,
                    result.migrated.len()
                ),
            );

            let mut out = String::new();
            let _ = writeln!(&mut out, "{}", result.plan_text);
            out.push('\n');
            out.push_str(&result.scaffold_log);
            out.push_str(&result.summary());
            return Ok(out);
        }

        if let Some(rest) = trimmed.strip_prefix("? ") {
            if let Some(model) = &self.model {
                let stats = self.index.stats();
                let recent = self.memory.recent(3);
                let mut prompt = String::new();
                
                // INJECT PERSONA DIRECTIVE
                let _ = writeln!(&mut prompt, "{}", self.persona.system_prompt());
                
                let _ = writeln!(
                    &mut prompt,
                    "\nYou are codex, a local codebase assistant. Project root: {:?}.",
                    self.root
                );
                let _ = writeln!(
                    &mut prompt,
                    "Indexed files: {}. Total lines: {}.",
                    stats.file_count, stats.total_lines
                );
                if !recent.is_empty() {
                    let _ = writeln!(&mut prompt, "Recent memory:");
                    for entry in recent {
                        let _ = writeln!(&mut prompt, "- [{}] {}", entry.kind, entry.content);
                    }
                }
                let _ = writeln!(&mut prompt, "User question: {}", rest);
                let answer = model.complete(&prompt)?;
                self.memory
                    .add("qa", format!("Q: {}\nA: {}", rest, answer));
                return Ok(answer);
            } else {
                return Ok("No language model is configured.".to_string());
            }
        }

        Ok(format!(
            "codex did not understand this yet:\n  {}\n\nTry one of:\n  :index\n  :summary\n  :memory\n  :migrate <src> <from> <to> <out> [--ai]\n  ? <question>",
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
                index.add_file(path, &contents);
            }
        }

        self.index = index;
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

        None
    }
}
