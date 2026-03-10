use std::path::Path;

use anyhow::{anyhow, Result};
use tree_sitter::{Language, Node, Parser};
use tree_sitter_rust as ts_rust;

pub fn is_rust_source(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => true,
        _ => false,
    }
}

pub struct RustSymbol {
    pub name: String,
    pub kind: RustSymbolKind,
}

pub enum RustSymbolKind {
    Struct,
    Enum,
    Function,
}

pub fn parse_rust_file(path: &Path, contents: &str) -> Result<Vec<RustSymbol>> {
    let _ = path;
    let mut parser = Parser::new();
    let lang: Language = Language::from(ts_rust::LANGUAGE);
    parser
        .set_language(&lang)
        .map_err(|e| anyhow!("failed to set Rust language: {:?}", e))?;

    let tree = parser
        .parse(contents, None)
        .ok_or_else(|| anyhow!("failed to parse Rust source"))?;

    let root = tree.root_node();
    let mut cursor = root.walk();
    let mut symbols = Vec::new();

    for child in root.children(&mut cursor) {
        match child.kind() {
            "struct_item" => {
                if let Some(name) = identifier_name(child, contents) {
                    symbols.push(RustSymbol {
                        name,
                        kind: RustSymbolKind::Struct,
                    });
                }
            }
            "enum_item" => {
                if let Some(name) = identifier_name(child, contents) {
                    symbols.push(RustSymbol {
                        name,
                        kind: RustSymbolKind::Enum,
                    });
                }
            }
            "function_item" => {
                if let Some(name) = identifier_name(child, contents) {
                    symbols.push(RustSymbol {
                        name,
                        kind: RustSymbolKind::Function,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(symbols)
}

fn identifier_name(node: Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string())
}
