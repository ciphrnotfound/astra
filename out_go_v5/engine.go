package main

import (
    "bufio"
    "fmt"
    "io"
    "log"
    "os"
    "path/filepath"
    "strings"

    "github.com/codex-engine/codex-go/git"
    "github.com/codex-engine/codex-go/migration"
    "github.com/codex-engine/codex-go/model"
    "github.com/codex-engine/codex-go/move"
    "github.com/codex-engine/codex-go/search"
    "github.com/codex-engine/codex-go/teams"
)

const skipDirs = []string{
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

type CodexEngine struct {
    Root   filepath.Path
    Index  *migration.CodeIndex
    Model  *model.CodexModel
    Search *search.SearchProvider
    Memory *move.MemoryStore
    Git    *git.GitRepo
    Persona Persona
}

func NewCodexEngine() *CodexEngine {
    return codexEngine()
}

func codexEngine() *CodexEngine {
    root := filepath.FromSlash(".")
    memoryPath := resolveMemoryPath(root)
    git := git.Discover(root).OK()
    persona := Persona.Load(root)
    return &CodexEngine{
        Root:   root,
        Index:  migration.NewCodeIndex(),
        Model:  nil,
        Search: nil,
        Memory: move.LoadMemoryStore(memoryPath),
        Git:    git,
        Persona: persona,
    }
}

func (ce *CodexEngine) WithRoot(root filepath.Path) *CodexEngine {
    return codexEngineWithRoot(root)
}

func codexEngineWithRoot(root filepath.Path) *CodexEngine {
    memoryPath := resolveMemoryPath(root)
    git := git.Discover(root).OK()
    persona := Persona.Load(root)
    return &CodexEngine{
        Root:   root,
        Index:  migration.NewCodeIndex(),
        Model:  nil,
        Search: nil,
        Memory: move.LoadMemoryStore(memoryPath),
        Git:    git,
        Persona: persona,
    }
}

func (ce *CodexEngine) WithModel(root filepath.Path, model model.CodexModel) *CodexEngine {
    return codexEngineWithModel(root, model)
}

func codexEngineWithModel(root filepath.Path, model model.CodexModel) *CodexEngine {
    memoryPath := resolveMemoryPath(root)
    git := git.Discover(root).OK()
    persona := Persona.Load(root)
    return &CodexEngine{
        Root:   root,
        Index:  migration.NewCodeIndex(),
        Model:  &model,
        Search: nil,
        Memory: move.LoadMemoryStore(memoryPath),
        Git:    git,
        Persona: persona,
    }
}

func (ce *CodexEngine) SetPersona(persona Persona) {
    ce.Persona = persona
}

func (ce *CodexEngine) SetModel(model model.CodexModel) {
    ce.Model = &model
}

func (ce *CodexEngine) SetSearch(search search.SearchProvider) {
    ce.Search = &search
}

type CodexModel interface {
    Complete(prompt string) (string, error)
}

type SearchProvider interface {
    Search(query string) (string, error)
}

type Persona struct {
    // fields
}

func PersonaLoad(root filepath.Path) Persona {
    // implementation
}

func PersonaLoadFile(filePath string) Persona {
    // implementation
}

func (ce *CodexEngine) IntentFor(query string) string {
    return ""
}

func resolveMemoryPath(root filepath.Path) filepath.Path {
    // implementation
}

func (ce *CodexEngine) RecordGitCommit() {
    if ce.Git != nil {
        // implementation
    }
}

func (ce *CodexEngine) RecordWorktreeSnapshot() {
    // implementation
}

func (ce *CodexEngine) HandleInput(input string) (string, error) {
    trimmed := strings.TrimSpace(input)
    if trimmed == "" {
        return "Say something about your codebase to get started.", nil
    }

    normalized := trimmed
    if normalized != "" && strings.HasPrefix(normalized, "›") {
        normalized = normalized[1:]
    }

    if strings.HasPrefix(strings.ToLower(normalized), "migrate") {
        tokens := strings.Fields(normalized)
        if len(tokens) >= 8 {
            // implementation
        }
    }

    if normalized == ":index" {
        return ce.buildIndex(), nil
    }

    if strings.HasPrefix(normalized, ":memory ") {
        // implementation
    }

    if normalized == ":memory" {
        // implementation
    }

    if normalized == ":files-by-lang" {
        // implementation
    }

    if strings.HasPrefix(normalized, ":web ") {
        // implementation
    }

    if normalized == ":summary" {
        // implementation
        return "", nil
    }

    return "", nil
}

func (ce *CodexEngine) BuildIndex() error {
    return nil
}
package main

import (
	"os"
	"path/filepath"

	"github.com/alecthomas/participle/v2/lexer"
	"github.com/alecthomas/participle/v2/selector"
	"go.astra.dev/astra/pkg/model"
	"go.astra.dev/astra/pkg/persona"
	"go.astra.dev/astra/pkg/teams/team_mgr"
	"astra.dev/go/astrapkg"
	"astra.dev/go/astrapkg/dot"
	"astra.dev/go/astrapkg/git"
	"astra.dev/go/astrapkg/migration"
	"astra.dev/go/astrapkg/migration/health"
	"astra.dev/go/astrapkg/time_travel"
)

func (m *Model) processCommand(trimmed string) error {
	if trimmed == ":summary" {
		symbolsbylang := m.index.SymbolsByLanguage()
		graphstats := m.index.GraphStats()
		summary := ""

		file_count := m.index.Stats().FileCount
		total_lines := m.index.Stats().TotalLines
		symbol_count := m.index.TotalSymbolCount()

		summary += fmt.Sprintf("Project root: %v\n", m.root)
		summary += fmt.Sprintf("Indexed files: %d\n", file_count)
		summary += fmt.Sprintf("Total lines: %d\n", total_lines)

		if symbol_count > 0 {
			summary += fmt.Sprintf("Symbols detected: %d\n", symbol_count)
		}

		if graphstats.NodeCount > 0 {
			summary += fmt.Sprintf("Semantic graph: %d nodes (%d files, %d symbols), %d edges\n",
				graphstats.NodeCount, graphstats.FileNodes, graphstats.SymbolNodes, graphstats.EdgeCount)
		}

		if has_git := m.Git.IsSome(); has_git {
			summary += fmt.Sprintf("Git repository detected: yes\n")
		} else {
			summary += fmt.Sprintf("Git repository detected: no\n")
		}

		if !symbolsbylang.IsEmpty() {
			summary += fmt.Sprintf("Symbols by language:\n")
			for lang, count := range symbolsbylang {
				summary += fmt.Sprintf("- %s: %d\n", lang, count)
			}
		}

		if !m.Memory.Recent(5).IsEmpty() {
			summary += fmt.Sprintf("Recent memory:\n")
			for entry := range m.Memory.Recent(5) {
				summary += fmt.Sprintf("- [%s] %s\n", entry.Kind, entry.Content)
			}
		}

		if model := m.Model.AsRef(); model != nil {
			prompt := fmt.Sprintf("You are Astra. Summarize this project information for the user:\n%s\n", summary)
			answer, err := model.Complete(prompt)
			if err != nil {
				return err
			}
			m.Memory.Add("summary", answer.Clone())
			return nil
		}

		m.Memory.Add("summary", summary)

		return nil
	}

	if trimmed == ":graph" {
		if file_count := m.Index.Stats().FileCount; file_count == 0 {
			return m.build_index()
		}

		graph_dot := m.Index.GraphDot()
		output_dir := m.root.Join(".astra")
		err := os.MkdirAll(output_dir, 0755)
		if err != nil {
			return err
		}

		output_path := output_dir.Join("graph.dot")
		err = astrapkg.Dot.WriteFile(graph_dot, output_path)
		if err != nil {
			return err
		}

		memory_message := fmt.Sprintf("Wrote semantic graph to %v", output_path)
		m.Memory.Add("graph", memory_message)

		return nil
	}

	if trimmed == ":git-commit-count" {
		if git := m.Git.IsSome(); git != nil {
			git := m.Git.AsRef()
			commit_count := git.TotalCommitCount()
			message := fmt.Sprintf("Total commits: %d", commit_count)
			m.Memory.Add("git", message)
			return nil
		}

		return fmt.Errorf("No git repository detected for this root.")
	}

	if trimmed == ":git-last-commit" {
		if git := m.Git.IsSome(); git != nil {
			git := m.Git.AsRef()
			commit_info := git.LastCommitInfo()
			message := fmt.Sprintf("Last commit: %s by %s at %s — %s",
				commit_info.Id, commit_info.Author, commit_info.Date, commit_info.Summary)
			m.Memory.Add("git", message)
			return nil
		}

		return fmt.Errorf("No git repository detected for this root.")
	}

	if trimmed == ":health" {
		if file_count := m.Index.Stats().FileCount; file_count == 0 {
			return m.build_index()
		}

		team := team_mgr.NewTeamManager(m.root)
		health_report := health.ComputeHealth(m.root, m.Index, m.Memory, team)

		m.Memory.AddEvent(
			"health",
			fmt.Sprintf("Health check: quality=%d test=%d drift=%d security=%d git=%d team=%d",
				health_report.Scores.CodeQuality, health_report.Scores.TestHealth,
				health_report.Scores.CrossLangDrift, health_report.Scores.SecuritySurface,
				health_report.Scores.GitHealth, health_report.Scores.TeamVelocity),
			astrapkg.MemoryEvent{HealthSnapshot: health_report.Scores},
		)

		return health_report.Render()
	}

	rest := strings.TrimPrefix(trimmed, ":bisect ")
	if rest != "" {
		if git := m.Git.IsSome(); git != nil {
			if model := m.Model.AsRef(); model != nil {
				time_travel.Bisect(git, model, rest)
				return nil
			}

			return fmt.Errorf("Semantic bisect requires an LLM to be configured (use Groq).")
		}

		return fmt.Errorf("No git repository detected. Semantic bisect requires git.")
	}

	vibe_name := strings.TrimPrefix(trimmed, ":vibe ")
	if vibe_name != "" {
		persona := persona.FromVibe(vibe_name)
		display_name := persona.Name.Clone()
		m.set_persona(persona)

		return fmt.Sprintf("Vibe changed! You are now talking to %s.", display_name)
	}

	rest = strings.TrimPrefix(trimmed, ":git-history ")
	if rest != "" {
		if git := m.Git.IsSome(); git != nil {
			relative := filepath.FromSlash(rest)
			commits := git.RecentCommitsForPath(relative, 5)

			if commits.IsNotEmpty() {
				out := ""

				for _, commit := range commits {
					out += fmt.Sprintf("%s %s by %s at %s\n", commit.Id, commit.Summary, commit.Author, commit.Time)
				}

				m.memory.Add("git", fmt.Sprintf("history for %s queried (%d commits)", rest, commits.Len()))
				return out
			}

			return fmt.Sprintf("No recent commits found touching %s", rest)
		}

		return fmt.Errorf("No git repository detected for this root.")
	}

	rest = strings.TrimPrefix(trimmed, ":rust-symbols ")
	if rest != "" {
		path := filepath.FromSlash(rest)
		contents, err := os.ReadFile(path)
		if err != nil {
			return err
		}

		symbols := parse_rust_file(path, string(contents))

		if symbols.IsNotEmpty() {
			out := fmt.Sprintf("Rust symbols in %v:\n", path)

			for _, symbol := range symbols {
				symbol_kind := ""
				switch symbol.Kind {
				case astrapkg.ParsedSymbolKindStruct:
					symbol_kind = "struct"
				case astrapkg.ParsedSymbolKindEnum:
					symbol_kind = "enum"
				case astrapkg.ParsedSymbolKindFunction:
					symbol_kind = "fn"
				case astrapkg.ParsedSymbolKindClass:
					symbol_kind = "class"
				case astrapkg.ParsedSymbolKindInterface:
					symbol_kind = "interface"
				case astrapkg.ParsedSymbolKindType:
					symbol_kind = "type"
				case astrapkg.ParsedSymbolKindConstant:
					symbol_kind = "const"
				}

				out += fmt.Sprintf("  %s %s\n", symbol_kind, symbol.Name)
			}

			return out
		}

		return fmt.Sprintf("No Rust symbols found in %v", path)
	}

	if trimmed == ":migrations" {
		migrations := migration.ListMigrations()

		if migrations.IsNotEmpty() {
			out := ""

			out += "Migrations:\n"
			for migration := range migrations {
				out += fmt.Sprintf("- %v: %v -> %v\n", migration.Id, migration.FromStack, migration.ToStack)
			}

			return out
		}

		return fmt.Sprintf("No migrations are registered.")
	}

	return nil
}
package main

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"

	"github.com/pkg/errors"
	"github.com/spf13/cobra"
)

type cleanupEngine interface {
	Clean(contents string, lang Language) ([]byte, []string, error)
}

func newCleanupEngine(model model) cleanupEngine {
	return migrate.Cleaner(model)
}

func stripMarkdownFences(text string) string {
	return strings.Trim(strings.TrimPrefix(strings.TrimSuffix(text, "\n"), "```"), "ts")
}

type command struct{}

func (cmd *command) run(ctx context.Context, root string, args []string) (string, error) {
	if len(args) == 0 {
		return usage(ctx)
	}

	base, rest := baseAndRest(args)
	switch base {
	case "clean":
		return clean(ctx, root, rest)
	case "fix":
		return fix(ctx, root, rest)
	case "migrate-ts-ai":
		return migrateTsAi(ctx, root, rest)
	case "scaffold":
		return scaffold(ctx, root, rest)
	case "migrations":
		return listMigrations(ctx)
	case "plan-migration":
		return planMigration(ctx, rest)
	}

	return usage(ctx)
}

func clean(ctx context.Context, root string, args []string) (string, error) {
	if len(args) < 2 {
		return usage(ctx)
	}

	path, err := filepath.Abs(args[0])
	if err != nil {
		return "", err
	}
	lang, err := LanguageFromArgs(args[1])
	if err != nil {
		return "", err
	}

	contents, err := fs.ReadFile(root, path)
	if err != nil {
		return "", err
	}

	cleaner := newCleanupEngine(model)
	cleaned, smells, err := cleaner.Clean(string(contents), lang)
	if err != nil {
		return "", err
	}

	filepath.Abs(root)
	if err := fs.RemoveFile(root, path); err != nil {
		return "", err
	}
	if err := fs.WriteFile(root, path, cleaned); err != nil {
		return "", err
	}

	return fmt.Sprintf("✓ Cleaned up %s\n", path) + printSmells(smells)
}

func fix(ctx context.Context, root string, args []string) (string, error) {
	if len(args) < 2 {
		return usage(ctx)
	}

	path, err := filepath.Abs(args[0])
	if err != nil {
		return "", err
	}
	desc := strings.Join(args[1:], " ")

	contents, err := fs.ReadFile(root, path)
	if err != nil {
		return "", err
	}

	cleaner := newCleanupEngine(model)
	prompt := fmt.Sprintf("\nYou are an expert software engineer. A bug has been reported in this file.\nBug or error description: %s\nBelow is the current file contents. Return a fixed version of the file that compiles and resolves the bug.\nRules:\n- Keep the same overall structure and public API.\n- Focus only on changes needed to fix the described bug.\n- Output only the full fixed file contents in the same language, with no explanations and no markdown fences.\n- Preserve existing formatting and style where possible.\n\nFile path: %s\nCurrent file contents:\n%s", desc, path, string(contents))

	fixed, err := model.Complete(prompt)
	if err != nil {
		return "", err
	}
	fixed = stripMarkdownFences(fixed)

	backupPath := path + ".astra.bak"
	if err := fs.WriteFile(root, backupPath, contents); err != nil {
		return "", err
	}
	if err := fs.WriteFile(root, path, fixed); err != nil {
		return "", err
	}

	cmd := exec.Command("cargo", "check").CurrentDir(root)
	status, err := cmd.Run()
	if status.Success() {
		return fmt.Sprintf("Applied AI-generated fix to %s (backup at %s).", path, backupPath), nil
	}

	if err := fs.WriteFile(root, backupPath, contents); err != nil {
		return "", err
	}

	return fmt.Sprintf("AI-generated fix for %s did not pass cargo check. Original file was restored; backup kept at %s.", path, backupPath), nil
}

func migrateTsAi(ctx context.Context, root string, args []string) (string, error) {
	if model == nil {
		return "No language model is configured.", nil
	}

	path := args[0]
	contents, err := fs.ReadFile(root, path)
	if err != nil {
		return "", err
	}

	prompt := fmt.Sprintf("\nYou are an expert code translator. Translate the following TypeScript code into equivalent Rust.\nPreserve function names and signatures as closely as possible.\nUse idiomatic Rust, but do not add comments or explanations.\nOutput only Rust code, with no markdown fences or extra text.\nDo not omit any logic or types.\nAvoid TODO stubs or placeholder code.\n\nTypeScript code:\n```ts\n%s\n```", string(contents))

	rustCode, err := model.Complete(prompt)
	if err != nil {
		return "", err
	}

	return stripMarkdownFences(rustCode), nil
}

func scaffold(ctx context.Context, root string, args []string) (string, error) {
	stack, err := filepath.Abs(args[0])
	if err != nil {
		return "", err
	}

	plan, err := migrate.PlanScaffold(stack)
	if err != nil {
		return "", err
	}

	suggestions := []string{}
	for _, cmd := range plan.Commands {
		suggestions = append(suggestions, cmd)
	}
	for _, note := range plan.Notes {
		suggestions = append(suggestions, note)
	}

	res := fmt.Sprintf("Scaffold plan for stack `%s`:\n", plan.Stack)
	if len(suggestions) > 0 {
		res += fmt.Sprintf(`Suggested shell commands:
  %s`, strings.Join(suggestions, "\n  "))
	}
	if len(plan.Notes) > 0 {
		res += fmt.Sprintf("\nNotes:\n- %s", strings.Join(plan.Notes, "\n- "))
	}

	return res, nil
}

func listMigrations(ctx context.Context) (string, error) {
	migrations, err := migration.ListMigrations()
	if err != nil {
		return "", err
	}

	if len(migrations) == 0 {
		return "No migrations are registered.", nil
	}

	res := "List of registered migrations:\n"
	for _, m := range migrations {
		res += fmt.Sprintf("%s: %s -> %s\n", m.ID, m.FromStack, m.ToStack)
	}

	return res, nil
}

func planMigration(ctx context.Context, id string) (string, error) {
	if migration, err := migration.FindMigration(id); err != nil {
		return fmt.Sprintf("Unknown migration id: %s", id), nil
	} else {
		id := migration.ID
		from := migration.FromStack
		to := migration.ToStack

		res := fmt.Sprintf("Migration %s:\n%s -> %s\n\nDescription:\n%s", id, from, to, migration.Description)
		res += "\nHigh-level steps:\n"
		for i, step := range migration.Steps {
			res += fmt.Sprintf("%d. %s\n", i+1, step)
		}

		return res, nil
	}
}

func usage(ctx context.Context) (string, error) {
	return "Usage: :fix <path> <bug or error description>\nUsage: :clean <path> <language>\nUsage: :migrate-ts-ai <path>\nUsage: :scaffold <stack>\nUsage: :migrations\nUsage: :plan-migration <id>", nil
}
func (s *Solver) updateFixResult(path, desc, contents, backup_path string) error {
    cmd := exec.Command("cargo", "check")
    cmd.Dir = s.root
    cmd.Env = s.solverEnv
    cmd.Args = append(cmd.Args, path)
    output, err := cmd.CombinedOutput()
    if err != nil {
        return errors.Errorf("error checking path %q: %v", path, err)
    }

    if string(output) != "" {
        s.memory.Add("fix", fmt.Sprintf("applied fix for %q: %q", path, desc))
        return fmt.Errorf("Applied AI-generated fix to %q (backup at %q).", path, backup_path)
    }

    fs.WriteFile(path, contents, 0644)
    return fmt.Errorf("AI-generated fix for %q did not pass cargo check. Original file was restored; backup kept at %q.", path, backup_path)
}

func (s *Solver) migrateTsAiToFile(ts_part, rust_part string) error {
    if len(strings.Split(ts_part, " ")) != 2 || len(strings.Split(rust_part, " ")) != 2 {
        return errors.Errorf("Usage: :migrate-ts-ai-to-file <ts-path> <rust-path>")
    }

    tsPath := path.Join(tsPart)
    rustPath := path.Join(rustPart)

    tsCode, err := os.ReadFile(tsPath)
    if err != nil {
        return err
    }

    if s.model != nil {
        prompt := fmt.Sprintf("%s\nYou are an expert code translator. Translate the following TypeScript code into equivalent Rust.\nPreserve function names and signatures as closely as possible.\nUse idiomatic Rust, but do not add comments or explanations.\nOutput only Rust code, with no markdown fences or extra text.\nDo not omit any logic or types.\nAvoid TODO stubs or placeholder code.\n\nTypeScript code:\n```ts\n%s\n```\n",
            s.model.SystemPrompt(),
            tsCode)
        completions, err := s.model.Complete(prompt)
        if err != nil {
            return err
        }
        completions = strings.ReplaceAll(completions, "```", "")
        fs.WriteFile(rustPath, []byte(completions), 0644)

        // Add event to memory
        memory.Add("migration-ai-file", fmt.Sprintf("from: %q to: %q", tsPath, rustPath))
        return fmt.Errorf("Wrote migrated Rust code to %q", rustPath)
    }

    return errors.New("No language model is configured.")
}

func (s *Solver) migrateTsFile(tsPath string) error {
    code, err := tsMigrate.TranslateTsFile(tsPath)
    if err != nil {
        return err
    }

    memory.Add("migration-generated", fmt.Sprintf("from: %q", tsPath))
    return errors.New(code)
}

func (s *Solver) migrate(fromLang, toLang string, args ...string) error {
    if len(args) < 4 {
        return errors.Errorf("Usage: :migrate <source-dir> <from-lang> <to-lang> <output-dir> [--ai]")
    }

    sourceDir := path.Join(args[0])
    outputDir := path.Join(args[3])

    config := MigrationConfig{
        SourceDir: sourceDir,
        OutputDir: outputDir,
        FromLang:  fromLang,
        ToLang:    toLang,
    }

    if s.model != nil {
        config.UseAi = true
    }

    migrator := migrate.NewMigrator()
    result, err := migrator.RunMigration(&config)
    if err != nil {
        return err
    }

    memory.Add("migration", fmt.Sprintf("%s → %s: %d files migrated", config.FromLang, config.ToLang, result.Migrated))

    out := strings.Builder{}
    _, err = out.WriteString(result.PlanText)
    out.WriteString("\n")
    out.WriteString(result.ScaffoldLog)
    out.WriteString(result.Summary())
    return errors.New(out.String())
}

func (s *Solver) answerQuestion(question string) error {
    if strings.HasPrefix(question, "? ") {
        trimmed := strings.TrimSpace(question)
        question = trimmed[2:]
    }

    if strings.Contains(strings.ToLower(question), "health") || strings.ToLower(question) == "health" {
        return s.handleInput(":health")
    }

    if strings.Contains(strings.ToLower(question), "graph") || strings.ToLower(question) == "graph" {
        return s.handleInput(":graph")
    }

    // ... rest of the function ...
}
func (s *Snapshot) walk(path string, stack *[]string) error {
    if f, err := s.fs.Open(path); err != nil {
        return err
    } else {
        defer f.Close()
    }

    if p, err := s.fs.ReadDir(path); err != nil {
        return err
    } else {
        for _, entry := range p {
            s.stack = append(s.stack, entry.Name())
            if s.isIndexablePath(entry.Name()) {
                s.loadFile(entry.Name())
            }
            stack = append(stack, entry.Name())
        }
    }
    return nil
}

func (s *Snapshot) loadFile(path string) error {
    data, err := s.fs.ReadFile(path)
    if err != nil {
        return err
    }

    file := s.addIndexable(path, string(data))

    if file.isHighPriority() {
        s.memory.add("source-doc", fmt.Sprintf("File: %s\nContents:\n%s", path, data))
    }

    s.index.addFile(path, data)
    s.index.addFileToIndex(path, file)

    return nil
}

func stripMarkdownFences(text string) string {
    if s := strings.TrimFunc(text, func(r rune) bool {
        return r == '\n' || r == '`'
    }); s != "" {
        pos := strings.IndexFunc(s, func(r rune) bool {
            return r == '\n'
        })
        if pos > -1 {
            s = s[pos+1:]
        }
        pos = strings.IndexFunc(s, func(r rune) bool {
            return r == '`'
        })
        if pos > -1 {
            s = s[:pos]
        }
        return s
    }
    return ""
}

func (s *Snapshot) answerQuestion(question string) error {
    matches := s.memory.Search(question, 6)
    stats := s.index.getStats()
    byLang := s.index.getFilesByLanguage()

    if matches.IsEmpty() && stats.fileCount == 0 {
        return nil
    }

    response := strings.Builder{}

    if stats.fileCount > 0 {
        _, err := fmt.Fprintln(&response, fmt.Sprintf("Project snapshot: %d files, %d lines.", stats.fileCount, stats.totalLines))
        if err != nil {
            return err
        }
    }

    if !byLang.IsEmpty() {
        _, err := fmt.Fprintln(&response, "Files by language:")
        if err != nil {
            return err
        }
        for lang, count := range byLang {
            _, err := fmt.Fprintln(&response, fmt.Sprintf("- %s: %d", lang, count))
            if err != nil {
                return err
            }
        }
    }

    if !matches.IsEmpty() {
        _, err := fmt.Fprintln(&response, "Memory matches:")
        if err != nil {
            return err
        }
        for entry := range matches {
            _, err := fmt.Fprintln(&response, fmt.Sprintf("- [%s] %s (ts: %s)", entry.Kind, entry.Content, entry.Timestamp))
            if err != nil {
                return err
            }
        }
    }

    if fileContent := s.memory.GetLatestEvent("source-doc"); fileContent != nil {
        _, err := fmt.Fprintln(&response, fileContent)
        if err != nil {
            return err
        }
    }

    return nil
}

type Intent string

func (s *Snapshot) intentFor(query string) Intent {
    trimmed := strings.TrimSpace(query)

    if trimmed == "" {
        return ""
    }

    if trimmed StartsWith(":") || trimmed StartsWith("? ") {
        return ""
    }

    lower := strings.ToLower(trimmed)

    if lower Contains("what do you remember") || (lower Contains("remember") && lower Contains("what")) {
        return ":memory"
    }

    if lower Contains("what do you know") || lower Contains("project summary") || lower Contains("project info") || lower Contains("project information") {
        return ":summary"
    }

    if (lower Contains("how many") && lower Contains("file")) || lower Contains("files by language") || lower Contains("files-by-lang") {
        return ":files-by-lang"
    }

    if lower Contains("git repo") || lower Contains("git repository") {
        return ":summary"
    }

    if (lower Contains("how many") && lower Contains("commit")) || lower Contains("commit count") {
        return ":git-commit-count"
    }

    if lower Contains("last commit") || lower Contains("recent commit") || lower Contains("most recent commit") || lower Contains("when did i make") || lower Contains("when did i commit") || lower Contains("when was my last commit") {
        return ":git-last-commit"
    }

    if lower Contains("health check") || lower == "health" {
        return ":health"
    }

    if lower Contains("graph") || lower Contains("semantic graph") {
        return ":graph"
    }

    return ""
}
package main

func (t *translator) resolve_query(lang string) string {
    lower := strings.ToLower(lang)
    if lower == "git repo" || lower == "git repository" {
        return ":summary"
    }

    if (lower.Contains("how many") && lower.Contains("commit")) || lower.Contains("commit count") {
        return ":git-commit-count"
    }

    if lower.Contains("last commit") || lower.Contains("recent commit") || lower.Contains("most recent commit") || lower.Contains("when did i make") || lower.Contains("when did i commit") || lower.Contains("when was my last commit") {
        return ":git-last-commit"
    }

    if lower.Contains("health check") || lower == "health" {
        return ":health"
    }

    if lower.Contains("graph") || lower.Contains("semantic graph") {
        return ":graph"
    }

    return ""
}

func (t *translator) record_worktree_snapshot() {
    git := t.git
    if git == nil {
        return
    }
    files := git.ChangedFiles()
    changed_files := len(files)
    last := t.memory.LatestEvent("worktree")
    if last != nil && last.Event != nil && *last.Event == &MemoryEvent{WorktreeSnapshot: &MemoryEvent_WorktreeSnapshot{ChangedFiles: changed_files, Files: files}} {
        return
    }
    t.memory.AddEvent("worktree", fmt.Sprintf("uncommitted files: %d", changed_files), &MemoryEvent{WorktreeSnapshot: &MemoryEvent_WorktreeSnapshot{ChangedFiles: changed_files, Files: files}})
}

func (t *translator) record_git_commit() {
    git := t.git
    if git == nil {
        return
    }
    head, err := git.GetHeadCommit()
    if err != nil {
        return
    }
    last := t.memory.LatestEvent("git-commit")
    if last != nil && last.Event != nil && *last.Event == &MemoryEvent{GitCommit: &MemoryEvent_GitCommit{Id: head.ToString(), Summary: "", Author: "", Date: ""}} {
        return
    }
    info, err := git.LastCommitInfo()
    if err != nil {
        return
    }
    content := fmt.Sprintf("%s by %s — %s", info.Id, info.Author, info.Summary)
    t.memory.AddEvent("git-commit", content, &MemoryEvent{GitCommit: &MemoryEvent_GitCommit{Id: info.Id, Summary: info.Summary, Author: info.Author, Date: info.Date}})
}

func resolve_memory_path(root *path.Path) string {
    preferred := ".astra/memory.json"
    if _, err := root.Join(".astra").Join("memory.json").Stat(); err == nil {
        return preferred
    }
    previous := ".forge/memory.json"
    if _, err := root.Join(".forge").Join("memory.json").Stat(); err == nil {
        return previous
    }
    legacy := ".codex/memory.json"
    if _, err := root.Join(".codex").Join("memory.json").Stat(); err == nil {
        return legacy
    }
    return preferred
}
