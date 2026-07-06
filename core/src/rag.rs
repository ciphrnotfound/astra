use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Chunk
// ---------------------------------------------------------------------------

/// A contiguous slice of a source file, sized for LLM context injection.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chunk {
    pub id: String,
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub language: String,
    /// Optional: embedding vector, present after embed() is called.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl Chunk {
    pub fn header(&self) -> String {
        format!(
            "// {} (lines {}-{})\n",
            self.path.display(),
            self.start_line,
            self.end_line
        )
    }

    pub fn with_header(&self) -> String {
        format!("{}{}", self.header(), self.content)
    }
}

// ---------------------------------------------------------------------------
// Chunker
// ---------------------------------------------------------------------------

const MAX_CHUNK_LINES: usize = 60;
const OVERLAP_LINES: usize = 8;

/// Split a file's content into overlapping chunks.
/// Tries to split at function/class boundaries; falls back to fixed windows.
pub fn chunk_file(path: &Path, content: &str, language: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let boundaries = find_boundaries(&lines, language);
    let mut chunks = Vec::new();
    let file_stem = path.to_string_lossy().replace('\\', "/");

    if boundaries.len() > 1 {
        // Merge boundaries into windows that stay under MAX_CHUNK_LINES
        let mut i = 0;
        let mut chunk_idx = 0;
        while i < boundaries.len() {
            let start = boundaries[i];
            let mut end = start;
            let mut j = i + 1;
            while j < boundaries.len() {
                let candidate_end = if j + 1 < boundaries.len() {
                    boundaries[j + 1].saturating_sub(1)
                } else {
                    lines.len().saturating_sub(1)
                };
                if candidate_end - start + 1 <= MAX_CHUNK_LINES {
                    end = candidate_end;
                    j += 1;
                } else {
                    break;
                }
            }
            if end == start && j < boundaries.len() {
                // single boundary block is already too large — take it as-is
                end = (boundaries[j].saturating_sub(1)).min(start + MAX_CHUNK_LINES - 1);
            }
            end = end.min(lines.len().saturating_sub(1));

            let chunk_lines: Vec<&str> = lines[start..=end].to_vec();
            let chunk_content = chunk_lines.join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk {
                    id: format!("{}::{}", file_stem, chunk_idx),
                    path: path.to_path_buf(),
                    start_line: start + 1,
                    end_line: end + 1,
                    content: chunk_content,
                    language: language.to_string(),
                    embedding: None,
                });
                chunk_idx += 1;
            }
            // advance past consumed boundaries, with overlap
            i = j.saturating_sub(1).max(i + 1);
        }
    } else {
        // No structural boundaries — use sliding window
        let mut start = 0usize;
        let mut chunk_idx = 0usize;
        while start < lines.len() {
            let end = (start + MAX_CHUNK_LINES).min(lines.len());
            let chunk_content = lines[start..end].join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk {
                    id: format!("{}::{}", file_stem, chunk_idx),
                    path: path.to_path_buf(),
                    start_line: start + 1,
                    end_line: end,
                    content: chunk_content,
                    language: language.to_string(),
                    embedding: None,
                });
                chunk_idx += 1;
            }
            if end >= lines.len() {
                break;
            }
            start = end.saturating_sub(OVERLAP_LINES);
        }
    }

    chunks
}

/// Return line indices (0-based) where new top-level declarations begin.
fn find_boundaries(lines: &[&str], language: &str) -> Vec<usize> {
    let mut boundaries = vec![0usize];
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_declaration_start(trimmed, language) {
            if i > 0 {
                boundaries.push(i);
            }
        }
    }
    boundaries
}

fn is_declaration_start(trimmed: &str, language: &str) -> bool {
    match language {
        "rust" => {
            (trimmed.starts_with("pub fn ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("pub impl ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("trait "))
                && !trimmed.starts_with("//")
        }
        "typescript" | "javascript" => {
            (trimmed.starts_with("function ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("export default function")
                || trimmed.starts_with("export const ")
                || trimmed.starts_with("export class ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("async function ")
                || trimmed.starts_with("export async function "))
                && !trimmed.starts_with("//")
        }
        "python" => {
            (trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("class "))
                && !trimmed.starts_with("#")
        }
        "go" => {
            (trimmed.starts_with("func ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("var ")
                || trimmed.starts_with("const "))
                && !trimmed.starts_with("//")
        }
        "java" | "kotlin" => {
            (trimmed.contains("class ")
                || trimmed.contains("interface ")
                || trimmed.contains("void ")
                || trimmed.contains("public ")
                || trimmed.contains("private ")
                || trimmed.contains("protected "))
                && trimmed.ends_with('{')
                && !trimmed.starts_with("//")
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Vector Store
// ---------------------------------------------------------------------------

/// Lightweight in-process vector store using cosine similarity.
/// Persisted as a JSON file alongside the index.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct VectorStore {
    pub chunks: Vec<Chunk>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let data = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn upsert_chunks(&mut self, new_chunks: Vec<Chunk>) {
        // Replace all chunks from the same file
        if let Some(first) = new_chunks.first() {
            let path = first.path.clone();
            self.chunks.retain(|c| c.path != path);
        }
        self.chunks.extend(new_chunks);
    }

    pub fn set_embedding(&mut self, chunk_id: &str, embedding: Vec<f32>) {
        if let Some(chunk) = self.chunks.iter_mut().find(|c| c.id == chunk_id) {
            chunk.embedding = Some(embedding);
        }
    }

    /// Semantic search: returns top-k chunks ranked by cosine similarity.
    /// Falls back to keyword search if no embeddings are available.
    pub fn search(&self, query: &str, query_embedding: Option<&[f32]>, top_k: usize) -> Vec<&Chunk> {
        if let Some(q_vec) = query_embedding {
            let has_embeddings = self.chunks.iter().any(|c| c.embedding.is_some());
            if has_embeddings {
                return self.semantic_search(q_vec, top_k);
            }
        }
        self.keyword_search(query, top_k)
    }

    fn semantic_search(&self, query_vec: &[f32], top_k: usize) -> Vec<&Chunk> {
        let mut scored: Vec<(f32, &Chunk)> = self
            .chunks
            .iter()
            .filter_map(|chunk| {
                chunk.embedding.as_ref().map(|emb| {
                    let score = cosine_similarity(query_vec, emb);
                    (score, chunk)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, c)| c).collect()
    }

    /// TF-IDF-style keyword search: scores chunks by term overlap with query.
    pub fn keyword_search(&self, query: &str, top_k: usize) -> Vec<&Chunk> {
        let query_terms: Vec<String> = tokenize(query);
        if query_terms.is_empty() {
            return self.chunks.iter().take(top_k).collect();
        }

        // Compute IDF weights across all chunks
        let total = self.chunks.len() as f32;
        let mut doc_freq: HashMap<&str, usize> = HashMap::new();
        for chunk in &self.chunks {
            let chunk_terms: Vec<String> = tokenize(&chunk.content);
            let unique: std::collections::HashSet<String> = chunk_terms.into_iter().collect();
            for term in &query_terms {
                if unique.contains(term) {
                    *doc_freq.entry(term.as_str()).or_insert(0) += 1;
                }
            }
        }

        let mut scored: Vec<(f32, &Chunk)> = self
            .chunks
            .iter()
            .map(|chunk| {
                let chunk_terms: Vec<String> = tokenize(&chunk.content);
                let term_count = chunk_terms.len() as f32;
                if term_count == 0.0 {
                    return (0.0, chunk);
                }
                let mut score = 0.0f32;
                for term in &query_terms {
                    let tf = chunk_terms.iter().filter(|t| *t == term).count() as f32 / term_count;
                    let df = *doc_freq.get(term.as_str()).unwrap_or(&0) as f32;
                    let idf = if df > 0.0 { (total / df).ln() + 1.0 } else { 0.0 };
                    score += tf * idf;
                }
                (score, chunk)
            })
            .filter(|(s, _)| *s > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_k).map(|(_, c)| c).collect()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn embedded_count(&self) -> usize {
        self.chunks.iter().filter(|c| c.embedding.is_some()).count()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Simple tokenizer: lowercase alphanumeric tokens, min length 2.
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            if current.len() >= 2 {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= 2 {
        tokens.push(current);
    }
    tokens
}

// ---------------------------------------------------------------------------
// Format helpers
// ---------------------------------------------------------------------------

/// Format retrieved chunks into a compact block for LLM prompt injection.
pub fn format_chunks_for_prompt(chunks: &[&Chunk], max_chars: usize) -> String {
    let mut out = String::new();
    let mut total = 0usize;
    for chunk in chunks {
        let block = format!(
            "```{}\n// {}: lines {}-{}\n{}\n```\n\n",
            chunk.language,
            chunk.path.display(),
            chunk.start_line,
            chunk.end_line,
            chunk.content.trim()
        );
        if total + block.len() > max_chars {
            break;
        }
        out.push_str(&block);
        total += block.len();
    }
    out
}
