/// Shared types and utilities — no internal dependencies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A rule registry entry from register.jsonl.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry_x {
    pub file: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub banned: Vec<String>,
    #[serde(default)]
    pub layer: u32,
    #[serde(default)]
    pub binding: bool,
    #[serde(default)]
    pub edges: HashMap<String, Vec<String>>,
}

/// Weighted search field for scoring.
#[derive(Debug, Clone)]
pub struct WeightedField_x {
    pub text: String,
    pub weight: u32,
}

/// Entry with search score for ranking.
#[derive(Debug, Clone)]
pub struct ScoredEntry_x {
    pub score: u32,
    pub entry: Entry_x,
}

impl Eq for ScoredEntry_x {}
impl PartialEq for ScoredEntry_x {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.entry.file == other.entry.file
    }
}
impl Ord for ScoredEntry_x {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.cmp(&self.score).then_with(|| self.entry.file.cmp(&other.entry.file))
    }
}
impl PartialOrd for ScoredEntry_x {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error_x {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("git: {0}")]
    Git(#[from] git2::Error),
}
