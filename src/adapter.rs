/// MCP Server adapter — exposes registry via Model Context Protocol.

use crate::core::Registry_core;
use crate::shared::Entry_x;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP Server state holding registry and repo path.
pub struct RulesMcpServer {
    pub registry: Arc<Mutex<Registry_core>>,
    pub repo_path: PathBuf,
}

impl RulesMcpServer {
    /// Generate help text showing available tools.
    pub async fn help(&self) -> String {
        let registry = self.registry.lock().await;
        let total_rules = registry.entries.len();
        let cats = registry.categories();

        format!(
            r#"# RulesMCP — AI coding standards lookup (Rust)

**{}** rules across **{}** categories

## Tools

| Tool | Purpose |
|------|---------|
| `help` | This overview |
| `search_rules` | Find rules by keyword |
| `get_rule` | Read full rule content |
| `get_context` | All rules for languages |
| `get_learning_path` | Phased reading order |
| `list_rules` | Browse available rules |
| `get_related` | Follow edges to related rules |

## Quick start

- **App architecture** → `get_context(["global"])`
- **New project setup** → `get_context(["global", "project-files"])`
- **Learn a language** → `get_learning_path(["rust"], phase=1)`
- **Search a topic** → `search_rules("error handling")`

## Categories

{}
"#,
            total_rules,
            cats.len(),
            cats.join(", ")
        )
    }

    /// Search rules by query and optional category.
    pub async fn search_rules(
        &self,
        query: &str,
        category: Option<&str>,
        limit: usize,
    ) -> String {
        let registry = self.registry.lock().await;
        let results = registry.search(query, category, limit);

        if results.is_empty() {
            return "No matching rules found.".to_string();
        }

        let mut lines = vec![];
        for entry in results {
            lines.push(format!("- **{}**: {}", entry.file, entry.title));
            if !entry.tags.is_empty() {
                let tags = entry.tags.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
                lines.push(format!("  tags: {}", tags));
            }
        }

        lines.join("\n")
    }

    /// Get full markdown content of a rule file.
    pub async fn get_rule(&self, file: &str) -> String {
        let path = self.repo_path.join(file);

        match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => format!("File not found: {}", file),
        }
    }

    /// Get combined rules context for given languages.
    pub async fn get_context(&self, languages: &[&str]) -> String {
        let registry = self.registry.lock().await;
        let matched = registry
            .list_files(None)
            .into_iter()
            .filter(|e| {
                languages.contains(&e.category.as_str())
                    || languages.contains(&"global")
            })
            .collect::<Vec<_>>();

        if matched.is_empty() {
            return "No rules found for the given languages.".to_string();
        }

        let mut sections = vec![];
        for entry in matched {
            let file_path = self.repo_path.join(&entry.file);
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                sections.push(format!("## {}\n", entry.file));
                if !entry.rules.is_empty() {
                    sections.push(format!("**RULES:** {}", entry.rules.join(" | ")));
                }
                if !entry.banned.is_empty() {
                    sections.push(format!("**BANNED:** {}", entry.banned.join(" | ")));
                }
                sections.push(content);
                sections.push("---".to_string());
            }
        }

        sections.join("\n\n")
    }

    /// Get learning path for given languages, optionally filtered by phase.
    pub async fn get_learning_path(&self, languages: &[&str], phase: Option<u32>) -> String {
        let registry = self.registry.lock().await;
        let layers = registry.learning_path(languages, phase);

        if layers.is_empty() {
            return "No rules found for the given languages.".to_string();
        }

        let mut sections = vec![format!(
            "# Learning Path: {} — {} rules\n",
            languages.join(", "),
            layers.iter().map(|l| l.len()).sum::<usize>()
        )];

        for (i, layer) in layers.iter().enumerate() {
            let phase_num = phase.unwrap_or(i as u32 + 1);
            sections.push(format!("## Phase {}: {} rules", phase_num, layer.len()));

            for entry in layer {
                let markers = [
                    if !entry.rules.is_empty() {
                        Some(format!("RULES: {}", entry.rules.len()))
                    } else {
                        None
                    },
                    if !entry.banned.is_empty() {
                        Some(format!("BANNED: {}", entry.banned.len()))
                    } else {
                        None
                    },
                ]
                .iter()
                .filter_map(|m| m.clone())
                .collect::<Vec<_>>();

                let marker_str = if markers.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", markers.join(", "))
                };

                sections.push(format!("- {}: {}{}", entry.file, entry.title, marker_str));
            }
            sections.push(String::new());
        }

        sections.join("\n")
    }

    /// List all rules, optionally filtered by category.
    pub async fn list_rules(&self, category: Option<&str>) -> String {
        let registry = self.registry.lock().await;
        let entries = registry.list_files(category);

        if entries.is_empty() {
            let available = registry.categories().join(", ");
            return format!("No rules found. Available categories: {}", available);
        }

        let mut lines = vec![];
        let mut current_cat = String::new();

        for entry in entries {
            if entry.category != current_cat {
                current_cat = entry.category.clone();
                lines.push(format!("\n### {}", current_cat));
            }
            lines.push(format!("- {}: {}", entry.file, entry.title));
        }

        lines.join("\n")
    }

    /// Get related rules by following edges from a specific rule file.
    pub async fn get_related(&self, file: &str) -> String {
        let registry = self.registry.lock().await;

        let entry = match registry.find_by_file(file) {
            Some(e) => e,
            None => return format!("File not found: {}", file),
        };

        let edges = &entry.edges;
        if edges.is_empty() {
            return format!("No edges found for {}", file);
        }

        let mut lines = vec![format!("# Edges for {}\n", file)];

        let labels = [
            ("requires", "Depends on (must read first)"),
            ("required_by", "Depended on by"),
            ("feeds", "Feeds into"),
            ("fed_by", "Fed by"),
            ("related", "Related"),
        ];

        for (edge_type, label) in &labels {
            if let Some(targets) = edges.get(*edge_type) {
                if !targets.is_empty() {
                    lines.push(format!("## {}", label));
                    for target in targets {
                        let title = registry
                            .find_by_file(target)
                            .map(|e| e.title)
                            .unwrap_or_else(|| "(not found)".to_string());
                        lines.push(format!("- {}: {}", target, title));
                    }
                    lines.push(String::new());
                }
            }
        }

        lines.join("\n")
    }
}
