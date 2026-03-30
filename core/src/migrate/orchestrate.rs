use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::detect::{discover_source_files, Language};
use super::scaffold::scaffold_project;
use super::translate::{HybridTranslator, Translator};
use super::clean::CleanupEngine;
use crate::model::CodexModel;
use crate::parser::{parse_rust_file, ParsedSymbolKind};

// ---------------------------------------------------------------------------
// Config & result types
// ---------------------------------------------------------------------------

pub struct MigrationConfig {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub from_lang: Language,
    pub to_lang: Language,
    pub use_ai: bool,
    pub use_clean: bool,
    pub use_fix: bool,
    pub knowledge: Option<String>,
}

pub struct MigrationResult {
    pub migrated: Vec<MigratedFile>,
    pub skipped: Vec<SkippedFile>,
    pub errors: Vec<MigrationError>,
    pub scaffold_log: String,
    pub plan_text: String,
}

pub struct GeneratedTests {
    pub files: Vec<PathBuf>,
}

pub struct MigratedFile {
    pub source: PathBuf,
    pub output: PathBuf,
    pub lines: usize,
}

pub struct SkippedFile {
    pub path: PathBuf,
    pub reason: String,
}

pub struct MigrationError {
    pub path: PathBuf,
    pub error: String,
}

impl MigrationResult {
    /// Summarize the result as a human-readable string.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(&mut s, "\n━━━ MIGRATION COMPLETE ━━━");
        let _ = writeln!(&mut s, "  ✓ Migrated : {} files", self.migrated.len());
        if !self.skipped.is_empty() {
            let _ = writeln!(&mut s, "  ⊘ Skipped  : {} files", self.skipped.len());
        }
        if !self.errors.is_empty() {
            let _ = writeln!(&mut s, "  ✗ Errors   : {} files", self.errors.len());
        }
        let total_lines: usize = self.migrated.iter().map(|f| f.lines).sum();
        let _ = writeln!(&mut s, "  ◎ Lines    : {}", total_lines);

        if !self.errors.is_empty() {
            let _ = writeln!(&mut s, "\nErrors:");
            for e in &self.errors {
                let _ = writeln!(&mut s, "  {:?}: {}", e.path, e.error);
            }
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Main migration entry point
// ---------------------------------------------------------------------------

pub fn run_migration(
    config: &MigrationConfig,
    model: Option<&(dyn CodexModel + Send + Sync)>,
    search: Option<&(dyn crate::model::SearchProvider + Send + Sync)>
) -> Result<MigrationResult> {
    let mut plan_text = String::new();

    if config.from_lang == config.to_lang {
        return Ok(MigrationResult {
            migrated: Vec::new(),
            skipped: Vec::new(),
            errors: vec![MigrationError {
                path: config.source_dir.clone(),
                error: "Source and target languages must be different.".to_string(),
            }],
            scaffold_log: String::new(),
            plan_text: "Migration aborted: source and target languages are the same.".to_string(),
        });
    }

    let _ = writeln!(
        &mut plan_text,
        "Migration plan: {} → {}",
        config.from_lang, config.to_lang
    );

    // 1. Discover source files
    let source_files = discover_source_files(&config.source_dir, config.from_lang);

    if source_files.is_empty() {
        return Ok(MigrationResult {
            migrated: Vec::new(),
            skipped: Vec::new(),
            errors: vec![MigrationError {
                path: config.source_dir.clone(),
                error: format!(
                    "No {} files found in {:?}",
                    config.from_lang, config.source_dir
                ),
            }],
            scaffold_log: String::new(),
            plan_text,
        });
    }

    let total_lines: usize = source_files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .map(|c| c.lines().count())
        .sum();

    let _ = writeln!(
        &mut plan_text,
        "Found {} source files (approx {} lines) in {:?}",
        source_files.len(),
        total_lines,
        config.source_dir
    );

    if config.from_lang == Language::Rust {
        let _ = writeln!(&mut plan_text);
        let _ = writeln!(&mut plan_text, "Key Rust modules:");
        for path in source_files.iter().take(5) {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(symbols) = parse_rust_file(path, &contents) {
                    if symbols.is_empty() {
                        continue;
                    }
                    let _ = writeln!(&mut plan_text, "  {:?}:", path);
                    for sym in symbols.iter().take(8) {
                        let kind = match sym.kind {
                            ParsedSymbolKind::Struct => "struct",
                            ParsedSymbolKind::Enum => "enum",
                            ParsedSymbolKind::Function => "fn",
                            ParsedSymbolKind::Class => "class",
                            ParsedSymbolKind::Interface => "interface",
                            ParsedSymbolKind::Type => "type",
                            ParsedSymbolKind::Constant => "const",
                        };
                        let _ = writeln!(&mut plan_text, "    {} {}", kind, sym.name);
                    }
                }
            }
        }
    }

    let _ = writeln!(&mut plan_text);
    let _ = writeln!(&mut plan_text, "Steps:");
    let _ = writeln!(
        &mut plan_text,
        "  1. Scaffold a new {} project at {:?}",
        config.to_lang, config.output_dir
    );
    let _ = writeln!(
        &mut plan_text,
        "  2. Translate all {} source files into {}",
        config.from_lang, config.to_lang
    );
    let _ = writeln!(
        &mut plan_text,
        "  3. Generate basic {} tests for migrated modules",
        config.to_lang
    );
    let _ = writeln!(
        &mut plan_text,
        "  4. Write a MIGRATION_REPORT.md summarizing files and any issues"
    );

    let use_ai_effective = if config.use_ai {
        if model.is_some() {
            true
        } else {
            let _ = writeln!(
                &mut plan_text,
                "AI requested but no model is configured; falling back to rule-based translation."
            );
            false
        }
    } else {
        false
    };

    // 2. Scaffold the output project
    let scaffold_log = scaffold_project(&config.output_dir, config.to_lang)?;

    // 3. Build the translator
    let translator = if use_ai_effective {
        HybridTranslator::new(model, search, config.knowledge.clone())
    } else {
        HybridTranslator::new(None, None, None)
    };

    // 4. Walk each file and translate
    let mut migrated = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    // Determine the src sub-directory to place translated files
    let src_subdir = match config.to_lang {
        Language::Rust => config.output_dir.join("src"),
        Language::Go => config.output_dir.clone(),
        Language::Python => config.output_dir.join("src"),
        Language::TypeScript => config.output_dir.join("src"),
        Language::JavaScript => config.output_dir.join("src"),
        Language::Java => config
            .output_dir
            .join("src")
            .join("main")
            .join("java"),
        Language::React | Language::NextJs | Language::Vue | Language::Svelte => config.output_dir.join("src"),
        Language::Cpp | Language::Assembly => config.output_dir.join("src"),
    };
    fs::create_dir_all(&src_subdir)?;

    for source_path in &source_files {
        // Map source path to output path
        let mut relative = match source_path.strip_prefix(&config.source_dir) {
            Ok(r) => {
                if r.as_os_str().is_empty() {
                    Path::new(source_path.file_name().unwrap())
                } else {
                    r
                }
            },
            Err(_) => {
                skipped.push(SkippedFile {
                    path: source_path.clone(),
                    reason: "Could not compute relative path".to_string(),
                });
                continue;
            }
        };

        let output_path = map_output_path(&src_subdir, relative, config.to_lang);

        // Read source
        let source_code = match fs::read_to_string(source_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(MigrationError {
                    path: source_path.clone(),
                    error: format!("Failed to read: {}", e),
                });
                continue;
            }
        };

        if source_code.trim().is_empty() {
            skipped.push(SkippedFile {
                path: source_path.clone(),
                reason: "Empty file".to_string(),
            });
            continue;
        }

        // Translate
        match translator.translate(&source_code, config.from_lang, config.to_lang) {
            Ok(translated) => {
                let lines = translated.lines().count();

                // Create parent dirs
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent).ok();
                }

                // Write
                if let Err(e) = fs::write(&output_path, &translated) {
                    errors.push(MigrationError {
                        path: source_path.clone(),
                        error: format!("Failed to write: {}", e),
                    });
                    continue;
                }

                // Cleanup if requested
                if config.use_clean {
                    if let Some(m) = model {
                        let cleaner = CleanupEngine::new(m);
                        if let Ok((cleaned, _smells)) = cleaner.clean(&translated, config.to_lang) {
                            let _ = fs::write(&output_path, &cleaned);
                        }
                    }
                }

                migrated.push(MigratedFile {
                    source: source_path.clone(),
                    output: output_path.clone(),
                    lines,
                });

                // Auto-fix if requested
                if config.use_fix {
                    if let Some(m) = model {
                        let fixer = super::fix::AutoFixer::new(m);
                        let _ = fixer.fix(config.to_lang, &output_path);
                    }
                }
            }
            Err(e) => {
                errors.push(MigrationError {
                    path: source_path.clone(),
                    error: format!("Translation failed: {}", e),
                });
            }
        }
    }

    // 5. Update the Rust lib.rs if target is Rust (add mod declarations)
    if config.to_lang == Language::Rust {
        update_rust_lib_rs(&config.output_dir, &migrated);
    }

    // 6. Optionally generate tests using the model
    if use_ai_effective {
        if let Some(m) = model {
        generate_ai_tests(m, config, &migrated).ok();
        }
    }

    // 7. Write migration report
    write_migration_report(&config.output_dir, config, &migrated, &skipped, &errors);

    Ok(MigrationResult {
        migrated,
        skipped,
        errors,
        scaffold_log,
        plan_text,
    })
}

fn generate_ai_tests(
    model: &(dyn CodexModel + Send + Sync),
    config: &MigrationConfig,
    migrated: &[MigratedFile],
) -> Result<GeneratedTests> {
    let mut created = Vec::new();

    for file in migrated {
        let code = match fs::read_to_string(&file.output) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let request = format!(
            "You are an expert test writer.\n\
             Write tests for the following {} code that capture its intent.\n\
             Rules:\n\
             - Use idiomatic {}\n\
             - Focus on edge cases and failure modes\n\
             - Output only test code, no explanations\n\n\
             Source code:\n{}\n",
            config.to_lang, config.to_lang, code
        );

        let tests = model.complete(&request)?;

        let test_path = match config.to_lang {
            Language::Rust => {
                let mut p = file.output.clone();
                let name = match p.file_stem().and_then(|s| s.to_str()) {
                    Some(n) => format!("{}_tests.rs", n),
                    None => "generated_tests.rs".to_string(),
                };
                p.set_file_name(name);
                p
            }
            Language::Python => {
                let mut p = config.output_dir.join("tests");
                let name = match file.output.file_stem().and_then(|s| s.to_str()) {
                    Some(n) => format!("test_{}.py", n),
                    None => "test_generated.py".to_string(),
                };
                p.push(name);
                p
            }
            Language::Go => {
                let mut p = file.output.clone();
                let name = match p.file_stem().and_then(|s| s.to_str()) {
                    Some(n) => format!("{}_test.go", n),
                    None => "generated_test.go".to_string(),
                };
                p.set_file_name(name);
                p
            }
            _ => continue,
        };

        if let Some(parent) = test_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        if fs::write(&test_path, tests).is_ok() {
            created.push(test_path);
        }
    }

    Ok(GeneratedTests { files: created })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_output_path(src_subdir: &Path, relative: &Path, to_lang: Language) -> PathBuf {
    let stem = relative.file_stem().unwrap_or_default();
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));

    let new_ext = to_lang.target_extension();
    let new_name = if to_lang == Language::Rust {
        format!("{}.{}", sanitize_rust_module_name(&stem.to_string_lossy()), new_ext)
    } else {
        format!("{}.{}", stem.to_string_lossy(), new_ext)
    };

    src_subdir.join(parent).join(new_name)
}

fn update_rust_lib_rs(output_dir: &Path, migrated: &[MigratedFile]) {
    let lib_path = output_dir.join("src").join("lib.rs");
    let mut content = String::from("// Auto-generated by astra migrate\n\n");

    for file in migrated {
        if let Some(stem) = file.output.file_stem() {
            let mod_name = sanitize_rust_module_name(&stem.to_string_lossy());
            if mod_name != "lib" && mod_name != "main" {
                content.push_str(&format!("pub mod {};\n", mod_name));
            }
        }
    }

    fs::write(lib_path, content).ok();
}

fn sanitize_rust_module_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "_module".to_string();
    }
    if out.chars().next().unwrap_or('_').is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

fn write_migration_report(
    output_dir: &Path,
    config: &MigrationConfig,
    migrated: &[MigratedFile],
    skipped: &[SkippedFile],
    errors: &[MigrationError],
) {
    let mut report = String::new();
    let _ = writeln!(&mut report, "# Migration Report");
    let _ = writeln!(&mut report);
    let _ = writeln!(
        &mut report,
        "- **From**: {} (`{:?}`)",
        config.from_lang, config.source_dir
    );
    let _ = writeln!(
        &mut report,
        "- **To**: {} (`{:?}`)",
        config.to_lang, config.output_dir
    );
    let _ = writeln!(&mut report, "- **AI-assisted**: {}", config.use_ai);
    let _ = writeln!(&mut report);

    let _ = writeln!(&mut report, "## Migrated Files ({}):", migrated.len());
    for f in migrated {
        let _ = writeln!(
            &mut report,
            "- `{:?}` → `{:?}` ({} lines)",
            f.source, f.output, f.lines
        );
    }

    if !skipped.is_empty() {
        let _ = writeln!(&mut report);
        let _ = writeln!(&mut report, "## Skipped ({}):", skipped.len());
        for f in skipped {
            let _ = writeln!(&mut report, "- `{:?}`: {}", f.path, f.reason);
        }
    }

    if !errors.is_empty() {
        let _ = writeln!(&mut report);
        let _ = writeln!(&mut report, "## Errors ({}):", errors.len());
        for e in errors {
            let _ = writeln!(&mut report, "- `{:?}`: {}", e.path, e.error);
        }
    }

    fs::write(output_dir.join("MIGRATION_REPORT.md"), report).ok();
}
