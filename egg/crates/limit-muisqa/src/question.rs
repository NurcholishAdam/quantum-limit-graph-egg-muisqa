// crates/limit-muisqa/src/question.rs
//! Question parsing and intent extraction

use serde::{Deserialize, Serialize};
use crate::dataset::Intent;

/// Intent type with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentType {
    pub intent: Intent,
    pub confidence: f64,
}

impl IntentType {
    pub fn new(intent: Intent, confidence: f64) -> Self {
        Self { intent, confidence }
    }
}

/// Parsed question with extracted intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub text: String,
    pub intents: Vec<IntentType>,
    pub keywords: Vec<String>,
    pub entities: Vec<String>,
}

impl Question {
    pub fn new(text: String) -> Self {
        Self {
            text,
            intents: Vec::new(),
            keywords: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn with_intents(mut self, intents: Vec<IntentType>) -> Self {
        self.intents = intents;
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.entities = entities;
        self
    }

    /// Get primary intent (highest confidence)
    pub fn primary_intent(&self) -> Option<&IntentType> {
        self.intents
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }

    /// Check if question has multiple intents
    pub fn is_multi_intent(&self) -> bool {
        self.intents.len() > 1
    }

    /// Get intents above confidence threshold
    pub fn intents_above_threshold(&self, threshold: f64) -> Vec<&IntentType> {
        self.intents
            .iter()
            .filter(|i| i.confidence >= threshold)
            .collect()
    }
}

/// Question parser for intent extraction
pub struct QuestionParser {
    intent_keywords: std::collections::HashMap<Intent, Vec<String>>,
}

impl QuestionParser {
    pub fn new() -> Self {
        let mut intent_keywords = std::collections::HashMap::new();

        intent_keywords.insert(
            Intent::Factual,
            vec!["what", "who", "when", "where", "which"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Comparison,
            vec!["compare", "difference", "versus", "vs", "better", "worse"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Explanation,
            vec!["why", "how", "explain", "reason", "because"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Procedural,
            vec!["how to", "steps", "process", "procedure", "method"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Opinion,
            vec!["think", "believe", "opinion", "view", "perspective"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Temporal,
            vec!["when", "before", "after", "during", "timeline"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Causal,
            vec!["cause", "effect", "result", "lead to", "due to"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        intent_keywords.insert(
            Intent::Hypothetical,
            vec!["if", "would", "could", "suppose", "imagine"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        Self { intent_keywords }
    }

    /// Parse question and extract intents
    pub fn parse(&self, text: &str) -> Question {
        let text_lower = text.to_lowercase();
        let mut intents = Vec::new();

        // Extract intents based on keywords
        for (intent, keywords) in &self.intent_keywords {
            let mut matches = 0;
            for keyword in keywords {
                if text_lower.contains(keyword) {
                    matches += 1;
                }
            }

            if matches > 0 {
                let confidence = (matches as f64 / keywords.len() as f64).min(1.0);
                intents.push(IntentType::new(intent.clone(), confidence));
            }
        }

        // Sort by confidence
        intents.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Extract keywords (simple tokenization)
        let keywords: Vec<String> = text_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .take(10)
            .map(|s| s.to_string())
            .collect();

        // Extract entities (capitalized words in original text)
        let entities: Vec<String> = text
            .split_whitespace()
            .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .map(|s| s.to_string())
            .collect();

        Question::new(text.to_string())
            .with_intents(intents)
            .with_keywords(keywords)
            .with_entities(entities)
    }

    /// Parse multiple questions
    pub fn parse_batch(&self, texts: &[String]) -> Vec<Question> {
        texts.iter().map(|t| self.parse(t)).collect()
    }
}

impl Default for QuestionParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_parser() {
        let parser = QuestionParser::new();
        let question = parser.parse("What is the difference between AI and ML?");

        assert!(!question.intents.is_empty());
        assert!(question.is_multi_intent());
        
        let primary = question.primary_intent().unwrap();
        assert!(primary.confidence > 0.0);
    }

    #[test]
    fn test_intent_extraction() {
        let parser = QuestionParser::new();
        
        let q1 = parser.parse("How does quantum computing work?");
        assert!(q1.intents.iter().any(|i| matches!(i.intent, Intent::Explanation)));

        let q2 = parser.parse("Compare Python and Rust");
        assert!(q2.intents.iter().any(|i| matches!(i.intent, Intent::Comparison)));
    }

    #[test]
    fn test_keyword_extraction() {
        let parser = QuestionParser::new();
        let question = parser.parse("What are the benefits of machine learning?");

        assert!(!question.keywords.is_empty());
        assert!(question.keywords.iter().any(|k| k.contains("benefits") || k.contains("machine")));
    }
}
