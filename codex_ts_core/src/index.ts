// TODO: use std::collections::HashMap;
// TODO: use std::path::{Path, PathBuf};

// #[derive(Default)]
export interface CodeIndex {
  files: any;
}


export interface FileSummary {
  line_count: number;
  language: string;
  approx_fn_count: number;
}


// impl CodeIndex {
export function new(): any {
  // Self::default()
}


export function add_file(&mut self: any, path: any, contents: string): void {
  // let line_count = contents.lines().count();
  // let language = language_for_path(&path);
  // let approx_fn_count = if language == "rust" {
  // contents.matches("fn ").count()
  // } else {
  // 0
  // };
  // let summary = FileSummary {
  // line_count,
  // language,
  // approx_fn_count,
  // };
  // self.files.insert(path, summary);
}


export function stats(&self: any): any {
  // let file_count = self.files.len();
  // let total_lines = self
  // .files
  // .values()
  // .map(|f| f.line_count)
  // .sum();
  // IndexStats {
  // file_count,
  // total_lines,
  // }
}


export function files_by_language(&self: any): any {
  // let mut counts: HashMap<String, usize> = HashMap::new();
  // for summary in self.files.values() {
  // *counts.entry(summary.language.clone()).or_insert(0) += 1;
  // }
  // counts
}


export function files(&self: any): any {
  // &self.files
}


export function lines_by_language(&self: any): any {
  // let mut counts: HashMap<String, usize> = HashMap::new();
  // for summary in self.files.values() {
  // *counts.entry(summary.language.clone()).or_insert(0) += summary.line_count;
  // }
  // counts
}


export function total_fn_count(&self: any): number {
  // self.files.values().map(|f| f.approx_fn_count).sum()
}

// }

export interface IndexStats {
  file_count: number;
  total_lines: number;
}


export function is_indexable_path(path: any): boolean {
  // path.is_file()
}


export function language_for_path(path: any): string {
  // match path.extension().and_then(|e| e.to_str()) {
  // Some("rs") => "rust".to_string(),
  // Some("ts") => "typescript".to_string(),
  // Some("tsx") => "typescript".to_string(),
  // Some("js") => "javascript".to_string(),
  // Some("jsx") => "javascript".to_string(),
  // Some("py") => "python".to_string(),
  // Some("go") => "go".to_string(),
  // Some("java") => "java".to_string(),
  // Some(ext) => ext.to_lowercase(),
  // None => "unknown".to_string(),
  // }
}

