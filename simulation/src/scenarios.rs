//! Scenario fixtures. Each scenario returns a fully-configured synthetic
//! adapter and a scenario manifest recording software / policy versions.

use aeon_sensor_adapter_sdk::adapters::synthetic::{SyntheticAdapter, SyntheticConfig};
use aeon_track_management::TrackPolicy;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioManifest {
    pub name: String,
    pub description: String,
    pub adapter_version: String,
    pub normalization_version: String,
    pub track_algorithm_version: String,
    pub policy_digest_hex: String,
}

pub struct Scenario {
    pub manifest: ScenarioManifest,
    pub adapter: SyntheticAdapter,
    pub policy: TrackPolicy,
}

fn policy_digest(p: &TrackPolicy) -> String {
    use sha2::{Digest, Sha256};
    let s = format!("{:?}", p);
    hex::encode(Sha256::digest(s.as_bytes()))
}

pub fn single_clean_track() -> Scenario {
    // Simulation clock is fixed, observations are at UNIX_EPOCH; disable
    // the latency guard so all synthetic observations flow through.
    let policy = TrackPolicy {
        max_accepted_latency_seconds: f64::INFINITY,
        ..TrackPolicy::default()
    };
    let manifest = ScenarioManifest {
        name: "single_clean_track".into(),
        description: "One synthetic aircraft flying due north at 100 m/s.".into(),
        adapter_version: env!("CARGO_PKG_VERSION").into(),
        normalization_version: aeon_normalization::NORMALIZATION_VERSION.into(),
        track_algorithm_version: aeon_track_management::ALGORITHM_VERSION.into(),
        policy_digest_hex: policy_digest(&policy),
    };
    let adapter = SyntheticAdapter::new(SyntheticConfig {
        sensor_id: "syn-1".into(),
        start_lat_deg: 40.0,
        start_lon_deg: -74.0,
        start_alt_m: 3000.0,
        v_north_ms: 100.0,
        v_east_ms: 0.0,
        v_up_ms: 0.0,
        max_observations: 20,
        tick_seconds: 1.0,
        start_time: OffsetDateTime::UNIX_EPOCH,
    });
    Scenario { manifest, adapter, policy }
}

pub fn crossing_two_tracks() -> Scenario {
    // A degenerate "crossing" — same adapter but the config runs long enough
    // that the second scenario when re-run separately produces a second
    // distinct track. Used to exercise multi-track determinism.
    let mut s = single_clean_track();
    s.manifest.name = "crossing_two_tracks".into();
    s.manifest.description = "Two independent synthetic aircraft (run in sequence).".into();
    s.adapter = SyntheticAdapter::new(SyntheticConfig {
        sensor_id: "syn-2".into(),
        start_lat_deg: 40.0,
        start_lon_deg: -73.5,
        start_alt_m: 3200.0,
        v_north_ms: -100.0,
        v_east_ms: 0.0,
        v_up_ms: 0.0,
        max_observations: 20,
        tick_seconds: 1.0,
        start_time: OffsetDateTime::UNIX_EPOCH,
    });
    s
}
