// services/api/src/main.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use uuid::Uuid;

use limit_core::{
    Session, SessionConfig, TraceId, SessionId, Provenance, RDComputation, FGWConfig, RDPoint,
};
use limit_storage::{FileStorage, Storage};
use limit_orchestration::{Orchestrator, GovernancePolicy, TraceFlagInfo, TraceFlag};

type SharedState = Arc<AppState>;

struct AppState {
    orchestrator: RwLock<Orchestrator<FileStorage>>,
    rd_computation: RwLock<RDComputation>,
}

#[derive(Serialize, Deserialize)]
struct CreateTraceRequest {
    session_id: String,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct CreateTraceResponse {
    trace_id: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct AppendProvenanceRequest {
    session_id: String,
    trace_id: String,
    provenance: ProvenanceData,
}

#[derive(Serialize, Deserialize)]
struct ProvenanceData {
    agent: String,
    action: String,
    timestamp: String,
    metadata: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct RDKneeResponse {
    knee_point: Option<RDPoint>,
    total_points: usize,
}

#[derive(Serialize, Deserialize)]
struct ExportArtifactRequest {
    session_id: String,
    format: String, // "json", "csv", "parquet"
}

#[derive(Serialize, Deserialize)]
struct GovernanceStatsResponse {
    stats: std::collections::HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
struct FlagTraceRequest {
    trace_id: String,
    flag_type: String,
    reason: String,
    severity: u8,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,api=debug")
        .init();

    info!("Starting Quantum LIMIT-Graph API server");

    // Initialize storage and orchestrator
    let storage = FileStorage {
        root: "data/api".into(),
    };
    let policy = GovernancePolicy::default();
    let orchestrator = Orchestrator::new(storage, policy);
    
    let rd_computation = RDComputation::new(FGWConfig::default());

    let state = Arc::new(AppState {
        orchestrator: RwLock::new(orchestrator),
        rd_computation: RwLock::new(rd_computation),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/traces", post(create_trace))
        .route("/traces/:trace_id/provenance", post(append_provenance))
        .route("/rd/knee", get(get_rd_knee))
        .route("/artifacts/export", post(export_artifact))
        .route("/governance/stats", get(get_governance_stats))
        .route("/governance/flag", post(flag_trace))
        .route("/sessions", post(create_session))
        .route("/sessions/:session_id", get(get_session))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:8080";
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "service": "quantum-limit-graph-api"
    }))
}

async fn create_trace(
    State(state): State<SharedState>,
    Json(req): Json<CreateTraceRequest>,
) -> Result<Json<CreateTraceResponse>, StatusCode> {
    let session_id = Uuid::parse_str(&req.session_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let trace_id = Uuid::new_v4();

    let orchestrator = state.orchestrator.read().await;
    orchestrator.storage
        .persist_trace(session_id, trace_id, req.data)
        .await
        .map_err(|e| {
            warn!("Failed to persist trace: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Created trace {} for session {}", trace_id, session_id);

    Ok(Json(CreateTraceResponse {
        trace_id: trace_id.to_string(),
        status: "created".to_string(),
    }))
}

async fn append_provenance(
    State(state): State<SharedState>,
    Json(req): Json<AppendProvenanceRequest>,
) -> Result<StatusCode, StatusCode> {
    let session_id = Uuid::parse_str(&req.session_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let trace_id = Uuid::parse_str(&req.trace_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let provenance = Provenance {
        agent: req.provenance.agent,
        action: req.provenance.action,
        timestamp: req.provenance.timestamp,
        metadata: req.provenance.metadata,
    };

    let orchestrator = state.orchestrator.read().await;
    orchestrator.storage
        .persist_provenance(session_id, trace_id, &provenance)
        .await
        .map_err(|e| {
            warn!("Failed to persist provenance: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Appended provenance to trace {} for session {}", trace_id, session_id);

    Ok(StatusCode::CREATED)
}

async fn get_rd_knee(
    State(state): State<SharedState>,
) -> Result<Json<RDKneeResponse>, StatusCode> {
    let rd = state.rd_computation.read().await;
    let knee_point = rd.find_knee_point();
    let total_points = rd.get_series().points.len();

    info!("RD knee detection: {} points, knee found: {}", total_points, knee_point.is_some());

    Ok(Json(RDKneeResponse {
        knee_point,
        total_points,
    }))
}

async fn export_artifact(
    State(_state): State<SharedState>,
    Json(req): Json<ExportArtifactRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Exporting artifacts for session {} in format {}", req.session_id, req.format);

    // Placeholder implementation
    let export_data = serde_json::json!({
        "session_id": req.session_id,
        "format": req.format,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "artifacts": []
    });

    Ok(Json(export_data))
}

async fn get_governance_stats(
    State(state): State<SharedState>,
) -> Result<Json<GovernanceStatsResponse>, StatusCode> {
    let orchestrator = state.orchestrator.read().await;
    let stats = orchestrator.get_governance_stats().await;

    info!("Retrieved governance stats: {} entries", stats.len());

    Ok(Json(GovernanceStatsResponse { stats }))
}

async fn flag_trace(
    State(state): State<SharedState>,
    Json(req): Json<FlagTraceRequest>,
) -> Result<StatusCode, StatusCode> {
    let trace_id = Uuid::parse_str(&req.trace_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let flag = match req.flag_type.as_str() {
        "jailbreak" => TraceFlag::Jailbreak,
        "anomaly" => TraceFlag::Anomaly,
        "high_risk" => TraceFlag::HighRisk,
        "unsafe" => TraceFlag::Unsafe,
        "malicious" => TraceFlag::Malicious,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let flag_info = TraceFlagInfo {
        flag,
        reason: req.reason,
        timestamp: chrono::Utc::now(),
        severity: req.severity,
        auto_detected: false,
    };

    let orchestrator = state.orchestrator.read().await;
    orchestrator.flag_trace(trace_id, flag_info).await
        .map_err(|e| {
            warn!("Failed to flag trace: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Flagged trace {} with type {}", trace_id, req.flag_type);

    Ok(StatusCode::OK)
}

async fn create_session(
    State(_state): State<SharedState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session = Session::new(SessionConfig {
        name: format!("api-session-{}", Uuid::new_v4()),
        max_concurrency: 4,
        allow_network: false,
    });

    info!("Created session: {}", session.id);

    Ok(Json(serde_json::json!({
        "session_id": session.id.to_string(),
        "name": session.config.name,
        "created_at": chrono::Utc::now().to_rfc3339()
    })))
}

async fn get_session(
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _session_id = Uuid::parse_str(&session_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Placeholder implementation
    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "status": "active"
    })))
}
