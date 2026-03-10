// TODO: use std::fmt;
// TODO: use std::path::Path;

// /// Supported programming languages for migration.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
export type Language =
  | "TypeScript"
  | "JavaScript"
  | "Python"
  | "Go"
  | "Rust"
  | "Java";


// impl Language {
// /// File extensions that belong to this language.
export function extensions(&self: any): any {
  // match self {
  // Language::TypeScript => &["ts", "tsx"],
  // Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
  // Language::Python => &["py"],
  // Language::Go => &["go"],
  // Language::Rust => &["rs"],
  // Language::Java => &["java"],
  // }
}


// /// Target file extension when generating code in this language.
export function target_extension(&self: any): any {
  // match self {
  // Language::TypeScript => "ts",
  // Language::JavaScript => "js",
  // Language::Python => "py",
  // Language::Go => "go",
  // Language::Rust => "rs",
  // Language::Java => "java",
  // }
}


// /// Parse a language string (case-insensitive).
export function from_str_loose(s: string): any {
  // match s.to_ascii_lowercase().as_str() {
  // "ts" | "typescript" => Some(Language::TypeScript),
  // "js" | "javascript" => Some(Language::JavaScript),
  // "py" | "python" => Some(Language::Python),
  // "go" | "golang" => Some(Language::Go),
  // "rs" | "rust" => Some(Language::Rust),
  // "java" => Some(Language::Java),
  // _ => None,
  // }
}


// /// Name of the CLI tool needed for this language's ecosystem.
export function required_tool(&self: any): any {
  // match self {
  // Language::TypeScript | Language::JavaScript => "npm",
  // Language::Python => "python",
  // Language::Go => "go",
  // Language::Rust => "cargo",
  // Language::Java => "javac",
  // }
}

// }

// impl fmt::Display for Language {
export function fmt(&self: any, f: any): any {
  // match self {
  // Language::TypeScript => write!(f, "TypeScript"),
  // Language::JavaScript => write!(f, "JavaScript"),
  // Language::Python => write!(f, "Python"),
  // Language::Go => write!(f, "Go"),
  // Language::Rust => write!(f, "Rust"),
  // Language::Java => write!(f, "Java"),
  // }
}

// }

// /// Detect the language of a file from its extension.
export function detect_language(path: any): any {
  // let ext = path.extension()?.to_str()?;
  // let ext_lower = ext.to_ascii_lowercase();
  // let all_languages = [
  // Language::TypeScript,
  // Language::JavaScript,
  // Language::Python,
  // Language::Go,
  // Language::Rust,
  // Language::Java,
  // ];
  // for lang in &all_languages {
  // if lang.extensions().contains(&ext_lower.as_str()) {
  // return Some(*lang);
  // }
  // }
  // None
}


// /// Directories to skip when discovering source files.
// const SKIP_DIRS: &[&str] = &[
// "node_modules",
// "target",
// ".git",
// "__pycache__",
// ".venv",
// "venv",
// "dist",
// "build",
// ".next",
// ".idea",
// ".vscode",
// ".codex",
// "vendor",
// "bin",
// "obj",
// ];

// /// Recursively discover all source files for a given language in a directory.
// pub fn discover_source_files(

// dir: &Path,
// lang: Language,
// ) -> Vec<std::path::PathBuf> {
// let mut files = Vec::new();
// discover_recursive(dir, lang, &mut files);
// files.sort();
// files
// }

// fn discover_recursive(

// dir: &Path,
// lang: Language,
// out: &mut Vec<std::path::PathBuf>,
// ) {
// let entries = match std::fs::read_dir(dir) {
// Ok(e) => e,
// Err(_) => return,
// };

// for entry in entries.flatten() {
// let path = entry.path();
// if path.is_dir() {
// let name = path
// .file_name()
// .and_then(|n| n.to_str())
// .unwrap_or("");
// if SKIP_DIRS.contains(&name) {
// continue;
// }
// discover_recursive(&path, lang, out);
// } else if let Some(detected) = detect_language(&path) {
// if detected == lang {
// out.push(path);
// }
// }
// }
// }

// /// Check whether a required CLI tool is available on the system.
export function tool_available(tool: string): boolean {
  // std::process::Command::new(tool)
  // .arg("--version")
  // .stdout(std::process::Stdio::null())
  // .stderr(std::process::Stdio::null())
  // .status()
  // .is_ok()
}

