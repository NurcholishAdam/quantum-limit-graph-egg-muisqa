// crates/limit-agents/src/agent.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub objective: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub coverage: f32,
    pub alignment: f32,
    pub distortion: f32,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn config(&self) -> &AgentConfig;
    async fn step(&mut self, ctx: serde_json::Value) -> anyhow::Result<AgentMetrics>;
}
