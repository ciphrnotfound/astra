use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::model::CodexModel;
use super::detect::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub name: String,
    pub description: String,
    pub line_hint: Option<usize>,
}

pub struct CleanupEngine<'a> {
    model: &'a (dyn CodexModel + Send + Sync),
}

impl<'a> CleanupEngine<'a> {
    pub fn new(model: &'a (dyn CodexModel + Send + Sync)) -> Self {
        Self { model }
    }

    pub fn detect_smells(&self, code: &str, lang: Language) -> Vec<CodeSmell> {
        let mut smells = Vec::new();

        match lang {
            Language::Python => {
                // Smell: JSON dump of objects
                if code.contains("json.dump(") && !code.contains("asdict(") && !code.contains("JSONEncoder") {
                    smells.push(CodeSmell {
                        name: "Custom Object Serialization".to_string(),
                        description: "Found `json.dump` without `asdict` or custom encoder. This will crash for custom classes.".to_string(),
                        line_hint: None,
                    });
                }
                // Smell: Missing dataclasses
                if code.contains("def __init__(self") && !code.contains("@dataclass") {
                    smells.push(CodeSmell {
                        name: "Missing Dataclasses".to_string(),
                        description: "Boilerplate __init__ mirrors source struct; consider using @dataclass.".to_string(),
                        line_hint: None,
                    });
                }
                // Smell: Static methods for defaults
                if code.contains("@staticmethod") && (code.contains("default") || code.contains("max_entries")) {
                    smells.push(CodeSmell {
                        name: "Rust-style Static Helpers".to_string(),
                        description: "Static methods like `default_max_entries` should be converted to default parameters.".to_string(),
                        line_hint: None,
                    });
                }
            }
            _ => {}
        }

        smells
    }

    pub fn clean(&self, code: &str, lang: Language) -> Result<(String, Vec<CodeSmell>)> {
        let smells = self.detect_smells(code, lang);
        if smells.is_empty() {
            return Ok((code.to_string(), Vec::new()));
        }

        let smells_desc = smells.iter()
            .map(|s| format!("- {}: {}", s.name, s.description))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are an expert Refactor Bot. Clean up the following {} code to be fully idiomatic.\n\n\
             WE DETECTED THESE MIGRATION SMELLS:\n\
             {}\n\n\
             CODE TO FIX:\n\
             {}\n\n\
             RULES:\n\
             - FIX ALL detected smells.\n\
             - Use standard idiomatic patterns (e.g., @dataclass, typing, proper serialization).\n\
             - Output ONLY the fixed code, no explanations or markdown fences.\n",
            lang, smells_desc, code
        );

        let cleaned = self.model.complete(&prompt)?;
        let cleaned = self.strip_markdown(&cleaned);

        Ok((cleaned, smells))
    }

    fn strip_markdown(&self, s: &str) -> String {
        let mut out = s.to_string();
        if out.starts_with("```") {
            if let Some(first_newline) = out.find('\n') {
                out = out[first_newline + 1..].to_string();
            }
            if out.ends_with("```") {
                out = out[..out.len() - 3].to_string();
            }
        }
        out.trim().to_string()
    }
}
