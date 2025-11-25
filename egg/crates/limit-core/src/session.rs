// crates/limit-core/src/session.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type SessionId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub name: String,
    pub max_concurrency: usize,
    pub allow_network: bool,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub cfg: SessionConfig,
}

impl Session {
    pub fn new(cfg: SessionConfig) -> Self {
        Self { id: Uuid::new_v4(), cfg }
    }
}
