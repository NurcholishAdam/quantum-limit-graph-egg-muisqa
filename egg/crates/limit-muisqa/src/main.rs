// crates/limit-muisqa/src/main.rs
//! MuISQA demonstration and testing

use anyhow::Result;
use limit_core::{Session, SessionConfig};
use limit_storage::FileStorage;
use limit_muisqa::{
    MuISQADataset, MuISQAAgent, MuISQAAgentConfig, QuestionParser,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,limit_muisqa=debug")
        .init();

    tracing::info!("=== MuISQA Demo ===\n");

    // Demo 1: Dataset Management
    demo_dataset().await?;

    // Demo 2: Question Parsing
    demo_question_parsing().await?;

    // Demo 3: Agent Integration
    demo_agent_integration().await?;

    // Demo 4: Governance Integration
    demo_governance().await?;

    tracing::info!("\n=== All demos completed successfully! ===");
    Ok(())
}

async fn demo_dataset() -> Result<()> {
    tracing::info!("--- Demo 1: Dataset Management ---");

    // Create synthetic dataset
    let dataset = MuISQADataset::synthetic(20);
    tracing::info!("Created synthetic dataset: {}", dataset.name);

    // Get statistics
    let stats = dataset.stats();
    tracing::info!("Dataset statistics:");
    tracing::info!("  Total entries: {}", stats.total_entries);
    tracing::info!("  Single-intent: {}", stats.single_intent_entries);
    tracing::info!("  Multi-intent: {}", stats.multi_intent_entries);
    tracing::info!("  Intent distribution:");
    for (intent, count) in &stats.intent_distribution {
        tracing::info!("    {}: {}", intent, count);
    }

    // Filter by intent
    let factual_questions = dataset.filter_by_intent(&limit_muisqa::Intent::Factual);
    tracing::info!("Factual questions: {}", factual_questions.len());

    // Get multi-intent entries
    let multi_intent = dataset.multi_intent_entries();
    tracing::info!("Multi-intent entries: {}", multi_intent.len());

    tracing::info!("✓ Dataset demo completed\n");
    Ok(())
}

async fn demo_question_parsing() -> Result<()> {
    tracing::info!("--- Demo 2: Question Parsing ---");

    let parser = QuestionParser::new();

    let questions = vec![
        "What is the difference between AI and ML?",
        "How does quantum computing work and why is it important?",
        "Compare Python and Rust for systems programming",
        "If we could travel faster than light, what would happen?",
    ];

    for question_text in questions {
        let question = parser.parse(question_text);
        
        tracing::info!("Question: {}", question.text);
        tracing::info!("  Intents ({}):", question.intents.len());
        for intent in &question.intents {
            tracing::info!(
                "    {} (confidence: {:.2})",
                intent.intent.as_str(),
                intent.confidence
            );
        }
        tracing::info!("  Keywords: {}", question.keywords.join(", "));
        tracing::info!("  Entities: {}", question.entities.join(", "));
        tracing::info!("");
    }

    tracing::info!("✓ Question parsing demo completed\n");
    Ok(())
}

async fn demo_agent_integration() -> Result<()> {
    tracing::info!("--- Demo 3: Agent Integration ---");

    // Create session
    let session = Session::new(SessionConfig {
        name: "muisqa-demo".into(),
        max_concurrency: 4,
        allow_network: false,
    });
    tracing::info!("Created session: {}", session.id);

    // Create storage
    let storage = FileStorage {
        root: "data/muisqa-demo".into(),
    };

    // Create agent
    let config = MuISQAAgentConfig {
        name: "demo-agent".to_string(),
        max_retrieval_docs: 5,
        intent_threshold: 0.3,
        enable_governance: true,
        block_unsafe_intents: true,
    };

    let agent = MuISQAAgent::new(config, storage, session);

    // Process questions
    let questions = vec![
        "What are the key differences between supervised and unsupervised learning?",
        "How can we implement a neural network from scratch?",
        "Compare the performance of different sorting algorithms",
    ];

    for question in questions {
        tracing::info!("Processing: {}", question);
        
        let response = agent.process_question(question).await?;
        
        tracing::info!("  Trace ID: {}", response.trace_id);
        tracing::info!("  Intents: {}", response.intents.join(", "));
        tracing::info!("  Retrieved docs: {}", response.retrieved_docs.len());
        tracing::info!("  Overall score: {:.3}", response.metrics.overall_score);
        tracing::info!("  Retrieval F1: {:.3}", response.metrics.retrieval.f1_score);
        tracing::info!("  Answer quality: {:.3}", 
            (response.metrics.answer.bleu_score + response.metrics.answer.rouge_l) / 2.0);
        tracing::info!("");
    }

    // Get governance stats
    let stats = agent.governance_stats().await;
    tracing::info!("Governance statistics:");
    for (key, value) in stats {
        tracing::info!("  {}: {}", key, value);
    }

    tracing::info!("✓ Agent integration demo completed\n");
    Ok(())
}

async fn demo_governance() -> Result<()> {
    tracing::info!("--- Demo 4: Governance Integration ---");

    let session = Session::new(SessionConfig {
        name: "governance-demo".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let storage = FileStorage {
        root: "data/governance-demo".into(),
    };

    let config = MuISQAAgentConfig {
        name: "governance-agent".to_string(),
        max_retrieval_docs: 5,
        intent_threshold: 0.3,
        enable_governance: true,
        block_unsafe_intents: true,
    };

    let agent = MuISQAAgent::new(config, storage, session);

    // Test with potentially unsafe questions
    let risky_questions = vec![
        "What do you think about this political issue?",
        "If we could hack into any system, what would you do?",
        "What is your opinion on this controversial topic?",
    ];

    for question in risky_questions {
        tracing::info!("Processing risky question: {}", question);
        
        match agent.process_question(question).await {
            Ok(response) => {
                tracing::info!("  ✓ Processed (score: {:.3})", response.metrics.overall_score);
            }
            Err(e) => {
                tracing::warn!("  ✗ Blocked by governance: {}", e);
            }
        }
    }

    let stats = agent.governance_stats().await;
    tracing::info!("Final governance statistics:");
    for (key, value) in stats {
        tracing::info!("  {}: {}", key, value);
    }

    tracing::info!("✓ Governance demo completed\n");
    Ok(())
}
