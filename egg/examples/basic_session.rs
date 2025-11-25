// examples/basic_session.rs
//! Basic session creation and execution example

use limit_core::{Session, SessionConfig, TraceId, RDPoint};
use limit_storage::FileStorage;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Create a new session
    let session = Session::new(SessionConfig {
        name: "basic-example".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    tracing::info!("Created session: {}", session.id);

    // Create storage
    let storage = FileStorage {
        root: "data/examples".into(),
    };

    // Simulate some work with trace points
    let trace_id = TraceId::new();
    let data = serde_json::json!({
        "task": "example_computation",
        "result": 42,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Persist trace
    storage.persist_trace(session.id, trace_id, data).await?;
    tracing::info!("Persisted trace: {}", trace_id);

    // Create RD series (Reward-Difficulty curve)
    let rd_series = limit_core::RDSeries {
        points: vec![
            RDPoint { reward: 0.5, difficulty: 1.0 },
            RDPoint { reward: 0.7, difficulty: 2.0 },
            RDPoint { reward: 0.9, difficulty: 3.0 },
        ],
    };

    storage.persist_rd_series(session.id, &rd_series).await?;
    tracing::info!("Persisted RD series with {} points", rd_series.points.len());

    tracing::info!("Example completed successfully!");
    Ok(())
}
