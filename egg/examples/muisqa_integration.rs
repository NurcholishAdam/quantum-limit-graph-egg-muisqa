// examples/muisqa_integration.rs
//! Complete MuISQA integration example demonstrating all features

use anyhow::Result;
use limit_core::{Session, SessionConfig, TraceId};
use limit_storage::FileStorage;
use limit_muisqa::{
    MuISQADataset, MuISQAAgent, MuISQAAgentConfig, QuestionParser,
    Intent, MuISQAEntry,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("=== Complete MuISQA Integration Example ===\n");

    // 1. Dataset Creation and Management
    demo_dataset_operations().await?;

    // 2. Question Parsing and Intent Extraction
    demo_question_parsing().await?;

    // 3. Agent-based Question Answering
    demo_agent_qa().await?;

    // 4. Governance and Safety
    demo_governance_integration().await?;

    // 5. Metrics and Evaluation
    demo_metrics_tracking().await?;

    tracing::info!("\n=== Integration example completed! ===");
    Ok(())
}

async fn demo_dataset_operations() -> Result<()> {
    tracing::info!("--- 1. Dataset Operations ---");

    // Create custom dataset
    let mut dataset = MuISQADataset::new("custom-dataset".to_string());

    // Add entries with different intent combinations
    dataset.add_entry(
        MuISQAEntry::new(
            "q1".to_string(),
            "What is machine learning and how does it differ from traditional programming?".to_string(),
            vec![Intent::Factual, Intent::Comparison, Intent::Explanation],
        )
        .with_context("Machine learning is a subset of AI...".to_string())
        .with_answer("Machine learning enables computers to learn from data...".to_string())
    );

    dataset.add_entry(
        MuISQAEntry::new(
            "q2".to_string(),
            "How can I implement a neural network from scratch using Python?".to_string(),
            vec![Intent::Procedural, Intent::Explanation],
        )
        .with_context("Neural networks consist of layers of interconnected nodes...".to_string())
    );

    dataset.add_entry(
        MuISQAEntry::new(
            "q3".to_string(),
            "If quantum computers become mainstream, what impact would they have on cryptography?".to_string(),
            vec![Intent::Hypothetical, Intent::Causal],
        )
    );

    // Get statistics
    let stats = dataset.stats();
    tracing::info!("Dataset: {}", dataset.name);
    tracing::info!("  Total entries: {}", stats.total_entries);
    tracing::info!("  Multi-intent: {}", stats.multi_intent_entries);
    tracing::info!("  Intent distribution:");
    for (intent, count) in &stats.intent_distribution {
        tracing::info!("    {}: {}", intent, count);
    }

    // Filter operations
    let factual = dataset.filter_by_intent(&Intent::Factual);
    tracing::info!("  Factual questions: {}", factual.len());

    let multi_intent = dataset.multi_intent_entries();
    tracing::info!("  Multi-intent questions: {}", multi_intent.len());

    tracing::info!("✓ Dataset operations completed\n");
    Ok(())
}

async fn demo_question_parsing() -> Result<()> {
    tracing::info!("--- 2. Question Parsing ---");

    let parser = QuestionParser::new();

    let test_questions = vec![
        "What are the main differences between supervised and unsupervised learning?",
        "How does backpropagation work in neural networks and why is it important?",
        "Compare the performance of bubble sort versus quicksort",
        "What would happen if we could travel faster than light?",
        "Explain the concept of quantum entanglement",
    ];

    for (i, question_text) in test_questions.iter().enumerate() {
        let question = parser.parse(question_text);
        
        tracing::info!("Question {}: {}", i + 1, question.text);
        tracing::info!("  Multi-intent: {}", question.is_multi_intent());
        
        if let Some(primary) = question.primary_intent() {
            tracing::info!("  Primary intent: {} ({:.2})", 
                primary.intent.as_str(), primary.confidence);
        }

        tracing::info!("  All intents:");
        for intent in &question.intents {
            tracing::info!("    - {} ({:.2})", 
                intent.intent.as_str(), intent.confidence);
        }

        tracing::info!("  Keywords: {}", question.keywords.join(", "));
        tracing::info!("");
    }

    tracing::info!("✓ Question parsing completed\n");
    Ok(())
}

async fn demo_agent_qa() -> Result<()> {
    tracing::info!("--- 3. Agent-based QA ---");

    // Create session with isolation
    let session = Session::new(SessionConfig {
        name: "muisqa-qa-session".into(),
        max_concurrency: 4,
        allow_network: false,
    });
    tracing::info!("Session ID: {}", session.id);

    // Create storage
    let storage = FileStorage {
        root: "data/muisqa-qa".into(),
    };

    // Configure agent
    let config = MuISQAAgentConfig {
        name: "qa-agent".to_string(),
        max_retrieval_docs: 10,
        intent_threshold: 0.3,
        enable_governance: true,
        block_unsafe_intents: true,
    };

    let agent = MuISQAAgent::new(config, storage, session);

    // Process questions
    let questions = vec![
        "What is the difference between AI, ML, and deep learning?",
        "How do transformers work in natural language processing?",
        "Compare Python and Rust for machine learning applications",
    ];

    for question in questions {
        tracing::info!("Processing: {}", question);
        
        let response = agent.process_question(question).await?;
        
        tracing::info!("  Trace: {}", response.trace_id);
        tracing::info!("  Intents: {}", response.intents.join(", "));
        tracing::info!("  Retrieved: {} docs", response.retrieved_docs.len());
        tracing::info!("  Metrics:");
        tracing::info!("    Overall score: {:.3}", response.metrics.overall_score);
        tracing::info!("    Retrieval F1: {:.3}", response.metrics.retrieval.f1_score);
        tracing::info!("    Retrieval NDCG: {:.3}", response.metrics.retrieval.ndcg);
        tracing::info!("    Answer BLEU: {:.3}", response.metrics.answer.bleu_score);
        tracing::info!("    Intent coverage: {:.3}", response.metrics.answer.intent_coverage);
        tracing::info!("");
    }

    tracing::info!("✓ Agent QA completed\n");
    Ok(())
}

async fn demo_governance_integration() -> Result<()> {
    tracing::info!("--- 4. Governance Integration ---");

    let session = Session::new(SessionConfig {
        name: "governance-session".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let storage = FileStorage {
        root: "data/muisqa-governance".into(),
    };

    let config = MuISQAAgentConfig {
        name: "governance-agent".to_string(),
        max_retrieval_docs: 5,
        intent_threshold: 0.3,
        enable_governance: true,
        block_unsafe_intents: true,
    };

    let agent = MuISQAAgent::new(config, storage, session);

    // Test various question types
    let test_cases = vec![
        ("Safe factual", "What is the capital of France?"),
        ("Safe procedural", "How do I sort a list in Python?"),
        ("Risky opinion", "What do you think about this political issue?"),
        ("Risky hypothetical", "If we could hack any system, what would happen?"),
    ];

    let mut passed = 0;
    let mut blocked = 0;

    for (category, question) in test_cases {
        tracing::info!("Testing [{}]: {}", category, question);
        
        match agent.process_question(question).await {
            Ok(response) => {
                passed += 1;
                tracing::info!("  ✓ Passed (score: {:.3})", response.metrics.overall_score);
            }
            Err(e) => {
                blocked += 1;
                tracing::warn!("  ✗ Blocked: {}", e);
            }
        }
    }

    tracing::info!("\nGovernance summary:");
    tracing::info!("  Passed: {}", passed);
    tracing::info!("  Blocked: {}", blocked);

    let stats = agent.governance_stats().await;
    tracing::info!("  Governance stats:");
    for (key, value) in stats {
        tracing::info!("    {}: {}", key, value);
    }

    tracing::info!("✓ Governance integration completed\n");
    Ok(())
}

async fn demo_metrics_tracking() -> Result<()> {
    tracing::info!("--- 5. Metrics Tracking ---");

    let session = Session::new(SessionConfig {
        name: "metrics-session".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let storage = FileStorage {
        root: "data/muisqa-metrics".into(),
    };

    let config = MuISQAAgentConfig::default();
    let agent = MuISQAAgent::new(config, storage, session);

    // Process multiple questions and track metrics
    let questions = vec![
        "What is quantum computing?",
        "How does machine learning work?",
        "Compare SQL and NoSQL databases",
        "Explain the concept of blockchain",
        "What are the benefits of cloud computing?",
    ];

    let mut all_scores = Vec::new();

    for question in questions {
        let response = agent.process_question(question).await?;
        all_scores.push(response.metrics.overall_score);
        
        tracing::info!("Question: {}", question);
        tracing::info!("  Score: {:.3}", response.metrics.overall_score);
    }

    // Calculate aggregate metrics
    let avg_score = all_scores.iter().sum::<f64>() / all_scores.len() as f64;
    let max_score = all_scores.iter().fold(0.0f64, |a, &b| a.max(b));
    let min_score = all_scores.iter().fold(1.0f64, |a, &b| a.min(b));

    tracing::info!("\nAggregate metrics:");
    tracing::info!("  Average score: {:.3}", avg_score);
    tracing::info!("  Max score: {:.3}", max_score);
    tracing::info!("  Min score: {:.3}", min_score);
    tracing::info!("  Questions processed: {}", all_scores.len());

    tracing::info!("✓ Metrics tracking completed\n");
    Ok(())
}
