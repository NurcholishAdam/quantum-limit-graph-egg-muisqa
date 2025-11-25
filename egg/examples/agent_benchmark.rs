// examples/agent_benchmark.rs
//! Agent benchmarking example

use limit_agents::{Agent, AgentConfig, BenchmarkRun, SerendipityTrace};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Create agent
    let agent = Agent::new(AgentConfig {
        name: "benchmark-agent".into(),
        timeout_ms: 5000,
    });

    tracing::info!("Created agent: {}", agent.config.name);

    // Create benchmark run
    let benchmark = BenchmarkRun {
        agent_name: agent.config.name.clone(),
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now() + chrono::Duration::seconds(10),
        total_tasks: 100,
        successful_tasks: 95,
        failed_tasks: 5,
        avg_latency_ms: 42.5,
        p95_latency_ms: 85.0,
        p99_latency_ms: 120.0,
    };

    tracing::info!("Benchmark results:");
    tracing::info!("  Total tasks: {}", benchmark.total_tasks);
    tracing::info!("  Successful: {}", benchmark.successful_tasks);
    tracing::info!("  Failed: {}", benchmark.failed_tasks);
    tracing::info!("  Avg latency: {:.2}ms", benchmark.avg_latency_ms);
    tracing::info!("  P95 latency: {:.2}ms", benchmark.p95_latency_ms);
    tracing::info!("  P99 latency: {:.2}ms", benchmark.p99_latency_ms);

    // Create serendipity trace
    let serendipity = SerendipityTrace {
        discovery_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        context: "Unexpected pattern discovered during benchmark".into(),
        significance_score: 0.85,
    };

    tracing::info!("Serendipity discovery: {}", serendipity.discovery_id);
    tracing::info!("  Significance: {:.2}", serendipity.significance_score);
    tracing::info!("  Context: {}", serendipity.context);

    tracing::info!("Agent benchmark completed successfully!");
    Ok(())
}
