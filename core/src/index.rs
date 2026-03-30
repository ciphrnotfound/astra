use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use anyhow::Result;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use crate::parser::{
    parse_go_file, parse_java_file, parse_javascript_file, parse_python_file,
    parse_rust_file, parse_typescript_file, ParsedSymbolKind,
    extract_rust_imports, extract_python_imports, extract_ts_imports,
    extract_go_imports, extract_java_imports, ParsedImport,
};

#[derive(Serialize, Deserialize)]
pub struct CodeIndex {
    files: HashMap<PathBuf, FileSummary>,
    graph: SemanticGraph,
}

#[derive(Serialize, Deserialize)]
pub struct FileSummary {
    pub line_count: usize,
    pub language: String,
    pub approx_fn_count: usize,
    pub symbols: Vec<SymbolSummary>,
    pub imports: Vec<ParsedImport>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Interface,
    Enum,
    Type,
    Constant,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SymbolSummary {
    pub name: String,
    pub kind: SymbolKind,
}

impl Default for CodeIndex {
    fn default() -> Self {
        Self {
            files: HashMap::new(),
            graph: SemanticGraph::new(),
        }
    }
}

impl CodeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, path: PathBuf, contents: &str) {
        let line_count = contents.lines().count();
        let language = language_for_path(&path);
        let symbols = extract_symbols(&language, contents, &path);
        let imports = extract_file_imports(&language, contents);
        let approx_fn_count = symbols
            .iter()
            .filter(|s| matches!(s.kind, SymbolKind::Function))
            .count();
        let summary = FileSummary {
            line_count,
            language,
            approx_fn_count,
            symbols,
            imports,
        };
        self.graph.add_file(&path, &summary);
        self.files.insert(path, summary);
    }

    /// Second pass: resolve cross-file import edges after all files are indexed.
    pub fn resolve_imports(&mut self) {
        self.graph.resolve_cross_file_references(&self.files);
    }

    /// Find all files that depend on a given symbol name.
    pub fn find_dependents(&self, symbol_name: &str) -> Vec<PathBuf> {
        self.files
            .iter()
            .filter(|(_, summary)| {
                summary.imports.iter().any(|imp| {
                    imp.symbols.iter().any(|s| s == symbol_name)
                        || imp.module.contains(symbol_name)
                })
            })
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Find all modules/symbols that a given file depends on.
    pub fn find_dependencies(&self, path: &Path) -> Vec<String> {
        if let Some(summary) = self.files.get(path) {
            summary.imports.iter().map(|imp| imp.module.clone()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn stats(&self) -> IndexStats {
        let file_count = self.files.len();
        let total_lines = self
            .files
            .values()
            .map(|f| f.line_count)
            .sum();
        IndexStats {
            file_count,
            total_lines,
        }
    }

    pub fn files_by_language(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for summary in self.files.values() {
            *counts.entry(summary.language.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn symbols_by_language(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for summary in self.files.values() {
            let entry = counts.entry(summary.language.clone()).or_insert(0);
            *entry += summary.symbols.len();
        }
        counts
    }

    pub fn files(&self) -> &HashMap<PathBuf, FileSummary> {
        &self.files
    }

    pub fn lines_by_language(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for summary in self.files.values() {
            *counts.entry(summary.language.clone()).or_insert(0) += summary.line_count;
        }
        counts
    }

    pub fn total_fn_count(&self) -> usize {
        self.files.values().map(|f| f.approx_fn_count).sum()
    }

    pub fn total_symbol_count(&self) -> usize {
        self.files.values().map(|f| f.symbols.len()).sum()
    }

    pub fn graph_stats(&self) -> GraphStats {
        self.graph.stats()
    }

    pub fn graph_dot(&self) -> String {
        self.graph.to_dot()
    }

    /// Return (path, line_count, fn_count) for every indexed file.
    pub fn all_file_stats(&self) -> Vec<(PathBuf, usize, usize)> {
        self.files
            .iter()
            .map(|(path, summary)| {
                (path.clone(), summary.line_count, summary.approx_fn_count)
            })
            .collect()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let index: Self = serde_json::from_str(&data)?;
        Ok(index)
    }
}

pub struct IndexStats {
    pub file_count: usize,
    pub total_lines: usize,
}

pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_nodes: usize,
    pub symbol_nodes: usize,
}

pub fn is_indexable_path(path: &Path) -> bool {
    path.is_file()
}

fn language_for_path(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("tsx") => "typescript".to_string(),
        Some("js") => "javascript".to_string(),
        Some("jsx") => "javascript".to_string(),
        Some("py") => "python".to_string(),
        Some("go") => "go".to_string(),
        Some("java") => "java".to_string(),
        Some(ext) => ext.to_lowercase(),
        None => "unknown".to_string(),
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum GraphNode {
    File { path: PathBuf, language: String },
    Symbol { name: String, kind: SymbolKind, language: String },
}

#[derive(Clone, Serialize, Deserialize)]
enum GraphEdge {
    Contains,
    Imports,
    References,
}

#[derive(Serialize, Deserialize)]
struct SemanticGraph {
    graph: Graph<GraphNode, GraphEdge>,
    file_nodes: HashMap<PathBuf, NodeIndex>,
}

impl SemanticGraph {
    fn new() -> Self {
        Self {
            graph: Graph::new(),
            file_nodes: HashMap::new(),
        }
    }

    fn add_file(&mut self, path: &PathBuf, summary: &FileSummary) {
        let file_node = match self.file_nodes.get(path) {
            Some(idx) => *idx,
            None => {
                let idx = self.graph.add_node(GraphNode::File {
                    path: path.clone(),
                    language: summary.language.clone(),
                });
                self.file_nodes.insert(path.clone(), idx);
                idx
            }
        };

        // Track symbol nodes for cross-file resolution
        for symbol in &summary.symbols {
            let symbol_node = self.graph.add_node(GraphNode::Symbol {
                name: symbol.name.clone(),
                kind: symbol.kind.clone(),
                language: summary.language.clone(),
            });
            self.graph.add_edge(file_node, symbol_node, GraphEdge::Contains);
        }
    }

    fn stats(&self) -> GraphStats {
        let mut file_nodes = 0;
        let mut symbol_nodes = 0;
        for node in self.graph.node_indices() {
            match self.graph[node] {
                GraphNode::File { .. } => file_nodes += 1,
                GraphNode::Symbol { .. } => symbol_nodes += 1,
            }
        }
        GraphStats {
            node_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
            file_nodes,
            symbol_nodes,
        }
    }

    fn to_dot(&self) -> String {
        let mut out = String::new();
        out.push_str("digraph astra {\n");
        for node in self.graph.node_indices() {
            let label = match &self.graph[node] {
                GraphNode::File { path, language } => {
                    format!("file\\n{}\\n{}", path.display(), language)
                }
                GraphNode::Symbol { name, kind, language } => {
                    format!("symbol\\n{} {}\\n{}", symbol_kind_name(kind), name, language)
                }
            };
            let _ = writeln!(
                &mut out,
                "  n{} [label=\"{}\"];",
                node.index(),
                escape_dot_label(&label)
            );
        }
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                let _ = writeln!(&mut out, "  n{} -> n{};", source.index(), target.index());
            }
        }
        out.push_str("}\n");
        out
    }

    fn resolve_cross_file_references(&mut self, files: &HashMap<PathBuf, FileSummary>) {
        // Build a map of symbol_name -> file that defines it
        let mut symbol_to_file: HashMap<String, NodeIndex> = HashMap::new();
        for node_idx in self.graph.node_indices() {
            if let GraphNode::Symbol { ref name, .. } = self.graph[node_idx] {
                // Find the parent file node
                for edge in self.graph.edges_directed(node_idx, petgraph::Direction::Incoming) {
                    if matches!(self.graph[edge.id()], GraphEdge::Contains) {
                        symbol_to_file.insert(name.clone(), edge.source());
                    }
                }
            }
        }

        // For each file, check its imports against known symbols
        for (path, summary) in files {
            if let Some(&importer_node) = self.file_nodes.get(path) {
                for imp in &summary.imports {
                    // Check if any imported symbol matches a defined symbol
                    for sym_name in &imp.symbols {
                        if let Some(&definer_node) = symbol_to_file.get(sym_name) {
                            if definer_node != importer_node {
                                self.graph.add_edge(importer_node, definer_node, GraphEdge::Imports);
                            }
                        }
                    }

                    // Also check if module name matches a file path
                    for (other_path, other_idx) in &self.file_nodes {
                        if other_path == path {
                            continue;
                        }
                        let stem = other_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("");
                        if imp.module.contains(stem) {
                            self.graph.add_edge(importer_node, *other_idx, GraphEdge::Imports);
                        }
                    }
                }
            }
        }
    }
}

fn symbol_kind_name(kind: &SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "fn",
        SymbolKind::Struct => "struct",
        SymbolKind::Class => "class",
        SymbolKind::Interface => "interface",
        SymbolKind::Enum => "enum",
        SymbolKind::Type => "type",
        SymbolKind::Constant => "const",
    }
}

fn escape_dot_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn extract_symbols(language: &str, contents: &str, path: &Path) -> Vec<SymbolSummary> {
    match language {
        "rust" => extract_rust_symbols(contents, path),
        "typescript" => extract_ts_symbols(contents),
        "javascript" => extract_js_symbols(contents),
        "python" => extract_python_symbols(contents),
        "go" => extract_go_symbols(contents),
        "java" => extract_java_symbols(contents),
        _ => Vec::new(),
    }
}

fn extract_file_imports(language: &str, contents: &str) -> Vec<ParsedImport> {
    match language {
        "rust" => extract_rust_imports(contents),
        "python" => extract_python_imports(contents),
        "typescript" | "javascript" => extract_ts_imports(contents),
        "go" => extract_go_imports(contents),
        "java" => extract_java_imports(contents),
        _ => Vec::new(),
    }
}

fn extract_rust_symbols(contents: &str, path: &Path) -> Vec<SymbolSummary> {
    match parse_rust_file(path, contents) {
        Ok(items) => map_parsed_symbols(items),
        Err(_) => Vec::new(),
    }
}

fn extract_ts_symbols(contents: &str) -> Vec<SymbolSummary> {
    if let Ok(items) = parse_typescript_file(contents) {
        return map_parsed_symbols(items);
    }
    let mut out = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        let mut clean = trimmed;
        if let Some(rest) = clean.strip_prefix("export ") {
            clean = rest.trim_start();
        }
        if let Some(rest) = clean.strip_prefix("default ") {
            clean = rest.trim_start();
        }
        if let Some(name) = name_after_keyword(clean, "class") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Class,
            });
            continue;
        }
        if let Some(name) = name_after_keyword(clean, "interface") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Interface,
            });
            continue;
        }
        if let Some(name) = name_after_keyword(clean, "type") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Type,
            });
            continue;
        }
        if let Some(name) = name_after_keyword(clean, "function") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Function,
            });
            continue;
        }
        if clean.starts_with("const ")
            || clean.starts_with("let ")
            || clean.starts_with("var ")
        {
            if clean.contains("=>") {
                if let Some(name) = name_after_keyword(clean, "const")
                    .or_else(|| name_after_keyword(clean, "let"))
                    .or_else(|| name_after_keyword(clean, "var"))
                {
                    out.push(SymbolSummary {
                        name,
                        kind: SymbolKind::Function,
                    });
                }
            } else if let Some(name) = name_after_keyword(clean, "const") {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Constant,
                });
            }
        }
    }
    out
}

fn extract_js_symbols(contents: &str) -> Vec<SymbolSummary> {
    if let Ok(items) = parse_javascript_file(contents) {
        return map_parsed_symbols(items);
    }
    extract_ts_symbols(contents)
}

fn extract_python_symbols(contents: &str) -> Vec<SymbolSummary> {
    if let Ok(items) = parse_python_file(contents) {
        return map_parsed_symbols(items);
    }
    let mut out = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(name) = name_after_keyword(trimmed, "def") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Function,
            });
            continue;
        }
        if let Some(name) = name_after_keyword(trimmed, "class") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Class,
            });
        }
    }
    out
}

fn extract_go_symbols(contents: &str) -> Vec<SymbolSummary> {
    if let Ok(items) = parse_go_file(contents) {
        return map_parsed_symbols(items);
    }
    let mut out = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("func ") {
            if let Some(name) = parse_go_func_name(trimmed) {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Function,
                });
            }
            continue;
        }
        if let Some(name) = name_after_keyword(trimmed, "type") {
            if trimmed.contains(" struct") {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Struct,
                });
            } else if trimmed.contains(" interface") {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Interface,
                });
            } else {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Type,
                });
            }
        }
    }
    out
}

fn extract_java_symbols(contents: &str) -> Vec<SymbolSummary> {
    if let Ok(items) = parse_java_file(contents) {
        return map_parsed_symbols(items);
    }
    let mut out = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        if let Some(name) = name_after_keyword_anywhere(trimmed, "class") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Class,
            });
            continue;
        }
        if let Some(name) = name_after_keyword_anywhere(trimmed, "interface") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Interface,
            });
            continue;
        }
        if let Some(name) = name_after_keyword_anywhere(trimmed, "enum") {
            out.push(SymbolSummary {
                name,
                kind: SymbolKind::Enum,
            });
            continue;
        }
        if looks_like_java_method(trimmed) {
            if let Some(name) = name_before_paren(trimmed) {
                out.push(SymbolSummary {
                    name,
                    kind: SymbolKind::Function,
                });
            }
        }
    }
    out
}

fn map_parsed_symbols(items: Vec<crate::parser::ParsedSymbol>) -> Vec<SymbolSummary> {
    items
        .into_iter()
        .map(|s| SymbolSummary {
            name: s.name,
            kind: match s.kind {
                ParsedSymbolKind::Function => SymbolKind::Function,
                ParsedSymbolKind::Struct => SymbolKind::Struct,
                ParsedSymbolKind::Class => SymbolKind::Class,
                ParsedSymbolKind::Interface => SymbolKind::Interface,
                ParsedSymbolKind::Enum => SymbolKind::Enum,
                ParsedSymbolKind::Type => SymbolKind::Type,
                ParsedSymbolKind::Constant => SymbolKind::Constant,
            },
        })
        .collect()
}

fn name_after_keyword(line: &str, keyword: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    while let Some(token) = parts.next() {
        if token == keyword {
            if let Some(next) = parts.next() {
                return sanitize_identifier(next);
            }
        }
    }
    None
}

fn name_after_keyword_anywhere(line: &str, keyword: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    while let Some(token) = parts.next() {
        if token == keyword {
            if let Some(next) = parts.next() {
                return sanitize_identifier(next);
            }
        }
    }
    None
}

fn sanitize_identifier(token: &str) -> Option<String> {
    let mut start = None;
    let mut end = None;
    for (idx, ch) in token.char_indices() {
        if ch.is_alphanumeric() || ch == '_' {
            if start.is_none() {
                start = Some(idx);
            }
            end = Some(idx + ch.len_utf8());
        }
    }
    match (start, end) {
        (Some(s), Some(e)) if s < e => Some(token[s..e].to_string()),
        _ => None,
    }
}

fn parse_go_func_name(line: &str) -> Option<String> {
    let rest = line.trim_start_matches("func").trim_start();
    if rest.starts_with('(') {
        if let Some(close) = rest.find(')') {
            let after = rest[close + 1..].trim_start();
            return first_identifier(after);
        }
        return None;
    }
    first_identifier(rest)
}

fn first_identifier(s: &str) -> Option<String> {
    let mut name = String::new();
    let mut started = false;
    for ch in s.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn looks_like_java_method(line: &str) -> bool {
    if !(line.contains('(') && line.contains(')') && line.contains('{')) {
        return false;
    }
    let lower = line.trim_start().to_ascii_lowercase();
    !(lower.starts_with("if ")
        || lower.starts_with("for ")
        || lower.starts_with("while ")
        || lower.starts_with("switch ")
        || lower.starts_with("catch "))
}

fn name_before_paren(line: &str) -> Option<String> {
    let before = line.split('(').next()?.trim_end();
    let mut last = None;
    for token in before.split_whitespace() {
        last = Some(token);
    }
    last.and_then(sanitize_identifier)
}
