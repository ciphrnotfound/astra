use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Result};
use crate::model::CodexModel;
use super::detect::Language;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub line: Option<usize>,
    pub message: String,
}

pub struct CompileOutput {
    pub success: bool,
    pub errors: Vec<CompileError>,
    pub raw_output: String,
}

/// Trait for running language-specific compilers and capturing errors.
pub trait CompilerRunner {
    fn run(&self, file_path: &Path) -> Result<CompileOutput>;
}

// --- TypeScript (tsc) Implementation ---
pub struct TscRunner;
impl CompilerRunner for TscRunner {
    fn run(&self, file_path: &Path) -> Result<CompileOutput> {
        // On Windows, npx is a batch script so we need npx.cmd
        let npx_cmd = if cfg!(target_os = "windows") { "npx.cmd" } else { "npx" };
        let output = Command::new(npx_cmd)
            .args(["tsc", file_path.to_str().unwrap(), "--noEmit", "--skipLibCheck", "--esModuleInterop", "--strict"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}{}", stdout, stderr);

        let success = output.status.success();
        let mut errors = Vec::new();

        // Basic parsing of tsc errors: "file.ts(line,col): error TSXXXX: message"
        for line in combined.lines() {
            if line.contains("error TS") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let msg = parts[2..].join(":").trim().to_string();
                    let line_num = line.find('(').and_then(|start| {
                        line[start+1..].find(',').map(|end| {
                            line[start+1..start+1+end].parse::<usize>().ok()
                        })
                    }).flatten();

                    errors.push(CompileError { line: line_num, message: msg });
                }
            }
        }

        Ok(CompileOutput { success, errors, raw_output: combined })
    }
}

// --- Rust (cargo check) Implementation ---
pub struct CargoRunner;
impl CompilerRunner for CargoRunner {
    fn run(&self, _file_path: &Path) -> Result<CompileOutput> {
        // Rust usually requires the whole crate context
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let mut errors = Vec::new();

        for line in stderr.lines() {
            if line.contains("error:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let line_num = parts[1].parse::<usize>().ok();
                    let msg = parts[3..].join(":").trim().to_string();
                    errors.push(CompileError { line: line_num, message: msg });
                }
            }
        }

        Ok(CompileOutput { success, errors, raw_output: stderr })
    }
}

/// The main logic for the Auto-Fixer
pub struct AutoFixer<'a> {
    model: &'a (dyn CodexModel + Send + Sync),
}

impl<'a> AutoFixer<'a> {
    pub fn new(model: &'a (dyn CodexModel + Send + Sync)) -> Self {
        Self { model }
    }

    pub fn fix(&self, lang: Language, file_path: &Path) -> Result<String> {
        let runner: Box<dyn CompilerRunner> = match lang {
            Language::TypeScript => Box::new(TscRunner),
            Language::Rust => Box::new(CargoRunner),
            _ => return Err(anyhow!("Auto-fixing not yet supported for {}", lang)),
        };

        let mut current_code = std::fs::read_to_string(file_path)?;
        let mut attempts = 0;
        let max_attempts = 3;

        println!(" \u{1f50d} Starting Auto-Fix pass for {}...", file_path.display());

        while attempts < max_attempts {
            let output = runner.run(file_path)?;
            if output.success {
                println!(" \u{2705} No compiler errors found!");
                return Ok(current_code);
            }

            println!(" \u{26a0}\u{fe0f} Found {} compiler errors. Attempting fix {}/{}...", output.errors.len(), attempts + 1, max_attempts);

            let error_report = output.errors.iter()
                .map(|e| format!("Line {}: {}", e.line.map(|l| l.to_string()).unwrap_or("?".to_string()), e.message))
                .collect::<Vec<_>>()
                .join("\n");

            let system_prompt = format!(
                "You are an expert {} developer and code fixer. \
                 The following code has COMPILER ERRORS. Your task is to fix them while preserving original logic.\n\n\
                 COMPILER REPORT:\n{}\n\n\
                 RULES:\n\
                 - FIX the errors mentioned above.\n\
                 - Output ONLY the full fixed code. NO explanations. NO markdown fences.",
                lang, error_report
            );

            let user_msg = format!("Original Code:\n{}", current_code);
            
            let fixed_code = self.model.complete_chat(&system_prompt, &user_msg)?;
            current_code = strip_markdown_fences(&fixed_code);
            
            // Save it back to disk for the next compiler run
            std::fs::write(file_path, &current_code)?;
            
            attempts += 1;
        }

        Err(anyhow!("Could not fix all compiler errors after {} attempts.", max_attempts))
    }
}

fn strip_markdown_fences(s: &str) -> String {
    let mut result = s.trim().to_string();
    if result.starts_with("```") {
        if let Some(first_newline) = result.find('\n') {
            result = result[first_newline..].trim().to_string();
        }
    }
    if result.ends_with("```") {
        result = result[..result.len() - 3].trim().to_string();
    }
    result
}
