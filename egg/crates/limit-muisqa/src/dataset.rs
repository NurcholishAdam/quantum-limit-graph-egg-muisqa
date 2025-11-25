// crates/limit-muisqa/src/dataset.rs
//! MuISQA dataset loading and management

use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::path::Path;

/// Intent type for multi-intent questions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Intent {
    Factual,
    Comparison,
    Explanation,
    Procedural,
    Opinion,
    Temporal,
    Causal,
    Hypothetical,
}

impl Intent {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "factual" => Some(Intent::Factual),
            "comparison" => Some(Intent::Comparison),
            "explanation" => Some(Intent::Explanation),
            "procedural" => Some(Intent::Procedural),
            "opinion" => Some(Intent::Opinion),
            "temporal" => Some(Intent::Temporal),
            "causal" => Some(Intent::Causal),
            "hypothetical" => Some(Intent::Hypothetical),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Intent::Factual => "factual",
            Intent::Comparison => "comparison",
            Intent::Explanation => "explanation",
            Intent::Procedural => "procedural",
            Intent::Opinion => "opinion",
            Intent::Temporal => "temporal",
            Intent::Causal => "causal",
            Intent::Hypothetical => "hypothetical",
        }
    }
}

/// A single MuISQA dataset entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuISQAEntry {
    pub id: String,
    pub question: String,
    pub intents: Vec<Intent>,
    pub context: Option<String>,
    pub gold_answer: Option<String>,
    pub metadata: serde_json::Value,
}

impl MuISQAEntry {
    pub fn new(id: String, question: String, intents: Vec<Intent>) -> Self {
        Self {
            id,
            question,
            intents,
            context: None,
            gold_answer: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_answer(mut self, answer: String) -> Self {
        self.gold_answer = Some(answer);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if entry has multiple intents
    pub fn is_multi_intent(&self) -> bool {
        self.intents.len() > 1
    }

    /// Get primary intent
    pub fn primary_intent(&self) -> Option<&Intent> {
        self.intents.first()
    }
}

/// MuISQA dataset manager
#[derive(Debug, Clone)]
pub struct MuISQADataset {
    pub entries: Vec<MuISQAEntry>,
    pub name: String,
}

impl MuISQADataset {
    pub fn new(name: String) -> Self {
        Self {
            entries: Vec::new(),
            name,
        }
    }

    /// Load dataset from JSON file
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read dataset file")?;
        let entries: Vec<MuISQAEntry> = serde_json::from_str(&content)
            .context("Failed to parse JSON dataset")?;
        
        Ok(Self {
            entries,
            name: path.as_ref()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    /// Load dataset from CSV file
    pub fn from_csv(path: impl AsRef<Path>) -> Result<Self> {
        let mut reader = csv::Reader::from_path(path.as_ref())
            .context("Failed to open CSV file")?;
        
        let mut entries = Vec::new();
        for result in reader.deserialize() {
            let record: CsvRecord = result.context("Failed to parse CSV record")?;
            entries.push(record.into_entry());
        }

        Ok(Self {
            entries,
            name: path.as_ref()
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    /// Create synthetic dataset for testing
    pub fn synthetic(size: usize) -> Self {
        let mut entries = Vec::new();
        
        for i in 0..size {
            let intents = if i % 3 == 0 {
                vec![Intent::Factual, Intent::Comparison]
            } else if i % 3 == 1 {
                vec![Intent::Explanation, Intent::Causal]
            } else {
                vec![Intent::Procedural]
            };

            let entry = MuISQAEntry::new(
                format!("q{}", i),
                format!("What is the answer to question {}?", i),
                intents,
            )
            .with_context(format!("Context for question {}", i))
            .with_answer(format!("Answer to question {}", i));

            entries.push(entry);
        }

        Self {
            entries,
            name: "synthetic".to_string(),
        }
    }

    /// Add entry to dataset
    pub fn add_entry(&mut self, entry: MuISQAEntry) {
        self.entries.push(entry);
    }

    /// Get entry by ID
    pub fn get_entry(&self, id: &str) -> Option<&MuISQAEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Filter entries by intent
    pub fn filter_by_intent(&self, intent: &Intent) -> Vec<&MuISQAEntry> {
        self.entries
            .iter()
            .filter(|e| e.intents.contains(intent))
            .collect()
    }

    /// Get multi-intent entries only
    pub fn multi_intent_entries(&self) -> Vec<&MuISQAEntry> {
        self.entries
            .iter()
            .filter(|e| e.is_multi_intent())
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> DatasetStats {
        let total = self.entries.len();
        let multi_intent = self.multi_intent_entries().len();
        let single_intent = total - multi_intent;

        let mut intent_counts = std::collections::HashMap::new();
        for entry in &self.entries {
            for intent in &entry.intents {
                *intent_counts.entry(intent.as_str()).or_insert(0) += 1;
            }
        }

        DatasetStats {
            total_entries: total,
            single_intent_entries: single_intent,
            multi_intent_entries: multi_intent,
            intent_distribution: intent_counts,
        }
    }

    /// Save dataset to JSON
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CsvRecord {
    id: String,
    question: String,
    intents: String,
    context: Option<String>,
    gold_answer: Option<String>,
}

impl CsvRecord {
    fn into_entry(self) -> MuISQAEntry {
        let intents: Vec<Intent> = self.intents
            .split(',')
            .filter_map(|s| Intent::from_str(s.trim()))
            .collect();

        let mut entry = MuISQAEntry::new(self.id, self.question, intents);
        if let Some(ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        if let Some(ans) = self.gold_answer {
            entry = entry.with_answer(ans);
        }
        entry
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStats {
    pub total_entries: usize,
    pub single_intent_entries: usize,
    pub multi_intent_entries: usize,
    pub intent_distribution: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_parsing() {
        assert_eq!(Intent::from_str("factual"), Some(Intent::Factual));
        assert_eq!(Intent::from_str("Comparison"), Some(Intent::Comparison));
        assert_eq!(Intent::from_str("unknown"), None);
    }

    #[test]
    fn test_entry_creation() {
        let entry = MuISQAEntry::new(
            "q1".to_string(),
            "What is AI?".to_string(),
            vec![Intent::Factual, Intent::Explanation],
        );

        assert!(entry.is_multi_intent());
        assert_eq!(entry.primary_intent(), Some(&Intent::Factual));
    }

    #[test]
    fn test_synthetic_dataset() {
        let dataset = MuISQADataset::synthetic(10);
        assert_eq!(dataset.entries.len(), 10);
        
        let stats = dataset.stats();
        assert_eq!(stats.total_entries, 10);
        assert!(stats.multi_intent_entries > 0);
    }
}
