// crates/limit-muisqa/src/lib.rs
//! Multi-Intent Information Seeking Question Answering (MuISQA) integration
//! 
//! This module provides support for multi-intent question answering with:
//! - Dataset loading and management
//! - Question parsing and intent extraction
//! - Retrieval and answer generation
//! - Comprehensive metrics tracking

pub mod dataset;
pub mod question;
pub mod metrics;
pub mod agent;

pub use dataset::{MuISQADataset, MuISQAEntry, Intent};
pub use question::{Question, QuestionParser, IntentType};
pub use metrics::{MuISQAMetrics, RetrievalMetrics, AnswerMetrics};
pub use agent::{MuISQAAgent, MuISQAAgentConfig};
