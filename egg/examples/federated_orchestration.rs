// examples/federated_orchestration.rs
//! Federated orchestration example with multiple sessions

use limit_core::{Session, SessionConfig, BackendRunner, RunnerKind};
use limit_storage::FileStorage;
use limit_orchestration::{Orchestrator, GovernancePolicy};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Create storage
    let storage = FileStorage {
        root: "data/federated".into(),
    };

    // Create orchestrator with governance policy
    let orchestrator = Orchestrator::new(storage, GovernancePolicy {
        block_unsafe_merge: true,
        require_provenance: true,
    });

    tracing::info!("Created orchestrator with governance policy");

    // Create multiple sessions for federated execution
    let sessions: Vec<Session> = (0..3)
        .map(|i| {
            Session::new(SessionConfig {
                name: format!("federated-session-{}", i),
                max_concurrency: 4,
                allow_network: false,
            })
        })
        .collect();

    tracing::info!("Created {} sessions for federated execution", sessions.len());

    // Create backend runners
    let local_runner = BackendRunner::new(RunnerKind::Local);
    tracing::info!("Created local backend runner");

    // Simulate federated task execution
    for (i, session) in sessions.iter().enumerate() {
        tracing::info!("Executing task on session {}: {}", i, session.id);
        
        // In a real implementation, this would execute actual tasks
        let task_data = serde_json::json!({
            "session_id": session.id.to_string(),
            "task_index": i,
            "backend": "local",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        tracing::info!("Task {} completed: {:?}", i, task_data);
    }

    tracing::info!("Federated orchestration completed successfully!");
    Ok(())
}
