//! Operator API binary.

use aeon_operator_api::{build_router, ApiState};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().init();

    let state = ApiState {
        engine: Arc::new(Mutex::new(aeon_track_management::TrackEngine::new(
            aeon_track_management::TrackPolicy::default(),
        ))),
        health: Arc::new(Mutex::new(default_health())),
        alerts: Arc::new(Mutex::new(vec![])),
        build_id: env!("CARGO_PKG_VERSION").into(),
        configuration_version: "cfg-0".into(),
        runtime_id: "aeon-operator-api-1".into(),
        replay_mode: Arc::new(Mutex::new(false)),
    };
    let app = build_router(state);
    let addr = std::env::var("AEON_API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "operator-api listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn default_health() -> aeon_contracts::health::SystemHealth {
    use aeon_contracts::health::*;
    use aeon_contracts::version::health_schema;
    SystemHealth {
        schema_version: health_schema(),
        captured_at: time::OffsetDateTime::now_utc(),
        runtime_state: RuntimeState::Ready,
        adapters: vec![],
        sensor_feeds: vec![],
        storage: StorageHealth::Healthy,
        relay: vec![],
        time_source: TimeSourceHealth::LocalOnly,
        active_model_ids: vec![],
        configuration_version: "cfg-0".into(),
        build_id: env!("CARGO_PKG_VERSION").into(),
        degraded_capabilities: vec![],
    }
}
