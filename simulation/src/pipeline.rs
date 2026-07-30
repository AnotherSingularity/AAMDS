//! End-to-end pipeline: adapter -> normalization -> track management.

use aeon_contracts::ids::{AdapterId, SensorId, SourceSystemId};
use aeon_contracts::provenance::{Integrity, Provenance};
use aeon_contracts::track::TrackUpdate;
use aeon_sensor_adapter_sdk::adapter::SensorAdapter;
use aeon_track_management::{DeterministicIdSource, IngestOutcome, TrackEngine, TrackPolicy};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOutcome {
    pub scenario_name: String,
    pub updates: Vec<TrackUpdate>,
    pub rejected_count: usize,
}

/// Drive an adapter through normalization and the track engine, returning
/// the ordered list of updates and the count of rejected observations.
pub fn run_pipeline<A: SensorAdapter>(
    scenario_name: &str,
    adapter: &mut A,
    policy: TrackPolicy,
    now_supplier: impl Fn() -> OffsetDateTime,
) -> anyhow::Result<PipelineOutcome> {
    adapter.connect().map_err(|e| anyhow::anyhow!("{e}"))?;
    // Deterministic id source seeded from the scenario name so that
    // running the same scenario twice produces bit-identical track
    // ids (RC2 finding 9).
    let mut engine =
        TrackEngine::with_id_source(policy, DeterministicIdSource::from_seed(scenario_name));
    let mut updates = Vec::new();
    let mut rejected = 0usize;
    let mut seq = 0u64;

    loop {
        let obs = match adapter.next_observation() {
            Ok(Some(o)) => o,
            Ok(None) => break,
            Err(_) => {
                rejected += 1;
                continue;
            }
        };
        seq += 1;

        let provenance = Provenance {
            source_observations: vec![obs.observation_id],
            source_system: SourceSystemId::new(obs.source_system_id.as_str()),
            sensor: SensorId::new(obs.sensor_id.as_str()),
            adapter: AdapterId::new(obs.adapter_id.as_str()),
            adapter_version: obs.adapter_version.clone(),
            receive_timestamp: obs.receive_timestamp,
            source_timestamp: obs.source_timestamp,
            normalization_version: aeon_normalization::NORMALIZATION_VERSION.into(),
            coordinate_transformation_version: aeon_normalization::COORD_TRANSFORM_VERSION.into(),
            track_algorithm_version: Some(aeon_track_management::ALGORITHM_VERSION.into()),
            model_versions: vec![],
            configuration_version: "sim-cfg-1".into(),
            build_id: env!("CARGO_PKG_VERSION").into(),
            processing_sequence: seq,
            integrity: Integrity::Verified,
            transformation_chain: vec![],
        };

        let normalized = match aeon_normalization::normalize(&obs, provenance) {
            Ok(n) => n,
            Err(_) => {
                rejected += 1;
                continue;
            }
        };
        match engine.ingest(&normalized, now_supplier()) {
            IngestOutcome::Initiated(u) | IngestOutcome::Updated(u) => updates.push(u),
            IngestOutcome::Rejected { .. } => rejected += 1,
        }
    }

    Ok(PipelineOutcome {
        scenario_name: scenario_name.into(),
        updates,
        rejected_count: rejected,
    })
}
