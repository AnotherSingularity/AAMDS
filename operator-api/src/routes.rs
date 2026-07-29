//! Read-oriented HTTP routes.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::ApiState;

pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/v1/version", get(version))
        .route("/api/v1/tracks", get(list_tracks))
        .route("/api/v1/tracks/:id", get(get_track))
        .route("/api/v1/health", get(get_health))
        .route("/api/v1/alerts", get(list_alerts))
        .route("/api/v1/alerts/:id/acknowledge", post(ack_alert))
        .route("/api/v1/replay/status", get(replay_status))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct VersionInfo {
    runtime_id: String,
    build_id: String,
    configuration_version: String,
    contracts: Vec<&'static str>,
}

async fn version(State(s): State<ApiState>) -> Json<VersionInfo> {
    Json(VersionInfo {
        runtime_id: s.runtime_id.clone(),
        build_id: s.build_id.clone(),
        configuration_version: s.configuration_version.clone(),
        contracts: vec![
            "observation:1.0", "normalized:1.0", "track:1.0",
            "track_update:1.0", "health:1.0", "alert:1.0",
            "relay_envelope:1.0", "audit:1.0",
        ],
    })
}

async fn list_tracks(State(s): State<ApiState>) -> Json<Vec<aeon_contracts::track::Track>> {
    Json(s.tracks())
}

async fn get_track(
    State(s): State<ApiState>,
    Path(id_str): Path<String>,
) -> Result<Json<aeon_contracts::track::Track>, (StatusCode, String)> {
    let uuid = Uuid::parse_str(&id_str).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let id = aeon_contracts::ids::TrackId(uuid);
    match s.engine.lock().unwrap().track(&id) {
        Some(t) => Ok(Json(t.clone())),
        None    => Err((StatusCode::NOT_FOUND, "not found".into())),
    }
}

async fn get_health(State(s): State<ApiState>) -> Json<aeon_contracts::health::SystemHealth> {
    Json(s.health.lock().unwrap().clone())
}

async fn list_alerts(State(s): State<ApiState>) -> Json<Vec<aeon_contracts::alert::Alert>> {
    Json(s.alerts.lock().unwrap().clone())
}

#[derive(Debug, Deserialize)]
struct AckBody {
    actor: String,
}

async fn ack_alert(
    State(s): State<ApiState>,
    Path(id_str): Path<String>,
    Json(body): Json<AckBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let uuid = Uuid::parse_str(&id_str).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let mut alerts = s.alerts.lock().unwrap();
    let a = alerts.iter_mut().find(|a| a.alert_id.0 == uuid).ok_or((StatusCode::NOT_FOUND, "no such alert".into()))?;
    a.acknowledgment = aeon_contracts::alert::AlertAcknowledgment::Acknowledged;
    Ok(Json(serde_json::json!({"ok": true, "actor": body.actor})))
}

async fn replay_status(State(s): State<ApiState>) -> impl IntoResponse {
    let in_replay = *s.replay_mode.lock().unwrap();
    Json(serde_json::json!({"replay_mode": in_replay}))
}
