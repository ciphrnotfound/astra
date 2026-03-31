use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::memory::MemoryStore;
use crate::model::CodexModel;
use crate::semantic_graph::TemporalGraph;

// ────────────────────────────────────────────────────────────────
//  CONCEPT CATEGORIES
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConceptCategory {
    Risk,       // "auth system is high risk"
    Coupling,   // "payments and auth are secretly coupled"
    Ownership,  // "@mike is the only person who understands processor.rs"
    Pattern,    // "Friday commits cause Monday incidents"
    Staleness,  // "processor.rs hasn't been touched in 47 days"
    Architecture, // "the auth service is the central dependency"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WatchPriority {
    Low,
    Medium,
    High,
    Critical,
}

// ────────────────────────────────────────────────────────────────
//  LAYER 2: SEMANTIC CONCEPTS
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConcept {
    pub id: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f32,
    pub last_updated: u64,
    pub category: ConceptCategory,
}

// ────────────────────────────────────────────────────────────────
//  LAYER 3: PROCEDURAL PATTERNS
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralPattern {
    pub id: String,
    pub description: String,
    pub evidence_count: usize,
    pub confidence: f32,
    pub category: String,
}

// ────────────────────────────────────────────────────────────────
//  LAYER 4: PROSPECTIVE WATCHES
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectiveWatch {
    pub description: String,
    pub trigger_condition: String,
    pub created_at: u64,
    pub priority: WatchPriority,
}

// ────────────────────────────────────────────────────────────────
//  THE CONCEPT STORE
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConceptStore {
    pub concepts: Vec<SemanticConcept>,
    pub patterns: Vec<ProceduralPattern>,
    pub watches: Vec<ProspectiveWatch>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl ConceptStore {
    pub fn new() -> Self {
        Self::default()
    }

    // ────────────────────────────────────────────────────────────
    //  LAYER 2: Derive concepts from memory + graph (1 LLM call)
    // ────────────────────────────────────────────────────────────

    /// Uses a SINGLE Gemini API call to synthesize concepts from
    /// recent episodic memory + temporal graph data.
    pub fn derive_concepts(
        &mut self,
        memory: &MemoryStore,
        graph: &TemporalGraph,
        model: &dyn CodexModel,
    ) -> Result<usize> {
        // 1. Gather raw evidence from episodic memory
        let recent_entries = memory.recent(30);
        let mut evidence_block = String::new();
        for entry in &recent_entries {
            let _ = writeln!(&mut evidence_block, "- [{}] {}", entry.kind, entry.content);
        }

        // 2. Gather evidence from the temporal graph
        let mut graph_block = String::new();

        // Developer stats
        for dev in graph.developers.values() {
            let _ = writeln!(
                &mut graph_block,
                "Developer '{}': {} commits, active from timestamp {} to {}",
                dev.name, dev.commit_count, dev.first_seen, dev.last_seen
            );
        }

        // Stale files
        for history in graph.file_histories.values() {
            if history.staleness_days > 30 {
                let _ = writeln!(
                    &mut graph_block,
                    "STALE FILE: {} — {} days since last change, {} total commits by {} authors",
                    history.path,
                    history.staleness_days,
                    history.total_changes,
                    history.authors.len()
                );
            }
        }

        // Co-change patterns
        for pair in graph.co_changes.iter().take(10) {
            let _ = writeln!(
                &mut graph_block,
                "COUPLING: {} <-> {} changed together {} times ({:.0}% coupling score)",
                pair.file_a, pair.file_b, pair.co_change_count, pair.coupling_score * 100.0
            );
        }

        // Single ownership risks
        for history in graph.file_histories.values() {
            if history.authors.len() == 1 && history.total_changes >= 5 {
                let owner = history.authors.first().map(|a| &a.name).unwrap();
                let _ = writeln!(
                    &mut graph_block,
                    "SINGLE OWNER RISK: {} is the ONLY person who has ever changed {} ({} commits)",
                    owner, history.path, history.total_changes
                );
            }
        }

        if evidence_block.is_empty() && graph_block.is_empty() {
            return Ok(0);
        }

        // 3. Single LLM call to extract concepts
        let prompt = format!(
            "You are Astra, a codebase intelligence engine. Analyze the following evidence and extract 3-7 semantic concepts.\n\n\
            RULES:\n\
            - Each concept must be a SHORT statement (1 sentence max)\n\
            - Each concept must have a confidence between 0.0 and 1.0\n\
            - Each concept must be categorized as: Risk, Coupling, Ownership, Pattern, Staleness, or Architecture\n\
            - Only state things the evidence actually supports. DO NOT hallucinate.\n\
            - Format EXACTLY as: CONCEPT|<category>|<confidence>|<description>\n\n\
            EPISODIC MEMORY:\n{}\n\n\
            TEMPORAL GRAPH DATA:\n{}\n\n\
            Output ONLY the concepts, one per line:",
            evidence_block, graph_block
        );

        let response = model.complete(&prompt)?;

        // 4. Parse the response into SemanticConcepts
        let mut new_count = 0;
        for line in response.lines() {
            let line = line.trim();
            if !line.starts_with("CONCEPT|") {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() < 4 {
                continue;
            }

            let category = match parts[1].trim().to_lowercase().as_str() {
                "risk" => ConceptCategory::Risk,
                "coupling" => ConceptCategory::Coupling,
                "ownership" => ConceptCategory::Ownership,
                "pattern" => ConceptCategory::Pattern,
                "staleness" => ConceptCategory::Staleness,
                "architecture" => ConceptCategory::Architecture,
                _ => continue,
            };

            let confidence: f32 = parts[2].trim().parse().unwrap_or(0.5);
            let description = parts[3].trim().to_string();

            if description.is_empty() {
                continue;
            }

            let id = format!(
                "{}-{}",
                description
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("-")
                    .to_lowercase(),
                now_secs() % 10000
            );

            self.concepts.push(SemanticConcept {
                id,
                description,
                evidence: Vec::new(), // Could attach raw evidence here
                confidence,
                last_updated: now_secs(),
                category,
            });
            new_count += 1;
        }

        Ok(new_count)
    }

    // ────────────────────────────────────────────────────────────
    //  LAYER 3: Detect procedural patterns (Zero API calls)
    // ────────────────────────────────────────────────────────────

    pub fn detect_patterns(&mut self, graph: &TemporalGraph) {
        self.patterns.clear();

        // Pattern: Single-owner files
        let single_owner_files: Vec<_> = graph
            .file_histories
            .values()
            .filter(|h| h.authors.len() == 1 && h.total_changes >= 3)
            .collect();

        if !single_owner_files.is_empty() {
            self.patterns.push(ProceduralPattern {
                id: "single-owner-risk".into(),
                description: format!(
                    "{} files have only ever been touched by a single developer — bus factor risk",
                    single_owner_files.len()
                ),
                evidence_count: single_owner_files.len(),
                confidence: 0.9,
                category: "ownership".into(),
            });
        }

        // Pattern: Frequently coupled files
        let high_coupling: Vec<_> = graph
            .co_changes
            .iter()
            .filter(|p| p.coupling_score > 0.5)
            .collect();

        if !high_coupling.is_empty() {
            self.patterns.push(ProceduralPattern {
                id: "hidden-coupling".into(),
                description: format!(
                    "{} file pairs have high coupling (>50%) — consider refactoring or documenting the dependency",
                    high_coupling.len()
                ),
                evidence_count: high_coupling.len(),
                confidence: 0.85,
                category: "coupling".into(),
            });
        }

        // Pattern: Stale critical files
        let stale_files: Vec<_> = graph
            .file_histories
            .values()
            .filter(|h| h.staleness_days > 60 && h.total_changes >= 5)
            .collect();

        if !stale_files.is_empty() {
            self.patterns.push(ProceduralPattern {
                id: "stale-critical-files".into(),
                description: format!(
                    "{} frequently-changed files haven't been touched in 60+ days — potential drift",
                    stale_files.len()
                ),
                evidence_count: stale_files.len(),
                confidence: 0.75,
                category: "staleness".into(),
            });
        }

        // Pattern: Developer concentration
        if let Some((top_dev, _)) = graph
            .developers
            .iter()
            .max_by_key(|(_, d)| d.commit_count)
        {
            let total_commits: usize = graph.developers.values().map(|d| d.commit_count).sum();
            if total_commits > 0 {
                let top_dev_node = &graph.developers[top_dev];
                let concentration = top_dev_node.commit_count as f32 / total_commits as f32;
                if concentration > 0.5 && graph.developers.len() > 1 {
                    self.patterns.push(ProceduralPattern {
                        id: "developer-concentration".into(),
                        description: format!(
                            "{} has written {:.0}% of all commits — high bus factor risk if they leave",
                            top_dev,
                            concentration * 100.0
                        ),
                        evidence_count: top_dev_node.commit_count,
                        confidence: 0.9,
                        category: "ownership".into(),
                    });
                }
            }
        }
    }

    // ────────────────────────────────────────────────────────────
    //  LAYER 4: Prospective watches (Zero API calls)
    // ────────────────────────────────────────────────────────────

    pub fn check_watches(&mut self, graph: &TemporalGraph) {
        self.watches.clear();

        // Watch: Stale files with history
        for history in graph.file_histories.values() {
            if history.staleness_days > 90 && history.total_changes >= 3 {
                self.watches.push(ProspectiveWatch {
                    description: format!(
                        "{} hasn't been touched in {} days but has {} historical changes",
                        history.path, history.staleness_days, history.total_changes
                    ),
                    trigger_condition: "staleness > 90 days".into(),
                    created_at: now_secs(),
                    priority: if history.staleness_days > 180 {
                        WatchPriority::High
                    } else {
                        WatchPriority::Medium
                    },
                });
            }
        }

        // Watch: Single-owner critical files
        for history in graph.file_histories.values() {
            if history.authors.len() == 1 && history.total_changes >= 10 {
                let owner = history
                    .primary_owner
                    .as_deref()
                    .unwrap_or("unknown");
                self.watches.push(ProspectiveWatch {
                    description: format!(
                        "If {} leaves, {} becomes orphaned ({} commits, no other contributor)",
                        owner, history.path, history.total_changes
                    ),
                    trigger_condition: "single owner with 10+ commits".into(),
                    created_at: now_secs(),
                    priority: WatchPriority::High,
                });
            }
        }

        // Watch: High coupling without explicit imports
        for pair in graph.co_changes.iter().take(5) {
            if pair.coupling_score > 0.6 {
                self.watches.push(ProspectiveWatch {
                    description: format!(
                        "{} and {} are secretly coupled ({:.0}% co-change rate) — " ,
                        pair.file_a, pair.file_b, pair.coupling_score * 100.0
                    ) + "changes to one will likely require changes to the other",
                    trigger_condition: "coupling score > 60%".into(),
                    created_at: now_secs(),
                    priority: WatchPriority::Medium,
                });
            }
        }
    }

    // ────────────────────────────────────────────────────────────
    //  DISPLAY
    // ────────────────────────────────────────────────────────────

    pub fn concepts_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "🧠 Semantic Concepts ({} total)", self.concepts.len());
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        if self.concepts.is_empty() {
            let _ = writeln!(
                &mut out,
                "\nNo concepts derived yet. Run :analyze to extract concepts from memory."
            );
            return out;
        }

        for concept in &self.concepts {
            let badge = match concept.category {
                ConceptCategory::Risk => "🔴",
                ConceptCategory::Coupling => "🔗",
                ConceptCategory::Ownership => "👤",
                ConceptCategory::Pattern => "📊",
                ConceptCategory::Staleness => "⏳",
                ConceptCategory::Architecture => "🏗️",
            };
            let _ = writeln!(
                &mut out,
                "\n{} {} (confidence: {:.0}%)",
                badge, concept.description, concept.confidence * 100.0
            );
        }

        if !self.patterns.is_empty() {
            let _ = writeln!(&mut out, "\n\n📊 Procedural Patterns ({} detected):", self.patterns.len());
            let _ = writeln!(&mut out, "────────────────────────────────────────");
            for pattern in &self.patterns {
                let _ = writeln!(
                    &mut out,
                    "  • {} (confidence: {:.0}%, evidence: {})",
                    pattern.description,
                    pattern.confidence * 100.0,
                    pattern.evidence_count
                );
            }
        }

        out
    }

    pub fn watches_report(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "👁️ Prospective Watches ({} active)", self.watches.len());
        let _ = writeln!(&mut out, "════════════════════════════════════════");

        if self.watches.is_empty() {
            let _ = writeln!(
                &mut out,
                "\nNo active watches. Run :index and :analyze first."
            );
            return out;
        }

        for watch in &self.watches {
            let priority_badge = match watch.priority {
                WatchPriority::Critical => "🔴 CRITICAL",
                WatchPriority::High => "🟠 HIGH",
                WatchPriority::Medium => "🟡 MEDIUM",
                WatchPriority::Low => "🟢 LOW",
            };
            let _ = writeln!(&mut out, "\n{}: {}", priority_badge, watch.description);
        }

        out
    }

    /// Save to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Load from disk.
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let store: Self = serde_json::from_str(&data)?;
        Ok(store)
    }
}
