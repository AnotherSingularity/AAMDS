//! Prohibited-concept registry.
//!
//! This module is the *canonical* list of concepts Aeon must not implement,
//! import, or relay. It is consulted by:
//!
//! - the secure-relay outbound validator (runtime),
//! - the `verify-scope-boundary` CI job (static source scan),
//! - contract tests that assert Aeon's public API does not surface any of
//!   these tokens as message types, function names, schema fields, or
//!   enum variants.
//!
//! The scope-boundary scanner explicitly exempts this file, the relay
//! allowlist, the boundary scanner itself, and the docs tree. Anywhere
//! else these tokens appear in a source file, the CI job fails the build.
//!
//! ## Adding a token
//!
//! Adding a token here narrows Aeon's permitted surface. It must never be
//! removed to allow a previously-prohibited capability without an explicit,
//! auditable decision recorded in `docs/architecture/SCOPE_BOUNDARY.md`.

use serde::{Deserialize, Serialize};

/// Canonical prohibited concept tokens.
///
/// Case-insensitive on match. Additional wording variants (hyphenated,
/// spaced) are checked by [`matches_prohibited`].
pub const PROHIBITED_TOKENS: &[&str] = &[
    "weapon_assignment",
    "weapon_recommendation",
    "engagement_ranking",
    "intercept_point",
    "intercept_calculation",
    "firing_solution",
    "fire_solution",
    "fire_control_bus",
    "launch_authorization",
    "launch_recommendation",
    "launch_command",
    "aimpoint_selection",
    "aimpoint",
    "probability_of_kill",
    "pk_optimization",
    "missile_guidance",
    "interceptor_guidance",
    "terminal_guidance",
    "terminal_course_correction",
    "autonomous_engagement",
    "engage_target",
    "engagement_authorization",
    "target_engagement",
    "actuate_weapon",
    "arm_weapon",
    "fire_weapon",
];

/// Return true if `s` (interpreted case-insensitively, with `-`, space, `.`
/// normalised to `_`) contains any prohibited token as a whole run.
pub fn matches_prohibited(s: &str) -> Option<&'static str> {
    let norm: String = s
        .chars()
        .map(|c| match c {
            '-' | ' ' | '.' | '/' => '_',
            c => c.to_ascii_lowercase(),
        })
        .collect();
    for tok in PROHIBITED_TOKENS {
        if norm.contains(tok) {
            return Some(tok);
        }
    }
    None
}

/// Reason a relay envelope may be rejected on scope grounds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeRejection {
    /// A prohibited token appeared in a string field of the envelope
    /// or its payload.
    ProhibitedContent { token: String, field_hint: String },
}

/// Scan a JSON-serialisable payload for prohibited content. Returns the
/// first offending token found, or `None` if the payload is clean.
///
/// This is a *belt-and-braces* check on top of the compile-time allowlist:
/// the allowlist rejects prohibited *message types*, and this scan rejects
/// prohibited *content* regardless of type.
pub fn scan_json(value: &serde_json::Value) -> Option<ScopeRejection> {
    fn walk(v: &serde_json::Value, path: &str) -> Option<ScopeRejection> {
        match v {
            serde_json::Value::String(s) => {
                matches_prohibited(s).map(|tok| ScopeRejection::ProhibitedContent {
                    token: tok.to_string(),
                    field_hint: path.to_string(),
                })
            }
            serde_json::Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    let p = format!("{path}[{i}]");
                    if let Some(r) = walk(x, &p) {
                        return Some(r);
                    }
                }
                None
            }
            serde_json::Value::Object(o) => {
                for (k, x) in o {
                    if let Some(tok) = matches_prohibited(k) {
                        return Some(ScopeRejection::ProhibitedContent {
                            token: tok.to_string(),
                            field_hint: format!("{path}.{k}"),
                        });
                    }
                    let p = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    if let Some(r) = walk(x, &p) {
                        return Some(r);
                    }
                }
                None
            }
            _ => None,
        }
    }
    walk(value, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_token_matches() {
        assert!(matches_prohibited("firing_solution").is_some());
        assert!(matches_prohibited("Fire-Solution").is_some());
        assert!(matches_prohibited("The system computes a Firing Solution").is_some());
    }

    #[test]
    fn benign_content_passes() {
        assert!(matches_prohibited("track_state").is_none());
        assert!(matches_prohibited("system_health").is_none());
        assert!(matches_prohibited("observation summary payload").is_none());
    }

    #[test]
    fn json_scan_detects_prohibited_field() {
        let v = serde_json::json!({
            "kind": "TrackState",
            "payload": { "firing_solution": {"x": 1} }
        });
        let r = scan_json(&v).unwrap();
        match r {
            ScopeRejection::ProhibitedContent { token, .. } => {
                assert_eq!(token, "firing_solution");
            }
        }
    }

    #[test]
    fn json_scan_passes_clean_track_state() {
        let v = serde_json::json!({
            "kind": "TrackState",
            "payload": {
                "track_id": "abc",
                "confidence": 0.7,
                "uncertainty_m": 42.0
            }
        });
        assert!(scan_json(&v).is_none());
    }
}
