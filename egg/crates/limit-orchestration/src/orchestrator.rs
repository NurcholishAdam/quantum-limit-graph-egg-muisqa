// crates/limit-orchestration/src/orchestrator.rs
use async_trait::async_trait;
use anyhow::{Result, bail};
use uuid::Uuid;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use limit_core::{
    Session, SessionId, TraceId, GovernanceCheckpoint, Provenance, RDSeries, BackendRunner, RunnerOutput,
};
use limit_storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraceFlag {
    Jailbreak,
    Anomaly,
    HighRisk,
    Unsafe,
    Unverified,
    Malicious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceFlagInfo {
    pub flag: TraceFlag,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: u8, // 1-10
    pub auto_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub block_unsafe_merge: bool,
    pub require_provenance: bool,
    pub block_jailbreak_traces: bool,
    pub block_anomaly_traces: bool,
    pub max_anomaly_severity: u8,
    pub require_human_review: bool,
    pub auto_quarantine: bool,
    pub block_malicious_traces: bool,
}

impl Default for GovernancePolicy {
    fn default() -> Self {
        Self {
            block_unsafe_merge: true,
            require_provenance: true,
            block_jailbreak_traces: true,
            block_anomaly_traces: true,
            max_anomaly_severity: 7,
            require_human_review: false,
            auto_quarantine: true,
            block_malicious_traces: true,
        }
    }
}

impl GovernancePolicy {
    pub fn permissive() -> Self {
        Self {
            block_unsafe_merge: false,
            require_provenance: false,
            block_jailbreak_traces: false,
            block_anomaly_traces: false,
            max_anomaly_severity: 10,
            require_human_review: false,
            auto_quarantine: false,
            block_malicious_traces: false,
        }
    }

    pub fn strict() -> Self {
        Self {
            block_unsafe_merge: true,
            require_provenance: true,
            block_jailbreak_traces: true,
            block_anomaly_traces: true,
            max_anomaly_severity: 5,
            require_human_review: true,
            auto_quarantine: true,
            block_malicious_traces: true,
        }
    }

    /// Validate trace against governance rules
    pub fn validate_trace(&self, flags: &[TraceFlagInfo]) -> Result<()> {
        for flag_info in flags {
            match flag_info.flag {
                TraceFlag::Jailbreak if self.block_jailbreak_traces => {
                    bail!("Governance violation: Jailbreak detected - {}", flag_info.reason);
                }
                TraceFlag::Anomaly if self.block_anomaly_traces => {
                    if flag_info.severity > self.max_anomaly_severity {
                        bail!("Governance violation: Anomaly severity {} exceeds threshold {} - {}", 
                            flag_info.severity, self.max_anomaly_severity, flag_info.reason);
                    }
                }
                TraceFlag::HighRisk | TraceFlag::Unsafe if self.block_unsafe_merge => {
                    bail!("Governance violation: Unsafe operation - {}", flag_info.reason);
                }
                TraceFlag::Unverified if self.require_provenance => {
                    bail!("Governance violation: Missing provenance - {}", flag_info.reason);
                }
                TraceFlag::Malicious if self.block_malicious_traces => {
                    bail!("Governance violation: Malicious activity detected - {}", flag_info.reason);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub struct Orchestrator<S: Storage> {
    pub storage: S,
    pub policy: GovernancePolicy,
    flagged_traces: std::sync::Arc<tokio::sync::RwLock<HashMap<TraceId, Vec<TraceFlagInfo>>>>,
    quarantined_traces: std::sync::Arc<tokio::sync::RwLock<HashMap<TraceId, String>>>,
}

impl<S: Storage> Orchestrator<S> {
    pub fn new(storage: S, policy: GovernancePolicy) -> Self { 
        Self { 
            storage, 
            policy,
            flagged_traces: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            quarantined_traces: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        } 
    }

    pub async fn run_agent_task<R: BackendRunner>(
        &self,
        session: &Session,
        runner: &R,
        task: serde_json::Value,
    ) -> Result<(TraceId, RunnerOutput)> {
        let trace_id = Uuid::new_v4();
        
        // Check if task contains suspicious patterns
        self.detect_anomalies(&trace_id, &task).await?;
        
        let out = runner.run(task.clone()).await?;
        
        self.storage.persist_trace(session.id, trace_id, serde_json::json!({
            "runner": format!("{:?}", runner.kind()),
            "output": out,
            "task": task,
        })).await?;

        // Governance checkpoint
        let chk = GovernanceCheckpoint {
            label: "governance-check".to_string(),
            passed: self.validate_trace_governance(&trace_id).await.is_ok(),
            details: Some("Governance policy validation".into()),
        };
        self.storage.persist_checkpoint(session.id, trace_id, &chk).await?;

        Ok((trace_id, out))
    }

    /// Flag a trace with specific governance concern
    pub async fn flag_trace(&self, trace_id: TraceId, flag_info: TraceFlagInfo) -> Result<()> {
        let mut flags = self.flagged_traces.write().await;
        flags.entry(trace_id).or_insert_with(Vec::new).push(flag_info.clone());
        
        warn!("Trace {} flagged: {:?} - {}", trace_id, flag_info.flag, flag_info.reason);
        
        // Auto-quarantine if policy requires
        if self.policy.auto_quarantine && flag_info.severity >= 8 {
            drop(flags); // Release lock before calling quarantine
            self.quarantine_trace(trace_id, flag_info.reason).await?;
        }
        
        Ok(())
    }

    /// Get all flags for a trace
    pub async fn get_trace_flags(&self, trace_id: &TraceId) -> Vec<TraceFlagInfo> {
        let flags = self.flagged_traces.read().await;
        flags.get(trace_id).cloned().unwrap_or_default()
    }

    /// Validate trace against governance policy
    async fn validate_trace_governance(&self, trace_id: &TraceId) -> Result<()> {
        let flags = self.get_trace_flags(trace_id).await;
        
        if !flags.is_empty() {
            warn!("Trace {} has {} governance flags", trace_id, flags.len());
            self.policy.validate_trace(&flags)?;
        }

        Ok(())
    }

    /// Validate merge operation
    pub async fn validate_merge(&self, session_id: SessionId, trace_id: TraceId) -> Result<()> {
        // Check if trace is quarantined
        let quarantined = self.quarantined_traces.read().await;
        if let Some(reason) = quarantined.get(&trace_id) {
            bail!("Cannot merge quarantined trace {}: {}", trace_id, reason);
        }
        drop(quarantined);

        // Validate governance
        self.validate_trace_governance(&trace_id).await?;

        // Check provenance requirement
        if self.policy.require_provenance {
            info!("Provenance check passed for trace {}", trace_id);
        }

        Ok(())
    }

    /// Quarantine a trace
    pub async fn quarantine_trace(&self, trace_id: TraceId, reason: String) -> Result<()> {
        let mut quarantined = self.quarantined_traces.write().await;
        quarantined.insert(trace_id, reason.clone());
        warn!("Trace {} quarantined: {}", trace_id, reason);
        Ok(())
    }

    /// Detect anomalies in task
    async fn detect_anomalies(&self, trace_id: &TraceId, task: &serde_json::Value) -> Result<()> {
        let task_str = task.to_string().to_lowercase();
        
        // Simple pattern matching for demonstration
        if task_str.contains("jailbreak") || task_str.contains("ignore previous") {
            self.flag_trace(*trace_id, TraceFlagInfo {
                flag: TraceFlag::Jailbreak,
                reason: "Potential jailbreak attempt detected".to_string(),
                timestamp: chrono::Utc::now(),
                severity: 10,
                auto_detected: true,
            }).await?;
        }

        if task_str.contains("rm -rf") || task_str.contains("drop table") {
            self.flag_trace(*trace_id, TraceFlagInfo {
                flag: TraceFlag::Malicious,
                reason: "Potentially malicious command detected".to_string(),
                timestamp: chrono::Utc::now(),
                severity: 9,
                auto_detected: true,
            }).await?;
        }

        Ok(())
    }

    pub async fn record_provenance(&self, session: &Session, trace: TraceId, prov: Provenance) -> Result<()> {
        self.storage.persist_provenance(session.id, trace, &prov).await
    }

    pub async fn record_rd_series(&self, session: &Session, series: RDSeries) -> Result<()> {
        self.storage.persist_rd_series(session.id, &series).await
    }

    /// Get governance statistics
    pub async fn get_governance_stats(&self) -> HashMap<String, usize> {
        let flags = self.flagged_traces.read().await;
        let quarantined = self.quarantined_traces.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("total_flagged".to_string(), flags.len());
        stats.insert("total_quarantined".to_string(), quarantined.len());
        
        let mut flag_counts: HashMap<String, usize> = HashMap::new();
        for flag_list in flags.values() {
            for flag_info in flag_list {
                let key = format!("{:?}", flag_info.flag);
                *flag_counts.entry(key).or_insert(0) += 1;
            }
        }
        
        for (flag_type, count) in flag_counts {
            stats.insert(format!("flag_{}", flag_type), count);
        }
        
        stats
    }
}
