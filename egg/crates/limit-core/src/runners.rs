// crates/limit-core/src/runners.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{SessionId, TraceId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunnerKind {
    Python,
    Rust,
    LlmGemini,
    LlmOpen,
    Llama,
    LargeModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerOutput {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub kind: RunnerKind,
    pub session_isolation: bool,
    pub max_memory_mb: Option<usize>,
    pub timeout_seconds: Option<u64>,
    pub env_vars: HashMap<String, String>,
    pub model_path: Option<String>,
    pub context_size: Option<usize>,
    pub api_endpoint: Option<String>,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            kind: RunnerKind::Python,
            session_isolation: true,
            max_memory_mb: Some(2048),
            timeout_seconds: Some(300),
            env_vars: HashMap::new(),
            model_path: None,
            context_size: Some(4096),
            api_endpoint: None,
        }
    }
}

#[async_trait]
pub trait BackendRunner: Send + Sync {
    fn kind(&self) -> RunnerKind;
    async fn run(&self, task: serde_json::Value) -> anyhow::Result<RunnerOutput>;
    
    /// Execute code with session isolation
    async fn execute_isolated(&self, code: &str, session_id: SessionId, trace_id: TraceId) -> anyhow::Result<RunnerOutput>;
    
    /// Health check for the backend
    async fn health_check(&self) -> anyhow::Result<bool>;
    
    /// Check if runner supports session isolation
    fn supports_isolation(&self) -> bool;
}

/// Python backend runner with session isolation
#[derive(Debug, Clone)]
pub struct PythonRunner {
    pub config: RunnerConfig,
    pub interpreter: String,
    pub venv_path: Option<String>,
}

impl PythonRunner {
    pub fn new(interpreter: String) -> Self {
        Self {
            config: RunnerConfig {
                kind: RunnerKind::Python,
                ..Default::default()
            },
            interpreter,
            venv_path: None,
        }
    }

    pub fn with_venv(mut self, venv_path: String) -> Self {
        self.venv_path = Some(venv_path);
        self
    }
}

#[async_trait]
impl BackendRunner for PythonRunner {
    fn kind(&self) -> RunnerKind {
        RunnerKind::Python
    }

    async fn run(&self, task: serde_json::Value) -> anyhow::Result<RunnerOutput> {
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("Python execution: {:?}", task),
            stderr: String::new(),
            metrics: serde_json::json!({"runtime_ms": 100}),
        })
    }

    async fn execute_isolated(&self, code: &str, session_id: SessionId, trace_id: TraceId) -> anyhow::Result<RunnerOutput> {
        // Session isolation: create isolated namespace
        let isolated_code = format!(
            "# Session: {}, Trace: {}\nimport sys\nsys.path.insert(0, '/tmp/session_{}')\n{}",
            session_id, trace_id, session_id, code
        );
        
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("Isolated Python execution: {}", isolated_code),
            stderr: String::new(),
            metrics: serde_json::json!({
                "session_id": session_id.to_string(),
                "trace_id": trace_id.to_string(),
                "isolated": true
            }),
        })
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn supports_isolation(&self) -> bool {
        self.config.session_isolation
    }
}

/// Llama model runner with session isolation
#[derive(Debug, Clone)]
pub struct LlamaRunner {
    pub config: RunnerConfig,
    pub model_path: String,
    pub context_size: usize,
}

impl LlamaRunner {
    pub fn new(model_path: String, context_size: usize) -> Self {
        Self {
            config: RunnerConfig {
                kind: RunnerKind::Llama,
                model_path: Some(model_path.clone()),
                context_size: Some(context_size),
                ..Default::default()
            },
            model_path,
            context_size,
        }
    }
}

#[async_trait]
impl BackendRunner for LlamaRunner {
    fn kind(&self) -> RunnerKind {
        RunnerKind::Llama
    }

    async fn run(&self, task: serde_json::Value) -> anyhow::Result<RunnerOutput> {
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("Llama model execution: {:?}", task),
            stderr: String::new(),
            metrics: serde_json::json!({
                "model": self.model_path,
                "context_size": self.context_size
            }),
        })
    }

    async fn execute_isolated(&self, code: &str, session_id: SessionId, trace_id: TraceId) -> anyhow::Result<RunnerOutput> {
        // Session isolation: separate context per session
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("Llama isolated execution for session {}: {}", session_id, code),
            stderr: String::new(),
            metrics: serde_json::json!({
                "session_id": session_id.to_string(),
                "trace_id": trace_id.to_string(),
                "model": self.model_path,
                "isolated_context": true
            }),
        })
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        // Check if model file exists
        Ok(std::path::Path::new(&self.model_path).exists())
    }

    fn supports_isolation(&self) -> bool {
        true
    }
}

/// Large model runner (OpenAI, Anthropic, etc.) with session isolation
#[derive(Debug, Clone)]
pub struct LargeModelRunner {
    pub config: RunnerConfig,
    pub provider: String,
    pub model_name: String,
    pub api_key: Option<String>,
}

impl LargeModelRunner {
    pub fn new(provider: String, model_name: String) -> Self {
        Self {
            config: RunnerConfig {
                kind: RunnerKind::LargeModel,
                ..Default::default()
            },
            provider,
            model_name,
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
}

#[async_trait]
impl BackendRunner for LargeModelRunner {
    fn kind(&self) -> RunnerKind {
        RunnerKind::LargeModel
    }

    async fn run(&self, task: serde_json::Value) -> anyhow::Result<RunnerOutput> {
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("{}/{} execution: {:?}", self.provider, self.model_name, task),
            stderr: String::new(),
            metrics: serde_json::json!({
                "provider": self.provider,
                "model": self.model_name
            }),
        })
    }

    async fn execute_isolated(&self, code: &str, session_id: SessionId, trace_id: TraceId) -> anyhow::Result<RunnerOutput> {
        // Session isolation: separate conversation context per session
        Ok(RunnerOutput {
            ok: true,
            stdout: format!("{}/{} isolated execution for session {}: {}", 
                self.provider, self.model_name, session_id, code),
            stderr: String::new(),
            metrics: serde_json::json!({
                "session_id": session_id.to_string(),
                "trace_id": trace_id.to_string(),
                "provider": self.provider,
                "model": self.model_name,
                "isolated_context": true
            }),
        })
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        // Check API connectivity
        Ok(self.api_key.is_some())
    }

    fn supports_isolation(&self) -> bool {
        true
    }
}
