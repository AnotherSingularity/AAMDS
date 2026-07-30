//! Deterministic-replay harness.
//!
//! RC2 correction (audit finding 9):
//!
//! - The digest now includes **track_id** as well as the previously
//!   excluded material fields (confidence, uncertainty magnitude,
//!   classification hypothesis count, rejected-observation count,
//!   integrity state, algorithm version). Track ids became
//!   deterministic in RC2 (see `track_management::DeterministicIdSource`),
//!   so including them no longer defeats reproducibility — it
//!   proves it.
//! - The digest is a canonical byte stream, not a stringified
//!   Debug rendering that can silently change under Rust compiler
//!   version bumps for parts of the state we care about.

use sha2::{Digest, Sha256};

use crate::pipeline::PipelineOutcome;

pub fn trace_digest(outcome: &PipelineOutcome) -> String {
    let mut h = Sha256::new();
    h.update(outcome.scenario_name.as_bytes());
    h.update((outcome.rejected_count as u64).to_be_bytes());
    h.update((outcome.updates.len() as u64).to_be_bytes());
    for u in &outcome.updates {
        // Every field below is material to the replay contract.
        h.update(u.deterministic_sequence.to_be_bytes());
        h.update(u.algorithm_version.as_bytes());
        h.update(u.track_id.0.as_bytes());
        h.update(format!("{:?}", u.correlation_rationale).as_bytes());
        h.update(format!("{:?}", u.new_state.status).as_bytes());
        h.update(format!("{:?}", u.new_state.freshness_state).as_bytes());
        h.update(format!("{:?}", u.new_state.conflict_state).as_bytes());
        // Position quantised to 1 cm — floats survive same-input reruns
        // on the same target without a per-CPU scale flag.
        if let aeon_contracts::unknown::Known::Known { value: p } =
            &u.new_state.kinematic_state.position
        {
            h.update(((p.latitude_deg * 1e7).round() as i64).to_be_bytes());
            h.update(((p.longitude_deg * 1e7).round() as i64).to_be_bytes());
            h.update(((p.altitude_m * 1e3).round() as i64).to_be_bytes());
        }
        // Confidence to 4 decimal places.
        let c = (u.new_state.confidence.get() * 10_000.0).round() as i64;
        h.update(c.to_be_bytes());
        // Uncertainty magnitude (position 2-norm) to 1 cm.
        if let aeon_contracts::unknown::Known::Known { value: pu } =
            &u.new_state.state_uncertainty.position
        {
            let mag =
                (pu.sigma_east_m.powi(2) + pu.sigma_north_m.powi(2) + pu.sigma_up_m.powi(2)).sqrt();
            h.update(((mag * 100.0).round() as i64).to_be_bytes());
        }
        // Classification hypotheses — ordered by label so the digest
        // does not depend on insertion order of the underlying Vec.
        let mut classes: Vec<(&str, i64, u32)> = u
            .new_state
            .classification_hypotheses
            .iter()
            .map(|c| {
                (
                    c.label.as_str(),
                    (c.confidence.get() * 10_000.0).round() as i64,
                    c.supporting_source_count,
                )
            })
            .collect();
        classes.sort_by(|a, b| a.0.cmp(b.0));
        h.update((classes.len() as u32).to_be_bytes());
        for (label, conf, sup) in &classes {
            h.update(label.as_bytes());
            h.update(conf.to_be_bytes());
            h.update(sup.to_be_bytes());
        }
        // Source contributions — ordered by (source_system, sensor).
        let mut sources: Vec<(&str, &str, i64)> = u
            .new_state
            .source_contributions
            .iter()
            .map(|s| {
                (
                    s.source_system.as_str(),
                    s.sensor.as_str(),
                    (s.weight * 1000.0).round() as i64,
                )
            })
            .collect();
        sources.sort();
        h.update((sources.len() as u32).to_be_bytes());
        for (sys, sensor, weight) in &sources {
            h.update(sys.as_bytes());
            h.update(sensor.as_bytes());
            h.update(weight.to_be_bytes());
        }
        // Integrity state and contributing / rejected observation counts.
        h.update(format!("{:?}", u.new_state.integrity).as_bytes());
        h.update((u.contributing_observations.len() as u32).to_be_bytes());
        h.update((u.rejected_observations.len() as u32).to_be_bytes());
    }
    hex::encode(h.finalize())
}
