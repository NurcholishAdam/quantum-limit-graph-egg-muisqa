// crates/limit-muisqa/src/metrics.rs
//! Metrics tracking for MuISQA evaluation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Retrieval metrics for multi-intent questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub mrr: f64, // Mean Reciprocal Rank
    pub ndcg: f64, // Normalized Discounted Cumulative Gain
    pub retrieved_count: usize,
    pub relevant_count: usize,
}

impl RetrievalMetrics {
    pub fn new() -> Self {
        Self {
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            mrr: 0.0,
            ndcg: 0.0,
            retrieved_count: 0,
            relevant_count: 0,
        }
    }

    /// Calculate metrics from retrieved and relevant items
    pub fn calculate(retrieved: &[String], relevant: &[String]) -> Self {
        let retrieved_set: std::collections::HashSet<_> = retrieved.iter().collect();
        let relevant_set: std::collections::HashSet<_> = relevant.iter().collect();

        let true_positives = retrieved_set.intersection(&relevant_set).count();
        let retrieved_count = retrieved.len();
        let relevant_count = relevant.len();

        let precision = if retrieved_count > 0 {
            true_positives as f64 / retrieved_count as f64
        } else {
            0.0
        };

        let recall = if relevant_count > 0 {
            true_positives as f64 / relevant_count as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Calculate MRR (Mean Reciprocal Rank)
        let mrr = Self::calculate_mrr(retrieved, relevant);

        // Calculate NDCG
        let ndcg = Self::calculate_ndcg(retrieved, relevant);

        Self {
            precision,
            recall,
            f1_score,
            mrr,
            ndcg,
            retrieved_count,
            relevant_count,
        }
    }

    fn calculate_mrr(retrieved: &[String], relevant: &[String]) -> f64 {
        let relevant_set: std::collections::HashSet<_> = relevant.iter().collect();
        
        for (i, item) in retrieved.iter().enumerate() {
            if relevant_set.contains(item) {
                return 1.0 / (i + 1) as f64;
            }
        }
        0.0
    }

    fn calculate_ndcg(retrieved: &[String], relevant: &[String]) -> f64 {
        let relevant_set: std::collections::HashSet<_> = relevant.iter().collect();
        
        let mut dcg = 0.0;
        for (i, item) in retrieved.iter().enumerate() {
            if relevant_set.contains(item) {
                dcg += 1.0 / ((i + 2) as f64).log2();
            }
        }

        let mut idcg = 0.0;
        for i in 0..relevant.len().min(retrieved.len()) {
            idcg += 1.0 / ((i + 2) as f64).log2();
        }

        if idcg > 0.0 {
            dcg / idcg
        } else {
            0.0
        }
    }
}

impl Default for RetrievalMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Answer quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerMetrics {
    pub bleu_score: f64,
    pub rouge_l: f64,
    pub exact_match: bool,
    pub semantic_similarity: f64,
    pub intent_coverage: f64,
    pub answer_length: usize,
}

impl AnswerMetrics {
    pub fn new() -> Self {
        Self {
            bleu_score: 0.0,
            rouge_l: 0.0,
            exact_match: false,
            semantic_similarity: 0.0,
            intent_coverage: 0.0,
            answer_length: 0,
        }
    }

    /// Calculate metrics from generated and gold answers
    pub fn calculate(generated: &str, gold: &str, intents_covered: usize, total_intents: usize) -> Self {
        let exact_match = generated.trim().to_lowercase() == gold.trim().to_lowercase();
        
        let bleu_score = Self::calculate_bleu(generated, gold);
        let rouge_l = Self::calculate_rouge_l(generated, gold);
        let semantic_similarity = Self::calculate_similarity(generated, gold);
        
        let intent_coverage = if total_intents > 0 {
            intents_covered as f64 / total_intents as f64
        } else {
            0.0
        };

        Self {
            bleu_score,
            rouge_l,
            exact_match,
            semantic_similarity,
            intent_coverage,
            answer_length: generated.len(),
        }
    }

    fn calculate_bleu(generated: &str, gold: &str) -> f64 {
        let gen_tokens: Vec<&str> = generated.split_whitespace().collect();
        let gold_tokens: Vec<&str> = gold.split_whitespace().collect();

        if gen_tokens.is_empty() || gold_tokens.is_empty() {
            return 0.0;
        }

        let gold_set: std::collections::HashSet<_> = gold_tokens.iter().collect();
        let matches = gen_tokens.iter().filter(|t| gold_set.contains(t)).count();

        matches as f64 / gen_tokens.len() as f64
    }

    fn calculate_rouge_l(generated: &str, gold: &str) -> f64 {
        let gen_tokens: Vec<&str> = generated.split_whitespace().collect();
        let gold_tokens: Vec<&str> = gold.split_whitespace().collect();

        if gen_tokens.is_empty() || gold_tokens.is_empty() {
            return 0.0;
        }

        // Simplified LCS-based ROUGE-L
        let lcs_length = Self::lcs_length(&gen_tokens, &gold_tokens);
        let precision = lcs_length as f64 / gen_tokens.len() as f64;
        let recall = lcs_length as f64 / gold_tokens.len() as f64;

        if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        }
    }

    fn lcs_length(a: &[&str], b: &[&str]) -> usize {
        let m = a.len();
        let n = b.len();
        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if a[i - 1] == b[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        dp[m][n]
    }

    fn calculate_similarity(generated: &str, gold: &str) -> f64 {
        // Simple Jaccard similarity
        let gen_set: std::collections::HashSet<_> = 
            generated.split_whitespace().collect();
        let gold_set: std::collections::HashSet<_> = 
            gold.split_whitespace().collect();

        let intersection = gen_set.intersection(&gold_set).count();
        let union = gen_set.union(&gold_set).count();

        if union > 0 {
            intersection as f64 / union as f64
        } else {
            0.0
        }
    }
}

impl Default for AnswerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive MuISQA metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuISQAMetrics {
    pub retrieval: RetrievalMetrics,
    pub answer: AnswerMetrics,
    pub per_intent_metrics: HashMap<String, IntentMetrics>,
    pub overall_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMetrics {
    pub intent: String,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub count: usize,
}

impl MuISQAMetrics {
    pub fn new(retrieval: RetrievalMetrics, answer: AnswerMetrics) -> Self {
        let overall_score = Self::calculate_overall_score(&retrieval, &answer);

        Self {
            retrieval,
            answer,
            per_intent_metrics: HashMap::new(),
            overall_score,
            timestamp: chrono::Utc::now(),
        }
    }

    fn calculate_overall_score(retrieval: &RetrievalMetrics, answer: &AnswerMetrics) -> f64 {
        // Weighted combination of metrics
        let retrieval_score = (retrieval.f1_score + retrieval.ndcg) / 2.0;
        let answer_score = (answer.bleu_score + answer.rouge_l + answer.semantic_similarity) / 3.0;
        
        0.4 * retrieval_score + 0.6 * answer_score
    }

    pub fn add_intent_metrics(&mut self, intent: String, metrics: IntentMetrics) {
        self.per_intent_metrics.insert(intent, metrics);
    }

    /// Get summary statistics
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            overall_score: self.overall_score,
            retrieval_f1: self.retrieval.f1_score,
            answer_quality: (self.answer.bleu_score + self.answer.rouge_l) / 2.0,
            intent_coverage: self.answer.intent_coverage,
            num_intents: self.per_intent_metrics.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub overall_score: f64,
    pub retrieval_f1: f64,
    pub answer_quality: f64,
    pub intent_coverage: f64,
    pub num_intents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_metrics() {
        let retrieved = vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()];
        let relevant = vec!["doc2".to_string(), "doc3".to_string(), "doc4".to_string()];

        let metrics = RetrievalMetrics::calculate(&retrieved, &relevant);

        assert!(metrics.precision > 0.0);
        assert!(metrics.recall > 0.0);
        assert!(metrics.f1_score > 0.0);
    }

    #[test]
    fn test_answer_metrics() {
        let generated = "The answer is 42";
        let gold = "The answer is forty-two";

        let metrics = AnswerMetrics::calculate(generated, gold, 2, 2);

        assert!(metrics.bleu_score > 0.0);
        assert_eq!(metrics.intent_coverage, 1.0);
    }

    #[test]
    fn test_muisqa_metrics() {
        let retrieval = RetrievalMetrics::new();
        let answer = AnswerMetrics::new();

        let metrics = MuISQAMetrics::new(retrieval, answer);

        assert!(metrics.overall_score >= 0.0);
        assert!(metrics.overall_score <= 1.0);
    }
}
