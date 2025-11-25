// examples/advanced_features.rs
//! Demonstrates all advanced features: backend runners, governance, RD computation, and storage

use limit_core::{
    Session, SessionConfig, PythonRunner, LlamaRunner, LargeModelRunner,
    Runner, RDComputation, FGWConfig, TraceId,
};
use limit_storage::{FileStorage, KVStorage};
use limit_orchestration::{Orchestrator, GovernancePolicy, TraceFlagInfo, TraceFlag};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tracing::info!("=== Advanced Features Demo ===\n");

    // 1. Backend Runners Demo
    demo_backend_runners().await?;

    // 2. Governance Policies Demo
    demo_governance_policies().await?;

    // 3. RD Computation Demo
    demo_rd_computation().await?;

    // 4. Storage Backends Demo
    demo_storage_backends().await?;

    tracing::info!("\n=== All demos completed successfully! ===");
    Ok(())
}

async fn demo_backend_runners() -> Result<()> {
    tracing::info!("--- 1. Backend Runners Demo ---");

    let session = Session::new(SessionConfig {
        name: "backend-runners-demo".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    // Python Runner
    let python_runner = PythonRunner::new("python3".to_string())
        .with_venv("/opt/venv".to_string());
    
    let result = python_runner.execute_isolated(
        "print('Hello from Python')",
        session.id,
        TraceId::new(),
    ).await?;
    tracing::info!("Python execution: {}", result.stdout);

    // Llama Runner
    let llama_runner = LlamaRunner::new(
        "/models/llama-7b.gguf".to_string(),
        4096,
    );
    
    let result = llama_runner.execute_isolated(
        "Explain quantum computing",
        session.id,
        TraceId::new(),
    ).await?;
    tracing::info!("Llama execution: {}", result.stdout);

    // Large Model Runner
    let gpt_runner = LargeModelRunner::new(
        "openai".to_string(),
        "gpt-4".to_string(),
    ).with_api_key("sk-...".to_string());
    
    let result = gpt_runner.execute_isolated(
        "Summarize this text",
        session.id,
        TraceId::new(),
    ).await?;
    tracing::info!("GPT-4 execution: {}", result.stdout);

    tracing::info!("✓ Backend runners demo completed\n");
    Ok(())
}

async fn demo_governance_policies() -> Result<()> {
    tracing::info!("--- 2. Governance Policies Demo ---");

    let storage = FileStorage {
        root: "data/governance-demo".into(),
    };

    // Strict policy
    let strict_policy = GovernancePolicy::strict();
    let orchestrator = Orchestrator::new(storage, strict_policy);

    let trace_id = TraceId::new();

    // Flag a trace as jailbreak
    orchestrator.flag_trace(trace_id, TraceFlagInfo {
        flag: TraceFlag::Jailbreak,
        reason: "Detected jailbreak attempt".to_string(),
        timestamp: chrono::Utc::now(),
        severity: 10,
        auto_detected: true,
    }).await?;

    tracing::info!("Flagged trace {} as jailbreak", trace_id);

    // Try to validate merge (should fail)
    match orchestrator.validate_merge(uuid::Uuid::new_v4(), trace_id).await {
        Ok(_) => tracing::warn!("Merge validation passed (unexpected)"),
        Err(e) => tracing::info!("Merge validation failed as expected: {}", e),
    }

    // Get governance stats
    let stats = orchestrator.get_governance_stats().await;
    tracing::info!("Governance stats: {:?}", stats);

    tracing::info!("✓ Governance policies demo completed\n");
    Ok(())
}

async fn demo_rd_computation() -> Result<()> {
    tracing::info!("--- 3. RD Computation Demo ---");

    let config = FGWConfig {
        alpha: 0.5,
        epsilon: 0.01,
        max_iter: 100,
        tol: 1e-6,
    };

    let mut rd_comp = RDComputation::new(config);

    // Simulate refinement steps
    let refinement_steps = vec![
        (1.0, 2.0),   // (distortion, variance)
        (0.8, 2.0),
        (0.5, 2.0),
        (0.3, 2.0),
        (0.1, 2.0),
    ];

    tracing::info!("Computing RD curve for {} refinement steps", refinement_steps.len());
    let series = rd_comp.compute_rd_curve(&refinement_steps);
    
    tracing::info!("RD Series points:");
    for (i, point) in series.points.iter().enumerate() {
        tracing::info!("  Step {}: Rate={:.4}, Distortion={:.4}", 
            i, point.reward, point.difficulty);
    }

    // Find knee point
    if let Some(knee) = rd_comp.find_knee_point() {
        tracing::info!("Knee point detected: Rate={:.4}, Distortion={:.4}", 
            knee.reward, knee.difficulty);
    } else {
        tracing::info!("No knee point detected");
    }

    // Compute FGW distortion
    let source_features = vec![1.0, 2.0, 3.0];
    let target_features = vec![1.1, 2.1, 3.1];
    let source_structure = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    let target_structure = vec![vec![0.0, 1.1], vec![1.1, 0.0]];

    let distortion = rd_comp.compute_fgw_distortion(
        &source_features,
        &target_features,
        &source_structure,
        &target_structure,
    );
    tracing::info!("FGW distortion: {:.6}", distortion);

    tracing::info!("✓ RD computation demo completed\n");
    Ok(())
}

async fn demo_storage_backends() -> Result<()> {
    tracing::info!("--- 4. Storage Backends Demo ---");

    let session_id = uuid::Uuid::new_v4();
    let trace_id = uuid::Uuid::new_v4();

    // File Storage
    tracing::info!("Testing FileStorage...");
    let file_storage = FileStorage {
        root: "data/file-storage-demo".into(),
    };
    file_storage.persist_trace(
        session_id,
        trace_id,
        serde_json::json!({"test": "file_storage"}),
    ).await?;
    tracing::info!("✓ FileStorage test passed");

    // KV Storage (sled)
    tracing::info!("Testing KVStorage...");
    let kv_storage = KVStorage::new("data/kv-storage-demo")?;
    kv_storage.persist_trace(
        session_id,
        trace_id,
        serde_json::json!({"test": "kv_storage"}),
    ).await?;
    tracing::info!("✓ KVStorage test passed");

    // SQLite Storage (if feature enabled)
    #[cfg(feature = "sqlite")]
    {
        tracing::info!("Testing SqliteStorage...");
        let sqlite_storage = limit_storage::SqliteStorage::new("sqlite::memory:").await?;
        sqlite_storage.persist_trace(
            session_id,
            trace_id,
            serde_json::json!({"test": "sqlite_storage"}),
        ).await?;
        tracing::info!("✓ SqliteStorage test passed");
    }

    tracing::info!("✓ Storage backends demo completed\n");
    Ok(())
}
