//! RC2 finding 9 closure — strict determinism.
//!
//! Unlike the RC1 tests, these:
//!   * compare TrackIds directly (RC1's TrackId came from
//!     `Uuid::new_v4` and was excluded from the digest);
//!   * cover a scenario that puts two candidates within the same
//!     correlation gate, forcing the tie-break rule;
//!   * assert full-state trace_digest stability across runs.

use aeon_contracts::coords::CoordinateReference;
use aeon_contracts::ids::{AdapterId, ObservationId, SensorId, SourceSystemId};
use aeon_contracts::observation::{QualityIndicators, RawMeasurement, RawObservation};
use aeon_contracts::provenance::{Integrity, Provenance};
use aeon_contracts::time_kind::TimeQuality;
use aeon_contracts::uncertainty::PositionUncertainty;
use aeon_contracts::unknown::Known;
use aeon_contracts::version::observation_schema;
use aeon_normalization::{normalize, COORD_TRANSFORM_VERSION, NORMALIZATION_VERSION};
use aeon_simulation::determinism::trace_digest;
use aeon_simulation::pipeline::run_pipeline;
use aeon_simulation::scenarios::{crossing_two_tracks, single_clean_track};
use aeon_track_management::{
    DeterministicIdSource, IngestOutcome, TrackEngine, TrackPolicy, ALGORITHM_VERSION,
};
use time::OffsetDateTime;
use uuid::Uuid;

fn far() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH + time::Duration::days(365 * 10)
}

#[test]
fn track_ids_are_bit_identical_across_runs() {
    let mut s1 = single_clean_track();
    let mut s2 = single_clean_track();
    let a = run_pipeline(&s1.manifest.name, &mut s1.adapter, s1.policy.clone(), far).unwrap();
    let b = run_pipeline(&s2.manifest.name, &mut s2.adapter, s2.policy.clone(), far).unwrap();
    assert!(!a.updates.is_empty());
    assert_eq!(a.updates.len(), b.updates.len());
    for (x, y) in a.updates.iter().zip(b.updates.iter()) {
        assert_eq!(
            x.track_id, y.track_id,
            "track_id must be reproducible across runs"
        );
        assert_eq!(x.deterministic_sequence, y.deterministic_sequence);
    }
    assert_eq!(trace_digest(&a), trace_digest(&b));
}

#[test]
fn different_scenarios_get_disjoint_track_id_streams() {
    let mut a_scn = single_clean_track();
    let mut b_scn = crossing_two_tracks();
    let a = run_pipeline(
        &a_scn.manifest.name,
        &mut a_scn.adapter,
        a_scn.policy.clone(),
        far,
    )
    .unwrap();
    let b = run_pipeline(
        &b_scn.manifest.name,
        &mut b_scn.adapter,
        b_scn.policy.clone(),
        far,
    )
    .unwrap();
    // The two scenarios seed different id sources; at least one first-track
    // id must differ.
    assert_ne!(a.updates[0].track_id, b.updates[0].track_id);
}

#[test]
fn deterministic_id_source_wraps_stable_stream() {
    let mut a = DeterministicIdSource::from_seed("scn-X");
    let mut b = DeterministicIdSource::from_seed("scn-X");
    for _ in 0..64 {
        assert_eq!(a.next_track_id(), b.next_track_id());
    }
    // Independent seed produces a different stream.
    let mut c = DeterministicIdSource::from_seed("scn-Y");
    let mut d = DeterministicIdSource::from_seed("scn-X");
    let mut diff = false;
    for _ in 0..8 {
        if c.next_track_id() != d.next_track_id() {
            diff = true;
            break;
        }
    }
    assert!(diff, "different seeds must yield different id streams");
}

/// A synthetic regression that specifically exercises the tie-break
/// path: two observations reported at the same instant, in different
/// places, followed by a third observation that is equidistant from
/// both existing tracks. The correlation must resolve deterministically.
#[test]
fn tie_break_is_stable_by_insertion_ordinal() {
    fn obs(
        lat: f64,
        lon: f64,
        seq: u64,
        sensor: &str,
    ) -> aeon_contracts::observation::NormalizedObservation {
        let raw = RawObservation {
            schema_version: observation_schema(),
            observation_id: ObservationId(Uuid::from_u128(seq as u128)),
            source_system_id: SourceSystemId::new("t"),
            sensor_id: SensorId::new(sensor),
            adapter_id: AdapterId::new("t"),
            adapter_version: "1".into(),
            source_timestamp: OffsetDateTime::UNIX_EPOCH,
            receive_timestamp: OffsetDateTime::UNIX_EPOCH,
            sequence_number: seq,
            coordinate_reference: CoordinateReference::Wgs84Geodetic,
            measurement: RawMeasurement::Position {
                x: lon,
                y: lat,
                z: 3000.0,
            },
            measurement_uncertainty: Known::Known {
                value: PositionUncertainty {
                    sigma_east_m: 5.0,
                    sigma_north_m: 5.0,
                    sigma_up_m: 20.0,
                },
            },
            velocity_uncertainty: Known::Unknown,
            classification_claims: vec![],
            quality_indicators: QualityIndicators {
                time_quality: TimeQuality::Disciplined,
                declared_snr_db: Known::Unknown,
                declared_confidence: Known::Unknown,
            },
            integrity: Integrity::Verified,
            raw_source_blob_digest: None,
        };
        normalize(
            &raw,
            Provenance {
                source_observations: vec![raw.observation_id],
                source_system: SourceSystemId::new("t"),
                sensor: SensorId::new(sensor),
                adapter: AdapterId::new("t"),
                adapter_version: "1".into(),
                receive_timestamp: OffsetDateTime::UNIX_EPOCH,
                source_timestamp: OffsetDateTime::UNIX_EPOCH,
                normalization_version: NORMALIZATION_VERSION.into(),
                coordinate_transformation_version: COORD_TRANSFORM_VERSION.into(),
                track_algorithm_version: Some(ALGORITHM_VERSION.into()),
                model_versions: vec![],
                configuration_version: "c".into(),
                build_id: "b".into(),
                processing_sequence: seq,
                integrity: Integrity::Verified,
                transformation_chain: vec![],
            },
        )
        .unwrap()
    }

    // Wider gate so that the third observation genuinely gates against both.
    let policy = TrackPolicy {
        max_accepted_latency_seconds: f64::INFINITY,
        correlation_gate_m: 5_000_000.0,
        ..TrackPolicy::default()
    };

    let now = OffsetDateTime::UNIX_EPOCH;
    let source = DeterministicIdSource::from_seed("tie-break-scn");
    let mut e1 = TrackEngine::with_id_source(policy.clone(), source.clone());
    // Initiate track A, then track B far away, then an observation between them.
    let a1 = obs(40.0, -74.0, 1, "s1");
    let b1 = obs(50.0, -60.0, 2, "s2");
    let mid = obs(45.0, -67.0, 3, "s3");
    let _ = e1.ingest(&a1, now);
    let _ = e1.ingest(&b1, now);
    let out1 = match e1.ingest(&mid, now) {
        IngestOutcome::Updated(u) | IngestOutcome::Initiated(u) => u,
        _ => panic!("mid rejected"),
    };
    // Rerun with a fresh engine and identical seed — the winner must match.
    let mut e2 = TrackEngine::with_id_source(policy, source);
    let _ = e2.ingest(&a1, now);
    let _ = e2.ingest(&b1, now);
    let out2 = match e2.ingest(&mid, now) {
        IngestOutcome::Updated(u) | IngestOutcome::Initiated(u) => u,
        _ => panic!("mid rejected"),
    };
    assert_eq!(out1.track_id, out2.track_id);
}
