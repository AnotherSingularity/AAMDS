//! Deterministic-replay harness.
//!
//! The harness takes a scenario builder (a closure that returns a fresh
//! scenario), runs the pipeline twice, computes a canonical trace digest
//! over the *stable* fields of each `TrackUpdate` (the fields that are
//! part of the deterministic contract — track_algorithm_version,
//! deterministic_sequence, correlation_rationale, new_state.position,
//! new_state.status, contributing observation count), and asserts the
//! two digests match.
//!
//! Fields that are intentionally non-deterministic (e.g. `TrackId`, which
//! is UUIDv4 per run) are excluded from the digest.

use sha2::{Digest, Sha256};

use crate::pipeline::PipelineOutcome;

pub fn trace_digest(outcome: &PipelineOutcome) -> String {
    let mut h = Sha256::new();
    h.update(outcome.scenario_name.as_bytes());
    h.update(outcome.rejected_count.to_be_bytes());
    for u in &outcome.updates {
        h.update(u.deterministic_sequence.to_be_bytes());
        h.update(u.algorithm_version.as_bytes());
        h.update(format!("{:?}", u.correlation_rationale).as_bytes());
        h.update(format!("{:?}", u.new_state.status).as_bytes());
        h.update(format!("{:?}", u.new_state.freshness_state).as_bytes());
        h.update(format!("{:?}", u.new_state.conflict_state).as_bytes());
        // position, quantised to ~1 cm — floats survive same-run repro
        // without a scale flag on the machine so this quantisation is
        // conservative.
        if let aeon_contracts::unknown::Known::Known { value: p } =
            &u.new_state.kinematic_state.position
        {
            h.update(((p.latitude_deg * 1e7).round() as i64).to_be_bytes());
            h.update(((p.longitude_deg * 1e7).round() as i64).to_be_bytes());
            h.update(((p.altitude_m * 1e3).round() as i64).to_be_bytes());
        }
        h.update(u.contributing_observations.len().to_be_bytes());
    }
    hex::encode(h.finalize())
}
