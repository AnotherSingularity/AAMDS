//! Integration tests for the operator API surface.

use aeon_contracts::alert::{
    Alert, AlertAcknowledgment, AlertCategory, AlertSeverity, RecommendedOperatorAction,
};
use aeon_contracts::ids::AlertId;
use aeon_contracts::version::alert_schema;
use aeon_operator_api::{build_router, ApiState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

fn state() -> ApiState {
    use aeon_contracts::health::*;
    use aeon_contracts::version::health_schema;
    ApiState {
        engine: Arc::new(Mutex::new(aeon_track_management::TrackEngine::new(
            aeon_track_management::TrackPolicy::default(),
        ))),
        health: Arc::new(Mutex::new(SystemHealth {
            schema_version: health_schema(),
            captured_at: time::OffsetDateTime::UNIX_EPOCH,
            runtime_state: RuntimeState::Ready,
            adapters: vec![],
            sensor_feeds: vec![],
            storage: StorageHealth::Healthy,
            relay: vec![],
            time_source: TimeSourceHealth::LocalOnly,
            active_model_ids: vec![],
            configuration_version: "cfg-0".into(),
            build_id: "b-0".into(),
            degraded_capabilities: vec![],
        })),
        alerts: Arc::new(Mutex::new(vec![])),
        build_id: "b-0".into(),
        configuration_version: "cfg-0".into(),
        runtime_id: "rt-0".into(),
        replay_mode: Arc::new(Mutex::new(false)),
    }
}

async fn body_string(r: axum::http::Response<Body>) -> String {
    let b = axum::body::to_bytes(r.into_body(), 65_536).await.unwrap();
    String::from_utf8(b.to_vec()).unwrap()
}

#[tokio::test]
async fn version_returns_build_and_config_and_contract_list() {
    let app = build_router(state());
    let r = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let s = body_string(r).await;
    assert!(s.contains("track:1.0"));
    assert!(s.contains("relay_envelope:1.0"));
    assert!(s.contains("cfg-0"));
}

#[tokio::test]
async fn tracks_endpoint_returns_empty_when_engine_empty() {
    let app = build_router(state());
    let r = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tracks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let s = body_string(r).await;
    assert_eq!(s, "[]");
}

#[tokio::test]
async fn get_track_404_when_missing() {
    let app = build_router(state());
    let r = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tracks/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn alert_acknowledgment_flips_state() {
    let s = state();
    let alert_id = AlertId::new();
    s.alerts.lock().unwrap().push(Alert {
        schema_version: alert_schema(),
        alert_id,
        severity: AlertSeverity::Warning,
        category: AlertCategory::Other,
        affected_component: "test".into(),
        human_summary: "test".into(),
        machine_reason: "test".into(),
        first_occurrence: time::OffsetDateTime::UNIX_EPOCH,
        last_occurrence: time::OffsetDateTime::UNIX_EPOCH,
        occurrence_count: 1,
        acknowledgment: AlertAcknowledgment::Unacknowledged,
        recommended_action: RecommendedOperatorAction::Acknowledge,
    });
    let app = build_router(s.clone());
    let uri = format!("/api/v1/alerts/{}/acknowledge", alert_id.0);
    let r = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"actor":"tester"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        s.alerts.lock().unwrap()[0].acknowledgment,
        AlertAcknowledgment::Acknowledged
    );
}

#[tokio::test]
async fn replay_status_defaults_false() {
    let app = build_router(state());
    let r = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/replay/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let s = body_string(r).await;
    assert!(s.contains("\"replay_mode\":false"));
}
