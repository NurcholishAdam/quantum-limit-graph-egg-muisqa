// tests/integration_test.rs
//! Integration tests for the egg federated orchestration system

use limit_core::{Session, SessionConfig, TraceId, RDPoint, RDSeries};
use limit_storage::FileStorage;
use limit_orchestration::{Orchestrator, GovernancePolicy};
use limit_agents::{Agent, AgentConfig};
use anyhow::Result;

#[tokio::test]
async fn test_session_creation() -> Result<()> {
    let session = Session::new(SessionConfig {
        name: "test-session".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    assert_eq!(session.config.name, "test-session");
    assert_eq!(session.config.max_concurrency, 4);
    assert!(!session.config.allow_network);
    Ok(())
}

#[tokio::test]
async fn test_storage_persistence() -> Result<()> {
    let storage = FileStorage {
        root: "data/test".into(),
    };

    let session = Session::new(SessionConfig {
        name: "storage-test".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let trace_id = TraceId::new();
    let data = serde_json::json!({"test": "data"});

    storage.persist_trace(session.id, trace_id, data).await?;
    Ok(())
}

#[tokio::test]
async fn test_rd_series_persistence() -> Result<()> {
    let storage = FileStorage {
        root: "data/test".into(),
    };

    let session = Session::new(SessionConfig {
        name: "rd-test".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let rd_series = RDSeries {
        points: vec![
            RDPoint { reward: 0.5, difficulty: 1.0 },
            RDPoint { reward: 0.7, difficulty: 2.0 },
        ],
    };

    storage.persist_rd_series(session.id, &rd_series).await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_creation() -> Result<()> {
    let storage = FileStorage {
        root: "data/test".into(),
    };

    let orchestrator = Orchestrator::new(storage, GovernancePolicy {
        block_unsafe_merge: true,
        require_provenance: true,
    });

    assert!(orchestrator.policy.block_unsafe_merge);
    assert!(orchestrator.policy.require_provenance);
    Ok(())
}

#[tokio::test]
async fn test_agent_creation() -> Result<()> {
    let agent = Agent::new(AgentConfig {
        name: "test-agent".into(),
        timeout_ms: 5000,
    });

    assert_eq!(agent.config.name, "test-agent");
    assert_eq!(agent.config.timeout_ms, 5000);
    Ok(())
}

#[tokio::test]
async fn test_multiple_sessions() -> Result<()> {
    let sessions: Vec<Session> = (0..5)
        .map(|i| {
            Session::new(SessionConfig {
                name: format!("session-{}", i),
                max_concurrency: 4,
                allow_network: false,
            })
        })
        .collect();

    assert_eq!(sessions.len(), 5);
    
    // Verify all sessions have unique IDs
    let mut ids: Vec<_> = sessions.iter().map(|s| s.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 5);
    
    Ok(())
}

#[tokio::test]
async fn test_provenance_tracking() -> Result<()> {
    let storage = FileStorage {
        root: "data/test".into(),
    };

    let session = Session::new(SessionConfig {
        name: "provenance-test".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let trace_id = TraceId::new();
    let provenance = limit_core::Provenance {
        trace_id,
        timestamp: chrono::Utc::now(),
        operation: "test_operation".into(),
        actor: "test_actor".into(),
        hash: "test_hash".into(),
    };

    storage.persist_provenance(session.id, trace_id, &provenance).await?;
    Ok(())
}

#[tokio::test]
async fn test_governance_checkpoint() -> Result<()> {
    let storage = FileStorage {
        root: "data/test".into(),
    };

    let session = Session::new(SessionConfig {
        name: "checkpoint-test".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    let trace_id = TraceId::new();
    let checkpoint = limit_core::GovernanceCheckpoint {
        trace_id,
        timestamp: chrono::Utc::now(),
        policy_check: "safety_validation".into(),
        passed: true,
        details: "All safety checks passed".into(),
    };

    storage.persist_checkpoint(session.id, trace_id, &checkpoint).await?;
    Ok(())
}
