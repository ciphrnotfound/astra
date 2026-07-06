//! Astra's pair-PM code reviewer — the "don't ship rubbish" gate.
//!
//! Designed for AI-assisted / "vibe" coders who move fast and don't read the
//! generated code. It scans the files about to be shipped, finds the classes
//! of mistakes that AI codegen commonly produces (leaked secrets, injection,
//! missing auth, unvalidated input, swallowed errors, footguns), and explains
//! each one in plain English with a concrete fix — then gives a clear
//! ship-readiness verdict so they know whether it's safe to push.

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::CodexModel;

const MAX_FILE_BYTES: usize = 40_000;
const MAX_FILES_DEEP_REVIEW: usize = 12;

// ---------------------------------------------------------------------------
// Severity & verdict
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn from_str_loose(s: &str) -> Severity {
        match s.trim().to_ascii_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" | "med" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Info,
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            Severity::Critical => "🟥",
            Severity::High => "🟧",
            Severity::Medium => "🟨",
            Severity::Low => "🟦",
            Severity::Info => "⬜",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Ship,      // nothing serious
    FixFirst,  // medium/high issues — fix before shipping
    Block,     // critical issues — do not ship
}

impl Verdict {
    pub fn banner(&self) -> String {
        match self {
            Verdict::Ship => "✅ SHIP — no blocking issues found.".to_string(),
            Verdict::FixFirst => "⚠️  FIX FIRST — issues found that should be fixed before shipping.".to_string(),
            Verdict::Block => "⛔ BLOCK — critical issues found. Do NOT ship this.".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Finding {
    pub file: String,
    #[serde(default)]
    pub line: usize,
    pub severity: Severity,
    pub category: String,    // e.g. "Secret", "SQL Injection", "Missing Auth"
    pub title: String,
    pub explanation: String, // plain English: WHY it's dangerous
    pub fix: String,         // concrete, copy-pasteable remediation
    /// Verification/context note added by the refine pass (git status, severity adjustments).
    #[serde(default)]
    pub note: String,
}

impl Finding {
    /// Stable identity across edits (line excluded — it shifts when code moves).
    pub fn fingerprint(&self) -> String {
        let title_norm: String = self
            .title
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect();
        format!("{}|{}|{}", self.file, self.category.to_ascii_lowercase(), title_norm.trim())
    }
}

#[derive(Debug, Clone)]
pub struct ReviewReport {
    pub findings: Vec<Finding>,
    pub files_scanned: usize,
    pub scope_label: String,
    pub verdict: Verdict,
    pub summary: Option<String>,
    /// Human-readable trend vs. the previous review (memory).
    pub trend: Option<String>,
    /// How many findings were hidden because the dev acknowledged them.
    pub suppressed: usize,
    /// How many raw findings the verification pass dropped as noise.
    pub filtered_noise: usize,
}

impl ReviewReport {
    pub fn compute_verdict(findings: &[Finding]) -> Verdict {
        let has_critical = findings.iter().any(|f| f.severity == Severity::Critical);
        let has_serious = findings
            .iter()
            .any(|f| matches!(f.severity, Severity::High | Severity::Medium));
        if has_critical {
            Verdict::Block
        } else if has_serious {
            Verdict::FixFirst
        } else {
            Verdict::Ship
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "\n╔══════════════════════════════════════════════════════════╗");
        let _ = writeln!(&mut out, "║  🔍 ASTRA CODE REVIEW — pair-PM ship gate                  ║");
        let _ = writeln!(&mut out, "╚══════════════════════════════════════════════════════════╝");
        let _ = writeln!(&mut out, "Scope: {}  ·  {} file(s) scanned", self.scope_label, self.files_scanned);

        // Counts by severity
        let crit = self.count(Severity::Critical);
        let high = self.count(Severity::High);
        let med = self.count(Severity::Medium);
        let low = self.count(Severity::Low);
        let _ = writeln!(
            &mut out,
            "Findings: {} critical · {} high · {} medium · {} low",
            crit, high, med, low
        );
        if let Some(trend) = &self.trend {
            let _ = writeln!(&mut out, "Since last review: {}", trend);
        }
        if self.suppressed > 0 {
            let _ = writeln!(&mut out, "({} acknowledged finding(s) hidden — see :review acked)", self.suppressed);
        }
        if self.filtered_noise > 0 {
            let _ = writeln!(&mut out, "({} low-confidence finding(s) filtered by verification)", self.filtered_noise);
        }
        let _ = writeln!(&mut out, "\n{}\n", self.verdict.banner());

        if self.findings.is_empty() {
            let _ = writeln!(&mut out, "🎉 Clean! Nothing blocking found in the reviewed code.");
            if self.suppressed > 0 {
                let _ = writeln!(&mut out, "\n(Note: {} finding(s) are hidden because you acknowledged them earlier.)", self.suppressed);
            }
            return out;
        }

        // Sort: most severe first
        let mut sorted = self.findings.clone();
        sorted.sort_by(|a, b| b.severity.cmp(&a.severity));

        for (i, f) in sorted.iter().enumerate() {
            let loc = if f.line > 0 {
                format!("{}:{}", f.file, f.line)
            } else {
                format!("{} (file-wide — no specific line)", f.file)
            };
            let _ = writeln!(
                &mut out,
                "#{} {} {} [{}]  {}",
                i + 1,
                f.severity.icon(),
                f.severity.label(),
                f.category,
                f.title
            );
            let _ = writeln!(&mut out, "   📍 {}", loc);
            if !f.note.is_empty() {
                let _ = writeln!(&mut out, "   🧠 {}", f.note);
            }
            let _ = writeln!(&mut out, "   ⚠️  {}", f.explanation);
            let _ = writeln!(&mut out, "   🔧 Fix: {}", f.fix);
            if i < sorted.len() - 1 {
                let _ = writeln!(&mut out);
            }
        }
        let _ = writeln!(&mut out, "\n💡 Acknowledge a false-positive/accepted-risk with:  :review ack <#>");

        if let Some(summary) = &self.summary {
            let _ = writeln!(&mut out, "\n──────────────────────────────────────────────────────────");
            let _ = writeln!(&mut out, "📋 PM Summary:\n{}", summary);
        }

        out
    }

    fn count(&self, sev: Severity) -> usize {
        self.findings.iter().filter(|f| f.severity == sev).count()
    }
}

// ===========================================================================
// TRUST LAYER — the difference between "helpful assistant" and "real gate"
// ===========================================================================

// ---------------------------------------------------------------------------
// Git context — is this secret actually exposed, or safe in a gitignored file?
// ---------------------------------------------------------------------------

use std::process::Command;

pub struct GitContext {
    root: PathBuf,
    is_repo: bool,
}

impl GitContext {
    pub fn new(root: &Path) -> Self {
        let is_repo = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        Self { root: root.to_path_buf(), is_repo }
    }

    /// True if the file is ignored by .gitignore (so its secrets never reach the remote).
    pub fn is_gitignored(&self, rel: &str) -> bool {
        if !self.is_repo {
            return false;
        }
        Command::new("git")
            .args(["check-ignore", "-q", rel])
            .current_dir(&self.root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// True if the file is tracked by git (committed / staged) — its history may hold secrets.
    pub fn is_tracked(&self, rel: &str) -> bool {
        if !self.is_repo {
            return false;
        }
        Command::new("git")
            .args(["ls-files", "--error-unmatch", rel])
            .current_dir(&self.root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Entropy — distinguish `API_KEY =` (a variable) from `API_KEY=sk-live-9f3a…`
// ---------------------------------------------------------------------------

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut counts = std::collections::HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0u32) += 1;
    }
    let len = s.chars().count() as f64;
    counts
        .values()
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Pull the value after `=` or `:` on a line and judge whether it's a REAL secret
/// (high entropy / known key prefix) vs. just a variable name or placeholder.
fn line_contains_real_secret(line: &str) -> bool {
    // Known unmistakable secret shapes
    let known_prefixes = [
        "sk-", "sk_live_", "sk_test_", "pk_live_", "ghp_", "gho_", "github_pat_",
        "AKIA", "ASIA", "xoxb-", "xoxp-", "AIza", "-----BEGIN", "eyJ", // JWT-ish
    ];
    for p in known_prefixes {
        if line.contains(p) {
            return true;
        }
    }

    // Otherwise, inspect the assigned value's entropy.
    let value = line
        .split_once('=')
        .or_else(|| line.split_once(':'))
        .map(|(_, v)| v.trim())
        .unwrap_or("");
    // Strip quotes and trailing commas/semicolons
    let value = value.trim_matches(|c| c == '"' || c == '\'' || c == ',' || c == ';' || c == ' ');

    // Placeholders are not secrets
    let lower = value.to_ascii_lowercase();
    let placeholders = [
        "", "your_api_key", "your-api-key", "xxx", "todo", "changeme", "none",
        "null", "example", "placeholder", "<your", "process.env", "os.environ",
        "env(", "getenv", "${", "test", "dummy", "fake",
    ];
    if placeholders.iter().any(|p| !p.is_empty() && lower.contains(p)) || value.len() < 12 {
        return false;
    }

    // Real secrets are long and high-entropy.
    let entropy = shannon_entropy(value);
    value.len() >= 16 && entropy >= 3.2
}

fn is_generated_file(rel: &str) -> bool {
    let l = rel.to_ascii_lowercase();
    l.ends_with("next-env.d.ts")
        || l.ends_with(".min.js")
        || l.ends_with(".d.ts") && l.contains("env")
        || l.contains("/generated/")
        || l.contains(".generated.")
}

// ---------------------------------------------------------------------------
// Verification / refine pass — deterministic, no extra LLM cost
// ---------------------------------------------------------------------------

/// The heart of the trust layer. Takes raw LLM findings and:
///  - kills hallucinations in generated files
///  - re-reads the cited line to confirm secret findings are real (entropy)
///  - re-scores secrets by git exposure (gitignored+untracked = safe → Info)
///  - downgrades unverifiable file-wide (line 0) High/Critical findings
///  - deduplicates
/// Returns (kept_findings, noise_dropped_count).
fn refine_findings(
    mut findings: Vec<Finding>,
    root: &Path,
    git: &GitContext,
) -> (Vec<Finding>, usize) {
    let before = findings.len();

    // Cache file contents so we can re-read cited lines cheaply.
    let mut file_cache: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut load_lines = |rel: &str| -> Vec<String> {
        if let Some(v) = file_cache.get(rel) {
            return v.clone();
        }
        let abs = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { root.join(rel) };
        let lines = fs::read_to_string(&abs)
            .map(|c| c.lines().map(|l| l.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        file_cache.insert(rel.to_string(), lines.clone());
        lines
    };

    let mut kept: Vec<Finding> = Vec::new();

    for mut f in findings.drain(..) {
        // 1. Drop findings in auto-generated files (next-env.d.ts etc.)
        if is_generated_file(&f.file) {
            continue;
        }

        let is_secret = f.category.to_ascii_lowercase().contains("secret")
            || f.title.to_ascii_lowercase().contains("api key")
            || f.title.to_ascii_lowercase().contains("password")
            || f.title.to_ascii_lowercase().contains("token");

        // 2. Verify secret findings against the actual cited line.
        //    A secret is only "verified" if we can point at a real high-entropy
        //    value on a real line. Everything else is an unproven claim.
        let mut verified_secret = false;
        if is_secret {
            if f.line > 0 {
                let lines = load_lines(&f.file);
                match lines.get(f.line - 1) {
                    Some(line) if line_contains_real_secret(line) => {
                        verified_secret = true;
                    }
                    Some(_) => {
                        // Line exists but has no real secret → likely a false positive.
                        f.severity = f.severity.min(Severity::Low);
                        f.note = "Unconfirmed: cited line has no high-entropy secret (looks like a variable name or placeholder).".to_string();
                    }
                    None => {
                        // Cited line doesn't exist → hallucinated location. Drop.
                        continue;
                    }
                }
            } else {
                // No line cited → cannot verify a secret at all. Never trust this
                // as Critical/High; it's an unproven lead. (This is the fix for the
                // bug where a fake file-wide "secret" got promoted to CRITICAL.)
                f.severity = f.severity.min(Severity::Low);
                if f.note.is_empty() {
                    f.note = "Unverified: claimed a secret but cited no specific line. Treat as a lead, not a confirmed leak.".to_string();
                }
            }
        }

        // 3. Re-score ONLY verified secrets by git exposure.
        if verified_secret {
            let ignored = git.is_gitignored(&f.file);
            let tracked = git.is_tracked(&f.file);
            if ignored && !tracked {
                // Safe: gitignored and never committed. Real secret, but not exposed.
                f.severity = Severity::Info;
                f.note = "gitignored & untracked — not shipped to the remote. Rotate only if this file was ever shared.".to_string();
            } else if tracked {
                // Dangerous: committed → it's in git history even if deleted later.
                f.severity = Severity::Critical;
                f.note = "⚠️ This file is TRACKED by git — the secret is in your commit history. Rotate the key AND purge history.".to_string();
            } else {
                // Untracked, not ignored: real secret, sitting in the working tree.
                f.severity = f.severity.max(Severity::High);
                f.note = "Real secret in an untracked file — don't commit it; move to an env var / secrets manager.".to_string();
            }
        }

        // 4. Downgrade unverifiable file-wide (no line) High/Critical findings.
        if f.line == 0 && f.severity >= Severity::High && !is_secret {
            f.severity = Severity::Medium;
            if f.note.is_empty() {
                f.note = "No specific line cited — treat as a lead to verify, not a confirmed defect.".to_string();
            }
        }

        kept.push(f);
    }

    // 5. Deduplicate by fingerprint (same file+category+title), keeping highest severity.
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut deduped: Vec<Finding> = Vec::new();
    for f in kept {
        let fp = f.fingerprint();
        if let Some(&idx) = seen.get(&fp) {
            if f.severity > deduped[idx].severity {
                deduped[idx] = f;
            }
        } else {
            seen.insert(fp, deduped.len());
            deduped.push(f);
        }
    }

    let dropped = before - deduped.len();
    (deduped, dropped)
}

// ---------------------------------------------------------------------------
// Review memory — Astra remembers past reviews (codebase memory)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ReviewMemory {
    /// Fingerprints the developer explicitly acknowledged (false positive / accepted risk).
    #[serde(default)]
    pub acknowledged: Vec<String>,
    /// Snapshot of the last review for trend comparison.
    #[serde(default)]
    pub last_counts: Option<(usize, usize, usize, usize)>, // crit, high, med, low
    #[serde(default)]
    pub last_run_ts: u64,
    /// Fingerprints present at the last review (to compute new vs. fixed).
    #[serde(default)]
    pub last_fingerprints: Vec<String>,
    /// The last render's ordered fingerprints, so `:review ack <n>` can map n → fingerprint.
    #[serde(default)]
    pub last_shown: Vec<String>,
}

impl ReviewMemory {
    pub fn path(root: &Path) -> PathBuf {
        root.join(".astra").join("review_memory.json")
    }

    pub fn load(root: &Path) -> Self {
        fs::read_to_string(Self::path(root))
            .ok()
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, root: &Path) {
        let p = Self::path(root);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(p, data);
        }
    }

    pub fn is_acknowledged(&self, fp: &str) -> bool {
        self.acknowledged.iter().any(|a| a == fp)
    }
}

/// Acknowledge finding #n from the last review so it stops alarming.
/// Returns a human message.
pub fn acknowledge(root: &Path, n: usize) -> String {
    let mut mem = ReviewMemory::load(root);
    if n == 0 || n > mem.last_shown.len() {
        return format!(
            "No finding #{} in the last review. Run `:review` first, then `:review ack <#>`.",
            n
        );
    }
    let fp = mem.last_shown[n - 1].clone();
    if mem.is_acknowledged(&fp) {
        return format!("Finding #{} is already acknowledged.", n);
    }
    mem.acknowledged.push(fp);
    mem.save(root);
    format!(
        "✅ Acknowledged finding #{}. It won't be counted against your ship-verdict anymore.\n   (Undo by editing .astra/review_memory.json)",
        n
    )
}

/// Compute a plain-English trend string vs. the previous review.
fn compute_trend(mem: &ReviewMemory, current: &[Finding]) -> Option<String> {
    let (last_c, last_h, last_m, last_l) = mem.last_counts?;
    let last_total = last_c + last_h + last_m + last_l;
    let cur_total = current.len();

    let current_fps: std::collections::HashSet<String> =
        current.iter().map(|f| f.fingerprint()).collect();
    let last_fps: std::collections::HashSet<String> =
        mem.last_fingerprints.iter().cloned().collect();

    let new_count = current_fps.difference(&last_fps).count();
    let fixed_count = last_fps.difference(&current_fps).count();

    let arrow = if cur_total < last_total {
        "📉 improving"
    } else if cur_total > last_total {
        "📈 more issues"
    } else {
        "➡️ unchanged"
    };
    Some(format!(
        "{} ({} → {} findings · {} new · {} fixed)",
        arrow, last_total, cur_total, new_count, fixed_count
    ))
}

// ---------------------------------------------------------------------------
// Heuristic pre-scan — cheap signals to prioritize which files to deep-review
// ---------------------------------------------------------------------------

/// A quick risk score for a file based on cheap pattern signals.
/// Higher = more likely to contain real problems worth an LLM pass.
fn risk_score(content: &str, path: &str) -> u32 {
    let lower = content.to_ascii_lowercase();
    let mut score = 0u32;

    // Secret-ish signals
    for needle in [
        "api_key", "apikey", "secret", "password", "passwd", "token",
        "private_key", "aws_access", "bearer ", "authorization",
    ] {
        if lower.contains(needle) {
            score += 3;
        }
    }
    // Injection / dangerous exec signals
    for needle in [
        "select * from", "execute(", "exec(", "eval(", "system(",
        "subprocess", "os.system", "child_process", "innerhtml",
        "dangerouslysetinnerhtml", "query(", "raw(", "f\"select", "'select",
    ] {
        if lower.contains(needle) {
            score += 2;
        }
    }
    // Auth / network surface
    for needle in [
        "jwt", "cors", "app.use", "router.", "@app.route", "fetch(",
        "axios", "http://", "verify=false", "rejectunauthorized",
    ] {
        if lower.contains(needle) {
            score += 1;
        }
    }
    // Files that touch sensitive areas by name
    let pl = path.to_ascii_lowercase();
    for needle in ["auth", "login", "payment", "admin", "config", "env", "db", "user"] {
        if pl.contains(needle) {
            score += 2;
        }
    }
    score
}

fn is_reviewable(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    // Skip lockfiles, binaries, generated, vendored
    let skip = [
        "package-lock.json", "yarn.lock", "cargo.lock", ".min.js",
        ".map", "node_modules/", "target/", "dist/", "build/", ".git/",
    ];
    if skip.iter().any(|s| lower.contains(s)) {
        return false;
    }
    let ok_ext = [
        ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".go", ".java",
        ".rb", ".php", ".cs", ".kt", ".sql", ".env", ".yml", ".yaml",
        ".sh", ".html", ".vue", ".svelte",
    ];
    ok_ext.iter().any(|e| lower.ends_with(e)) || lower.ends_with(".env")
}

// ---------------------------------------------------------------------------
// Deep review via LLM
// ---------------------------------------------------------------------------

/// Review a set of files. `files` are paths relative to `root`.
pub fn review_files(
    root: &Path,
    files: &[String],
    scope_label: &str,
    model: Option<&(dyn CodexModel + Send + Sync)>,
) -> ReviewReport {
    // Filter + load + score candidate files
    let mut candidates: Vec<(String, String, u32)> = Vec::new();
    for rel in files {
        if !is_reviewable(rel) {
            continue;
        }
        let abs: PathBuf = if Path::new(rel).is_absolute() {
            PathBuf::from(rel)
        } else {
            root.join(rel)
        };
        let content = match fs::read_to_string(&abs) {
            Ok(mut c) => {
                if c.len() > MAX_FILE_BYTES {
                    // Truncate at the nearest char boundary at or below the limit.
                    let mut end = MAX_FILE_BYTES;
                    while end > 0 && !c.is_char_boundary(end) {
                        end -= 1;
                    }
                    c.truncate(end);
                }
                c
            }
            Err(_) => continue,
        };
        let score = risk_score(&content, rel);
        candidates.push((rel.clone(), content, score));
    }

    let files_scanned = candidates.len();

    // Prioritize highest-risk files for the (more expensive) deep review
    candidates.sort_by(|a, b| b.2.cmp(&a.2));
    candidates.truncate(MAX_FILES_DEEP_REVIEW);

    let mut raw_findings = Vec::new();
    let mut file_contents: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    if let Some(model) = model {
        for (rel, content, _score) in &candidates {
            file_contents.insert(rel.clone(), content.clone());
            if let Ok(mut file_findings) = deep_review_file(model, rel, content) {
                raw_findings.append(&mut file_findings);
            }
        }
    } else {
        // No model — fall back to heuristic-only findings
        for (rel, content, _score) in &candidates {
            raw_findings.extend(heuristic_findings(rel, content));
        }
    }

    // ── TRUST LAYER ────────────────────────────────────────────────────────
    let raw_count = raw_findings.len();

    // 0. Skeptical verify: a hostile second LLM pass rejects hallucinated findings
    //    (e.g. "command injection" on shell-free Rust Command::args). Anti-slop gate.
    let raw_findings = if let Some(model) = model {
        skeptical_verify(model, raw_findings, &file_contents)
    } else {
        raw_findings
    };

    // 1. Refine: git-context re-scoring, entropy verification, drop hallucinations, dedup.
    let git = GitContext::new(root);
    let (mut findings, _refine_dropped) = refine_findings(raw_findings, root, &git);

    // Total noise removed = everything the first eager pass produced minus what survived.
    let filtered_noise = raw_count.saturating_sub(findings.len());

    // 2. Suppression: hide findings the developer has acknowledged (codebase memory).
    let mut mem = ReviewMemory::load(root);
    let suppressed = findings
        .iter()
        .filter(|f| mem.is_acknowledged(&f.fingerprint()))
        .count();
    findings.retain(|f| !mem.is_acknowledged(&f.fingerprint()));

    // 3. Trend vs. last review (before we overwrite memory).
    let trend = compute_trend(&mem, &findings);

    let verdict = ReviewReport::compute_verdict(&findings);

    // Optional PM-style summary
    let summary = if let (Some(model), false) = (model, findings.is_empty()) {
        pm_summary(model, &findings, &verdict).ok()
    } else {
        None
    };

    // 4. Persist this review into codebase memory (sorted the way it's shown).
    let mut shown = findings.clone();
    shown.sort_by(|a, b| b.severity.cmp(&a.severity));
    mem.last_shown = shown.iter().map(|f| f.fingerprint()).collect();
    mem.last_fingerprints = findings.iter().map(|f| f.fingerprint()).collect();
    mem.last_counts = Some((
        findings.iter().filter(|f| f.severity == Severity::Critical).count(),
        findings.iter().filter(|f| f.severity == Severity::High).count(),
        findings.iter().filter(|f| f.severity == Severity::Medium).count(),
        findings.iter().filter(|f| f.severity == Severity::Low).count(),
    ));
    mem.last_run_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    mem.save(root);

    ReviewReport {
        findings,
        files_scanned,
        scope_label: scope_label.to_string(),
        verdict,
        summary,
        trend,
        suppressed,
        filtered_noise,
    }
}

/// Send one file to the LLM for a thorough, vibe-coder-friendly review.
fn deep_review_file(
    model: &(dyn CodexModel + Send + Sync),
    path: &str,
    content: &str,
) -> anyhow::Result<Vec<Finding>> {
    // Number the lines so the model can cite exact locations.
    let numbered: String = content
        .lines()
        .enumerate()
        .map(|(i, l)| format!("{:>4}| {}", i + 1, l))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"You are Astra, a senior engineer reviewing code that was likely written
quickly with an AI assistant by a developer who does NOT read it carefully.
Your job is to catch anything that would be unsafe, insecure, or embarrassing
to ship — and explain it so a non-expert understands.

Review this file. Look specifically for:
- Hardcoded secrets / API keys / passwords / tokens
- SQL / command / path injection (unsanitized input in queries or shell)
- Missing or broken authentication / authorization / access control
- Unvalidated or untrusted user input
- Swallowed errors, missing error handling, crashes on bad input
- Dangerous patterns: eval/exec, disabled TLS verification, permissive CORS,
  debug mode on, secrets in logs, XSS (innerHTML/dangerouslySetInnerHTML)
- Obvious logic bugs that would break in production

FILE: {}

```
{}
```

Respond with ONLY a JSON array (no prose, no markdown fences). Each item:
{{
  "line": <line number, or 0 if file-wide>,
  "severity": "critical" | "high" | "medium" | "low" | "info",
  "category": "short category e.g. Secret / SQL Injection / Missing Auth",
  "title": "one-line description of the problem",
  "explanation": "plain-English WHY this is dangerous, for a non-expert (1-2 sentences)",
  "fix": "concrete, specific, copy-pasteable fix or instruction"
}}

If the file is genuinely clean, respond with: []
Be precise. Do NOT invent issues. Only report real problems. Severity must
reflect real-world risk: leaked live secret or injection = critical/high."#,
        path, numbered
    );

    let response = model.complete(&prompt)?;
    let json = extract_json_array(&response);

    #[derive(Deserialize)]
    struct RawFinding {
        #[serde(default)]
        line: usize,
        severity: String,
        category: String,
        title: String,
        explanation: String,
        fix: String,
    }

    let raw: Vec<RawFinding> = serde_json::from_str(&json).unwrap_or_default();
    let findings = raw
        .into_iter()
        .map(|r| Finding {
            file: path.to_string(),
            line: r.line,
            severity: Severity::from_str_loose(&r.severity),
            category: r.category,
            title: r.title,
            explanation: r.explanation,
            fix: r.fix,
            note: String::new(),
        })
        .collect();
    Ok(findings)
}

/// SKEPTICAL VERIFY PASS — the anti-slop gate.
///
/// A second LLM pass that acts as a hostile skeptic. It re-reads the real code
/// and, for each claimed finding, tries to REJECT it unless it can prove the
/// issue with a specific line and concrete mechanism. This is what kills the
/// "safe Rust Command::args is command injection" class of hallucination that
/// the first (eager) pass produces.
///
/// One LLM call per file that has findings. On any parse failure it keeps the
/// original findings for that file (fail-open — never silently lose a real bug).
fn skeptical_verify(
    model: &(dyn CodexModel + Send + Sync),
    findings: Vec<Finding>,
    file_contents: &std::collections::HashMap<String, String>,
) -> Vec<Finding> {
    use std::collections::HashMap;

    // Group findings by file, preserving order.
    let mut by_file: HashMap<String, Vec<Finding>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for f in findings {
        if !by_file.contains_key(&f.file) {
            order.push(f.file.clone());
        }
        by_file.entry(f.file.clone()).or_default().push(f);
    }

    let mut verified: Vec<Finding> = Vec::new();

    for file in order {
        let file_findings = by_file.remove(&file).unwrap_or_default();
        let content = match file_contents.get(&file) {
            Some(c) => c,
            None => {
                // No content to check against — keep as-is.
                verified.extend(file_findings);
                continue;
            }
        };

        // Number the code so the skeptic can cite exact lines.
        let numbered: String = content
            .lines()
            .enumerate()
            .map(|(i, l)| format!("{:>4}| {}", i + 1, l))
            .collect::<Vec<_>>()
            .join("\n");

        // List the claims to adjudicate.
        let claims: String = file_findings
            .iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "{}. [{}] {} — {} (claimed line {})",
                    i,
                    f.severity.label(),
                    f.category,
                    f.title,
                    if f.line > 0 { f.line.to_string() } else { "none".to_string() }
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"You are a SKEPTICAL senior code reviewer. Another reviewer flagged some
issues in this file. Many AI-generated findings are false positives that
pattern-match on scary words without understanding the language's safety model
(e.g. claiming Rust's std::process::Command::args is "command injection" when it
does NOT spawn a shell, or claiming "missing auth" on an internal CLI function
that has no network surface).

Your job: for EACH claim, decide CONFIRM or REJECT. Default to REJECT unless you
can point at the specific line and explain the concrete exploit/failure mechanism.

FILE: {}

```
{}
```

CLAIMS TO ADJUDICATE:
{}

Respond with ONLY a JSON array, one object per claim (same index order):
{{
  "index": <claim number>,
  "verdict": "confirm" | "reject",
  "line": <the real line number if confirmed, else 0>,
  "severity": "critical" | "high" | "medium" | "low" | "info",
  "reason": "one sentence: the concrete mechanism if confirmed, or why it's a false positive if rejected"
}}

Rules:
- REJECT anything you cannot tie to a specific line with a real mechanism.
- REJECT "missing auth / add OAuth/JWT" on code with no network/endpoint surface.
- REJECT injection claims where inputs never reach a shell/SQL string interpolation.
- Do NOT invent NEW issues. Only adjudicate the claims listed."#,
            file, numbered, claims
        );

        let response = match model.complete(&prompt) {
            Ok(r) => r,
            Err(_) => {
                // Model failed — keep originals (fail-open).
                verified.extend(file_findings);
                continue;
            }
        };

        #[derive(Deserialize)]
        struct Verdict {
            index: usize,
            #[serde(default)]
            verdict: String,
            #[serde(default)]
            line: usize,
            #[serde(default)]
            severity: String,
            #[serde(default)]
            reason: String,
        }

        let json = extract_json_array(&response);
        let verdicts: Vec<Verdict> = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(_) => {
                // Couldn't parse the skeptic — keep originals (fail-open).
                verified.extend(file_findings);
                continue;
            }
        };

        // Build a map index -> verdict.
        let mut vmap: HashMap<usize, &Verdict> = HashMap::new();
        for v in &verdicts {
            vmap.insert(v.index, v);
        }

        for (i, mut f) in file_findings.into_iter().enumerate() {
            match vmap.get(&i) {
                Some(v) if v.verdict.eq_ignore_ascii_case("reject") => {
                    // Skeptic rejected it — drop entirely.
                    continue;
                }
                Some(v) => {
                    // Confirmed (or unknown verdict treated as confirm) — apply corrections.
                    if v.line > 0 {
                        f.line = v.line;
                    }
                    if !v.severity.is_empty() {
                        f.severity = Severity::from_str_loose(&v.severity);
                    }
                    if !v.reason.is_empty() {
                        f.note = format!("verified: {}", v.reason);
                    }
                    verified.push(f);
                }
                None => {
                    // Skeptic didn't mention it — be conservative, keep but downgrade note.
                    f.note = "not re-confirmed by verification pass — lower confidence.".to_string();
                    if f.severity >= Severity::High {
                        f.severity = Severity::Medium;
                    }
                    verified.push(f);
                }
            }
        }
    }

    verified
}

/// Fallback when no LLM is available: surface the riskiest raw lines.
fn heuristic_findings(path: &str, content: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let patterns: [(&str, Severity, &str, &str); 6] = [
        ("api_key=", Severity::High, "Secret", "Possible hardcoded API key"),
        ("password=", Severity::High, "Secret", "Possible hardcoded password"),
        ("secret=", Severity::High, "Secret", "Possible hardcoded secret"),
        ("select * from", Severity::Medium, "SQL", "Raw SQL — check for injection"),
        ("eval(", Severity::High, "Dangerous Exec", "Use of eval()"),
        ("http://", Severity::Low, "Insecure Transport", "Plain HTTP reference"),
    ];
    for (i, line) in content.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        for (needle, sev, cat, title) in &patterns {
            if lower.contains(needle) {
                out.push(Finding {
                    file: path.to_string(),
                    line: i + 1,
                    severity: sev.clone(),
                    category: (*cat).to_string(),
                    title: (*title).to_string(),
                    explanation: "Flagged by fast pattern scan (no AI model configured for deep review).".to_string(),
                    fix: "Configure an LLM (Groq/Gemini/OpenAI) and re-run :review for a precise fix.".to_string(),
                    note: String::new(),
                });
            }
        }
    }
    out
}

/// A short PM-style wrap-up after the findings.
fn pm_summary(
    model: &(dyn CodexModel + Send + Sync),
    findings: &[Finding],
    verdict: &Verdict,
) -> anyhow::Result<String> {
    let mut list = String::new();
    for f in findings.iter().take(20) {
        let _ = writeln!(&mut list, "- [{}] {} ({})", f.severity.label(), f.title, f.category);
    }
    let verdict_word = match verdict {
        Verdict::Ship => "SHIP",
        Verdict::FixFirst => "FIX FIRST",
        Verdict::Block => "BLOCK",
    };
    let prompt = format!(
        r#"You are Astra, a friendly but firm technical project manager talking to a
developer who codes fast with AI and may not understand security deeply.

The automated review verdict is: {}

Findings:
{}

Write a short (3-5 line) summary: what are the 1-3 MOST important things to fix
first and why, in plain encouraging language. Be specific and actionable. If the
verdict is SHIP, congratulate briefly and note anything minor to watch."#,
        verdict_word, list
    );
    model.complete(&prompt)
}

// ---------------------------------------------------------------------------
// JSON extraction
// ---------------------------------------------------------------------------

fn extract_json_array(text: &str) -> String {
    let trimmed = text.trim();
    // strip ```json fences
    let body = if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        after.find("```").map(|e| &after[..e]).unwrap_or(after)
    } else if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        after.find("```").map(|e| &after[..e]).unwrap_or(after)
    } else {
        trimmed
    };
    let body = body.trim();
    // Find the first [ ... ] array
    if let Some(start) = body.find('[') {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in body[start..].char_indices() {
            if escape { escape = false; continue; }
            if ch == '\\' && in_string { escape = true; continue; }
            if ch == '"' { in_string = !in_string; continue; }
            if in_string { continue; }
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return body[start..=start + i].to_string();
                    }
                }
                _ => {}
            }
        }
    }
    "[]".to_string()
}
