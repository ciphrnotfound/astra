package migration

import (
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/your-project/model"
)

type MigrationConfig struct {
	SourceDir     string
	OutputDir     string
	FromLang      Language
	ToLang        Language
	UseAI         bool
	UseClean      bool
}

type MigrationResult struct {
	Migrated      []MigratedFile
	Skipped       []SkippedFile
	Errors        []MigrationError
	ScaffoldLog   string
	PlanText      string
}

type MigratedFile struct {
	Source   string
	Output   string
	 Lines   int
}

type SkippedFile struct {
	Path string
	Reason string
}

type MigrationError struct {
	Path string
	Err string
}

func RunMigration(cfg *MigrationConfig, model *model.CodexModel) (*MigrationResult, error) {
	var (
		planText string
	)

	if cfg.FromLang == cfg.ToLang {
		return nil, fmt.Errorf("source and target languages must be different")
	}

	planText += fmt.Sprintf("Migration plan: %s → %s\n", cfg.FromLang, cfg.ToLang)

	// 1. Discover source files
	sourceFiles, err := discoverSourceFiles(cfg.SourceDir, cfg.FromLang)
	if err != nil {
		return nil, err
	}
	if len(sourceFiles) == 0 {
		return nil, fmt.Errorf("no %s files found in %s", cfg.FromLang, cfg.SourceDir)
	}

	totalLines := 0
	for _, pf := range sourceFiles {
		if contents, err := os.ReadFile(pf); err == nil {
			totalLines += len(strings.Split(string(contents), "\n"))
			continue
		}
		log.Printf("Error reading file %s: %v", pf, err)
	}

	planText += fmt.Sprintf("Found %d source files (approx %d lines) in %s\n",
		len(sourceFiles), totalLines, cfg.SourceDir)

	if cfg.FromLang == LanguageRust {
		planText += "\nKey Rust modules:\n"
		for _, pf := range sourceFiles[:5] {
			if contents, err := os.ReadFile(pf); err == nil {
				if symbols, err := parseRustFile(pf, string(contents)); err == nil {
					if len(symbols) == 0 {
						continue
					}
					planText += fmt.Sprintf("  %s:\n", pf)
					for _, sym := range symbols[:8] {
						planText += fmt.Sprintf("    %s %s\n", kindString(sym.Kind), sym.Name)
					}
				}
			}
		}
	}

	planText += "\nSteps:\n"
	planText += fmt.Sprintf("  1. Scaffold a new %s project at %s\n", cfg.ToLang, cfg.OutputDir)
	planText += fmt.Sprintf("  2. Translate all %s source files into %s\n", cfg.FromLang, cfg.ToLang)
	planText += fmt.Sprintf("  3. Generate basic %s tests for migrated modules\n", cfg.ToLang)
	planText += "  4. Write a MIGRATION_REPORT.md summarizing files and any issues"

	useAIEffective := true
	if cfg.UseAI {
		if model == nil {
			planText += "\nAI requested but no model is configured; falling back to rule-based translation."
			useAIEffective = false
		}
	}

	// 2. Scaffold the output project
	scaffoldLog, err := scaffoldProject(cfg.OutputDir, cfg.ToLang)
	if err != nil {
		return nil, err
	}

	// 3. Build the translator
	translator := HybridTranslator{}
	if useAIEffective {
		translator = HybridTranslator{Model: model}
	}

	// 4. Walk each file and translate
	var (
		migrated []MigratedFile
		skipped  []SkippedFile
		errors   []MigrationError
	)

	// Determine the src sub-directory to place translated files
	srcSubdir := filepath.Join(cfg.OutputDir, "src")
	if cfg.ToLang == LanguageGo {
		srcSubdir = cfg.OutputDir
	}
	if err := os.MkdirAll(srcSubdir, 0755); err != nil {
		return nil, err
	}

	for _, sourceFile := range sourceFiles {
		// Map source path to output path
		relativePath := sourceFile
		if relative, err := filepath.Rel(cfg.SourceDir, relativePath); err == nil {
			relativePath = relative
		}

		outputPath := mapOutputPath(srcSubdir, relativePath, cfg.ToLang)
		// Read source
		sourceCode, err := os.ReadFile(sourceFile)
		if err != nil {
			errors = append(errors, MigrationError{Path: sourceFile, Err: err.Error()})
			continue
		}

		if len(strings.TrimSpace(string(sourceCode))) == 0 {
			skipped = append(skipped, SkippedFile{Path: sourceFile, Reason: "Empty file"})
			continue
		}

		// Translate
		translated, err := translator.Translate(string(sourceCode), cfg.FromLang, cfg.ToLang)
		if err != nil {
			errors = append(errors, MigrationError{Path: sourceFile, Err: err.Error()})
			continue
		}

		lines := len(strings.Split(translated, "\n"))

		// Create parent dirs
	 parentDir := filepath.Dir(outputPath)
		if parentDir != "" {
			if err := os.MkdirAll(parentDir, 0755); err != nil {
				return nil, err
			}
		}

		// Write
		if err := os.WriteFile(outputPath, []byte(translated), 0644); err != nil {
			errors = append(errors, MigrationError{Path: sourceFile, Err: err.Error()})
			continue
		}

		// Cleanup if requested
		if cfg.UseClean {
			if model != nil {
				cleaner := model.NewCleanupEngine()
				if cleaned, smells, err := cleaner.Clean(translated, cfg.ToLang); err == nil {
					if err = os.WriteFile(outputPath, cleaned, 0644); err != nil {
						return nil, err
					}
				}
			}
		}

		migrated = append(migrated, MigratedFile{Path: sourceFile, Output: outputPath, Lines: lines})
	}

	return &MigrationResult{
		Migrated:      migrated,
		Skipped:       skipped,
		Errors:        errors,
		ScaffoldLog:   scaffoldLog,
		PlanText:      planText,
	}, nil
}

func discoverSourceFiles(root, lang string) ([]string, error) {
	// implementation omitted
}

func scaffoldProject(root, lang string) (string, error) {
	// implementation omitted
}

func parseRustFile(path, content string) ([]Symbol, error) {
	// implementation omitted
}

func mapOutputPath(root, relative, lang string) string {
	// implementation omitted
}

type Symbol struct {
	Kind ParsedSymbolKind
	Name string
}

type ParsedSymbolKind string

const (
	SymbolKindStruct  ParsedSymbolKind = "struct"
	SymbolKindEnum    ParsedSymbolKind = "enum"
	SymbolKindFunction ParsedSymbolKind = "fn"
)

func kindString(kind ParsedSymbolKind) string {
	return strings.Title(string(kind))
}
func part2(config *MigrationConfig, skiplist *[]SkippedFile, errors *[]MigrationError, file MigratedFile, translated string) {
    lines := len(strings.Split(translated, "\n"))

    // Create parent dirs
    parent := file.OutputDir().Parent()
    if parent != nil {
        utils.CreateDirAll(parent)
    }

    // Write
    if err := fsutil.Write(&file.Output, translated); err != nil {
        *errors = append(*errors, MigrationError{
            Path:     file.Source().Clone(),
            Error:    fmt.Sprintf("Failed to write: %v", err),
        })
        return
    }

    // Cleanup if requested
    if config.UseClean {
        if model != nil {
            cleaner = CleanupEngine{
                Model: model,
            }
            cleaned, _ := cleaner.Clean(&translated, config.ToLang)
            utils.Write(&file.Output, cleaned)
        }
    }

    *migrated = append(*migrated, MigratedFile{
        Source: file.Source().Clone(),
        Output: file.Output,
        Lines:  lines,
    })
}

func generateAITests(model CodexModel, config *MigrationConfig, migrated []MigratedFile) (*GeneratedTests, error) {
    created := make([]string, 0)

    for _, file := range migrated {
        code, err := utils.ReadFile(file.Output)
        if err != nil {
            continue
        }

        request := fmt.Sprintf("You are an expert test writer.\n\
         Write tests for the following %s code that capture its intent.\n\
         Rules:\n\
         - Use idiomatic %s\n\
         - Focus on edge cases and failure modes\n\
         - Output only test code, no explanations\n\n\
         Source code:\n%s\n",
            config.ToLang, config.ToLang, code)

        tests, err := model.Complete(&request)
        if err != nil {
            return nil, err
        }

        testPath := mapOutputPath(config.OutputDir, file.Output)
        if err := utils.CreateDirAll(testPath.Parent()); err != nil {
            return nil, err
        }

        if err := fsutil.Write(&testPath, tests); err != nil {
            return nil, err
        }

        created = append(created, testPath.String())
    }

    return &GeneratedTests{
        Files: created,
    }, nil
}

func mapOutputPath(srcSubdir string, relative string, lang Language) string {
    stem := path.Base(relative)
    parent := path.Dir(relative)

    newExt := lang.TargetExtension()

    if lang == LanguageRust {
        newStem := sanitizeRustModuleName(stem)
        return path.Join(srcSubdir, parent, fmt.Sprintf("%s.%s", newStem, newExt))
    } else {
        return path.Join(srcSubdir, parent, fmt.Sprintf("%s.%s", stem, newExt))
    }
}

func sanitizeRustModuleName(name string) string {
    out := ""
    for _, r := range name {
        if unicode.IsASCII(r) && (unicode.IsLetter(r) || unicode.IsNumber(r) || r == '_') {
            out += string(r)
        } else {
            out += "_"
        }
    }
    if out == "" {
        return "_module"
    }
    if unicode.IsDigit(r) {
        out = "_" + out
    }
    return out
}

func updateRustLibRs(outputDir string, migrated []MigratedFile) {
    libPath := path.Join(outputDir, "src", "lib.rs")
    content := "// Auto-generated by astra migrate\n\n"

    for _, file := range migrated {
        stem := path.Base(file.Output)
        module := sanitizeRustModuleName(stem)
        if module != "lib" && module != "main" {
            content += fmt.Sprintf("pub mod %s;\n", module)
        }
    }

    if err := fsutil.Write(&libPath, content); err != nil {
        return
    }
}

func writeMigrationReport(outputDir string, config *MigrationConfig, migrated []MigratedFile, skipped []SkippedFile, errors []MigrationError) {
    report := ""

    // ... (rest of the report generation remains the same)
}
