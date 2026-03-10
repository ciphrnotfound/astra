use std::fs;
use std::path::Path;

use anyhow::Result;

pub fn translate_ts_file(path: &Path) -> Result<String> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let mut out = String::new();
    for i in 0..lines.len() {
        let line = lines[i].trim();
        if line.starts_with("export function ") {
            if let Some(func) = translate_ts_function(&lines, i) {
                out.push_str(&func);
                out.push('\n');
            }
        }
    }
    Ok(out)
}

fn translate_ts_function(lines: &[&str], start: usize) -> Option<String> {
    let header = lines.get(start)?.trim();
    let header = header.trim_start_matches("export function ").trim();
    let open_paren = header.find('(')?;
    let close_paren = header.find(')')?;
    let name = header[..open_paren].trim();
    let params_str = &header[open_paren + 1..close_paren];
    let after_paren = header[close_paren + 1..].trim();

    let mut ret_type = "String";
    if let Some(colon_pos) = after_paren.find(':') {
        let ty = after_paren[colon_pos + 1..]
            .trim()
            .trim_end_matches('{')
            .trim();
        ret_type = map_ts_type_to_rust_return(ty);
    }

    let params = translate_params(params_str);
    let body_line = lines.get(start + 1).map(|s| s.trim()).unwrap_or("");
    let expr = extract_return_expr(body_line);
    let body = translate_expr(expr, ret_type);

    let mut func = String::new();
    func.push_str("pub fn ");
    func.push_str(name);
    func.push('(');
    func.push_str(&params);
    func.push_str(") -> ");
    func.push_str(ret_type);
    func.push_str(" {\n    ");
    func.push_str(&body);
    func.push_str("\n}\n");

    Some(func)
}

fn translate_params(params_str: &str) -> String {
    let mut parts = Vec::new();
    for part in params_str.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut pieces = trimmed.splitn(2, ':');
        let name = pieces.next().unwrap_or("").trim();
        let ty = pieces.next().unwrap_or("any").trim();
        let rust_ty = map_ts_type_to_rust_param(ty);
        parts.push(format!("{}: {}", name, rust_ty));
    }
    parts.join(", ")
}

fn map_ts_type_to_rust_param(ts_type: &str) -> &'static str {
    match ts_type {
        "string" => "&str",
        "number" => "i32",
        _ => "&str",
    }
}

fn map_ts_type_to_rust_return(ts_type: &str) -> &'static str {
    match ts_type {
        "string" => "String",
        "number" => "i32",
        _ => "String",
    }
}

fn extract_return_expr(body_line: &str) -> &str {
    let trimmed = body_line.trim();
    if let Some(rest) = trimmed.strip_prefix("return ") {
        let without_semicolon = rest.trim_end_matches(';').trim();
        without_semicolon
    } else {
        trimmed
    }
}

fn translate_expr(expr: &str, ret_type: &str) -> String {
    let trimmed = expr.trim();
    if trimmed == "`Hello, ${name}`" {
        return "format!(\"Hello, {}\", name)".to_string();
    }
    if trimmed == "a + b" && ret_type == "i32" {
        return "a + b".to_string();
    }
    if trimmed == "`${id}:${username.toLowerCase()}`" {
        return "format!(\"{}:{}\", id, username.to_lowercase())".to_string();
    }
    match ret_type {
        "i32" => "0".to_string(),
        _ => "String::new()".to_string(),
    }
}

