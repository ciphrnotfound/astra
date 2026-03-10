use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct CodeIndex {
    files: HashMap<PathBuf, FileSummary>,
}

pub struct FileSummary {
    pub line_count: usize,
    pub language: String,
    pub approx_fn_count: usize,
}

impl CodeIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, path: PathBuf, contents: &str) {
        let line_count = contents.lines().count();
        let language = language_for_path(&path);
        let approx_fn_count = if language == "rust" {
            contents.matches("fn ").count()
        } else {
            0
        };
        let summary = FileSummary {
            line_count,
            language,
            approx_fn_count,
        };
        self.files.insert(path, summary);
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
}

pub struct IndexStats {
    pub file_count: usize,
    pub total_lines: usize,
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
