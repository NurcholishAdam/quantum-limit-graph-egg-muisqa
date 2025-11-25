// crates/limit-muisqa/src/agent.rs
//! MuISQA agent integration with limit-agents

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use limit_core::{Session, SessionId, TraceId};
use limit_storage::Storage;
use limit_orchestration::{Orchestrator, GovernancePolicy, TraceFlagInfo, TraceFlag};
use crate::{Question, QuestionParser, MuISQAMetrics, RetrievalMetrics, AnswerMetrics};

/// MuISQA agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuISQAAgentConfig {
    pub name: String,
    pub max_retrieval_docs: usize,
    pub intent_threshold: f64,
    pub enable_governance: bool,
    pub block_unsafe_intents: bool,
}

impl Default for MuISQAAgentConfig {
    fn default() -> Self {
        Self {
            name: "muisqa-agent".to_string(),
            max_retrieval_docs: 10,
            intent_threshold: 0.3,
            enable_governance: true,
            block_unsafe_intents: true,
        }
    }
}

/// MuISQA agent for multi-intent question answering
pub struct MuISQAAgent<S: Storage> {
    config: MuISQAAgentConfig,
    parser: QuestionParser,
    orchestrator: Orchestrator<S>,
    session: Session,
}

impl<S: Storage> MuISQAAgent<S> {
    pub fn new(
        config: MuISQAAgentConfig,
        storage: S,
        session: Session,
    ) -> Self {
        let policy = if config.enable_governance {
            GovernancePolicy::default()
        } else {
            GovernancePolicy::permissive()
        };

        let orchestrator = Orchestrator::new(storage, policy);

        Self {
            config,
            parser: QuestionParser::new(),
            orchestrator,
            session,
        }
    }

    /// Process a question and generate answer
    pub async fn process_question(&self, question_text: &str) -> Result<MuISQAResponse> {
        let trace_id = TraceId::new();

        // Parse question
        let question = self.parser.parse(question_text);
        tracing::info!(
            "Parsed question with {} intents",
            question.intents.len()
        );

        // Check governance for unsafe intents
        if self.config.block_unsafe_intents {
            self.check_intent_safety(&question, trace_id).await?;
        }

        // Simulate retrieval
        let retrieved_docs = self.retrieve_documents(&question).await?;

        // Simulate answer generation
        let answer = self.generate_answer(&question, &retrieved_docs).await?;

        // Calculate metrics
        let retrieval_metrics = RetrievalMetrics::calculate(
            &retrieved_docs,
            &vec!["doc1".to_string(), "doc2".to_string()], // Mock relevant docs
        );

        let answer_metrics = AnswerMetrics::calculate(
            &answer,
            "Mock gold answer",
            question.intents.len(),
            question.intents.len(),
        );

        let metrics = MuISQAMetrics::new(retrieval_metrics, answer_metrics);

        // Persist metrics
        self.persist_metrics(trace_id, &metrics).await?;

        Ok(MuISQAResponse {
            question: question_text.to_string(),
            answer,
            retrieved_docs,
            intents: question.intents.iter().map(|i| i.intent.as_str().to_string()).collect(),
            metrics,
            trace_id,
        })
    }

    async fn check_intent_safety(&self, question: &Question, trace_id: TraceId) -> Result<()> {
        // Check for potentially unsafe intents
        for intent_type in &question.intents {
            if intent_type.confidence > 0.7 {
                // Flag high-confidence opinion or hypothetical questions
                if matches!(intent_type.intent, crate::dataset::Intent::Opinion | crate::dataset::Intent::Hypothetical) {
                    self.orchestrator.flag_trace(trace_id, TraceFlagInfo {
                        flag: TraceFlag::HighRisk,
                        reason: format!("High-confidence {} intent detected", intent_type.intent.as_str()),
                        timestamp: chrono::Utc::now(),
                        severity: 6,
                        auto_detected: true,
                    }).await?;
                }
            }
        }

        // Validate merge
        self.orchestrator.validate_merge(self.session.id, trace_id).await?;

        Ok(())
    }

    async fn retrieve_documents(&self, question: &Question) -> Result<Vec<String>> {
        // Simulate document retrieval based on keywords
        let mut docs = Vec::new();
        
        for (i, keyword) in question.keywords.iter().enumerate().take(self.config.max_retrieval_docs) {
            docs.push(format!("doc_{}_about_{}", i, keyword));
        }

        tracing::debug!("Retrieved {} documents", docs.len());
        Ok(docs)
    }

    async fn generate_answer(&self, question: &Question, docs: &[String]) -> Result<String> {
        // Simulate answer generation
        let primary_intent = question.primary_intent()
            .map(|i| i.intent.as_str())
            .unwrap_or("unknown");

        let answer = format!(
            "Answer to '{}' (primary intent: {}). Based on {} retrieved documents: {}",
            question.text,
            primary_intent,
            docs.len(),
            docs.join(", ")
        );

        Ok(answer)
    }

    async fn persist_metrics(&self, trace_id: TraceId, metrics: &MuISQAMetrics) -> Result<()> {
        let metrics_json = serde_json::to_value(metrics)?;
        
        self.orchestrator.storage
            .persist_trace(self.session.id, trace_id, metrics_json)
            .await?;

        tracing::info!("Persisted metrics for trace {}", trace_id);
        Ok(())
    }

    /// Get session ID
    pub fn session_id(&self) -> SessionId {
        self.session.id
    }

    /// Get governance statistics
    pub async fn governance_stats(&self) -> std::collections::HashMap<String, usize> {
        self.orchestrator.get_governance_stats().await
    }
}

/// Response from MuISQA agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuISQAResponse {
    pub question: String,
    pub answer: String,
    pub retrieved_docs: Vec<String>,
    pub intents: Vec<String>,
    pub metrics: MuISQAMetrics,
    pub trace_id: TraceId,
}

impl MuISQAResponse {
    pub fn summary(&self) -> String {
        format!(
            "Question: {}\nIntents: {}\nAnswer: {}\nScore: {:.3}",
            self.question,
            self.intents.join(", "),
            self.answer,
            self.metrics.overall_score
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use limit_core::SessionConfig;
    use limit_storage::FileStorage;

    #[tokio::test]
    async fn test_muisqa_agent() {
        let config = MuISQAAgentConfig::default();
        let storage = FileStorage {
            root: "data/test-muisqa".into(),
        };
        let session = Session::new(SessionConfig {
            name: "test-session".into(),
            max_concurrency: 4,
            allow_network: false,
        });

        let agent = MuISQAAgent::new(config, storage, session);

        let response = agent
            .process_question("What is the difference between AI and ML?")
            .await
            .unwrap();

        assert!(!response.answer.is_empty());
        assert!(!response.intents.is_empty());
        assert!(response.metrics.overall_score >= 0.0);
    }
}
