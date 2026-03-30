use std::fmt::Write as FmtWrite;
use anyhow::{anyhow, Result};

use super::detect::Language;
use super::mapping::{LibraryRegistry, ConceptRegistry};
use crate::model::{CodexModel, SearchProvider};

// ---------------------------------------------------------------------------
// Translator trait
// ---------------------------------------------------------------------------

/// A translator converts source code from one language to another.
pub trait Translator {
    fn translate(
        &self,
        source_code: &str,
        from: Language,
        to: Language,
    ) -> Result<String>;
}

// ---------------------------------------------------------------------------
// Rule-based translator
// ---------------------------------------------------------------------------

pub struct RuleBasedTranslator;

impl RuleBasedTranslator {
    pub fn new() -> Self {
        Self
    }
}

impl Translator for RuleBasedTranslator {
    fn translate(
        &self,
        source_code: &str,
        from: Language,
        to: Language,
    ) -> Result<String> {
        match (from, to) {
            (Language::TypeScript, Language::Rust) => Ok(translate_ts_to_rust(source_code)),
            (Language::JavaScript, Language::Rust) => Ok(translate_ts_to_rust(source_code)),
            (Language::Python, Language::Rust) => Ok(translate_py_to_rust(source_code)),
            (Language::TypeScript, Language::Python) => Ok(translate_ts_to_python(source_code)),
            (Language::TypeScript, Language::Go) => Ok(translate_ts_to_go(source_code)),
            (Language::Python, Language::Go) => Ok(translate_py_to_go(source_code)),
            (Language::Python, Language::TypeScript) => Ok(translate_py_to_ts(source_code)),
            (Language::Rust, Language::TypeScript) => Ok(translate_rust_to_ts(source_code)),
            _ => Err(anyhow!(
                "Rule-based translation from {} to {} is not yet supported",
                from,
                to
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// AI-powered translator
// ---------------------------------------------------------------------------

pub struct AiTranslator<'a> {
    model: &'a (dyn CodexModel + Send + Sync),
    search: Option<&'a (dyn SearchProvider + Send + Sync)>,
    registry: LibraryRegistry,
    concepts: ConceptRegistry,
    knowledge: Option<String>,
}

impl<'a> AiTranslator<'a> {
    pub fn new(
        model: &'a (dyn CodexModel + Send + Sync), 
        search: Option<&'a (dyn SearchProvider + Send + Sync)>,
        knowledge: Option<String>
    ) -> Self {
        Self { 
            model,
            search,
            registry: LibraryRegistry::new(),
            concepts: ConceptRegistry::new(),
            knowledge,
        }
    }

    fn build_mapping_context(&self, source_code: &str, from: Language, to: Language) -> String {
        let mut context = String::new();

        // 1. Standard Language Idioms (Concepts)
        let mappings = self.concepts.get_mappings(from, to);
        if !mappings.is_empty() {
            let _ = writeln!(&mut context, "### REQUIRED {} TO {} IDIOMS:", from, to);
            for m in mappings {
                let _ = writeln!(&mut context, "- {}: {}", m.concept, m.pattern);
            }
            let _ = writeln!(&mut context);
        }

        // 2. Library-specific mappings
        let _ = writeln!(&mut context, "### LIBRARY MAPPINGS:");
        // Dynamically detect crates from `use xxx::` statements and emit known mappings
        let mut emitted_libs = std::collections::HashSet::new();
        for line in source_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                let parts: Vec<&str> = trimmed[4..].split("::").collect();
                if !parts.is_empty() {
                    let crate_name = parts[0].trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
                    if !crate_name.is_empty() && !emitted_libs.contains(crate_name) {
                        if let Some(m) = self.registry.get(crate_name, to) {
                            let _ = writeln!(
                                &mut context,
                                "- Rust '{}' → {}: {}. Import: '{}'. NOTE: {}",
                                crate_name, m.target_lib, to, m.import_path, m.notes
                            );
                            emitted_libs.insert(crate_name.to_string());
                        }
                    }
                }
            }
        }

        // 3. Autonomous Web Search Fallback for Unknown Dependencies
        if from == Language::Rust && self.search.is_some() {
            let mut unknown_crates = Vec::new();
            let std_libs = ["std", "core", "alloc", "crate", "super", "self"];
            
            // Simple string scanning for `use xxx::`
            for line in source_code.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("use ") {
                    let parts: Vec<&str> = trimmed[4..].split("::").collect();
                    if !parts.is_empty() {
                        let crate_name = parts[0].trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
                        if !crate_name.is_empty() 
                            && !std_libs.contains(&crate_name) 
                            && self.registry.get(crate_name, to).is_none() 
                            && !unknown_crates.contains(&crate_name.to_string()) 
                        {
                            unknown_crates.push(crate_name.to_string());
                        }
                    }
                }
            }

            if !unknown_crates.is_empty() {
                let _ = writeln!(&mut context, "### \u{1f310} AUTONOMOUS RESEARCH RESULTS:");
                for missing in unknown_crates.iter().take(3) { // Limit to top 3 to avoid spam
                    println!(" \u{1f50e} Unknown dependency '{}' detected. Searching for {} equivalent...", missing, to);
                    let query = format!("Idiomatic {} equivalent library or pattern for Rust crate '{}'", to, missing);
                    if let Ok(results) = self.search.unwrap().search(&query) {
                        // Ask LLM to summarize the search to save token space
                        let summary_prompt = format!(
                            "Based on these search results, what is the single MOST standard/idiomatic {} library equivalent to Rust's '{}'? \
                             Give a 1-sentence answer with the npm package name or standard pattern to use.\n\n{}", 
                            to, missing, &results[..results.len().min(1500)]
                        );
                        if let Ok(summary) = self.model.complete(&summary_prompt) {
                            println!("   \u{21b3} Found: {}", summary.trim());
                            let _ = writeln!(&mut context, "- For '{}', use: {}", missing, summary.trim());
                        }
                    }
                }
                let _ = writeln!(&mut context);
            }
        }
        if let Some(k) = &self.knowledge {
            let truncated_k = if k.len() > 3000 {
                let mut temp = k.chars().take(3000).collect::<String>();
                temp.push_str("... [TRUNCATED]");
                temp
            } else {
                k.clone()
            };
            let _ = writeln!(&mut context, "- Use these learned patterns for {}:\n{}", to, truncated_k);
        }
        context
    }

    fn translate_in_chunks(&self, source_code: &str, from: Language, to: Language) -> Result<String> {
        let mut result = String::new();
        let chunk_size = 10000;
        let overlap = 1000;
        
        let mapping_context = self.build_mapping_context(source_code, from, to);
        
        let mut start = 0;
        let mut chunk_idx = 1;
        while start < source_code.len() {
            let end = (start + chunk_size).min(source_code.len());
            let chunk = &source_code[start..end];
            
            let system_prompt = format!(
                "You are an expert code translator. You are translating a LARGE file in parts. This is Part {} of a {} to {} migration.\n\n\
                 RULES:\n\
                 - OUTPUT ONLY THE TRANSLATED {} CODE.\n\
                 - NEVER hallucinate fake libraries or imports.\n\
                 - DO NOT include ANY {} code or explanations.\n\
                 - Ensure Part {} connects logically to Part {}.\n\n\
                 {}",
                chunk_idx, from, to, to, from, chunk_idx, chunk_idx - 1, mapping_context
            );

            let user_msg = format!("Translate this source chunk (Part {}):\n{}", chunk_idx, chunk);
            
            // --- PASS 1: Initial Translation ---
            let draft = self.model.complete_chat(&system_prompt, &user_msg)?;
            let draft = strip_markdown_fences(&draft);

            // --- PASS 2: Self-Correction Pass ---
            let rewrite_system = format!(
                "You are an expert {} Developer. Rewrite the ENTIRE code chunk below to FIX common migration errors.\n\
                 Ensure it remains idiomatic {} and doesn't contain Rust syntax.\n\
                 RULES:\n\
                 1. You MUST rewrite and output the ENTIRE code chunk. Do not omit anything.\n\
                 2. Do NOT output explanations or comments about fixes.\n\
                 3. Output ONLY code, no markdown.",
                to, to
            );

            let rewrite_user = format!("Review and fix this {} code chunk:\n\n{}", to, draft);
            let final_chunk = self.model.complete_chat(&rewrite_system, &rewrite_user)?;
            
            let stripped = strip_markdown_fences(&final_chunk);
            result.push_str(&stripped);
            result.push('\n');
            
            if end == source_code.len() {
                break;
            }
            start += chunk_size - overlap;
            chunk_idx += 1;
        }
        Ok(result)
    }
}

impl<'a> Translator for AiTranslator<'a> {
    fn translate(
        &self,
        source_code: &str,
        from: Language,
        to: Language,
    ) -> Result<String> {
        if source_code.len() > 15000 {
            return self.translate_in_chunks(source_code, from, to);
        }

        let mapping_context = self.build_mapping_context(source_code, from, to);

        let system_prompt = format!(
            "You are a world-class senior software engineer performing a {} to {} code migration.\n\n\
             ABSOLUTE RULES (violations = failure):\n\
             1. Output ONLY valid, compilable {} code. No markdown fences, no explanations, no comments about the translation.\n\
             2. Translate ALL code. Every struct, impl, trait, function, and method MUST appear in the output.\n\
             3. NEVER comment out code. NEVER leave TODO placeholders. Every Rust function must become a working {} function.\n\
             4. NEVER hallucinate or invent libraries that don't exist.\n\
             5. Rust structs become TypeScript classes with explicit field declarations and constructors.\n\
             6. Rust 'impl' blocks become methods INSIDE the class body.\n\
             7. Rust trait impls become 'class X implements Y'.\n\
             8. Rust `HashMap<K, V>` becomes built-in JS `Map<K, V>`. NEVER import `Map` or `collections`.\n\
             9. CRITICAL: Convert ALL `snake_case` methods AND parameters to `camelCase`. Examples: `execute_batch` -> `executeBatch`, `user_id` -> `userId`.\n\
             10. Rust generic bounds `<S: Trait>` MUST become `<S extends Trait>` in TypeScript.\n\
             11. CRITICAL: Rust Enums with data MUST become Discriminated Union `type`s. NEVER create a `class DatabaseError` or `class CustomError`.\n\
             12. Arc<RwLock<T>> and Mutex<T> do NOT exist in TypeScript. Use plain Map/object. Add a `// Note: Concurrency lock removed` comment.\n\
             13. All async fn must use 'async' keyword on method and return Promise<T>.\n\n\
             {}",
            from, to, to, to, mapping_context
        );

        let user_msg = format!("Translate this {} code:\n{}", from, source_code);

        // --- PASS 1: Initial Translation ---
        let draft = self.model.complete_chat(&system_prompt, &user_msg)?;
        let draft = strip_markdown_fences(&draft);

        // --- PASS 2: Self-Correction Pass ---
        let rewrite_system = format!(
            "You are an expert {} Developer. Rewrite the ENTIRE provided code from top to bottom to FIX common migration errors.\n\n\
             RECHECK AND FIX THESE ERRORS:\n\
             1. NO 'async' keyword in interface methods (use Promise<T> return type).\n\
             2. Enums with data MUST be Discriminated Union `type`s. IF YOU WROTE `class CustomError` or `class DatabaseError`, DELETE IT AND MAKE IT A UNION `type`.\n\
             3. NO 'Option<T>' or 'Result<T, E>'. Use 'T | null' or Tuples.\n\
             4. Generic static methods must have <T> on the method signature.\n\
             5. CRITICAL: IF YOU HAVE ANY `snake_case` METHODS OR PARAMETERS (like `execute_batch` or `user_id`), RENAME THEM TO `camelCase` IMMEDIATELY.\n\
             6. USE built-in JS `Map<K, V>`. If you added `import * as Map...`, DELETE THE IMPORT.\n\
             7. Rust generic bounds `<S: Trait>` MUST become `<S extends Trait>`.\n\n\
             RULES:\n\
             - You MUST rewrite and output the ENTIRE file. Do not comment out or remove functionality.\n\
             - Do NOT output explanations or markdown fences. Output ONLY the raw fixed {} code.",
            to, to
        );

        let rewrite_user = format!("Review and fix this {} code:\n\n{}", to, draft);
        let final_code = self.model.complete_chat(&rewrite_system, &rewrite_user)?;

        Ok(strip_markdown_fences(&final_code))
    }
}

// ---------------------------------------------------------------------------
// Hybrid translator: AI first, rules fallback
// ---------------------------------------------------------------------------

pub struct HybridTranslator<'a> {
    rules: RuleBasedTranslator,
    ai: Option<AiTranslator<'a>>,
}

impl<'a> HybridTranslator<'a> {
    pub fn new(model: Option<&'a (dyn CodexModel + Send + Sync)>, search: Option<&'a (dyn SearchProvider + Send + Sync)>, knowledge: Option<String>) -> Self {
        Self {
            rules: RuleBasedTranslator::new(),
            ai: model.map(|m| AiTranslator::new(m, search, knowledge)),
        }
    }
}

impl<'a> Translator for HybridTranslator<'a> {
    fn translate(
        &self,
        source_code: &str,
        from: Language,
        to: Language,
    ) -> Result<String> {
        let mut ai_error = None;

        // If we have an AI model, try it first.
        if let Some(ai) = &self.ai {
            match ai.translate(source_code, from, to) {
                Ok(result) if !result.trim().is_empty() => return Ok(result),
                Ok(_) => {
                    ai_error = Some("AI returned empty output".to_string());
                }
                Err(e) => {
                    ai_error = Some(format!("AI error: {}", e));
                }
            }
        }

        // Fall back to rule-based translator
        match self.rules.translate(source_code, from, to) {
            Ok(result) if !result.trim().is_empty() => Ok(result),
            _ => {
                let context = if let Some(err) = ai_error {
                    format!(" ({} and no rules found)", err)
                } else {
                    " (no AI model and no rules found)".to_string()
                };
                Err(anyhow!(
                    "Translation failed for {} → {}{}",
                    from,
                    to,
                    context
                ))
            }
        }
    }
}

// ===========================================================================
// Language-specific rule-based translators
// ===========================================================================

// ---------------------------------------------------------------------------
// TypeScript / JavaScript → Rust
// ---------------------------------------------------------------------------

fn translate_ts_to_rust(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines / comments
        if line.is_empty() {
            out.push('\n');
            i += 1;
            continue;
        }

        // Import statements → use statements (as comments for now)
        if line.starts_with("import ") {
            i += 1;
            continue;
        }

        // Interface / type → struct
        if line.starts_with("export interface ") || line.starts_with("interface ") {
            let (rust_struct, consumed) = translate_ts_interface_to_rust(&lines, i);
            out.push_str(&rust_struct);
            out.push('\n');
            i += consumed;
            continue;
        }

        // Type alias
        if line.starts_with("export type ") || line.starts_with("type ") {
            if let Some((name, rhs)) = parse_ts_type_alias(line) {
                let rust_ty = map_ts_type_rust(rhs);
                out.push_str(&format!("pub type {} = {};\n", name, rust_ty));
            }
            i += 1;
            continue;
        }

        // Class → struct + impl
        if line.starts_with("export class ") || line.starts_with("class ") {
            let (rust_code, consumed) = translate_ts_class_to_rust(&lines, i);
            out.push_str(&rust_code);
            out.push('\n');
            i += consumed;
            continue;
        }

        // export function / function
        if line.starts_with("export function ")
            || line.starts_with("function ")
            || line.starts_with("export async function ")
            || line.starts_with("async function ")
        {
            let (rust_fn, consumed) = translate_ts_function_to_rust(&lines, i);
            out.push_str(&rust_fn);
            out.push('\n');
            i += consumed;
            continue;
        }

        // Arrow functions: export const name = (...) => { ... }
        if (line.starts_with("export const ") || line.starts_with("const "))
            && line.contains("=>")
        {
            let (rust_fn, consumed) = translate_ts_arrow_to_rust(&lines, i);
            out.push_str(&rust_fn);
            out.push('\n');
            i += consumed;
            continue;
        }

        // Variable declarations
        if line.starts_with("export const ")
            || line.starts_with("const ")
            || line.starts_with("let ")
            || line.starts_with("var ")
        {
            out.push_str(&translate_ts_variable_to_rust(line));
            out.push('\n');
            i += 1;
            continue;
        }

        // Anything else: pass through as comment
        i += 1;
    }

    out
}

// ---------------------------------------------------------------------------
// Rust → TypeScript
// ---------------------------------------------------------------------------

fn translate_rust_to_ts(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            out.push('\n');
            i += 1;
            continue;
        }

        if trimmed.starts_with("use ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("extern crate ")
        {
            out.push_str(&format!("// TODO: {}\n", trimmed));
            i += 1;
            continue;
        }

        if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
            let (ts_iface, consumed) = translate_rust_struct_to_ts(&lines, i);
            out.push_str(&ts_iface);
            out.push('\n');
            i += consumed;
            continue;
        }

        if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
            let (ts_enum, consumed) = translate_rust_enum_to_ts(&lines, i);
            out.push_str(&ts_enum);
            out.push('\n');
            i += consumed;
            continue;
        }

        if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
            let (ts_fn, consumed) = translate_rust_fn_to_ts(&lines, i);
            out.push_str(&ts_fn);
            out.push('\n');
            i += consumed;
            continue;
        }

        out.push_str(&format!("// {}\n", trimmed));
        i += 1;
    }

    out
}

fn translate_rust_struct_to_ts(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let name = header
        .split_whitespace()
        .skip_while(|p| *p != "struct")
        .nth(1)
        .unwrap_or("Unknown");

    let mut out = String::new();
    out.push_str(&format!("export interface {} {{\n", name));

    let mut i = start + 1;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with('}') {
            i += 1;
            break;
        }
        if line.is_empty() || line.starts_with("//") {
            i += 1;
            continue;
        }
        if let Some((field_name, ty)) = line.split_once(':') {
            let field_name = field_name
                .trim_start_matches("pub ")
                .trim()
                .trim_end_matches(',');
            let ty = ty.trim().trim_end_matches(',').trim_end_matches(',');
            let ts_type = map_rust_type_ts(ty);
            out.push_str(&format!("  {}: {};\n", field_name, ts_type));
        }
        i += 1;
    }

    out.push_str("}\n");
    (out, i - start)
}

fn translate_rust_enum_to_ts(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let name = header
        .split_whitespace()
        .skip_while(|p| *p != "enum")
        .nth(1)
        .unwrap_or("Unknown");

    let mut variants = Vec::new();
    let mut i = start + 1;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with('}') {
            i += 1;
            break;
        }
        if line.is_empty() || line.starts_with("//") {
            i += 1;
            continue;
        }
        let clean = line.trim_end_matches(',').trim();
        if !clean.is_empty() && !clean.contains('{') && !clean.contains('(') {
            variants.push(clean.to_string());
        }
        i += 1;
    }

    let mut out = String::new();
    if variants.is_empty() {
        out.push_str(&format!("export type {} = string;\n", name));
    } else {
        out.push_str(&format!("export type {} =\n", name));
        for (idx, v) in variants.iter().enumerate() {
            if idx == variants.len() - 1 {
                out.push_str(&format!("  | \"{}\";\n", v));
            } else {
                out.push_str(&format!("  | \"{}\"\n", v));
            }
        }
    }
    (out, i - start)
}

fn translate_rust_fn_to_ts(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let clean = header
        .trim_start_matches("pub ")
        .trim_start_matches("fn ")
        .trim();

    let open_paren = match clean.find('(') {
        Some(p) => p,
        None => return (format!("// {}\n", header), 1),
    };
    let close_paren = match clean.find(')') {
        Some(p) => p,
        None => return (format!("// {}\n", header), 1),
    };

    let name = &clean[..open_paren];
    let params_str = &clean[open_paren + 1..close_paren];
    let after_paren = clean[close_paren + 1..].trim();

    let ret_type = if let Some(arrow_pos) = after_paren.find("->") {
        let ty = after_paren[arrow_pos + 2..].trim().trim_end_matches('{').trim();
        map_rust_type_ts(ty)
    } else {
        "void"
    };

    let params = translate_rust_params_ts(params_str);
    let (body, consumed) = collect_braced_body(lines, start);

    let mut out = String::new();
    out.push_str(&format!("export function {}({}): {} {{\n", name, params, ret_type));
    if body.is_empty() {
        out.push_str("  // TODO: implement\n");
    } else {
        for body_line in &body {
            out.push_str(&format!("  // {}\n", body_line.trim()));
        }
    }
    out.push_str("}\n");

    (out, consumed)
}

fn translate_ts_interface_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let name = header
        .trim_start_matches("export ")
        .trim_start_matches("interface ")
        .split(|c: char| c == '{' || c == ' ' || c == '<')
        .next()
        .unwrap_or("Unknown")
        .trim();

    let mut out = String::new();
    out.push_str(&format!("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct {} {{\n", name));

    let mut i = start + 1;
    while i < lines.len() {
        let line = lines[i].trim();
        if line == "}" || line.starts_with("}") {
            i += 1;
            break;
        }
        // Parse field: name: type; or name?: type;
        if let Some((field_name, ts_type)) = parse_ts_field(line) {
            let optional = field_name.ends_with('?');
            let clean_name = field_name.trim_end_matches('?');
            let rust_type = map_ts_type_rust(ts_type);
            if optional {
                out.push_str(&format!("    pub {}: Option<{}>,\n", to_snake_case(clean_name), rust_type));
            } else {
                out.push_str(&format!("    pub {}: {},\n", to_snake_case(clean_name), rust_type));
            }
        }
        i += 1;
    }

    out.push_str("}\n");
    (out, i - start)
}

fn translate_ts_class_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let name = header
        .trim_start_matches("export ")
        .trim_start_matches("class ")
        .split(|c: char| c == '{' || c == ' ' || c == '<')
        .next()
        .unwrap_or("Unknown")
        .trim();

    let mut fields = Vec::new();

    let mut i = start + 1;
    let mut brace_depth = 1;

    while i < lines.len() && brace_depth > 0 {
        let line = lines[i].trim();

        for c in line.chars() {
            match c {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }

        if brace_depth == 0 {
            i += 1;
            break;
        }

        // Constructor fields
        if line.starts_with("constructor(") || line.contains("constructor(") {
            // Extract params as fields
            if let Some(params_start) = line.find('(') {
                if let Some(params_end) = line.find(')') {
                    let params = &line[params_start + 1..params_end];
                    for param in params.split(',') {
                        let p = param
                            .trim()
                            .trim_start_matches("private ")
                            .trim_start_matches("public ")
                            .trim_start_matches("readonly ");
                        if let Some((name, ty)) = p.split_once(':') {
                            fields.push((
                                name.trim().to_string(),
                                map_ts_type_rust(ty.trim()).to_string(),
                            ));
                        }
                    }
                }
            }
        }
        // Class property
        else if line.contains(':') && !line.contains('(') && !line.starts_with("//") {
            let clean = line.trim_start_matches("private ")
                .trim_start_matches("public ")
                .trim_start_matches("readonly ");
            if let Some((name, ty)) = clean.split_once(':') {
                let ty_clean = ty.trim().trim_end_matches(';');
                fields.push((
                    name.trim().to_string(),
                    map_ts_type_rust(ty_clean).to_string(),
                ));
            }
        }
        i += 1;
    }

    let mut out = String::new();
    out.push_str(&format!("pub struct {} {{\n", name));
    for (fname, ftype) in &fields {
        out.push_str(&format!("    pub {}: {},\n", to_snake_case(fname), ftype));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl {} {{\n", name));
    out.push_str("    pub fn new(");
    let params: Vec<String> = fields
        .iter()
        .map(|(n, t)| format!("{}: {}", to_snake_case(n), t))
        .collect();
    out.push_str(&params.join(", "));
    out.push_str(&format!(") -> Self {{\n        Self {{\n"));
    for (fname, _) in &fields {
        let snake = to_snake_case(fname);
        out.push_str(&format!("            {},\n", snake));
        }
    out.push_str("        }\n    }\n");

    out.push_str("}\n");

    (out, i - start)
}

fn translate_ts_function_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let clean = header
        .trim_start_matches("export ")
        .trim_start_matches("async ")
        .trim_start_matches("function ")
        .trim();

    let open_paren = match clean.find('(') {
        Some(p) => p,
        None => return (String::new(), 1),
    };
    let close_paren = match clean.find(')') {
        Some(p) => p,
        None => return (String::new(), 1),
    };

    let name = &clean[..open_paren];
    let params_str = &clean[open_paren + 1..close_paren];
    let after_paren = clean[close_paren + 1..].trim();

    let ret_type = if let Some(colon_pos) = after_paren.find(':') {
        let ty = after_paren[colon_pos + 1..]
            .trim()
            .trim_end_matches('{')
            .trim()
            .trim_start_matches("Promise<")
            .trim_end_matches('>');
        map_ts_type_rust(ty)
    } else {
        "()"
    };

    let params = translate_ts_params_rust(params_str);

    // Collect body
    let (body, consumed) = collect_braced_body(lines, start);

    let mut out = String::new();
    out.push_str(&format!("pub fn {}({}) -> {} {{\n", to_snake_case(name), params, ret_type));
    let mut wrote_body = false;
    for body_line in &body {
        if let Some(stmt) = translate_ts_statement_to_rust(body_line) {
            out.push_str("    ");
            out.push_str(&stmt);
            out.push('\n');
            wrote_body = true;
        }
    }
    if !wrote_body {
        match ret_type {
            "()" => {}
            "String" => out.push_str("    String::new()\n"),
            "i32" | "i64" | "f64" => out.push_str("    0\n"),
            "bool" => out.push_str("    false\n"),
            _ => out.push_str("    Default::default()\n"),
        }
    }
    out.push_str("}\n");

    (out, consumed)
}

fn translate_ts_arrow_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let clean = header
        .trim_start_matches("export ")
        .trim_start_matches("const ")
        .trim();

    // name = (params) => ...
    let eq_pos = match clean.find('=') {
        Some(p) => p,
        None => return (String::new(), 1),
    };

    let name = clean[..eq_pos].trim();
    let after_eq = clean[eq_pos + 1..].trim();

    // Try to extract params from (params): RetType =>
    let (params_str, ret_hint) = if let Some(paren_start) = after_eq.find('(') {
        if let Some(paren_end) = after_eq.find(')') {
            let p = &after_eq[paren_start + 1..paren_end];
            let after = after_eq[paren_end + 1..].trim();
            let ret = if let Some(colon_pos) = after.find(':') {
                let ty = after[colon_pos + 1..]
                    .trim()
                    .split("=>")
                    .next()
                    .unwrap_or("")
                    .trim();
                map_ts_type_rust(ty)
            } else {
                "()"
            };
            (p.to_string(), ret)
        } else {
            (String::new(), "()")
        }
    } else {
        (String::new(), "()")
    };

    let params = translate_ts_params_rust(&params_str);
    let (body, consumed) = collect_braced_body(lines, start);

    let mut out = String::new();
    out.push_str(&format!("pub fn {}({}) -> {} {{\n", to_snake_case(name), params, ret_hint));
    let mut wrote_body = false;
    for body_line in &body {
        if let Some(stmt) = translate_ts_statement_to_rust(body_line) {
            out.push_str("    ");
            out.push_str(&stmt);
            out.push('\n');
            wrote_body = true;
        }
    }
    if !wrote_body {
        match ret_hint {
            "()" => {}
            "String" => out.push_str("    String::new()\n"),
            "i32" | "i64" | "f64" => out.push_str("    0\n"),
            "bool" => out.push_str("    false\n"),
            _ => out.push_str("    Default::default()\n"),
        }
    }
    out.push_str("}\n");

    (out, consumed)
}

fn translate_ts_variable_to_rust(line: &str) -> String {
    let clean = line
        .trim()
        .trim_start_matches("export ")
        .trim_start_matches("const ")
        .trim_start_matches("let ")
        .trim_start_matches("var ")
        .trim_end_matches(';');

    if let Some((name, value)) = clean.split_once('=') {
        let name = name.split(':').next().unwrap_or(name).trim();
        let value = value.trim();
        let const_name = sanitize_const_name(name);
        if let Some((ty, lit)) = infer_ts_literal_rust(value) {
            format!("pub const {}: {} = {};", const_name, ty, lit)
        } else if let Some(fmt_expr) = translate_ts_template_to_rust(value) {
            format!(
                "pub fn {}() -> String {{ {} }}",
                to_snake_case(name),
                fmt_expr
            )
        } else {
            format!(
                "pub fn {}() -> String {{ String::new() }}",
                to_snake_case(name)
            )
        }
    } else {
        String::new()
    }
}

// ---------------------------------------------------------------------------
// Python → Rust
// ---------------------------------------------------------------------------

fn translate_py_to_rust(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            out.push('\n');
            i += 1;
            continue;
        }

        // Import → comment
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            i += 1;
            continue;
        }

        // Class
        if trimmed.starts_with("class ") {
            let (rust_code, consumed) = translate_py_class_to_rust(&lines, i);
            out.push_str(&rust_code);
            out.push('\n');
            i += consumed;
            continue;
        }

        // Function
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            let (rust_fn, consumed) = translate_py_function_to_rust(&lines, i);
            out.push_str(&rust_fn);
            out.push('\n');
            i += consumed;
            continue;
        }

        // Decorator
        if trimmed.starts_with('@') {
            i += 1;
            continue;
        }

        // Anything else
        i += 1;
    }

    out
}

fn translate_py_function_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let clean = header
        .trim_start_matches("async ")
        .trim_start_matches("def ")
        .trim_end_matches(':');

    let open_paren = match clean.find('(') {
        Some(p) => p,
        None => return (String::new(), 1),
    };
    let close_paren = match clean.rfind(')') {
        Some(p) => p,
        None => return (String::new(), 1),
    };

    let name = &clean[..open_paren];
    let params_str = &clean[open_paren + 1..close_paren];
    let after_close = clean[close_paren + 1..].trim();

    let ret_type = if let Some(arrow_pos) = after_close.find("->") {
        let ty = after_close[arrow_pos + 2..].trim();
        map_py_type_rust(ty)
    } else {
        "()"
    };

    let params = translate_py_params_rust(params_str);

    // Collect indented body
    let (body, consumed) = collect_py_body(lines, start);

    let mut out = String::new();
    out.push_str(&format!("pub fn {}({}) -> {} {{\n", to_snake_case(name), params, ret_type));
    let mut wrote_body = false;
    for body_line in &body {
        if let Some(stmt) = translate_py_statement_to_rust(body_line) {
            out.push_str("    ");
            out.push_str(&stmt);
            out.push('\n');
            wrote_body = true;
        }
    }
    if !wrote_body {
        match ret_type {
            "()" => {}
            "String" => out.push_str("    String::new()\n"),
            "i32" | "i64" | "f64" => out.push_str("    0\n"),
            "bool" => out.push_str("    false\n"),
            _ => out.push_str("    Default::default()\n"),
        }
    }
    out.push_str("}\n");

    (out, consumed)
}

fn translate_py_class_to_rust(lines: &[&str], start: usize) -> (String, usize) {
    let header = lines[start].trim();
    let name = header
        .trim_start_matches("class ")
        .split(|c: char| c == '(' || c == ':' || c == ' ')
        .next()
        .unwrap_or("Unknown")
        .trim();

    let mut fields: Vec<(String, String)> = Vec::new();
    let mut methods: Vec<String> = Vec::new();

    let (body_lines, consumed) = collect_py_body(lines, start);

    for bline in &body_lines {
        let t = bline.trim();
        if t.starts_with("self.") && t.contains('=') {
            if let Some(dot_field) = t.strip_prefix("self.") {
                if let Some((fname, _)) = dot_field.split_once('=') {
                    let fname = fname.trim().trim_end_matches(':').trim();
                    // Avoid duplicates
                    if !fields.iter().any(|(n, _)| n == fname) {
                        fields.push((fname.to_string(), "String".to_string()));
                    }
                }
            }
        }
        if t.starts_with("def ") || t.starts_with("async def ") {
            methods.push(t.to_string());
        }
    }

    let mut out = String::new();
    out.push_str(&format!("pub struct {} {{\n", name));
    for (fname, ftype) in &fields {
        out.push_str(&format!("    pub {}: {},\n", to_snake_case(fname), ftype));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl {} {{\n", name));
    out.push_str("}\n");

    (out, consumed)
}

fn translate_py_params_rust(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() || trimmed == "self" {
            continue;
        }
        if let Some((name, ty)) = trimmed.split_once(':') {
            let name = name.trim().trim_start_matches('*');
            let ty = ty.split('=').next().unwrap_or("").trim();
            let rust_ty = map_py_type_rust(ty);
            parts.push(format!("{}: {}", name, rust_ty));
        } else {
            let name = trimmed.split('=').next().unwrap_or(trimmed).trim();
            parts.push(format!("{}: String", name));
        }
    }
    parts.join(", ")
}

fn map_py_type_rust(py_type: &str) -> &'static str {
    match py_type.trim() {
        "str" => "String",
        "int" => "i64",
        "float" => "f64",
        "bool" => "bool",
        "None" => "()",
        "list" | "List" => "Vec<String>",
        "dict" | "Dict" => "std::collections::HashMap<String, String>",
        "bytes" => "Vec<u8>",
        _ => "String",
    }
}

// ---------------------------------------------------------------------------
// TypeScript → Python
// ---------------------------------------------------------------------------

fn translate_ts_to_python(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            out.push('\n');
            i += 1;
            continue;
        }

        if line.starts_with("import ") {
            i += 1;
            continue;
        }

        if line.starts_with("export function ") || line.starts_with("function ") {
            let clean = line
                .trim_start_matches("export ")
                .trim_start_matches("async ")
                .trim_start_matches("function ")
                .trim();

            if let Some(open) = clean.find('(') {
                if let Some(close) = clean.find(')') {
                    let name = &clean[..open];
                    let params = &clean[open + 1..close];
                    let py_params = translate_ts_params_python(params);
                    let (body_lines, consumed) = collect_braced_body(lines.as_slice(), i);
                    out.push_str(&format!("def {}({}):\n", to_snake_case(name), py_params));
                    let mut wrote_body = false;
                    for bl in &body_lines {
                        if let Some(stmt) = translate_ts_statement_to_python(bl) {
                            out.push_str("    ");
                            out.push_str(&stmt);
                            out.push('\n');
                            wrote_body = true;
                        }
                    }
                    if !wrote_body {
                        out.push_str("    pass\n");
                    }
                    out.push('\n');
                    i += consumed;
                    continue;
                }
            }
        }

        i += 1;
    }

    out
}

// ---------------------------------------------------------------------------
// TypeScript → Go
// ---------------------------------------------------------------------------

fn translate_ts_to_go(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut body = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            body.push('\n');
            i += 1;
            continue;
        }

        if line.starts_with("import ") {
            i += 1;
            continue;
        }

        if line.starts_with("export function ") || line.starts_with("function ") {
            let clean = line
                .trim_start_matches("export ")
                .trim_start_matches("async ")
                .trim_start_matches("function ");

            if let Some(open) = clean.find('(') {
                if let Some(close) = clean.find(')') {
                    let name = &clean[..open];
                    let params = &clean[open + 1..close];
                    let after = clean[close + 1..].trim();
                    let ret_type = if let Some(colon) = after.find(':') {
                        let ty = after[colon + 1..].trim().trim_end_matches('{').trim();
                        map_ts_type_go(ty)
                    } else {
                        ""
                    };
                    let go_params = translate_ts_params_go(params);
                    let (body_lines, consumed) = collect_braced_body(lines.as_slice(), i);

                    let go_name = to_pascal_case(name);
                    if ret_type.is_empty() {
                        body.push_str(&format!("func {}({}) {{\n", go_name, go_params));
                    } else {
                        body.push_str(&format!("func {}({}) {} {{\n", go_name, go_params, ret_type));
                    }
                    for bl in &body_lines {
                        if let Some(stmt) = translate_ts_statement_to_go(bl) {
                            body.push_str("\t");
                            body.push_str(&stmt);
                            body.push('\n');
                        }
                    }
                    body.push_str("}\n\n");
                    i += consumed;
                    continue;
                }
            }
        }

        i += 1;
    }

    let mut out = String::new();
    out.push_str("package main\n\n");
    if body.contains("fmt.") {
        out.push_str("import \"fmt\"\n\n");
    }
    out.push_str(&body);
    out
}

// ---------------------------------------------------------------------------
// Python → Go
// ---------------------------------------------------------------------------

fn translate_py_to_go(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut body = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            body.push('\n');
            i += 1;
            continue;
        }

        if line.starts_with("import ") || line.starts_with("from ") {
            i += 1;
            continue;
        }

        if line.starts_with("def ") || line.starts_with("async def ") {
            let clean = line
                .trim_start_matches("async ")
                .trim_start_matches("def ")
                .trim_end_matches(':');

            if let Some(open) = clean.find('(') {
                if let Some(close) = clean.rfind(')') {
                    let name = &clean[..open];
                    let params = &clean[open + 1..close];
                    let after = clean[close + 1..].trim();
                    let ret_type = if let Some(arrow) = after.find("->") {
                        map_py_type_go(after[arrow + 2..].trim())
                    } else {
                        ""
                    };
                    let go_params = translate_py_params_go(params);
                    let (body_lines, consumed) = collect_py_body(lines.as_slice(), i);

                    let go_name = to_pascal_case(name);
                    if ret_type.is_empty() {
                        body.push_str(&format!("func {}({}) {{\n", go_name, go_params));
                    } else {
                        body.push_str(&format!("func {}({}) {} {{\n", go_name, go_params, ret_type));
                    }
                    for bl in &body_lines {
                        if let Some(stmt) = translate_py_statement_to_go(bl) {
                            body.push_str("\t");
                            body.push_str(&stmt);
                            body.push('\n');
                        }
                    }
                    body.push_str("}\n\n");
                    i += consumed;
                    continue;
                }
            }
        }

        i += 1;
    }

    let mut out = String::new();
    out.push_str("package main\n\n");
    if body.contains("fmt.") {
        out.push_str("import \"fmt\"\n\n");
    }
    out.push_str(&body);
    out
}

// ---------------------------------------------------------------------------
// Python → TypeScript
// ---------------------------------------------------------------------------

fn translate_py_to_ts(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            out.push('\n');
            i += 1;
            continue;
        }

        if line.starts_with("import ") || line.starts_with("from ") {
            i += 1;
            continue;
        }

        if line.starts_with("def ") || line.starts_with("async def ") {
            let is_async = line.starts_with("async ");
            let clean = line
                .trim_start_matches("async ")
                .trim_start_matches("def ")
                .trim_end_matches(':');

            if let Some(open) = clean.find('(') {
                if let Some(close) = clean.rfind(')') {
                    let name = &clean[..open];
                    let params = &clean[open + 1..close];
                    let after = clean[close + 1..].trim();
                    let ret_type = if let Some(arrow) = after.find("->") {
                        map_py_type_ts(after[arrow + 2..].trim())
                    } else {
                        "void"
                    };
                    let ts_params = translate_py_params_ts(params);
                    let (body_lines, consumed) = collect_py_body(lines.as_slice(), i);

                    let prefix = if is_async { "export async function" } else { "export function" };
                    out.push_str(&format!("{} {}({}): {} {{\n", prefix, name, ts_params, ret_type));
                    for bl in &body_lines {
                        if let Some(stmt) = translate_py_statement_to_ts(bl) {
                            out.push_str("    ");
                            out.push_str(&stmt);
                            out.push('\n');
                        }
                    }
                    out.push_str("}\n\n");
                    i += consumed;
                    continue;
                }
            }
        }

        i += 1;
    }

    out
}

// ===========================================================================
// Shared helpers
// ===========================================================================

fn parse_ts_type_alias(line: &str) -> Option<(String, &str)> {
    let clean = line
        .trim()
        .trim_start_matches("export ")
        .trim_start_matches("type ")
        .trim();
    let (name, rhs) = clean.split_once('=')?;
    Some((name.trim().to_string(), rhs.trim().trim_end_matches(';')))
}

fn sanitize_const_name(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        return "CONST_VALUE".to_string();
    }
    if out.chars().next().unwrap_or('_').is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

fn infer_ts_literal_rust(value: &str) -> Option<(&'static str, String)> {
    let trimmed = value.trim();
    if let Some(lit) = normalize_ts_string_literal(trimmed) {
        return Some(("&str", lit));
    }
    if trimmed == "true" || trimmed == "false" {
        return Some(("bool", trimmed.to_string()));
    }
    if let Ok(_) = trimmed.parse::<i64>() {
        return Some(("i64", trimmed.to_string()));
    }
    if let Ok(_) = trimmed.parse::<f64>() {
        return Some(("f64", trimmed.to_string()));
    }
    None
}

fn normalize_ts_string_literal(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let (inner, is_template) = if trimmed.starts_with('"') && trimmed.ends_with('"') {
        (&trimmed[1..trimmed.len() - 1], false)
    } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        (&trimmed[1..trimmed.len() - 1], false)
    } else if trimmed.starts_with('`') && trimmed.ends_with('`') {
        (&trimmed[1..trimmed.len() - 1], true)
    } else {
        return None;
    };
    if is_template && inner.contains("${") {
        return None;
    }
    let escaped = inner.replace('\\', "\\\\").replace('"', "\\\"");
    Some(format!("\"{}\"", escaped))
}

fn translate_ts_template_to_rust(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if !(trimmed.starts_with('`') && trimmed.ends_with('`')) {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if !inner.contains("${") {
        let lit = normalize_ts_string_literal(trimmed)?;
        return Some(format!("String::from({})", lit));
    }
    let mut fmt = String::new();
    let mut args: Vec<String> = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = inner.chars().collect();
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            let mut j = i + 2;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j >= chars.len() {
                return None;
            }
            let expr: String = chars[i + 2..j].iter().collect();
            let mapped = map_ts_expr_rust(expr.trim());
            args.push(mapped);
            fmt.push_str("{}");
            i = j + 1;
        } else {
            fmt.push(chars[i]);
            i += 1;
        }
    }
    let mut escaped = fmt.replace('\\', "\\\\").replace('"', "\\\"");
    escaped = escaped.replace('{', "{{").replace('}', "}}");
    if escaped.contains("{{}}") {
        escaped = escaped.replace("{{}}", "{}");
    }
    let args_joined = if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    };
    Some(format!("format!(\"{}\"{})", escaped, args_joined))
}

fn map_ts_expr_rust(expr: &str) -> String {
    let mut out = expr.trim().to_string();
    if let Some(rest) = out.strip_prefix("await ") {
        out = rest.trim().to_string();
    }
    if out == "null" || out == "undefined" {
        return "None".to_string();
    }
    out.replace("this.", "self.")
}

fn translate_ts_statement_to_rust(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if trimmed == "{" || trimmed == "}" {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        let expr = map_ts_expr_rust(rest);
        return Some(format!("return {};", expr));
    }
    if trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("var ") {
        let clean = trimmed
            .trim_start_matches("const ")
            .trim_start_matches("let ")
            .trim_start_matches("var ");
        if let Some((name, expr)) = clean.split_once('=') {
            let name = name.split(':').next().unwrap_or(name).trim();
            let expr = map_ts_expr_rust(expr);
            return Some(format!("let {} = {};", to_snake_case(name), expr));
        }
    }
    if trimmed.starts_with("console.log(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("console.log(").trim_end_matches(')');
        let args: Vec<String> = inner
            .split(',')
            .map(|s| map_ts_expr_rust(s))
            .collect();
        if args.len() <= 1 {
            let expr = args.first().cloned().unwrap_or_else(|| "\"\"".to_string());
            return Some(format!("println!(\"{{:?}}\", {});", expr));
        }
        return Some(format!("println!(\"{{:?}}\", ({}));", args.join(", ")));
    }
    if trimmed.ends_with(')') {
        return Some(format!("{};", map_ts_expr_rust(trimmed)));
    }
    None
}

fn map_py_expr_rust(expr: &str) -> String {
    let out = expr.trim().to_string();
    if out == "None" {
        return "None".to_string();
    }
    if out == "True" {
        return "true".to_string();
    }
    if out == "False" {
        return "false".to_string();
    }
    out
}

fn translate_py_statement_to_rust(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        let expr = map_py_expr_rust(rest);
        return Some(format!("return {};", expr));
    }
    if trimmed.starts_with("print(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("print(").trim_end_matches(')');
        let args: Vec<String> = inner.split(',').map(|s| map_py_expr_rust(s)).collect();
        if args.len() <= 1 {
            let expr = args.first().cloned().unwrap_or_else(|| "\"\"".to_string());
            return Some(format!("println!(\"{{:?}}\", {});", expr));
        }
        return Some(format!("println!(\"{{:?}}\", ({}));", args.join(", ")));
    }
    if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") {
        if let Some((name, expr)) = trimmed.split_once('=') {
            let name = name.trim();
            let expr = map_py_expr_rust(expr);
            return Some(format!("let {} = {};", to_snake_case(name), expr));
        }
    }
    if trimmed.ends_with(')') {
        return Some(format!("{};", map_py_expr_rust(trimmed)));
    }
    None
}

fn map_ts_expr_python(expr: &str) -> String {
    let out = expr.trim().to_string();
    if out == "true" {
        return "True".to_string();
    }
    if out == "false" {
        return "False".to_string();
    }
    if out == "null" || out == "undefined" {
        return "None".to_string();
    }
    out
}

fn translate_ts_statement_to_python(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Some(format!("return {}", map_ts_expr_python(rest)));
    }
    if trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("var ") {
        let clean = trimmed
            .trim_start_matches("const ")
            .trim_start_matches("let ")
            .trim_start_matches("var ");
        if let Some((name, expr)) = clean.split_once('=') {
            let name = to_snake_case(name.split(':').next().unwrap_or(name).trim());
            return Some(format!("{} = {}", name, map_ts_expr_python(expr)));
        }
    }
    if trimmed.starts_with("console.log(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("console.log(").trim_end_matches(')');
        return Some(format!("print({})", map_ts_expr_python(inner)));
    }
    if trimmed.ends_with(')') {
        return Some(map_ts_expr_python(trimmed));
    }
    None
}

fn map_ts_expr_go(expr: &str) -> String {
    let out = expr.trim().to_string();
    if out == "true" || out == "false" {
        return out;
    }
    if out == "null" || out == "undefined" {
        return "nil".to_string();
    }
    out
}

fn translate_ts_statement_to_go(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Some(format!("return {}", map_ts_expr_go(rest)));
    }
    if trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("var ") {
        let clean = trimmed
            .trim_start_matches("const ")
            .trim_start_matches("let ")
            .trim_start_matches("var ");
        if let Some((name, expr)) = clean.split_once('=') {
            let name = name.split(':').next().unwrap_or(name).trim();
            return Some(format!("{} := {}", name, map_ts_expr_go(expr)));
        }
    }
    if trimmed.starts_with("console.log(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("console.log(").trim_end_matches(')');
        return Some(format!("fmt.Println({})", map_ts_expr_go(inner)));
    }
    if trimmed.ends_with(')') {
        return Some(format!("{}", map_ts_expr_go(trimmed)));
    }
    None
}

fn map_py_expr_go(expr: &str) -> String {
    let out = expr.trim().to_string();
    if out == "None" {
        return "nil".to_string();
    }
    if out == "True" {
        return "true".to_string();
    }
    if out == "False" {
        return "false".to_string();
    }
    out
}

fn translate_py_statement_to_go(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Some(format!("return {}", map_py_expr_go(rest)));
    }
    if trimmed.starts_with("print(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("print(").trim_end_matches(')');
        return Some(format!("fmt.Println({})", map_py_expr_go(inner)));
    }
    if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") {
        if let Some((name, expr)) = trimmed.split_once('=') {
            let name = name.trim();
            return Some(format!("{} := {}", name, map_py_expr_go(expr)));
        }
    }
    if trimmed.ends_with(')') {
        return Some(format!("{}", map_py_expr_go(trimmed)));
    }
    None
}

fn map_py_expr_ts(expr: &str) -> String {
    let out = expr.trim().to_string();
    if out == "None" {
        return "null".to_string();
    }
    if out == "True" {
        return "true".to_string();
    }
    if out == "False" {
        return "false".to_string();
    }
    out.replace("self.", "this.")
}

fn translate_py_statement_to_ts(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("return ") {
        return Some(format!("return {};", map_py_expr_ts(rest)));
    }
    if trimmed.starts_with("print(") && trimmed.ends_with(')') {
        let inner = trimmed.trim_start_matches("print(").trim_end_matches(')');
        return Some(format!("console.log({});", map_py_expr_ts(inner)));
    }
    if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") {
        if let Some((name, expr)) = trimmed.split_once('=') {
            let name = name.trim();
            return Some(format!("const {} = {};", name, map_py_expr_ts(expr)));
        }
    }
    if trimmed.ends_with(')') {
        return Some(format!("{};", map_py_expr_ts(trimmed)));
    }
    None
}

fn parse_ts_field(line: &str) -> Option<(&str, &str)> {
    let clean = line.trim().trim_end_matches(';').trim_end_matches(',');
    let colon = clean.find(':')?;
    let name = clean[..colon].trim();
    let ty = clean[colon + 1..].trim();
    Some((name, ty))
}

fn translate_ts_params_rust(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut pieces = trimmed.splitn(2, ':');
        let name = pieces.next().unwrap_or("").trim();
        let ty = pieces.next().unwrap_or("any").trim();
        let clean_name = name.trim_end_matches('?');
        let rust_ty = map_ts_type_rust(ty);
        parts.push(format!("{}: {}", clean_name, rust_ty));
    }
    parts.join(", ")
}

fn translate_ts_params_python(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut pieces = trimmed.splitn(2, ':');
        let name = pieces.next().unwrap_or("").trim().trim_end_matches('?');
        let ty = pieces.next().unwrap_or("").trim();
        let py_ty = map_ts_type_python(ty);
        if py_ty.is_empty() {
            parts.push(name.to_string());
        } else {
            parts.push(format!("{}: {}", name, py_ty));
        }
    }
    parts.join(", ")
}

fn translate_ts_params_go(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut pieces = trimmed.splitn(2, ':');
        let name = pieces.next().unwrap_or("").trim().trim_end_matches('?');
        let ty = pieces.next().unwrap_or("any").trim();
        let go_ty = map_ts_type_go(ty);
        parts.push(format!("{} {}", name, go_ty));
    }
    parts.join(", ")
}

fn translate_py_params_go(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() || trimmed == "self" {
            continue;
        }
        if let Some((name, ty)) = trimmed.split_once(':') {
            let name = name.trim();
            let ty = ty.split('=').next().unwrap_or("").trim();
            let go_ty = map_py_type_go(ty);
            parts.push(format!("{} {}", name, go_ty));
        } else {
            let name = trimmed.split('=').next().unwrap_or(trimmed).trim();
            parts.push(format!("{} interface{{}}", name));
        }
    }
    parts.join(", ")
}

fn translate_py_params_ts(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() || trimmed == "self" {
            continue;
        }
        if let Some((name, ty)) = trimmed.split_once(':') {
            let name = name.trim();
            let ty = ty.split('=').next().unwrap_or("").trim();
            let ts_ty = map_py_type_ts(ty);
            parts.push(format!("{}: {}", name, ts_ty));
        } else {
            let name = trimmed.split('=').next().unwrap_or(trimmed).trim();
            parts.push(format!("{}: any", name));
        }
    }
    parts.join(", ")
}

fn translate_rust_params_ts(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((name, ty)) = trimmed.split_once(':') {
            let name = name.trim();
            let ty = ty.trim();
            let ts_ty = map_rust_type_ts(ty);
            parts.push(format!("{}: {}", name, ts_ty));
        } else {
            parts.push(format!("{}: any", trimmed));
        }
    }
    parts.join(", ")
}

// ---------------------------------------------------------------------------
// Type mapping tables
// ---------------------------------------------------------------------------

fn map_ts_type_rust(ts_type: &str) -> &'static str {
    match ts_type.trim() {
        "string" => "String",
        "number" => "i32",
        "boolean" | "bool" => "bool",
        "void" => "()",
        "any" | "unknown" => "String",
        "null" | "undefined" => "()",
        "string[]" | "Array<string>" => "Vec<String>",
        "number[]" | "Array<number>" => "Vec<i32>",
        "boolean[]" | "Array<boolean>" => "Vec<bool>",
        "Date" => "String",
        "Buffer" | "Uint8Array" => "Vec<u8>",
        "object" | "Record<string, any>" => "std::collections::HashMap<String, String>",
        "Promise<string>" => "String",
        "Promise<number>" => "i32",
        "Promise<void>" => "()",
        _ => "String",
    }
}

fn map_rust_type_ts(rust_type: &str) -> &'static str {
    let t = rust_type.trim().trim_end_matches(',');
    match t {
        "String" | "&str" => "string",
        "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "f32" | "f64" => "number",
        "bool" => "boolean",
        "()" => "void",
        s if s.starts_with("Option<") => "any",
        s if s.starts_with("Vec<") => "any[]",
        s if s.starts_with("&[") => "any[]",
        _ => "any",
    }
}

fn map_ts_type_python(ts_type: &str) -> &'static str {
    match ts_type.trim() {
        "string" => "str",
        "number" => "int",
        "boolean" | "bool" => "bool",
        "void" => "None",
        "any" | "unknown" => "",
        "string[]" | "Array<string>" => "list[str]",
        "number[]" | "Array<number>" => "list[int]",
        _ => "",
    }
}

fn map_ts_type_go(ts_type: &str) -> &'static str {
    match ts_type.trim() {
        "string" => "string",
        "number" => "int",
        "boolean" | "bool" => "bool",
        "void" => "",
        "any" | "unknown" => "interface{}",
        "string[]" | "Array<string>" => "[]string",
        "number[]" | "Array<number>" => "[]int",
        _ => "interface{}",
    }
}

fn map_py_type_go(py_type: &str) -> &'static str {
    match py_type.trim() {
        "str" => "string",
        "int" => "int",
        "float" => "float64",
        "bool" => "bool",
        "None" => "",
        "list" | "List" => "[]interface{}",
        "dict" | "Dict" => "map[string]interface{}",
        _ => "interface{}",
    }
}

fn map_py_type_ts(py_type: &str) -> &'static str {
    match py_type.trim() {
        "str" => "string",
        "int" | "float" => "number",
        "bool" => "boolean",
        "None" => "void",
        "list" | "List" => "any[]",
        "dict" | "Dict" => "Record<string, any>",
        _ => "any",
    }
}

// ---------------------------------------------------------------------------
// Body collectors
// ---------------------------------------------------------------------------

/// Collect braces-delimited body lines for JS/TS/Go/Rust/Java.
fn collect_braced_body(lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let mut body = Vec::new();
    let mut brace_depth = 0;
    let mut found_open = false;
    let mut i = start;

    while i < lines.len() {
        let line = lines[i];
        for c in line.chars() {
            if c == '{' {
                brace_depth += 1;
                found_open = true;
            } else if c == '}' {
                brace_depth -= 1;
            }
        }

        if found_open && i > start {
            if brace_depth > 0 {
                body.push(line.to_string());
            } else {
                // closing brace line — don't include it
                i += 1;
                break;
            }
        }

        i += 1;

        if found_open && brace_depth == 0 {
            break;
        }
    }

    let consumed = i - start;
    (body, consumed)
}

/// Collect indentation-based body for Python.
fn collect_py_body(lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let mut body = Vec::new();
    let base_indent = leading_spaces(lines[start]);
    let mut i = start + 1;

    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            body.push(String::new());
            i += 1;
            continue;
        }
        let indent = leading_spaces(line);
        if indent <= base_indent {
            break;
        }
        body.push(line.to_string());
        i += 1;
    }

    let consumed = i - start;
    (body, consumed)
}

fn leading_spaces(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

// ---------------------------------------------------------------------------
// Name converters
// ---------------------------------------------------------------------------

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut s = first.to_uppercase().to_string();
                    s.extend(chars);
                    s
                }
            }
        })
        .collect()
}

fn strip_markdown_fences(text: &str) -> String {
    let mut s = text.trim().to_string();

    // 1. Strip leading ```typescript / ```ts / ``` etc.
    if s.starts_with("```") {
        if let Some(pos) = s.find('\n') {
            s = s[pos + 1..].to_string();
        }
    }
    
    // 2. Strip trailing ``` fence
    if s.ends_with("```") {
        if let Some(pos) = s.rfind("```") {
            s = s[..pos].to_string();
        }
    }
    
    // 3. If there's a ``` in the middle (LLM closed code then wrote prose), cut at the fence
    if let Some(pos) = s.find("\n```\n") {
        s = s[..pos].to_string();
    }
    if let Some(pos) = s.find("\n```") {
        if s[pos + 4..].trim_start().starts_with('\n') || s[pos + 4..].trim().is_empty() || !s[pos + 4..].trim().starts_with(|c: char| c == '{' || c == '(' || c == '[') {
            s = s[..pos].to_string();
        }
    }

    // 4. Strip trailing LLM commentary (lines that start with "This", "Note:", "The code", etc.)
    let lines: Vec<&str> = s.lines().collect();
    let mut end = lines.len();
    while end > 0 {
        let line = lines[end - 1].trim();
        if line.is_empty() {
            end -= 1;
            continue;
        }
        // If the line looks like prose (starts with capital letter, no code-like syntax), strip it
        let is_prose = !line.starts_with("//") 
            && !line.starts_with("/*")
            && !line.starts_with("import ")
            && !line.starts_with("export ")
            && !line.starts_with("class ")
            && !line.starts_with("interface ")
            && !line.starts_with("type ")
            && !line.starts_with("function ")
            && !line.starts_with("const ")
            && !line.starts_with("let ")
            && !line.starts_with("var ")
            && !line.starts_with("async ")
            && !line.starts_with("}")
            && !line.starts_with(")")
            && !line.starts_with("]")
            && !line.starts_with("return ")
            && !line.starts_with("throw ")
            && !line.starts_with("await ")
            && !line.starts_with("if ")
            && !line.starts_with("else")
            && !line.starts_with("for ")
            && !line.starts_with("while ")
            && !line.starts_with("switch ")
            && !line.starts_with("case ")
            && !line.starts_with("default:")
            && !line.starts_with("try ")
            && !line.starts_with("catch ")
            && !line.starts_with("finally ")
            && !line.starts_with("public ")
            && !line.starts_with("private ")
            && !line.starts_with("protected ")
            && !line.starts_with("static ")
            && !line.starts_with("readonly ")
            && !line.starts_with("abstract ")
            && !line.starts_with("@")
            && !line.starts_with("  ")    // indented code
            && !line.starts_with("\t")     // tab-indented code
            && !line.starts_with("*")      // JSDoc
            && (line.starts_with("This ") 
                || line.starts_with("Note:")
                || line.starts_with("Note ")
                || line.starts_with("The ")
                || line.starts_with("I ")
                || line.starts_with("Some ")
                || line.starts_with("However")
                || line.starts_with("In ")
                || line.starts_with("Here ")
                || line.starts_with("Above ")
                || line.starts_with("Below ")
                || line.ends_with(".")
                || line.ends_with(":")
            );
        if is_prose {
            end -= 1;
        } else {
            break;
        }
    }
    
    lines[..end].join("\n").trim().to_string()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_function_to_rust() {
        let ts = r#"export function greet(name: string): string {
    return `Hello, ${name}`;
}

export function add(a: number, b: number): number {
    return a + b;
}
"#;
        let rust = translate_ts_to_rust(ts);
        assert!(rust.contains("pub fn greet("));
        assert!(rust.contains("-> String"));
        assert!(rust.contains("pub fn add("));
        assert!(rust.contains("-> i32"));
    }

    #[test]
    fn test_ts_interface_to_rust() {
        let ts = r#"export interface User {
    id: number;
    name: string;
    email?: string;
}
"#;
        let rust = translate_ts_to_rust(ts);
        assert!(rust.contains("pub struct User"));
        assert!(rust.contains("pub id: i32"));
        assert!(rust.contains("pub name: String"));
        assert!(rust.contains("Option<String>"));
    }

    #[test]
    fn test_py_function_to_rust() {
        let py = r#"def greet(name: str) -> str:
    return f"Hello, {name}"

def add(a: int, b: int) -> int:
    return a + b
"#;
        let rust = translate_py_to_rust(py);
        assert!(rust.contains("pub fn greet("));
        assert!(rust.contains("-> String"));
        assert!(rust.contains("pub fn add("));
        assert!(rust.contains("-> i64"));
    }

    #[test]
    fn test_ts_to_python() {
        let ts = r#"export function greet(name: string): string {
    return `Hello, ${name}`;
}
"#;
        let py = translate_ts_to_python(ts);
        assert!(py.contains("def greet(name: str):"));
    }

    #[test]
    fn test_ts_to_go() {
        let ts = r#"export function greet(name: string): string {
    return `Hello, ${name}`;
}
"#;
        let go_code = translate_ts_to_go(ts);
        assert!(go_code.contains("package main"));
        assert!(go_code.contains("func Greet("));
        assert!(go_code.contains("string"));
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("getUserName"), "get_user_name");
        assert_eq!(to_snake_case("greet"), "greet");
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("get_user_name"), "GetUserName");
        assert_eq!(to_pascal_case("greet"), "Greet");
    }
}
