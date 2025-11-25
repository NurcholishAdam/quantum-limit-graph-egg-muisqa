# limit-muisqa: Multi-Intent Information Seeking Question Answering

Multi-Intent Information Seeking Question Answering (MuISQA) integration for the Quantum LIMIT-Graph egg architecture.

## Overview

MuISQA extends the egg folder with comprehensive support for multi-intent question answering, including:

- **Dataset Management**: Load, create, and manage MuISQA datasets
- **Question Parsing**: Extract intents, keywords, and entities from questions
- **Metrics Tracking**: Comprehensive evaluation metrics for retrieval and answer quality
- **Agent Integration**: Seamless integration with limit-agents for QA workflows
- **Governance**: Safety checks and intent-based governance policies

## Features

### 1. Dataset Management

```rust
use limit_muisqa::{MuISQADataset, MuISQAEntry, Intent};

// Create dataset
let mut dataset = MuISQADataset::new("my-dataset".to_string());

// Add entries
dataset.add_entry(
    MuISQAEntry::new(
        "q1".to_string(),
        "What is AI and how does it work?".to_string(),
        vec![Intent::Factual, Intent::Explanation],
    )
);

// Get statistics
let stats = dataset.stats();
println!("Total: {}, Multi-intent: {}", 
    stats.total_entries, stats.multi_intent_entries);
```

### 2. Question Parsing

```rust
use limit_muisqa::QuestionParser;

let parser = QuestionParser::new();
let question = parser.parse("What is the difference between AI and ML?");

// Access intents
for intent in &question.intents {
    println!("{}: {:.2}", intent.intent.as_str(), intent.confidence);
}

// Get primary intent
if let Some(primary) = question.primary_intent() {
    println!("Primary: {}", primary.intent.as_str());
}
```

### 3. Metrics Tracking

```rust
use limit_muisqa::{RetrievalMetrics, AnswerMetrics, MuISQAMetrics};

// Retrieval metrics
let retrieval = RetrievalMetrics::calculate(&retrieved_docs, &relevant_docs);
println!("F1: {:.3}, NDCG: {:.3}", retrieval.f1_score, retrieval.ndcg);

// Answer metrics
let answer = AnswerMetrics::calculate(generated, gold, intents_covered, total_intents);
println!("BLEU: {:.3}, ROUGE-L: {:.3}", answer.bleu_score, answer.rouge_l);

// Combined metrics
let metrics = MuISQAMetrics::new(retrieval, answer);
println!("Overall score: {:.3}", metrics.overall_score);
```

### 4. Agent Integration

```rust
use limit_muisqa::{MuISQAAgent, MuISQAAgentConfig};
use limit_core::{Session, SessionConfig};
use limit_storage::FileStorage;

// Create session
let session = Session::new(SessionConfig {
    name: "qa-session".into(),
    max_concurrency: 4,
    allow_network: false,
});

// Create agent
let config = MuISQAAgentConfig::default();
let storage = FileStorage { root: "data".into() };
let agent = MuISQAAgent::new(config, storage, session);

// Process question
let response = agent.process_question("What is quantum computing?").await?;
println!("Answer: {}", response.answer);
println!("Score: {:.3}", response.metrics.overall_score);
```

## Intent Types

MuISQA supports 8 intent types:

| Intent | Description | Example |
|--------|-------------|---------|
| `Factual` | Seeking factual information | "What is the capital of France?" |
| `Comparison` | Comparing entities | "What's the difference between X and Y?" |
| `Explanation` | Seeking explanations | "How does photosynthesis work?" |
| `Procedural` | Step-by-step instructions | "How do I install Python?" |
| `Opinion` | Seeking opinions | "What do you think about...?" |
| `Temporal` | Time-related questions | "When did World War II end?" |
| `Causal` | Cause-effect relationships | "Why does ice float?" |
| `Hypothetical` | Hypothetical scenarios | "What if we could time travel?" |

## Integration with Egg Components

### limit-core Integration

```rust
// Use Session and TraceId for isolation
let session = Session::new(config);
let trace_id = TraceId::new();

// Wrap MuISQA runs with session context
agent.process_question(question).await?;
```

### limit-storage Integration

```rust
// Persist metrics and provenance
storage.persist_trace(session_id, trace_id, metrics_json).await?;

// Support multiple storage backends
let storage = FileStorage { root: "data".into() };
// or
let storage = KVStorage::new("data/kv")?;
```

### limit-orchestration Integration

```rust
// Governance checkpoints for multi-intent retrieval
orchestrator.flag_trace(trace_id, TraceFlagInfo {
    flag: TraceFlag::HighRisk,
    reason: "High-confidence opinion intent".to_string(),
    severity: 6,
    auto_detected: true,
    timestamp: chrono::Utc::now(),
}).await?;

// Block unsafe merges
orchestrator.validate_merge(session_id, trace_id).await?;
```

### limit-agents Integration

```rust
// Agents consume MuISQA questions
let agent = MuISQAAgent::new(config, storage, session);

// Simulate retrieval
let docs = agent.retrieve_documents(&question).await?;

// Generate answers
let answer = agent.generate_answer(&question, &docs).await?;
```

## Metrics

### Retrieval Metrics

- **Precision**: Fraction of retrieved documents that are relevant
- **Recall**: Fraction of relevant documents that are retrieved
- **F1 Score**: Harmonic mean of precision and recall
- **MRR**: Mean Reciprocal Rank
- **NDCG**: Normalized Discounted Cumulative Gain

### Answer Metrics

- **BLEU Score**: N-gram overlap with reference answer
- **ROUGE-L**: Longest common subsequence
- **Exact Match**: Binary exact match indicator
- **Semantic Similarity**: Jaccard similarity
- **Intent Coverage**: Fraction of intents addressed

### Overall Score

Weighted combination:
```
overall_score = 0.4 * retrieval_score + 0.6 * answer_score
```

## Governance

### Safety Checks

```rust
// Configure governance
let config = MuISQAAgentConfig {
    enable_governance: true,
    block_unsafe_intents: true,
    ..Default::default()
};

// Automatic flagging of risky intents
// - Opinion questions (severity 6)
// - Hypothetical questions (severity 6)
```

### Governance Policies

- **Permissive**: No restrictions
- **Default**: Balanced security
- **Strict**: Maximum security with human review

## Examples

### Run the demo

```bash
cargo run --bin muisqa-demo
```

### Run the integration example

```bash
cargo run --example muisqa_integration
```

### Run tests

```bash
cargo test -p limit-muisqa
```

## Architecture

```
┌─────────────────────────────────────────┐
│         MuISQA Agent                    │
│  ┌─────────────────────────────────┐   │
│  │  Question Parser                │   │
│  │  - Intent extraction            │   │
│  │  - Keyword extraction           │   │
│  │  - Entity recognition           │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Retrieval Engine               │   │
│  │  - Document retrieval           │   │
│  │  - Relevance ranking            │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Answer Generator               │   │
│  │  - Multi-intent synthesis       │   │
│  │  - Context integration          │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │  Metrics Tracker                │   │
│  │  - Retrieval metrics            │   │
│  │  - Answer metrics               │   │
│  │  - Intent coverage              │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
         │              │              │
         ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ limit-core   │ │ limit-storage│ │ limit-orch   │
│ - Session    │ │ - Persistence│ │ - Governance │
│ - TraceId    │ │ - Provenance │ │ - Safety     │
└──────────────┘ └──────────────┘ └──────────────┘
```

## Performance

- **Question Parsing**: ~1-5ms per question
- **Intent Extraction**: ~0.5-2ms per question
- **Metrics Calculation**: ~0.1-1ms per evaluation
- **Agent Processing**: ~10-50ms per question (excluding LLM calls)

## Best Practices

1. **Use Session Isolation**: Always wrap MuISQA runs in sessions
2. **Enable Governance**: Use governance for production deployments
3. **Track Metrics**: Persist metrics for analysis and improvement
4. **Handle Multi-Intent**: Design answers to address all detected intents
5. **Validate Intents**: Check intent confidence before processing

## Future Enhancements

- [ ] Neural intent classification
- [ ] Advanced retrieval strategies
- [ ] Multi-hop reasoning
- [ ] Cross-lingual support
- [ ] Real-time learning from feedback

## License

MIT License - See [LICENSE](../../../../LICENSE) for details.
