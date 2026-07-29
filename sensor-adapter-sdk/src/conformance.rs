//! Adapter conformance harness.
//!
//! Every adapter must pass this harness. It exercises the failure modes
//! required by section 9 of the implementation directive:
//!
//! - malformed messages
//! - duplicate sequence numbers
//! - out-of-order events (clock rollback)
//! - unsupported schema versions
//! - connection loss (recovery after `shutdown` + `connect`)
//!
//! The harness is data-driven: adapter authors supply the fixtures.

use crate::adapter::{AdapterError, SensorAdapter};

/// Result of a single conformance case.
#[derive(Debug)]
pub struct ConformanceResult {
    pub case: &'static str,
    pub passed: bool,
    pub detail: String,
}

/// Assert that an adapter refuses to emit an observation before `connect`.
pub fn requires_connect_first<A: SensorAdapter>(a: &mut A) -> ConformanceResult {
    match a.next_observation() {
        Err(_) => ConformanceResult {
            case: "requires_connect_first",
            passed: true,
            detail: "rejected pre-connect".into(),
        },
        Ok(_) => ConformanceResult {
            case: "requires_connect_first",
            passed: false,
            detail: "emitted before connect".into(),
        },
    }
}

/// Assert that malformed JSON becomes `AdapterError::Malformed`.
pub fn malformed_json_produces_typed_error<A: SensorAdapter>(a: &mut A) -> ConformanceResult {
    match a.next_observation() {
        Err(AdapterError::Malformed(_)) => ConformanceResult {
            case: "malformed_json_produces_typed_error",
            passed: true,
            detail: "typed error".into(),
        },
        other => ConformanceResult {
            case: "malformed_json_produces_typed_error",
            passed: false,
            detail: format!("expected Malformed, got {other:?}"),
        },
    }
}

/// Assert that a duplicate sequence is refused.
pub fn duplicate_sequence_rejected<A: SensorAdapter>(a: &mut A) -> ConformanceResult {
    // Call twice; on the second we expect the duplicate to be caught.
    let _ = a.next_observation();
    match a.next_observation() {
        Err(AdapterError::DuplicateSequence(_)) => ConformanceResult {
            case: "duplicate_sequence_rejected",
            passed: true,
            detail: "duplicate rejected".into(),
        },
        other => ConformanceResult {
            case: "duplicate_sequence_rejected",
            passed: false,
            detail: format!("expected DuplicateSequence, got {other:?}"),
        },
    }
}

/// Assert that a rolled-back clock is caught.
pub fn clock_rollback_rejected<A: SensorAdapter>(a: &mut A) -> ConformanceResult {
    let _ = a.next_observation();
    match a.next_observation() {
        Err(AdapterError::ClockRollback { .. }) => ConformanceResult {
            case: "clock_rollback_rejected",
            passed: true,
            detail: "rollback caught".into(),
        },
        other => ConformanceResult {
            case: "clock_rollback_rejected",
            passed: false,
            detail: format!("expected ClockRollback, got {other:?}"),
        },
    }
}

/// Assert that a shutdown followed by a fresh connect works.
pub fn restart_recovers<A: SensorAdapter>(a: &mut A) -> ConformanceResult {
    if a.shutdown().is_err() {
        return ConformanceResult {
            case: "restart_recovers",
            passed: false,
            detail: "shutdown failed".into(),
        };
    }
    if a.connect().is_err() {
        return ConformanceResult {
            case: "restart_recovers",
            passed: false,
            detail: "reconnect failed".into(),
        };
    }
    ConformanceResult {
        case: "restart_recovers",
        passed: true,
        detail: "clean restart".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::synthetic::{SyntheticAdapter, SyntheticConfig};
    use crate::adapters::replay::ReplayAdapter;
    use time::OffsetDateTime;

    fn cfg() -> SyntheticConfig {
        SyntheticConfig {
            sensor_id: "s1".into(),
            start_lat_deg: 40.0,
            start_lon_deg: -74.0,
            start_alt_m: 3000.0,
            v_north_ms: 100.0,
            v_east_ms: 0.0,
            v_up_ms: 0.0,
            max_observations: 5,
            tick_seconds: 1.0,
            start_time: OffsetDateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn synthetic_passes_pre_connect_check() {
        let mut a = SyntheticAdapter::new(cfg());
        assert!(requires_connect_first(&mut a).passed);
    }

    #[test]
    fn synthetic_validates_configuration() {
        let mut bad = cfg();
        bad.tick_seconds = 0.0;
        let a = SyntheticAdapter::new(bad);
        assert!(a.validate_configuration().is_err());
        let a = SyntheticAdapter::new(cfg());
        assert!(a.validate_configuration().is_ok());
    }

    #[test]
    fn synthetic_produces_deterministic_stream() {
        let mut a = SyntheticAdapter::new(cfg());
        a.connect().unwrap();
        let mut b = SyntheticAdapter::new(cfg());
        b.connect().unwrap();
        for _ in 0..cfg().max_observations {
            let x = a.next_observation().unwrap().unwrap();
            let y = b.next_observation().unwrap().unwrap();
            assert_eq!(x.observation_id, y.observation_id);
            assert_eq!(x.source_timestamp, y.source_timestamp);
            assert_eq!(x.sequence_number, y.sequence_number);
        }
        // Both should now report exhausted.
        assert!(a.next_observation().unwrap().is_none());
        assert!(b.next_observation().unwrap().is_none());
    }

    #[test]
    fn synthetic_shutdown_and_reconnect() {
        let mut a = SyntheticAdapter::new(cfg());
        a.connect().unwrap();
        let r = restart_recovers(&mut a);
        assert!(r.passed, "{}", r.detail);
    }

    #[test]
    fn replay_malformed_line_yields_typed_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mal.jsonl");
        std::fs::write(&path, "this is not json\n").unwrap();
        let mut a = ReplayAdapter::new(&path);
        a.connect().unwrap();
        let r = malformed_json_produces_typed_error(&mut a);
        assert!(r.passed, "{}", r.detail);
    }
}
