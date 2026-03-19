/// Core domain logic — rule registry, search, and learning paths.

use crate::shared::{Entry_x, ScoredEntry_x, WeightedField_x, Error_x};
use std::path::Path;

/// In-memory index of register.jsonl entries.
#[derive(Debug, Clone)]
pub struct Registry_core {
    pub entries: Vec<Entry_x>,
}

impl Registry_core {
    /// Create new empty registry.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load register.jsonl from Rules repo path.
    pub async fn load(&mut self, repo_path: &Path) -> Result<(), Error_x> {
        let jsonl_path = repo_path.join("register.jsonl");
        let content = tokio::fs::read_to_string(&jsonl_path).await?;

        self.entries.clear();
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let entry: Entry_x = serde_json::from_str(line)?;
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    /// Search entries by query tokens — weighted scoring across tags, concepts, keywords, title, etc.
    pub fn search(&self, query: &str, category: Option<&str>, limit: usize) -> Vec<Entry_x> {
        let tokens: Vec<String> = query.to_lowercase().split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if tokens.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<ScoredEntry_x> = Vec::new();
        for entry in &self.entries {
            if let Some(cat) = category {
                if entry.category != cat {
                    continue;
                }
            }
            let score = score_entry(entry, &tokens);
            if score > 0 {
                scored.push(ScoredEntry_x {
                    score,
                    entry: entry.clone(),
                });
            }
        }

        scored.sort();
        scored.into_iter().take(limit).map(|s| s.entry).collect()
    }

    /// List all entries, optionally filtered by category.
    pub fn list_files(&self, category: Option<&str>) -> Vec<Entry_x> {
        self.entries.iter()
            .filter(|e| category.is_none() || e.category == category.unwrap_or(""))
            .cloned()
            .collect()
    }

    /// Get all unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.entries.iter()
            .map(|e| e.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Find entry by file path.
    pub fn find_by_file(&self, file: &str) -> Option<Entry_x> {
        self.entries.iter()
            .find(|e| e.file == file)
            .cloned()
    }

    /// Get learning path grouped by layer for given languages.
    /// Returns Vec of layers (each layer is Vec of entries), grouped by layer number.
    pub fn learning_path(&self, languages: &[&str], phase: Option<u32>) -> Vec<Vec<Entry_x>> {
        let lang_set: std::collections::HashSet<_> =
            languages.iter().map(|s| s.to_lowercase()).collect();

        let mut include_cats = lang_set.clone();
        include_cats.insert("global".to_string());
        include_cats.insert("project-files".to_string());
        include_cats.insert("gateway".to_string());
        include_cats.insert("adapter".to_string());
        include_cats.insert("core".to_string());
        include_cats.insert("pal".to_string());

        let relevant: Vec<Entry_x> = self.entries.iter()
            .filter(|e| include_cats.contains(&e.category.to_lowercase()))
            .cloned()
            .collect();

        if relevant.is_empty() {
            return Vec::new();
        }

        let mut layer_groups: std::collections::HashMap<u32, Vec<Entry_x>> =
            std::collections::HashMap::new();

        for e in relevant {
            let layer = if e.layer > 0 { e.layer } else { 4 };
            layer_groups.entry(layer).or_insert_with(Vec::new).push(e);
        }

        let mut layers: Vec<Vec<Entry_x>> = Vec::new();
        let mut layer_nums: Vec<u32> = layer_groups.keys().copied().collect();
        layer_nums.sort();

        for layer_num in layer_nums {
            if let Some(mut entries) = layer_groups.remove(&layer_num) {
                entries.sort_by(|a, b| a.file.cmp(&b.file));
                layers.push(entries);
            }
        }

        if let Some(p) = phase {
            if p > 0 && (p as usize) <= layers.len() {
                return vec![layers[(p - 1) as usize].clone()];
            }
        }

        layers
    }
}

/// Score entry based on token matches in weighted fields.
fn score_entry(entry: &Entry_x, tokens: &[String]) -> u32 {
    let fields = build_weighted_fields(entry);
    let mut score = 0u32;

    for token in tokens {
        for field in &fields {
            if matches_bidirectional(token, &field.text) {
                score = score.saturating_add(field.weight);
            }
        }
    }

    if score > 0 && entry.binding {
        score = score.saturating_add(10);
    }

    score
}

/// Build (text, weight) pairs from entry fields.
fn build_weighted_fields(entry: &Entry_x) -> Vec<WeightedField_x> {
    let mut fields = Vec::new();

    // File path (weight 3)
    fields.push(WeightedField_x {
        text: entry.file.to_lowercase(),
        weight: 3,
    });

    // Title (weight 3)
    fields.push(WeightedField_x {
        text: entry.title.to_lowercase(),
        weight: 3,
    });

    // Subtitle (weight 1)
    if !entry.subtitle.is_empty() {
        fields.push(WeightedField_x {
            text: entry.subtitle.to_lowercase(),
            weight: 1,
        });
    }

    // Tags (weight 2 each)
    for tag in &entry.tags {
        fields.push(WeightedField_x {
            text: tag.to_lowercase(),
            weight: 2,
        });
    }

    // Concepts (weight 2 each)
    for concept in &entry.concepts {
        fields.push(WeightedField_x {
            text: concept.to_lowercase(),
            weight: 2,
        });
    }

    // Keywords (weight 1 each)
    for kw in &entry.keywords {
        fields.push(WeightedField_x {
            text: kw.to_lowercase(),
            weight: 1,
        });
    }

    // Axioms (weight 2 each)
    for axiom in &entry.axioms {
        fields.push(WeightedField_x {
            text: axiom.to_lowercase(),
            weight: 2,
        });
    }

    // Category (weight 1)
    fields.push(WeightedField_x {
        text: entry.category.to_lowercase(),
        weight: 1,
    });

    fields
}

/// Bidirectional substring match.
fn matches_bidirectional(token: &str, field: &str) -> bool {
    token.contains(field) || field.contains(token)
}
