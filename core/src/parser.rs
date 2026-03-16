use std::path::Path;

use anyhow::{anyhow, Result};
use tree_sitter::{Language, Node, Parser, TreeCursor};
use tree_sitter_go as ts_go;
use tree_sitter_java as ts_java;
use tree_sitter_javascript as ts_javascript;
use tree_sitter_python as ts_python;
use tree_sitter_rust as ts_rust;
use tree_sitter_typescript as ts_typescript;

pub struct ParsedSymbol {
    pub name: String,
    pub kind: ParsedSymbolKind,
}

pub enum ParsedSymbolKind {
    Function,
    Struct,
    Class,
    Interface,
    Enum,
    Type,
    Constant,
}

pub fn parse_rust_file(path: &Path, contents: &str) -> Result<Vec<ParsedSymbol>> {
    let _ = path;
    parse_with_language(Language::from(ts_rust::LANGUAGE), contents, |node, source, out| {
        match node.kind() {
            "struct_item" => push_symbol(node, source, ParsedSymbolKind::Struct, out),
            "enum_item" => push_symbol(node, source, ParsedSymbolKind::Enum, out),
            "function_item" => push_symbol(node, source, ParsedSymbolKind::Function, out),
            _ => {}
        }
    })
}

pub fn parse_typescript_file(contents: &str) -> Result<Vec<ParsedSymbol>> {
    parse_with_language(Language::from(ts_typescript::LANGUAGE_TYPESCRIPT), contents, |node, source, out| {
        match node.kind() {
            "class_declaration" => push_symbol(node, source, ParsedSymbolKind::Class, out),
            "interface_declaration" => push_symbol(node, source, ParsedSymbolKind::Interface, out),
            "type_alias_declaration" => push_symbol(node, source, ParsedSymbolKind::Type, out),
            "enum_declaration" => push_symbol(node, source, ParsedSymbolKind::Enum, out),
            "function_declaration" | "method_definition" => {
                push_symbol(node, source, ParsedSymbolKind::Function, out)
            }
            "lexical_declaration" | "variable_declaration" => {
                push_ts_variable_symbols(node, source, out)
            }
            _ => {}
        }
    })
}

pub fn parse_javascript_file(contents: &str) -> Result<Vec<ParsedSymbol>> {
    parse_with_language(Language::from(ts_javascript::LANGUAGE), contents, |node, source, out| {
        match node.kind() {
            "class_declaration" => push_symbol(node, source, ParsedSymbolKind::Class, out),
            "function_declaration" | "method_definition" => {
                push_symbol(node, source, ParsedSymbolKind::Function, out)
            }
            "lexical_declaration" | "variable_declaration" => {
                push_ts_variable_symbols(node, source, out)
            }
            _ => {}
        }
    })
}

pub fn parse_python_file(contents: &str) -> Result<Vec<ParsedSymbol>> {
    parse_with_language(Language::from(ts_python::LANGUAGE), contents, |node, source, out| {
        match node.kind() {
            "function_definition" => push_symbol(node, source, ParsedSymbolKind::Function, out),
            "class_definition" => push_symbol(node, source, ParsedSymbolKind::Class, out),
            _ => {}
        }
    })
}

pub fn parse_go_file(contents: &str) -> Result<Vec<ParsedSymbol>> {
    parse_with_language(Language::from(ts_go::LANGUAGE), contents, |node, source, out| {
        match node.kind() {
            "function_declaration" | "method_declaration" => {
                push_symbol(node, source, ParsedSymbolKind::Function, out)
            }
            "type_spec" => {
                if node
                    .child_by_field_name("type")
                    .map(|t| t.kind() == "struct_type")
                    .unwrap_or(false)
                {
                    push_symbol(node, source, ParsedSymbolKind::Struct, out)
                } else if node
                    .child_by_field_name("type")
                    .map(|t| t.kind() == "interface_type")
                    .unwrap_or(false)
                {
                    push_symbol(node, source, ParsedSymbolKind::Interface, out)
                } else {
                    push_symbol(node, source, ParsedSymbolKind::Type, out)
                }
            }
            _ => {}
        }
    })
}

pub fn parse_java_file(contents: &str) -> Result<Vec<ParsedSymbol>> {
    parse_with_language(Language::from(ts_java::LANGUAGE), contents, |node, source, out| {
        match node.kind() {
            "class_declaration" => push_symbol(node, source, ParsedSymbolKind::Class, out),
            "interface_declaration" => push_symbol(node, source, ParsedSymbolKind::Interface, out),
            "enum_declaration" => push_symbol(node, source, ParsedSymbolKind::Enum, out),
            "method_declaration" => push_symbol(node, source, ParsedSymbolKind::Function, out),
            _ => {}
        }
    })
}

fn parse_with_language<F>(
    lang: Language,
    contents: &str,
    mut on_node: F,
) -> Result<Vec<ParsedSymbol>>
where
    F: FnMut(Node, &str, &mut Vec<ParsedSymbol>),
{
    let mut parser = Parser::new();
    parser
        .set_language(&lang)
        .map_err(|e| anyhow!("failed to set language: {:?}", e))?;

    let tree = parser
        .parse(contents, None)
        .ok_or_else(|| anyhow!("failed to parse source"))?;

    let root = tree.root_node();
    let mut symbols = Vec::new();
    let mut cursor = root.walk();
    walk_nodes(&mut cursor, contents, &mut on_node, &mut symbols);
    Ok(symbols)
}

fn walk_nodes<F>(
    cursor: &mut TreeCursor<'_>,
    source: &str,
    on_node: &mut F,
    symbols: &mut Vec<ParsedSymbol>,
)
where
    F: FnMut(Node, &str, &mut Vec<ParsedSymbol>),
{
    loop {
        let node = cursor.node();
        on_node(node, source, symbols);

        if cursor.goto_first_child() {
            walk_nodes(cursor, source, on_node, symbols);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

fn push_symbol(node: Node, source: &str, kind: ParsedSymbolKind, out: &mut Vec<ParsedSymbol>) {
    if let Some(name) = identifier_name(node, source) {
        out.push(ParsedSymbol { name, kind });
    }
}

fn identifier_name(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string())
}

fn push_ts_variable_symbols(node: Node, source: &str, out: &mut Vec<ParsedSymbol>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator" {
            let is_function = child
                .child_by_field_name("value")
                .map(|v| v.kind() == "arrow_function" || v.kind() == "function")
                .unwrap_or(false);
            if let Some(name) = child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            {
                out.push(ParsedSymbol {
                    name: name.to_string(),
                    kind: if is_function {
                        ParsedSymbolKind::Function
                    } else {
                        ParsedSymbolKind::Constant
                    },
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Import / dependency extraction
// ---------------------------------------------------------------------------

/// Represents a single import found in source code.
#[derive(Debug, Clone)]
pub struct ParsedImport {
    /// The module or path being imported (e.g. "std::fs", "./utils", "fmt")
    pub module: String,
    /// Specific symbols imported, if any (e.g. ["HashMap", "HashSet"])
    pub symbols: Vec<String>,
}

/// Extract imports from a Rust file.
pub fn extract_rust_imports(contents: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") {
            let rest = trimmed.trim_start_matches("use ").trim_end_matches(';').trim();
            if let Some(brace_start) = rest.find('{') {
                let module = rest[..brace_start].trim_end_matches("::").to_string();
                let end = if rest.ends_with('}') { rest.len() - 1 } else { rest.len() };
                let inner = if brace_start + 1 <= end { &rest[brace_start + 1..end] } else { "" };
                let symbols: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                imports.push(ParsedImport { module, symbols });
            } else {
                let parts: Vec<&str> = rest.rsplitn(2, "::").collect();
                if parts.len() == 2 {
                    imports.push(ParsedImport {
                        module: parts[1].to_string(),
                        symbols: vec![parts[0].to_string()],
                    });
                } else {
                    imports.push(ParsedImport {
                        module: rest.to_string(),
                        symbols: Vec::new(),
                    });
                }
            }
        }
    }
    imports
}

/// Extract imports from a Python file.
pub fn extract_python_imports(contents: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("from ") {
            // from foo.bar import baz, qux
            let rest = trimmed.trim_start_matches("from ").trim();
            if let Some(idx) = rest.find(" import ") {
                let module = rest[..idx].trim().to_string();
                let symbols: Vec<String> = rest[idx + 8..]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                imports.push(ParsedImport { module, symbols });
            }
        } else if trimmed.starts_with("import ") {
            let rest = trimmed.trim_start_matches("import ").trim();
            for part in rest.split(',') {
                let module = part.split(" as ").next().unwrap_or("").trim().to_string();
                if !module.is_empty() {
                    imports.push(ParsedImport {
                        module,
                        symbols: Vec::new(),
                    });
                }
            }
        }
    }
    imports
}

/// Extract imports from a TypeScript/JavaScript file.
pub fn extract_ts_imports(contents: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("import") {
            continue;
        }
        // import { Foo, Bar } from './module'
        // import Foo from 'module'
        // import * as Foo from 'module'
        if let Some(from_idx) = trimmed.find("from ") {
            let module_part = &trimmed[from_idx + 5..];
            let module = module_part
                .trim()
                .trim_end_matches(';')
                .trim_matches(|c| c == '\'' || c == '"')
                .to_string();

            let mut symbols = Vec::new();
            if let Some(brace_start) = trimmed.find('{') {
                if let Some(brace_end) = trimmed.find('}') {
                    symbols = trimmed[brace_start + 1..brace_end]
                        .split(',')
                        .map(|s| s.split(" as ").next().unwrap_or("").trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
            imports.push(ParsedImport { module, symbols });
        }
    }
    imports
}

/// Extract imports from a Go file.
pub fn extract_go_imports(contents: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    let mut in_import_block = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "import (" {
            in_import_block = true;
            continue;
        }
        if in_import_block {
            if trimmed == ")" {
                in_import_block = false;
                continue;
            }
            let module = trimmed
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
            if !module.is_empty() {
                imports.push(ParsedImport {
                    module,
                    symbols: Vec::new(),
                });
            }
        } else if trimmed.starts_with("import \"") || trimmed.starts_with("import\t\"") {
            let module = trimmed
                .trim_start_matches("import")
                .trim()
                .trim_matches('"')
                .to_string();
            if !module.is_empty() {
                imports.push(ParsedImport {
                    module,
                    symbols: Vec::new(),
                });
            }
        }
    }
    imports
}

/// Extract imports from a Java file.
pub fn extract_java_imports(contents: &str) -> Vec<ParsedImport> {
    let mut imports = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            let rest = trimmed
                .trim_start_matches("import ")
                .trim_start_matches("static ")
                .trim_end_matches(';')
                .trim()
                .to_string();
            if !rest.is_empty() {
                let parts: Vec<&str> = rest.rsplitn(2, '.').collect();
                if parts.len() == 2 {
                    imports.push(ParsedImport {
                        module: parts[1].to_string(),
                        symbols: vec![parts[0].to_string()],
                    });
                } else {
                    imports.push(ParsedImport {
                        module: rest,
                        symbols: Vec::new(),
                    });
                }
            }
        }
    }
    imports
}

