// TODO: use std::fs;
// TODO: use std::path::{Path, PathBuf};

// TODO: use crate::index::CodeIndex;
// TODO: use crate::model::CodexModel;

export interface SecurityIssue {
  file: any;
  line_number: number;
  severity: any;
  description: string;
  snippet: string;
}


export interface SecurityReport {
  issues: any[];
  files_scanned: number;
  ai_analysis: any;
}


// const HIGH_RISK_PATTERNS: &[(&str, &str, &str)] = &[
// ("api_key\\s*=", "High", "Hardcoded API key detected"),
// ("password\\s*=", "High", "Hardcoded password detected"),
// ("secret\\s*=", "High", "Hardcoded secret detected"),
// ("http://", "Medium", "Unencrypted HTTP connection used instead of HTTPS"),
// ("\\.execute\\(.*?\\+.*?", "High", "Potential SQL injection via string concatenation"),
// ];

// pub fn run_security_scan(

// root: &Path,
// index: &CodeIndex,
// model: Option<&(dyn CodexModel + Send + Sync)>,
// ) -> SecurityReport {
// let mut issues = Vec::new();
// let mut files_scanned = 0;

// // Convert primitive patterns into something we can check easily without the regex crate
// // To keep dependencies light, we'll do simple substring/contains checks for now,
// // simulating the regex intent.
// let simple_patterns: &[(&str, &str, &str)] = &[
// ("api_key =", "High", "Hardcoded API Key"),
// ("api_key=", "High", "Hardcoded API Key"),
// ("password =", "High", "Hardcoded Password"),
// ("password=", "High", "Hardcoded Password"),
// ("secret =", "High", "Hardcoded Secret"),
// ("secret=", "High", "Hardcoded Secret"),
// ("http://", "Medium", "Unencrypted HTTP (should be HTTPS)"),
// ("SELECT * FROM", "Medium", "Raw SQL query detected (check for injection risk)"),
// ];

// for (rel_path, _summary) in index.files() {
// files_scanned += 1;
// let abs_path = if rel_path.is_absolute() {
// rel_path.clone()
// } else {
// root.join(rel_path)
// };

// if let Ok(contents) = fs::read_to_string(&abs_path) {
// for (line_idx, line) in contents.lines().enumerate() {
// let lower = line.to_lowercase();

// for (pattern, severity, desc) in simple_patterns {
// if lower.contains(pattern) {
// // Avoid flagging our own security keywords array or dummy env files
// if !abs_path.to_string_lossy().contains("security.rs")
// && !abs_path.to_string_lossy().contains(".env.example") {
// issues.push(SecurityIssue {
// file: rel_path.clone(),
// line_number: line_idx + 1,
// severity,
// description: desc.to_string(),
// snippet: line.trim().to_string(),
// });
// }
// }
// }
// }
// }
// }

// let mut ai_analysis = None;

// if !issues.is_empty() {
// if let Some(m) = model {
// let mut prompt = String::new();
// prompt.push_str("You are an expert security researcher. Review these potential vulnerabilities found by a static scanner and summarize the overall risk level and next steps for the developer:\n\n");

// for (i, issue) in issues.iter().enumerate().take(15) {
// prompt.push_str(&format!("{}. [{}] {}:{} - {}\n   Code: {}\n",
// i + 1, issue.severity, issue.file.display(), issue.line_number, issue.description, issue.snippet));
// }

// if let Ok(analysis) = m.complete(&prompt) {
// ai_analysis = Some(analysis);
// }
// }
// }

// SecurityReport {
// issues,
// files_scanned,
// ai_analysis,
// }
// }

// impl SecurityReport {
export function render(&self: any): string {
  // use std::fmt::Write;
  // let mut out = String::new();
  // 
  // let _ = writeln!(&mut out, "🛡️  **SECURITY HUNTER REPORT** 🛡️\n");
  // let _ = writeln!(&mut out, "Scanned {} files.\n", self.files_scanned);
  // 
  // if self.issues.is_empty() {
  // let _ = writeln!(&mut out, "✅ No obvious security vulnerabilities found.");
  // return out;
  // }
  // 
  // let high_count = self.issues.iter().filter(|i| i.severity == "High").count();
  // let med_count = self.issues.iter().filter(|i| i.severity == "Medium").count();
  // 
  // let _ = writeln!(&mut out, "Found {} vulnerabilities ({} High, {} Medium).\n", self.issues.len(), high_count, med_count);
  // 
  // for issue in &self.issues {
  // let icon = if issue.severity == "High" { "🔴" } else { "🟡" };
  // let _ = writeln!(&mut out, "{} **{} Risk** in `{}:{}`", icon, issue.severity, issue.file.display(), issue.line_number);
  // let _ = writeln!(&mut out, "   Issue: {}", issue.description);
  // let _ = writeln!(&mut out, "   Code:  `{}`\n", issue.snippet);
  // }
  // 
  // if let Some(analysis) = &self.ai_analysis {
  // let _ = writeln!(&mut out, "\n🤖 **AI Security Analysis:**\n{}", analysis);
  // } else {
  // let _ = writeln!(&mut out, "\n*(Run with an LLM enabled for deeper AI analysis of these findings)*");
  // }
  // 
  // out
}

// }
