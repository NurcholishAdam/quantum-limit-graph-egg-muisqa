// crates/limit-storage/src/storage.rs
use async_trait::async_trait;
use anyhow::Result;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use serde_json::json;
use limit_core::{RDSeries, TraceId, SessionId, Provenance, GovernanceCheckpoint};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn persist_trace(&self, session: SessionId, trace: TraceId, data: serde_json::Value) -> Result<()>;
    async fn persist_rd_series(&self, session: SessionId, series: &RDSeries) -> Result<()>;
    async fn persist_provenance(&self, session: SessionId, trace: TraceId, prov: &Provenance) -> Result<()>;
    async fn persist_checkpoint(&self, session: SessionId, trace: TraceId, chk: &GovernanceCheckpoint) -> Result<()>;
}

pub struct FileStorage { pub root: String }

#[async_trait]
impl Storage for FileStorage {
    async fn persist_trace(&self, session: SessionId, trace: TraceId, data: serde_json::Value) -> Result<()> {
        let p = format!("{}/{}/trace-{}.json", self.root, session, trace);
        fs::create_dir_all(format!("{}/{}", self.root, session)).await?;
        fs::write(&p, serde_json::to_vec(&data)?).await?;
        Ok(())
    }
    async fn persist_rd_series(&self, session: SessionId, series: &RDSeries) -> Result<()> {
        let p = format!("{}/{}/rd-series.json", self.root, session);
        fs::create_dir_all(format!("{}/{}", self.root, session)).await?;
        fs::write(&p, serde_json::to_vec(series)?).await?;
        Ok(())
    }
    async fn persist_provenance(&self, session: SessionId, trace: TraceId, prov: &Provenance) -> Result<()> {
        let p = format!("{}/{}/prov-{}.jsonl", self.root, session, trace);
        fs::create_dir_all(format!("{}/{}", self.root, session)).await?;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&p).await?;
        f.write_all(serde_json::to_string(prov)?.as_bytes()).await?;
        f.write_all(b"\n").await?;
        Ok(())
    }
    async fn persist_checkpoint(&self, session: SessionId, trace: TraceId, chk: &GovernanceCheckpoint) -> Result<()> {
        let p = format!("{}/{}/chk-{}.jsonl", self.root, session, trace);
        fs::create_dir_all(format!("{}/{}", self.root, session)).await?;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&p).await?;
        f.write_all(serde_json::to_string(chk)?.as_bytes()).await?;
        f.write_all(b"\n").await?;
        Ok(())
    }
}
