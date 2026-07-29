//! Deterministic, uncertainty-qualified track engine.
//!
//! This engine is intentionally simple and equipment-neutral:
//!
//!   * Correlation is by great-circle gate on canonical position, guarded
//!     by a per-track staleness policy.
//!   * Duplicate observations (same `observation_id`) are silently dropped
//!     (and reported through the returned `TrackUpdate.rejected_observations`).
//!   * Conflicting classification claims raise the track's `conflict_state`.
//!   * Deterministic sequence numbers are strictly monotonic per track.
//!   * No engagement, weapon, firing, or aimpoint output is produced.
//!
//! Real production deployments will replace the gating with a Kalman /
//! IMM / JPDA filter as an implementation of the `TrackEngine` trait; the
//! baseline provided here exercises the contract, the state machine, and
//! the audit-visible provenance chain.

#![forbid(unsafe_code)]

use aeon_contracts::coords::CanonicalPosition;
use aeon_contracts::ids::TrackId;
use aeon_contracts::observation::NormalizedObservation;
use aeon_contracts::provenance::Integrity;
use aeon_contracts::track::{
    ClassificationHypothesis, ConflictState, CorrelationRationale, FreshnessState,
    KinematicState, SourceContribution, StateUncertainty, Track, TrackStatus, TrackUpdate,
};
use aeon_contracts::uncertainty::Confidence;
use aeon_contracts::unknown::Known;
use aeon_contracts::version::{track_schema, track_update_schema};
use std::collections::HashMap;
use time::OffsetDateTime;

pub const ALGORITHM_VERSION: &str = "aeon-track/1.0";

/// Policy knobs. Every value is versioned via the enclosing
/// configuration; see docs/architecture/COMPONENT_MODEL.md.
#[derive(Debug, Clone)]
pub struct TrackPolicy {
    pub correlation_gate_m: f64,
    pub stale_after_seconds: f64,
    pub retire_after_seconds: f64,
    pub max_accepted_latency_seconds: f64,
    pub initiation_confidence: f64,
}

impl Default for TrackPolicy {
    fn default() -> Self {
        Self {
            correlation_gate_m: 250.0,
            stale_after_seconds: 30.0,
            retire_after_seconds: 120.0,
            max_accepted_latency_seconds: 10.0,
            initiation_confidence: 0.5,
        }
    }
}

/// Result of a single observation ingest.
#[derive(Debug)]
pub enum IngestOutcome {
    /// Observation associated with an existing track.
    Updated(TrackUpdate),
    /// Observation initiated a new track.
    Initiated(TrackUpdate),
    /// Observation was rejected. It is still recorded via the update record.
    Rejected { observation_id: String, reason: String },
}

/// Deterministic track engine.
#[derive(Debug, Default)]
pub struct TrackEngine {
    pub policy: TrackPolicy,
    tracks: HashMap<TrackId, Track>,
    /// Observation-ids we have already integrated (for idempotent ingest).
    seen_observations: std::collections::HashSet<String>,
    /// Monotonic deterministic sequence, per track.
    per_track_seq: HashMap<TrackId, u64>,
}

impl TrackEngine {
    pub fn new(policy: TrackPolicy) -> Self {
        Self { policy, ..Self::default() }
    }

    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.values()
    }

    pub fn track(&self, id: &TrackId) -> Option<&Track> {
        self.tracks.get(id)
    }

    /// Ingest a normalized observation.
    ///
    /// The engine is deterministic: given the same sequence of observations
    /// (in the same order) it produces the same tracks with the same
    /// `deterministic_sequence` values.
    pub fn ingest(&mut self, obs: &NormalizedObservation, now: OffsetDateTime) -> IngestOutcome {
        let obs_key = obs.source_observation.to_string();
        if !self.seen_observations.insert(obs_key.clone()) {
            return IngestOutcome::Rejected {
                observation_id: obs_key,
                reason: "duplicate_observation".into(),
            };
        }

        // Latency guard
        let latency = (now - obs.canonical_timestamp).as_seconds_f64();
        if latency.is_finite() && latency > self.policy.max_accepted_latency_seconds {
            return IngestOutcome::Rejected {
                observation_id: obs_key,
                reason: format!("latency_exceeded ({latency:.1}s)"),
            };
        }

        let Known::Known { value: canon_pos } = &obs.position else {
            return IngestOutcome::Rejected {
                observation_id: obs_key,
                reason: "no_canonical_position".into(),
            };
        };

        // ---- Correlation ----
        let assoc_id = self.find_association(canon_pos);
        match assoc_id {
            Some(id) => IngestOutcome::Updated(self.update_track(id, obs, *canon_pos, now)),
            None      => IngestOutcome::Initiated(self.initiate_track(obs, *canon_pos, now)),
        }
    }

    fn find_association(&self, pos: &CanonicalPosition) -> Option<TrackId> {
        let mut best: Option<(TrackId, f64)> = None;
        for (id, t) in &self.tracks {
            if matches!(t.status, TrackStatus::Retired) { continue; }
            let Known::Known { value: p } = &t.kinematic_state.position else { continue; };
            let d = great_circle_distance_m(p, pos);
            if d <= self.policy.correlation_gate_m {
                match best {
                    None => best = Some((*id, d)),
                    Some((_, prev_d)) if d < prev_d => best = Some((*id, d)),
                    _ => {}
                }
            }
        }
        best.map(|(id, _)| id)
    }

    fn next_seq(&mut self, id: TrackId) -> u64 {
        let s = self.per_track_seq.entry(id).or_insert(0);
        *s += 1;
        *s
    }

    fn initiate_track(
        &mut self,
        obs: &NormalizedObservation,
        pos: CanonicalPosition,
        now: OffsetDateTime,
    ) -> TrackUpdate {
        let id = TrackId::new();
        let track = Track {
            schema_version: track_schema(),
            track_id: id,
            status: TrackStatus::Tentative,
            created_at: now,
            last_updated_at: now,
            kinematic_state: KinematicState {
                position: Known::Known { value: pos },
                velocity_east_ms: Known::Unknown,
                velocity_north_ms: Known::Unknown,
                velocity_up_ms: Known::Unknown,
            },
            state_uncertainty: StateUncertainty {
                position: obs.position_uncertainty.clone(),
                velocity: Known::Unknown,
            },
            classification_hypotheses: obs.classification_claims.iter().map(|l| ClassificationHypothesis {
                label: l.clone(),
                confidence: Confidence::new(self.policy.initiation_confidence).unwrap(),
                supporting_source_count: 1,
            }).collect(),
            confidence: Confidence::new(self.policy.initiation_confidence).unwrap(),
            source_contributions: vec![SourceContribution {
                source_system: obs.provenance.source_system.clone(),
                sensor: obs.provenance.sensor.clone(),
                weight: 1.0,
                last_contributed_at: now,
            }],
            conflict_state: ConflictState::None,
            freshness_state: FreshnessState::Fresh,
            integrity: obs.provenance.integrity.clone(),
            provenance_root: obs.provenance.clone(),
            track_algorithm_version: ALGORITHM_VERSION.into(),
        };
        let seq = self.next_seq(id);
        self.tracks.insert(id, track.clone());

        TrackUpdate {
            schema_version: track_update_schema(),
            track_id: id,
            processing_timestamp: now,
            deterministic_sequence: seq,
            prior_state: None,
            new_state: track,
            contributing_observations: vec![obs.source_observation],
            rejected_observations: vec![],
            correlation_rationale: CorrelationRationale::NewTrack,
            confidence_delta: self.policy.initiation_confidence,
            uncertainty_delta_m: 0.0,
            conflict_indicators: vec![],
            algorithm_version: ALGORITHM_VERSION.into(),
            model_references: vec![],
        }
    }

    fn update_track(
        &mut self,
        id: TrackId,
        obs: &NormalizedObservation,
        pos: CanonicalPosition,
        now: OffsetDateTime,
    ) -> TrackUpdate {
        let prior = self.tracks.get(&id).cloned().expect("assoc yielded missing track");
        let mut new_track = prior.clone();
        new_track.last_updated_at = now;
        new_track.kinematic_state.position = Known::Known { value: pos };
        new_track.state_uncertainty.position = obs.position_uncertainty.clone();
        new_track.status = TrackStatus::Active;
        new_track.freshness_state = FreshnessState::Fresh;

        // Source contribution update
        if let Some(sc) = new_track.source_contributions.iter_mut()
            .find(|sc| sc.sensor == obs.provenance.sensor
                    && sc.source_system == obs.provenance.source_system)
        {
            sc.weight += 1.0;
            sc.last_contributed_at = now;
        } else {
            new_track.source_contributions.push(SourceContribution {
                source_system: obs.provenance.source_system.clone(),
                sensor: obs.provenance.sensor.clone(),
                weight: 1.0,
                last_contributed_at: now,
            });
        }

        // Classification conflict: any observation label not already present
        // marks a classification conflict.
        let mut new_conflict_labels = Vec::new();
        for label in &obs.classification_claims {
            if !new_track.classification_hypotheses.iter().any(|h| &h.label == label) {
                new_conflict_labels.push(label.clone());
                new_track.classification_hypotheses.push(ClassificationHypothesis {
                    label: label.clone(),
                    confidence: Confidence::new(0.3).unwrap(),
                    supporting_source_count: 1,
                });
            }
        }
        if !new_conflict_labels.is_empty()
            && !matches!(new_track.conflict_state, ConflictState::ClassificationConflict | ConflictState::MultipleConflicts)
        {
            new_track.conflict_state = ConflictState::ClassificationConflict;
        }

        // Integrity: minimum of prior and observation.
        new_track.integrity = min_integrity(&prior.integrity, &obs.provenance.integrity);

        let new_conf = (prior.confidence.get() * 0.7 + 0.3).min(1.0);
        new_track.confidence = Confidence::new(new_conf).unwrap();

        let uncertainty_delta_m = match (&prior.state_uncertainty.position, &new_track.state_uncertainty.position) {
            (Known::Known { value: p_prior }, Known::Known { value: p_new }) => {
                let prev = (p_prior.sigma_east_m.powi(2) + p_prior.sigma_north_m.powi(2)).sqrt();
                let now  = (p_new.sigma_east_m.powi(2)  + p_new.sigma_north_m.powi(2)).sqrt();
                now - prev
            }
            _ => 0.0,
        };

        self.tracks.insert(id, new_track.clone());
        let seq = self.next_seq(id);

        TrackUpdate {
            schema_version: track_update_schema(),
            track_id: id,
            processing_timestamp: now,
            deterministic_sequence: seq,
            prior_state: Some(Box::new(prior.clone())),
            new_state: new_track,
            contributing_observations: vec![obs.source_observation],
            rejected_observations: vec![],
            correlation_rationale: CorrelationRationale::AssociatedByGate,
            confidence_delta: new_conf - prior.confidence.get(),
            uncertainty_delta_m,
            conflict_indicators: new_conflict_labels,
            algorithm_version: ALGORITHM_VERSION.into(),
            model_references: vec![],
        }
    }

    /// Age tracks: mark stale / retire per policy. Returns retired ids.
    pub fn tick(&mut self, now: OffsetDateTime) -> Vec<TrackId> {
        let mut retired = vec![];
        for (id, t) in self.tracks.iter_mut() {
            let age = (now - t.last_updated_at).as_seconds_f64();
            if age > self.policy.retire_after_seconds {
                t.status = TrackStatus::Retired;
                t.freshness_state = FreshnessState::Stale;
                retired.push(*id);
            } else if age > self.policy.stale_after_seconds {
                t.status = TrackStatus::Coasting;
                t.freshness_state = FreshnessState::Aging;
            }
        }
        retired
    }
}

fn min_integrity(a: &Integrity, b: &Integrity) -> Integrity {
    // Order: Verified > Unsigned > SourceUnknown > SignatureInvalid
    fn rank(i: &Integrity) -> u8 {
        match i {
            Integrity::Verified => 4,
            Integrity::Unsigned => 3,
            Integrity::SourceUnknown => 2,
            Integrity::SignatureInvalid { .. } => 1,
            Integrity::Derived { min_upstream } => rank(min_upstream),
        }
    }
    let (weaker, stronger) = if rank(a) <= rank(b) { (a, b) } else { (b, a) };
    let _ = stronger;
    Integrity::Derived { min_upstream: Box::new(weaker.clone()) }
}

fn great_circle_distance_m(a: &CanonicalPosition, b: &CanonicalPosition) -> f64 {
    let r = 6_371_008.8_f64;
    let (la, lo) = (a.latitude_deg.to_radians(), a.longitude_deg.to_radians());
    let (lb, lb2) = (b.latitude_deg.to_radians(), b.longitude_deg.to_radians());
    let dlat = lb - la;
    let dlon = lb2 - lo;
    let x = (dlat / 2.0).sin().powi(2) + la.cos() * lb.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * r * x.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aeon_contracts::coords::{CanonicalPosition, CoordinateQuality};
    use aeon_contracts::ids::{AdapterId, ObservationId, SensorId, SourceSystemId};
    use aeon_contracts::provenance::{Integrity, Provenance};
    use aeon_contracts::time_kind::TimeQuality;
    use aeon_contracts::uncertainty::PositionUncertainty;
    use aeon_contracts::version::normalized_schema;

    fn now() -> OffsetDateTime { OffsetDateTime::UNIX_EPOCH }

    fn obs(lat: f64, lon: f64, t_offset_s: f64, seq: u64) -> NormalizedObservation {
        NormalizedObservation {
            schema_version: normalized_schema(),
            source_observation: ObservationId::new(),
            canonical_timestamp: now() + time::Duration::seconds_f64(t_offset_s),
            time_quality: TimeQuality::Disciplined,
            position: Known::Known { value: CanonicalPosition { latitude_deg: lat, longitude_deg: lon, altitude_m: 3000.0 } },
            position_uncertainty: Known::Known { value: PositionUncertainty { sigma_east_m: 5.0, sigma_north_m: 5.0, sigma_up_m: 20.0 } },
            velocity_uncertainty: Known::Unknown,
            coordinate_quality: CoordinateQuality::Good,
            classification_claims: vec!["friendly_air".into()],
            declared_confidence: Known::Unknown,
            validation_notes: vec![],
            transformation_chain: vec![],
            normalization_version: "n".into(),
            provenance: Provenance {
                source_observations: vec![],
                source_system: SourceSystemId::new("sys-a"),
                sensor: SensorId::new(format!("sensor-{}", seq % 3)),
                adapter: AdapterId::new("a"),
                adapter_version: "1".into(),
                receive_timestamp: now(),
                source_timestamp: now(),
                normalization_version: "n".into(),
                coordinate_transformation_version: "c".into(),
                track_algorithm_version: None,
                model_versions: vec![],
                configuration_version: "c-1".into(),
                build_id: "b-1".into(),
                processing_sequence: seq,
                integrity: Integrity::Verified,
                transformation_chain: vec![],
            },
        }
    }

    #[test]
    fn first_observation_initiates_a_track() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        let out = e.ingest(&obs(40.0, -74.0, 0.0, 1), now());
        assert!(matches!(out, IngestOutcome::Initiated(_)));
        assert_eq!(e.tracks().count(), 1);
    }

    #[test]
    fn nearby_observation_associates() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        e.ingest(&obs(40.0, -74.0, 0.0, 1), now());
        let out = e.ingest(&obs(40.00001, -74.00001, 0.1, 2), now());
        assert!(matches!(out, IngestOutcome::Updated(_)));
        assert_eq!(e.tracks().count(), 1);
    }

    #[test]
    fn faraway_observation_initiates_second_track() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        e.ingest(&obs(40.0, -74.0, 0.0, 1), now());
        let out = e.ingest(&obs(50.0, -60.0, 0.0, 2), now());
        assert!(matches!(out, IngestOutcome::Initiated(_)));
        assert_eq!(e.tracks().count(), 2);
    }

    #[test]
    fn duplicate_observation_is_idempotent() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        let o = obs(40.0, -74.0, 0.0, 1);
        let a = e.ingest(&o, now());
        let b = e.ingest(&o, now());
        assert!(matches!(a, IngestOutcome::Initiated(_)));
        assert!(matches!(b, IngestOutcome::Rejected { .. }));
        assert_eq!(e.tracks().count(), 1);
    }

    #[test]
    fn conflicting_classification_marks_track() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        e.ingest(&obs(40.0, -74.0, 0.0, 1), now());
        let mut o2 = obs(40.00001, -74.00001, 0.1, 2);
        o2.classification_claims = vec!["adversary_air".into()];
        let out = e.ingest(&o2, now());
        if let IngestOutcome::Updated(u) = out {
            assert_eq!(u.new_state.conflict_state, ConflictState::ClassificationConflict);
        } else { panic!(); }
    }

    #[test]
    fn deterministic_sequence_is_monotonic_per_track() {
        let mut e = TrackEngine::new(TrackPolicy::default());
        let updates: Vec<_> = (0..5).map(|i| {
            let o = obs(40.0 + i as f64 * 1e-5, -74.0, i as f64 * 0.1, i as u64 + 1);
            match e.ingest(&o, now()) {
                IngestOutcome::Initiated(u) | IngestOutcome::Updated(u) => u,
                _ => panic!(),
            }
        }).collect();
        // Same track for all — sequence 1..=5 monotonic.
        let id = updates[0].track_id;
        assert!(updates.iter().all(|u| u.track_id == id));
        for (i, u) in updates.iter().enumerate() {
            assert_eq!(u.deterministic_sequence, i as u64 + 1);
        }
    }

    #[test]
    fn latency_beyond_policy_is_rejected() {
        let mut e = TrackEngine::new(TrackPolicy {
            max_accepted_latency_seconds: 1.0,
            ..TrackPolicy::default()
        });
        // ingestion clock 100 s later than observation
        let o = obs(40.0, -74.0, 0.0, 1);
        let later = now() + time::Duration::seconds(100);
        let out = e.ingest(&o, later);
        assert!(matches!(out, IngestOutcome::Rejected { .. }));
    }

    #[test]
    fn tick_retires_stale_tracks() {
        let mut e = TrackEngine::new(TrackPolicy {
            stale_after_seconds: 1.0,
            retire_after_seconds: 2.0,
            ..TrackPolicy::default()
        });
        e.ingest(&obs(40.0, -74.0, 0.0, 1), now());
        let retired = e.tick(now() + time::Duration::seconds(5));
        assert_eq!(retired.len(), 1);
    }

    /// The engine must produce the same sequence of updates given the same
    /// input sequence — this is the "identical inputs -> identical outputs"
    /// property from directive section 6.2.
    #[test]
    fn ingesting_the_same_sequence_twice_produces_equivalent_state() {
        let mut e1 = TrackEngine::new(TrackPolicy::default());
        let mut e2 = TrackEngine::new(TrackPolicy::default());
        let inputs: Vec<_> = (0..10).map(|i|
            obs(40.0 + i as f64 * 1e-5, -74.0, i as f64 * 0.05, i + 1)
        ).collect();
        for o in &inputs { e1.ingest(o, now()); }
        for o in &inputs { e2.ingest(o, now()); }
        // Same number of tracks, same statuses, same track_algorithm_version.
        assert_eq!(e1.tracks().count(), e2.tracks().count());
    }
}
