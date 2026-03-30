use std::fmt;
use std::path::Path;

/// Supported programming languages for migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
    Java,
    React,
    NextJs,
    Vue,
    Svelte,
    Cpp,
    Assembly,
}

impl Language {
    /// File extensions that belong to this language.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Rust => &["rs"],
            Language::Java => &["java"],
            Language::React => &["jsx", "tsx"],
            Language::NextJs => &["jsx", "tsx"],
            Language::Vue => &["vue"],
            Language::Svelte => &["svelte"],
            Language::Cpp => &["cpp", "hpp", "cc", "h", "cxx"],
            Language::Assembly => &["asm", "s"],
        }
    }

    /// Target file extension when generating code in this language.
    pub fn target_extension(&self) -> &'static str {
        match self {
            Language::TypeScript => "ts",
            Language::JavaScript => "js",
            Language::Python => "py",
            Language::Go => "go",
            Language::Rust => "rs",
            Language::Java => "java",
            Language::React => "tsx",
            Language::NextJs => "tsx",
            Language::Vue => "vue",
            Language::Svelte => "svelte",
            Language::Cpp => "cpp",
            Language::Assembly => "asm",
        }
    }

    /// Parse a language string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Language> {
        match s.to_ascii_lowercase().as_str() {
            "ts" | "typescript" => Some(Language::TypeScript),
            "js" | "javascript" => Some(Language::JavaScript),
            "py" | "python" => Some(Language::Python),
            "go" | "golang" => Some(Language::Go),
            "rs" | "rust" => Some(Language::Rust),
            "java" => Some(Language::Java),
            "react" => Some(Language::React),
            "next" | "nextjs" => Some(Language::NextJs),
            "vue" => Some(Language::Vue),
            "svelte" => Some(Language::Svelte),
            "cpp" | "c++" => Some(Language::Cpp),
            "asm" | "assembly" => Some(Language::Assembly),
            _ => None,
        }
    }

    /// Name of the CLI tool needed for this language's ecosystem.
    pub fn required_tool(&self) -> &'static str {
        match self {
            Language::TypeScript | Language::JavaScript => "npm",
            Language::Python => "python",
            Language::Go => "go",
            Language::Rust => "cargo",
            Language::Java => "javac",
            Language::React | Language::NextJs | Language::Vue | Language::Svelte => "npm",
            Language::Cpp => "g++",
            Language::Assembly => "nasm",
        }
    }

    pub fn from_path(path: &Path) -> Language {
        detect_language(path).unwrap_or(Language::TypeScript)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Python => write!(f, "Python"),
            Language::Go => write!(f, "Go"),
            Language::Rust => write!(f, "Rust"),
            Language::Java => write!(f, "Java"),
            Language::React => write!(f, "React"),
            Language::NextJs => write!(f, "Next.js"),
            Language::Vue => write!(f, "Vue"),
            Language::Svelte => write!(f, "Svelte"),
            Language::Cpp => write!(f, "C++"),
            Language::Assembly => write!(f, "Assembly"),
        }
    }
}

/// Detect the language of a file from its extension.
pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    let ext_lower = ext.to_ascii_lowercase();
    let all_languages = [
        Language::TypeScript,
        Language::JavaScript,
        Language::Python,
        Language::Go,
        Language::Rust,
        Language::Java,
        Language::React,
        Language::NextJs,
        Language::Vue,
        Language::Svelte,
        Language::Cpp,
        Language::Assembly,
    ];
    for lang in &all_languages {
        if lang.extensions().contains(&ext_lower.as_str()) {
            return Some(*lang);
        }
    }
    None
}

/// Directories to skip when discovering source files.
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    ".idea",
    ".vscode",
    ".astra",
    ".forge",
    ".codex",
    "vendor",
    "bin",
    "obj",
];

pub fn discover_source_files(
    path: &Path,
    lang: Language,
) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    
    // Check if the path points to an existing file directly
    if path.is_file() || std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
        if let Some(detected) = detect_language(path) {
            if detected == lang {
                files.push(path.to_path_buf());
            }
        }
    } else {
        discover_recursive(path, lang, &mut files);
    }
    files.sort();
    files
}


fn discover_recursive(
    dir: &Path,
    lang: Language,
    out: &mut Vec<std::path::PathBuf>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if SKIP_DIRS.contains(&name) {
                continue;
            }
            discover_recursive(&path, lang, out);
        } else if let Some(detected) = detect_language(&path) {
            if detected == lang {
                out.push(path);
            }
        }
    }
}

/// Check whether a required CLI tool is available on the system.
pub fn tool_available(tool: &str) -> bool {
    std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
