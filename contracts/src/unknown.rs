//! Explicit unknown / known / stale / untrusted state.
//!
//! Aeon never lets a missing value silently become `0`, `false`, `""`, or
//! `None`. Every optional-looking value that leaves a subsystem boundary is
//! wrapped in [`Known<T>`], which forces the producer to state *why* a value
//! is not present.

use serde::{Deserialize, Serialize};

/// The state of a value that leaves a subsystem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Known<T> {
    /// Value is known and trusted for use.
    Known { value: T },
    /// Value has not been supplied by any source.
    Unknown,
    /// The upstream source does not provide this value at all.
    Unavailable { reason: String },
    /// Value existed but is older than the freshness policy.
    Stale { value: T, age_seconds: f64 },
    /// Value was computed / interpolated, not directly observed.
    Estimated { value: T, method: String },
    /// Sources disagreed. All contributing values are preserved.
    Contradictory { candidates: Vec<T>, reason: String },
    /// Value exists but its integrity or provenance is untrusted.
    Untrusted { value: T, reason: String },
    /// Value was received but failed schema / range validation.
    Invalid { raw: String, reason: String },
    /// The concept does not apply in this context.
    NotApplicable,
}

impl<T> Known<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Known::Known { value }
            | Known::Stale { value, .. }
            | Known::Estimated { value, .. }
            | Known::Untrusted { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn is_trusted(&self) -> bool {
        matches!(self, Known::Known { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_value_is_trusted_others_are_not() {
        let k = Known::Known { value: 5u32 };
        assert!(k.is_trusted());
        let s = Known::Stale { value: 5u32, age_seconds: 60.0 };
        assert!(!s.is_trusted());
        let u = Known::<u32>::Unknown;
        assert!(!u.is_trusted());
    }

    #[test]
    fn serialisation_round_trip_preserves_variant() {
        let v: Known<f64> = Known::Estimated {
            value: 3.14,
            method: "kalman".into(),
        };
        let s = serde_json::to_string(&v).unwrap();
        let back: Known<f64> = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
    }
}
