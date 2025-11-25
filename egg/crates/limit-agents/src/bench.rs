// crates/limit-agents/src/bench.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerendipityTrace {
    pub id: Uuid,
    pub branches: Vec<String>,  // labels for anomaly/defection branches
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub trace: SerendipityTrace,
    pub rd_series: super::super::limit_core::RDSeries, // use re-export in real code
}
