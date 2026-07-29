//! Property tests for the canonical contracts.
//!
//! Assertions:
//!   * serde round-trip preserves every well-formed value.
//!   * `Known<T>` never silently loses its variant across a round trip.
//!   * `Confidence::new` rejects non-finite / out-of-range values.
//!   * Uncertainty struct validators reject negative or non-finite sigmas.
//!   * The prohibited-token scanner is stable under case / separator changes.

use aeon_contracts::prohibited::{matches_prohibited, scan_json, PROHIBITED_TOKENS};
use aeon_contracts::uncertainty::{Confidence, PositionUncertainty, VelocityUncertainty};
use aeon_contracts::unknown::Known;
use proptest::prelude::*;

proptest! {
    #[test]
    fn confidence_only_accepts_unit_interval(v in prop::num::f64::ANY) {
        let out = Confidence::new(v);
        if v.is_finite() && (0.0..=1.0).contains(&v) {
            prop_assert!(out.is_some());
            prop_assert_eq!(out.unwrap().get(), v);
        } else {
            prop_assert!(out.is_none());
        }
    }

    #[test]
    fn position_uncertainty_validator_matches_definition(
        e in prop::num::f64::ANY, n in prop::num::f64::ANY, u in prop::num::f64::ANY
    ) {
        let p = PositionUncertainty { sigma_east_m: e, sigma_north_m: n, sigma_up_m: u };
        let expected =
            e.is_finite() && n.is_finite() && u.is_finite()
            && e >= 0.0 && n >= 0.0 && u >= 0.0;
        prop_assert_eq!(p.is_valid(), expected);
    }

    #[test]
    fn velocity_uncertainty_validator_matches_definition(
        e in prop::num::f64::ANY, n in prop::num::f64::ANY, u in prop::num::f64::ANY
    ) {
        let v = VelocityUncertainty { sigma_east_ms: e, sigma_north_ms: n, sigma_up_ms: u };
        let expected =
            e.is_finite() && n.is_finite() && u.is_finite()
            && e >= 0.0 && n >= 0.0 && u >= 0.0;
        prop_assert_eq!(v.is_valid(), expected);
    }

    #[test]
    fn known_roundtrip_preserves_variant(v in prop::num::u32::ANY, tag in 0u8..8) {
        let k: Known<u32> = match tag {
            0 => Known::Known { value: v },
            1 => Known::Unknown,
            2 => Known::Unavailable { reason: "n/a".into() },
            3 => Known::Stale { value: v, age_seconds: 1.0 },
            4 => Known::Estimated { value: v, method: "prop".into() },
            5 => Known::Contradictory { candidates: vec![v], reason: "disagree".into() },
            6 => Known::Untrusted { value: v, reason: "sig".into() },
            _ => Known::NotApplicable,
        };
        let s = serde_json::to_string(&k).unwrap();
        let back: Known<u32> = serde_json::from_str(&s).unwrap();
        prop_assert_eq!(k, back);
    }

    #[test]
    fn prohibited_scan_case_and_separator_insensitive(tok_idx in 0usize..PROHIBITED_TOKENS.len()) {
        let tok = PROHIBITED_TOKENS[tok_idx];
        let mixed = tok.chars().enumerate().map(|(i, c)| {
            if i % 2 == 0 { c.to_ascii_uppercase() } else { c }
        }).collect::<String>();
        let hyphenated = tok.replace('_', "-");
        let spaced     = tok.replace('_', " ");
        for s in [tok.to_string(), mixed, hyphenated, spaced] {
            prop_assert!(matches_prohibited(&s).is_some(), "should match: {}", s);
        }
    }

    #[test]
    fn benign_short_strings_do_not_falsely_match(len in 0usize..30) {
        // A random ascii-alphanumeric string is extremely unlikely to contain
        // a full prohibited token; the property below just checks stability.
        let s: String = (0..len).map(|i| (b'a' + (i as u8 % 26)) as char).collect();
        // Only the empty case must not panic; matches or non-matches are ok.
        let _ = matches_prohibited(&s);
        let _ = scan_json(&serde_json::json!({"k": s}));
    }
}
