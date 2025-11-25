// crates/limit-storage/src/db_storage.rs
//! Database storage implementations (SQLite, PostgreSQL)

use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use limit_core::{RDSeries, TraceId, SessionId, Provenance, GovernanceCheckpoint};
use crate::Storage;

#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;

#[cfg(feature = "postgres")]
use sqlx::PgPool;

/// SQLite storage backend
#[cfg(feature = "sqlite")]
pub struct SqliteStorage {
    pool: SqlitePool,
}

#[cfg(feature = "sqlite")]
impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traces (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (session_id, trace_id)
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rd_series (
                session_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS provenance (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[cfg(feature = "sqlite")]
#[async_trait]
impl Storage for SqliteStorage {
    async fn persist_trace(&self, session: SessionId, trace: TraceId, data: Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO traces (session_id, trace_id, data) VALUES (?, ?, ?)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(serde_json::to_string(&data)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_rd_series(&self, session: SessionId, series: &RDSeries) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO rd_series (session_id, data) VALUES (?, ?)"
        )
        .bind(session.to_string())
        .bind(serde_json::to_string(series)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_provenance(&self, session: SessionId, trace: TraceId, prov: &Provenance) -> Result<()> {
        sqlx::query(
            "INSERT INTO provenance (session_id, trace_id, data) VALUES (?, ?, ?)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(serde_json::to_string(prov)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_checkpoint(&self, session: SessionId, trace: TraceId, chk: &GovernanceCheckpoint) -> Result<()> {
        sqlx::query(
            "INSERT INTO checkpoints (session_id, trace_id, data) VALUES (?, ?, ?)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(serde_json::to_string(chk)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// PostgreSQL storage backend
#[cfg(feature = "postgres")]
pub struct PostgresStorage {
    pool: PgPool,
}

#[cfg(feature = "postgres")]
impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traces (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (session_id, trace_id)
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rd_series (
                session_id TEXT PRIMARY KEY,
                data JSONB NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS provenance (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                session_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                data JSONB NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_traces_session ON traces(session_id)")
            .execute(&pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_provenance_session ON provenance(session_id)")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl Storage for PostgresStorage {
    async fn persist_trace(&self, session: SessionId, trace: TraceId, data: Value) -> Result<()> {
        sqlx::query(
            "INSERT INTO traces (session_id, trace_id, data) VALUES ($1, $2, $3)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_rd_series(&self, session: SessionId, series: &RDSeries) -> Result<()> {
        sqlx::query(
            "INSERT INTO rd_series (session_id, data) VALUES ($1, $2) 
             ON CONFLICT (session_id) DO UPDATE SET data = $2, updated_at = CURRENT_TIMESTAMP"
        )
        .bind(session.to_string())
        .bind(serde_json::to_value(series)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_provenance(&self, session: SessionId, trace: TraceId, prov: &Provenance) -> Result<()> {
        sqlx::query(
            "INSERT INTO provenance (session_id, trace_id, data) VALUES ($1, $2, $3)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(serde_json::to_value(prov)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn persist_checkpoint(&self, session: SessionId, trace: TraceId, chk: &GovernanceCheckpoint) -> Result<()> {
        sqlx::query(
            "INSERT INTO checkpoints (session_id, trace_id, data) VALUES ($1, $2, $3)"
        )
        .bind(session.to_string())
        .bind(trace.to_string())
        .bind(serde_json::to_value(chk)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// Key-Value storage using sled (embedded database)
pub struct KVStorage {
    db: sled::Db,
}

impl KVStorage {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    fn make_key(&self, prefix: &str, session: SessionId, trace: Option<TraceId>) -> String {
        if let Some(t) = trace {
            format!("{}:{}:{}", prefix, session, t)
        } else {
            format!("{}:{}", prefix, session)
        }
    }
}

#[async_trait]
impl Storage for KVStorage {
    async fn persist_trace(&self, session: SessionId, trace: TraceId, data: Value) -> Result<()> {
        let key = self.make_key("trace", session, Some(trace));
        let value = serde_json::to_vec(&data)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }

    async fn persist_rd_series(&self, session: SessionId, series: &RDSeries) -> Result<()> {
        let key = self.make_key("rd", session, None);
        let value = serde_json::to_vec(series)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }

    async fn persist_provenance(&self, session: SessionId, trace: TraceId, prov: &Provenance) -> Result<()> {
        let key = self.make_key("prov", session, Some(trace));
        let value = serde_json::to_vec(prov)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }

    async fn persist_checkpoint(&self, session: SessionId, trace: TraceId, chk: &GovernanceCheckpoint) -> Result<()> {
        let key = self.make_key("chk", session, Some(trace));
        let value = serde_json::to_vec(chk)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }
}
