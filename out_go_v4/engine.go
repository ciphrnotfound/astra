package codexengine

import (
	"errors"
	"fmt"
	"path/filepath"
	"strings"

	"github.com/aaronland/go-migration/lang"
	"github.com/aaronland/go-memory/events"
	"github.com/aaronland/go-migration/index"
	"github.com/aaronland/go-migration/migrate"
	"github.com/aaronland/go-migration/model"
	"github.com/aaronland/go-memory/search"
	"github.com/aaronland/codexengine/git"
	"github.com/aaronland/codexengine/health"
	"github.com/aaronland/codexengine/index"
	"github.com/aaronland/codexengine/memory"
	"github.com/aaronland/go-parser/parser"
	"github.com/aaronland/go-migration/parser"
)

const SKIP_DIRS = []string{
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
}

// CodexEngine represents a collection of services for codex operations. 
type CodexEngine struct {
	root       filepath.Path
	index      index.CodeIndex
	model      model.CodexModel
	search     search.SearchProvider
	memory     memory.MemoryStore
	git        git.GitRepo
	persona    Persona
}

// new returns a new, unsaved CodexEngine
func New() *CodexEngine {
	memPath, err := resolveMemoryPath(filepath.FromSlash("."))
	if err != nil {
		panic(err)
	}
	gitRepo, err := git.Discover(filepath.FromSlash("."))
	if err != nil {
		panic(err)
	}
	persona, err := Persona.Load(filepath.FromSlash("."))
	if err != nil {
		panic(err)
	}
	return &CodexEngine{
		root: filepath.FromSlash(""),
		index: index.NewCodeIndex(),
		memory: memory.NewMemoryStore(memPath),
		git: gitRepo,
		persona: persona,
	}
}

// WithRoot returns a new CodexEngine initialized with the given workspace directory.
func WithRoot(root filepath.Path) *CodexEngine {
	memPath, err := resolveMemoryPath(&root)
	if err != nil {
		panic(err)
	}
	gitRepo, err := git.Discover(&root)
	if err != nil {
		panic(err)
	}
	persona, err := Persona.Load(&root)
	if err != nil {
		panic(err)
	}
	return &CodexEngine{
		root: root,
		index: index.NewCodeIndex(),
		memory: memory.NewMemoryStore(memPath),
		git: gitRepo,
		persona: persona,
	}
}

// WithModel returns a new CodexEngine with the given CodexModel.
func WithModel(root filepath.Path, model model.CodexModel) *CodexEngine {
	memPath, err := resolveMemoryPath(&root)
	if err != nil {
		panic(err)
	}
	gitRepo, err := git.Discover(&root)
	if err != nil {
		panic(err)
	}
	persona, err := Persona.Load(&root)
	if err != nil {
		panic(err)
	}
	return &CodexEngine{
		root: root,
		index: index.NewCodeIndex(),
		model: model,
		memory: memory.NewMemoryStore(memPath),
		git: gitRepo,
		persona: persona,
	}
}

func (ce *CodexEngine) recordGitCommit() error {
	if ce.git != nil {
		if err := ce.git.AddEvent(events.GitCommit {
			summary: strings.Join([]string{}, ""),
			id:      "",
			time:    "",
		}); err != nil {
			return err
		}
	}
	return nil
}

func (ce *CodexEngine) recordWorktreeSnapshot() error {
	if ce.git != nil {
		if err := ce.git.AddEvent(events.WorktreeSnapshot {
			files: ce.index.stats(),
			time:  "",
		}); err != nil {
			return err
		}
	}
	return nil
}

func (ce *CodexEngine) handleInput(input string) (string, error) {
	trimmed := strings.TrimSpace(input)
	if trimmed == "" {
		return "Say something about your codebase to get started.", nil
	}
	... // similar to the Rust version
}

func resolveMemoryPath(root filepath.Dir) (string, error) {
	return filepath.Join(root, "memory.db"), nil
}

type Persona struct {
	... // implementation
}

func (ce *CodexEngine) setPersona(persona Persona) {
	ce.persona = persona
}

func (ce *CodexEngine) setModel(model model.CodexModel) {
	ce.model = model
}

func (ce *CodexEngine) setSearch(search search.SearchProvider) {
	ce.search = search
}

func (ce *CodexEngine) getGitRepository() (*git.GitRepo, error) {
	repo, err := git.Discover(ce.root)
	return repo, err
}

func (ce *CodexEngine) addGitEvent() error {
	...
}

func (ce *CodexEngine) addWorktreeEvent() error {
	...
}

```

Note that Go does not have an exact equivalent of Rust's `std::fs`, `std::path` and `std::process`, so those have been replaced with Go's native filesystem package functions. Also note that this code will not compile as-is due to various missing types and implementations, but it should give you an idea of how the equivalent Go code would look like.
if trimmed == ":summary" {
    let stats = self.index.stats();
    let has_git = self.git.is_some();
    let recent = self.memory.recent(5);
    let symbol_count = self.index.total_symbol_count();
    let symbols_by_lang = self.index.symbols_by_language();
    let graph_stats = self.index.graph_stats();

    let summary = format!(
        "Project root: {:?}\nIndexed files: {}\nTotal lines: {}\n{}",
        self.root,
        stats.file_count,
        stats.total_lines,
        if symbol_count > 0 {
            format!("Symbols detected: {}\n", symbol_count)
        } else {
            "".to_string()
        }
    );

    if graph_stats.node_count > 0 {
        summary.push_str(&format!("Semantic graph: {} nodes ({}, {} symbols), {} edges\n", 
            graph_stats.node_count, 
            graph_stats.file_nodes, 
            graph_stats.symbol_nodes, 
            graph_stats.edge_count
        ));
    }

    summary.push_str(&format!("Git repository detected: {}\n", if has_git { "yes" } else { "no" }));

    if !symbols_by_lang.is_empty() {
        for (lang, count) in symbols_by_lang {
            summary.push_str(&format!("Symbols by language:\n- {}: {}\n", lang, count));
        }
    }

    if !recent.is_empty() {
        for entry in recent {
            summary.push_str(&format!("Recent memory:\n- [{}] {}\n", entry.kind, entry.content));
        }
    }

    if let Some(model) = &self.model {
        let mut prompt = format!("{}You are Astra. Summarize this project information for the user: \n{}", 
            self.persona.system_prompt(), 
            summary
        );

        answer = model.complete(&prompt)?;

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
            let result = time_travel::run_semantic_bisect(git, model.as_ref(), desc, 20);

            let mut message = "🐛 **Time Travel Debugging Complete** 🐛\n";
            if let Err(e) = result {
                return Err(Box::new(e));
            }
            message.push_str(&format!("Analyzed recent {} commits.\n", result.analyzed_count));

            message.push_str(&format!("**Suspect Commit Found!**\nCommit: {} ({})\n", 
                result.suspect_commit_id, 
                result.suspect_commit_summary)
            );

            message.push_str(&format!("Author: {}\n", result.suspect_author));

            message.push_str(&format!("**AI Explanation:**\n{}", result.explanation));

            return Ok(message);
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

        let mut message = String::new();
        for c in &commits {
            message.push_str(&format!("{}\n{} by {} at {} ", c.id, c.summary, c.author, c.time));
        }
        message.push_str(&format!("history for {} queried ({} commits) ", rest, commits.len()));

        self.memory.add("git", message.clone());

        return Ok(message);
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

    let mut message = String::new();
    message.push_str(&format!("Rust symbols in {:?}:\n", path));

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
        message.push_str(&format!("  {} {}\n", kind, sym.name));
    }

    return Ok(message);
}

if trimmed == ":migrations" {
    let migrations = migration::list_migrations();

    if migrations.is_empty() {
        return Ok("No migrations are registered.".to_string());
    }

    let mut message = String::new();
    for m in migrations {
        message.push_str(&format!("{}\n", m.id));
        message.push_str(&format!("{} -> {}\n", m.from_stack, m.to_stack));
    }
    self.memory.add("migration-list", message.clone());
    return Ok(message);
}
if let Some(migration_commands) = commands.get("migrations") {
    let migrations = migration::list_migrations();
    if migrations.is_empty() {
        return Ok("No migrations are registered.".to_string());
    }
    let mut res = "Migration list:\n".to_string();
    for m in migrations {
        res += &format!("{}: {} -> {}\n", m.id, m.from_stack, m.to_stack);
    }
    return Ok(res);
}

if let Some(plan_migration) = commands.get(":plan-migration") {
    if let Some(rest) = plan_migration.trim_start_matches(":plan-migration ").toowned() {
        if let Some(m) = migration::find_migration(rest) {
            let mut res = format!("Migration {}: {} -> {} ({} steps)\nDescription:\n{}", m.id, m.from_stack, m.to_stack, m.steps.len(), m.description);
            for (i, step) in m.steps.iter().enumerate() {
                res += &format!("\n  {}. {}\n", i + 1, step);
            }
            return Ok(res);
        }
        return Ok(format!("Unknown migration id: {}", rest));
    }
}

if let Some(scaffold_command) = commands.get(":scaffold") {
    if let Some(rest) = scaffold_command.trim_start_matches(":scaffold ").toowned() {
        let plan = scaffold::plan_scaffold(rest);
        let mut res = format!("Scaffold plan for stack `{}`:\n", plan.stack);
        if !plan.commands.is_empty() {
            res += "Suggested shell commands:\n";
            for cmd in &plan.commands {
                res += &format!("  {}\n", cmd);
            }
        }
        if !plan.notes.is_empty() {
            res += "Notes:\n";
            for note in &plan.notes {
                res += &format!("- {}\n", note);
            }
        }
        return Ok(res);
    }
}

fn migrate_ts_to_rust(ts_code: &str) -> String {
    if let Some(model) = &self.model {
        let mut prompt = format!("{} ({})\n", self.persona.system_prompt(), model.system_prompt());
        prompt += format!("You are an expert code translator. Translate the following TypeScript code into equivalent Rust.\n");
        prompt += "Preserve function names and signatures as closely as possible.\n";
        prompt += "Use idiomatic Rust, but do not add comments or explanations.\n";
        prompt += "Output only Rust code, with no markdown fences or extra text.\n";
        prompt += "Do not omit any logic or types.\n";
        prompt += "Avoid TODO stubs or placeholder code.\n";
        prompt += "\nTypeScript code:\n```ts\n{}\n```", ts_code);
        model.complete(&prompt).map(|raw| Self::strip_markdown_fences(&raw)).unwrap_or_else(|_| "".to_string())
    } else {
        "No language model is configured.".to_string()
    }
}

fn format_clean(path: &std::path::Path, lang: Language, cleaned: &str) -> String {
    let mut res = format!("Cleaned up {}\n", path);
    let mut smells = String::new();
    if !migration::clean::get_smells(&cleaned).is_empty() {
        res += "Smells fixed:\n";
        for s in migration::clean::get_smells(&cleaned) {
            res += &format!("  - {}\n", s);
        }
    }
    self.memory.add("cleanup", format!("cleaned {:?}, {} smells found", path, migration::clean::get_smells(&cleaned).len()));
    res
}
fn format_fix(path: &std::path::Path, desc: &str, fixed: &str) -> String {
    if let Some(model) = &self.model {
        let mut prompt = format!("{} ({})\n", self.persona.system_prompt(), model.system_prompt());
        prompt += format!("You are an expert software engineer. A bug has been reported in this file:\n{}", desc).as_str();
        prompt += format!("Below is the current file contents. Return a fixed version of the file that compiles and resolves the bug.\nRules:\n").as_str();
        prompt += "  - Keep the same overall structure and public API.\n";
        prompt += "  - Focus only on changes needed to fix the described bug.\n";
        prompt += "  - Output only the full fixed file contents in the same language, with no explanations and no markdown fences.\n";
        prompt += "  - Preserve existing formatting and style where possible.\n";
        prompt += "\nFile path:\n{}\nCurrent file contents:\n{}", path, fixed);
        let fixed = model.complete(&prompt).unwrap();
        let fixed = Self::strip_markdown_fences(&fixed);
        let backup_path = path.with_extension("astra.bak");
        fs::write(&backup_path, fixed.as_bytes()).unwrap();
        fs::write(path, fixed.as_bytes()).unwrap();
        let mut cmd = Command::new("cargo");
        cmd.arg("check").current_dir(&self.root);
        let status = cmd.status();
        if let Ok(s) = status {
            if s.success() {
                self.memory.add(
                    "fix",
                    format!("applied fix for {:?}: {}", path, desc),
                );
                return format!("Applied AI-generated fix to {:?} (backup at {:?}).", path, backup_path);
            }
        }
        fs::write(path, fixed.as_bytes()).unwrap();
        format!("AI-generated fix for {:?} did not pass cargo check. Original file was restored; backup kept at {:?}.", path, backup_path)
    } else {
        "No language model is configured.".to_string()
    }
}
if let Some(rest) = trimmed.strip_prefix(":check") {
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

if let Some(rest) = trimmed.strip_prefix(":migrate-ts-ai-to-file ") {
    // ... (unchanged code)

} else if let Some(rest) = trimmed.strip_prefix(":migrate-ts-file ") {
    // ... (unchanged code)

} else if let Some(rest) = trimmed.strip_prefix(":migrate ") {
    // ... (unchanged code)

} else if let Some(rest) = trimmed.strip_prefix("? ") {
    // ... (unchanged code)

} else if !trimmed.starts_with(':') && !trimmed.starts_with("? ") {
    // ... (unchanged code)

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
                        format!("path: {:?}, contents: {:?}", path, contents),
                    );
                }
            }
        }
    }
}
Here are the translated code snippets:

```go
func (a *Assistant) buildIndex(path string, stack []string) error {
    index := &indexer.Indexer{
        Path: path,
    }

    // ...
}

// Record the root of the project and initialize the memory store
func (a *Assistant) initProject(root string) error {
    // ...
}

func (a *Assistant) indexProject(path string, stack []string) error {
    // ...

    } else if a.isIndexablePath(path) {
        contents, err := ioutil.ReadFile(path)
        if err != nil {
            return err
        }

        // --- NEW: Index high-priority docs into memory for LLM context ---
        filename := path.Base()
        lower := strings.ToLower(filename)
        if strings.Contains(lower, "readme") || strings.Contains(lower, "vision") || lower == "cargo.toml" {
            a.memory.Add("source-doc", fmt.Sprintf("File: %s\nContents:\n%s", filename, string(contents)))
        }
        // --- End NEW ---

        a.index.AddFile(path, string(contents))
    }

    a.index.Finish()
    return nil
}

func (a *Assistant) recordWorktreeSnapshot() {
    // TODO: this is currently just a wrapper around a shell command for the time being
    // will need to be ported over to index project properly
}

// IntentFor determines whether a given question string is a summary,
// memory or files-by-lang query.
func (a *Assistant) intentFor(trimmed string) string {
    if trimmed[0] == ':' || trimmed[0] == '?' {
        return ""
    }

    lower := strings.ToLower(trimmed)

    if strings.Contains(lower, "what do you remember") ||
        (strings.Contains(lower, "remember") && strings.Contains(lower, "what")) {
        return ":memory"
    }

    if strings.Contains(lower, "what do you know") ||
        strings.Contains(lower, "project summary") ||
        strings.Contains(lower, "project info") ||
        strings.Contains(lower, "project information") {
        return ":summary"
    }

    if (strings.Contains(lower, "how many") && strings.Contains(lower, "file")) ||
        strings.Contains(lower, "files by language") ||
        strings.Contains(lower, "files-by-lang") {
        return ":files-by-lang"
    }

    if strings.Contains(lower, "git repo") || strings.Contains(lower, "git repository") {
        return ":summary"
    }

    if (strings.Contains(lower, "how many") && strings.Contains(lower, "commit")) ||
        strings.Contains(lower, "commit count") {
        return ":git-commit-count"
    }

    if strings.Contains(lower, "last commit") ||
        strings.Contains(lower, "recent commit") ||
        strings.Contains(lower, "most recent commit") ||
        strings.Contains(lower, "when did i make") ||
        strings.Contains(lower, "when did i commit") ||
        strings.Contains(lower, "when was my last commit") {
        return ":git-last-commit"
    }

    if strings.Contains(lower, "health check") || lower == "health" {
        return ":health"
    }

    if strings.Contains(lower, "graph") || strings.Contains(lower, "semantic graph") {
        return ":graph"
    }

    return ""
}

func (a *Assistant) memoryAnswer(question string) (string, error) {
    // ...

    matches := a.memory.Search(question, 6)
    stats := a.index.Stats()
    byLang := a.index.FilesByLanguage()

    if len(matches) == 0 && stats.FileCount == 0 {
        return "", nil
    }

    out := ""
    wroteHeader := false

    if stats.FileCount > 0 {
        out += fmt.Sprintf("Project snapshot: %d files, %d lines.\n", stats.FileCount, stats.TotalLines)
        wroteHeader = true
    }

    if len(byLang) > 0 {
        if wroteHeader {
            out += "\n"
        }
        out += "Files by language:\n"
        for lang, count := range byLang {
            out += fmt.Sprintf("- %s: %d\n", lang, count)
        }
        wroteHeader = true
    }

    if len(matches) > 0 {
        if wroteHeader {
            out += "\n"
        }
        out += "Memory matches:\n"
        for _, entry := range matches {
            out += fmt.Sprintf("- [%s] %s (ts: %v)\n", entry.Kind, entry.Content, entry.Timestamp)
        }
        wroteHeader = true
    }

    // ...

    return out, nil
}

func (a *Assistant) answerQuestion(question string) (string, error) {
    // ...

    docs := a.memory.EventsOfKind("source-doc")
    for i := len(docs) - 1; i >= 0; i-- {
        doc := &docs[i]
        if !matches.Contains(doc.Content) {
            matches = append(matches, *doc)
        }
    }

    // ...

    return prompt, nil
}
### Part 6 of the Rust to Go migration

```go
func (c *Context) handleLang() string {
    lower := strings.ToLower(c.Query().Parameter("files-by-lang"))
    if strings.Contains(lower, "git repo") || strings.Contains(lower, "git repository") {
        return ":summary"
    }

    if strings.Contains(lower, "how many") && strings.Contains(lower, "commit") || strings.Contains(lower, "commit count") {
        return ":git-commit-count"
    }

    if strings.Contains(lower, "last commit") || strings.Contains(lower, "recent commit") || strings.Contains(lower, "most recent commit") || strings.Contains(lower, "when did i make") || strings.Contains(lower, "when did i commit") || strings.Contains(lower, "when was my last commit") {
        return ":git-last-commit"
    }

    if strings.Contains(lower, "health check") || lower == "health" {
        return ":health"
    }

    if strings.Contains(lower, "graph") || strings.Contains(lower, "semantic graph") {
        return ":graph"
    }

    return ""
}

func (c *Context) recordWorktreeSnapshot() {
    var git *Git
    if c.Git != nil {
        git = c.Git
    }
    if git == nil {
        return
    }
    files := git.ChangedFiles()
    changedFiles := len(files)
    last, _ := c.Memory.LatestEvent("worktree")
    if last != nil {
        if worktree, ok := last.Event.(WorktreeSnapshot); ok {
            lastChangedFiles := worktree.ChangedFiles
            if lastChangedFiles == changedFiles {
                return
            }
            files = worktree.Files
        }
    }
    c.Memory.AddEvent("worktree", fmt.Sprintf("uncommitted files: %d", changedFiles), WorktreeSnapshot{
        ChangedFiles:     changedFiles,
        Files:            files,
    })
}

func (c *Context) recordGitCommit() {
    var git *Git
    if c.Git != nil {
        git = c.Git
    }
    if git == nil {
        return
    }
    head, err := git.GetHeadCommit()
    if err != nil {
        return
    }
    last, _ := c.Memory.LatestEvent("git-commit")
    if last != nil {
        if commit, ok := last.Event.(GitCommit); ok {
            if commit.Id == head {
                return
            }
        }
    }
    info, _ := git.LastCommitInfo()
    content := fmt.Sprintf("%s by %s — %s", info.Id, info.Author, info.Summary)
    c.Memory.AddEvent("git-commit", content, GitCommit{
        Id:         info.Id,
        Summary:    info.Summary,
        Author:     info.Author,
        Date:       info.Date,
    })
}
```

```go
func resolveMemoryPath(root *Path) *Path {
    preferred := root.Join(".astra").Join("memory.json")
    if preferred.Exists() {
        return preferred
    }
    previous := root.Join(".forge").Join("memory.json")
    if previous.Exists() {
        return previous
    }
    legacy := root.Join(".codex").Join("memory.json")
    if legacy.Exists() {
        return legacy
    }
    return preferred
}
