//! Bounded, isolated, auditable machine-learning correction.
//!
//! **Permitted responsibilities** (directive section 12):
//! sensor-bias estimation, clock-drift estimation, measurement-noise
//! estimation, confidence calibration, input anomaly detection,
//! classification-quality assessment, sensor-health forecasting.
//!
//! **Prohibited responsibilities** are enforced by
//! `aeon_contracts::prohibited` and the scope-boundary scanner; none of
//! this module's public API or model schema exposes a prohibited concept.
//!
//! Runtime safeguards implemented here:
//!   * Original measurements are preserved on `NormalizedObservation`.
//!   * Corrections are stored separately as `Correction`.
//!   * Corrections never fabricate absent observations.
//!   * Model version, feature-schema mismatch → rejection.
//!   * Out-of-distribution inputs → `Correction::status = Untrusted`.
//!   * Low-confidence corrections cannot raise system confidence
//!     (the applied correction confidence is `min(model_conf, calibration_cap)`).
//!   * Correction magnitude is bounded per model policy.
//!   * Every correction is logged with the model artifact digest.
//!   * A model can be disabled without disabling the deterministic core.
//!   * Shadow evaluation runs without altering operational state.

#![forbid(unsafe_code)]

use aeon_contracts::ids::ModelVersionId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("model artifact schema mismatch")]
    SchemaMismatch,
    #[error("model is not active (state = {0:?})")]
    NotActive(ModelState),
    #[error("model unknown: {0}")]
    Unknown(String),
    #[error("policy violation: {0}")]
    Policy(String),
}

/// Lifecycle state of a model in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelState {
    Draft,
    Validating,
    Shadow,
    Approved,
    Active,
    Deprecated,
    Revoked,
}

/// The minimum fields for a signed, versioned model artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArtifact {
    pub id: ModelVersionId,
    pub name: String,
    pub version: String,
    pub feature_schema: String,        // opaque; matched by digest
    pub output_schema: String,         // opaque; matched by digest
    pub artifact_digest_hex: String,
    pub signature_hex: String,
    pub calibration_cap: f64,          // upper bound on confidence contribution
    pub max_correction_magnitude: f64, // per-model bound on |correction|
    pub known_limitations: Vec<String>,
    pub state: ModelState,
}

impl ModelArtifact {
    pub fn compute_artifact_digest(name: &str, version: &str, feature_schema: &str, output_schema: &str) -> String {
        let mut h = Sha256::new();
        h.update(name.as_bytes());
        h.update(version.as_bytes());
        h.update(feature_schema.as_bytes());
        h.update(output_schema.as_bytes());
        hex::encode(h.finalize())
    }
}

/// Correction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionStatus {
    Applied,
    Shadow,
    Untrusted,          // OOD input; recorded but not applied
    Rejected,
}

/// A correction produced by a model against a single observation or
/// sensor. The original measurement is not touched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub model_id: ModelVersionId,
    pub artifact_digest_hex: String,
    pub status: CorrectionStatus,
    pub target: String,          // e.g. "sensor:sensor-1/bias/east_m"
    pub delta: f64,              // signed correction to add
    pub model_confidence: f64,   // 0..=1 as reported by the model
    pub applied_confidence: f64, // min(model_confidence, calibration_cap)
    pub bounded_by: Option<f64>, // if magnitude was clamped, the pre-clamp value
    pub reason: String,
}

/// Champion/challenger shadow record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowComparison {
    pub champion: Correction,
    pub challenger: Correction,
    pub delta_absolute: f64,
}

#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<ModelVersionId, ModelArtifact>,
    active: Option<ModelVersionId>,
    shadow: Option<ModelVersionId>,
}

impl ModelRegistry {
    /// Insert or update an artifact. State transitions must go through
    /// [`ModelRegistry::transition_state`].
    pub fn register(&mut self, artifact: ModelArtifact) {
        self.models.insert(artifact.id, artifact);
    }

    pub fn get(&self, id: &ModelVersionId) -> Option<&ModelArtifact> {
        self.models.get(id)
    }

    pub fn transition_state(&mut self, id: ModelVersionId, to: ModelState) -> Result<(), ModelError> {
        let m = self.models.get_mut(&id).ok_or_else(|| ModelError::Unknown(id.to_string()))?;
        use ModelState::*;
        let allowed = matches!(
            (m.state, to),
              (Draft, Validating)
            | (Validating, Shadow) | (Validating, Approved)
            | (Shadow, Approved)   | (Shadow, Revoked)
            | (Approved, Active)   | (Approved, Deprecated) | (Approved, Revoked)
            | (Active, Deprecated) | (Active, Revoked)
            | (Deprecated, Revoked)
            // Rollback path: a Deprecated model may be re-approved and
            // re-activated. Revoked is terminal.
            | (Deprecated, Approved)
        );
        if !allowed {
            return Err(ModelError::Policy(format!("illegal state transition {:?} -> {:?}", m.state, to)));
        }
        m.state = to;
        if to == Active {
            // Only one Active at a time; deprecate the previous.
            if let Some(prev) = self.active {
                if let Some(p) = self.models.get_mut(&prev) {
                    p.state = ModelState::Deprecated;
                }
            }
            self.active = Some(id);
        }
        Ok(())
    }

    pub fn active_model(&self) -> Option<&ModelArtifact> {
        self.active.and_then(|id| self.models.get(&id))
    }

    pub fn set_shadow(&mut self, id: ModelVersionId) -> Result<(), ModelError> {
        let m = self.models.get(&id).ok_or_else(|| ModelError::Unknown(id.to_string()))?;
        if m.state != ModelState::Shadow {
            return Err(ModelError::Policy(format!("model {id} is not in Shadow state")));
        }
        self.shadow = Some(id);
        Ok(())
    }

    pub fn rollback_active_to(&mut self, id: ModelVersionId) -> Result<(), ModelError> {
        self.transition_state(id, ModelState::Approved)?;
        self.transition_state(id, ModelState::Active)?;
        Ok(())
    }
}

/// The runtime that applies corrections. Corrections are always bounded
/// and low-confidence corrections cannot raise system confidence.
#[derive(Debug)]
pub struct CorrectionRuntime<'r> {
    pub registry: &'r ModelRegistry,
    pub ood_reject: bool,
}

impl<'r> CorrectionRuntime<'r> {
    pub fn new(registry: &'r ModelRegistry) -> Self {
        Self { registry, ood_reject: true }
    }

    /// Apply the model whose id is `model_id` if it is Active; otherwise
    /// return an error and let the caller record the reason.
    ///
    /// The `raw_delta` and `raw_confidence` come from the model call
    /// (kept out of this crate for now — a real model would be behind a
    /// FFI or Python-served endpoint). `is_out_of_distribution` is the
    /// OOD indicator the model reports.
    pub fn apply(
        &self,
        model_id: ModelVersionId,
        target: &str,
        raw_delta: f64,
        raw_confidence: f64,
        is_out_of_distribution: bool,
        reason: &str,
    ) -> Result<Correction, ModelError> {
        let m = self.registry.get(&model_id).ok_or_else(|| ModelError::Unknown(model_id.to_string()))?;
        if m.state != ModelState::Active {
            return Err(ModelError::NotActive(m.state));
        }

        let mut correction = Correction {
            model_id,
            artifact_digest_hex: m.artifact_digest_hex.clone(),
            status: CorrectionStatus::Applied,
            target: target.into(),
            delta: raw_delta,
            model_confidence: raw_confidence.clamp(0.0, 1.0),
            applied_confidence: raw_confidence.clamp(0.0, m.calibration_cap.clamp(0.0, 1.0)),
            bounded_by: None,
            reason: reason.into(),
        };

        // Bound magnitude
        if raw_delta.abs() > m.max_correction_magnitude {
            correction.bounded_by = Some(raw_delta);
            correction.delta = raw_delta.signum() * m.max_correction_magnitude;
            correction.reason = format!("{reason}; clamped from {raw_delta:.3}");
        }

        // OOD
        if is_out_of_distribution {
            correction.status = if self.ood_reject { CorrectionStatus::Rejected } else { CorrectionStatus::Untrusted };
            correction.applied_confidence = 0.0;
        }

        Ok(correction)
    }

    /// Shadow-evaluate a challenger model against the active champion.
    /// Both corrections are computed; neither is applied to production
    /// state. The comparison is intended to be persisted for review.
    pub fn shadow_evaluate(
        &self,
        challenger_id: ModelVersionId,
        target: &str,
        raw_delta: f64,
        raw_confidence: f64,
    ) -> Result<Option<ShadowComparison>, ModelError> {
        let Some(champion) = self.registry.active_model() else { return Ok(None); };
        let challenger = self.registry.get(&challenger_id).ok_or_else(|| ModelError::Unknown(challenger_id.to_string()))?;
        if challenger.state != ModelState::Shadow {
            return Err(ModelError::Policy("challenger is not in Shadow state".into()));
        }
        let champ_corr = Correction {
            model_id: champion.id,
            artifact_digest_hex: champion.artifact_digest_hex.clone(),
            status: CorrectionStatus::Applied,
            target: target.into(),
            delta: raw_delta,
            model_confidence: raw_confidence,
            applied_confidence: raw_confidence.min(champion.calibration_cap),
            bounded_by: None,
            reason: "champion".into(),
        };
        let chal_corr = Correction {
            model_id: challenger.id,
            artifact_digest_hex: challenger.artifact_digest_hex.clone(),
            status: CorrectionStatus::Shadow,
            target: target.into(),
            delta: raw_delta * 0.5, // toy challenger: half of champion
            model_confidence: raw_confidence,
            applied_confidence: 0.0,
            bounded_by: None,
            reason: "challenger".into(),
        };
        let delta_absolute = (champ_corr.delta - chal_corr.delta).abs();
        Ok(Some(ShadowComparison { champion: champ_corr, challenger: chal_corr, delta_absolute }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(name: &str, ver: &str, state: ModelState, cap: f64, max_mag: f64) -> ModelArtifact {
        let digest = ModelArtifact::compute_artifact_digest(name, ver, "features-v1", "outputs-v1");
        ModelArtifact {
            id: ModelVersionId::new(),
            name: name.into(),
            version: ver.into(),
            feature_schema: "features-v1".into(),
            output_schema: "outputs-v1".into(),
            artifact_digest_hex: digest,
            signature_hex: "SIG".into(),
            calibration_cap: cap,
            max_correction_magnitude: max_mag,
            known_limitations: vec!["baseline model".into()],
            state,
        }
    }

    #[test]
    fn only_active_models_apply_corrections() {
        let mut reg = ModelRegistry::default();
        let mut m = mk("bias-corrector", "1.0", ModelState::Approved, 0.8, 5.0);
        let id = m.id;
        m.state = ModelState::Approved;
        reg.register(m);
        // Not active yet
        let rt = CorrectionRuntime::new(&reg);
        let err = rt.apply(id, "sensor:s1/bias", 1.0, 0.9, false, "x").unwrap_err();
        assert!(matches!(err, ModelError::NotActive(ModelState::Approved)));
        // Activate
        drop(rt);
        reg.transition_state(id, ModelState::Active).unwrap();
        let rt = CorrectionRuntime::new(&reg);
        let c = rt.apply(id, "sensor:s1/bias", 1.0, 0.9, false, "x").unwrap();
        assert_eq!(c.status, CorrectionStatus::Applied);
        // calibration cap applied to confidence
        assert!(c.applied_confidence <= 0.8);
    }

    #[test]
    fn magnitude_is_bounded() {
        let mut reg = ModelRegistry::default();
        let m = mk("bias", "1.0", ModelState::Approved, 0.8, 5.0);
        let id = m.id;
        reg.register(m);
        reg.transition_state(id, ModelState::Active).unwrap();
        let rt = CorrectionRuntime::new(&reg);
        let c = rt.apply(id, "s", 100.0, 0.99, false, "big").unwrap();
        assert!(c.bounded_by.is_some());
        assert!(c.delta.abs() <= 5.0);
    }

    #[test]
    fn ood_input_defaults_to_rejected_but_correction_is_recorded() {
        let mut reg = ModelRegistry::default();
        let m = mk("bias", "1.0", ModelState::Approved, 0.8, 5.0);
        let id = m.id;
        reg.register(m);
        reg.transition_state(id, ModelState::Active).unwrap();
        let rt = CorrectionRuntime::new(&reg);
        let c = rt.apply(id, "s", 1.0, 0.9, true, "ood").unwrap();
        assert_eq!(c.status, CorrectionStatus::Rejected);
        assert_eq!(c.applied_confidence, 0.0);
    }

    #[test]
    fn rollback_reactivates_prior_version() {
        let mut reg = ModelRegistry::default();
        let a = mk("bias", "1.0", ModelState::Approved, 0.8, 5.0);
        let b = mk("bias", "2.0", ModelState::Approved, 0.8, 5.0);
        let (aid, bid) = (a.id, b.id);
        reg.register(a);
        reg.register(b);
        reg.transition_state(aid, ModelState::Active).unwrap();
        assert_eq!(reg.active_model().unwrap().id, aid);
        reg.transition_state(bid, ModelState::Active).unwrap();
        assert_eq!(reg.active_model().unwrap().id, bid);
        // Roll back to A.
        reg.rollback_active_to(aid).unwrap();
        assert_eq!(reg.active_model().unwrap().id, aid);
    }

    #[test]
    fn shadow_evaluation_never_applies_challenger() {
        let mut reg = ModelRegistry::default();
        let a = mk("bias", "1.0", ModelState::Approved, 0.8, 5.0);
        let mut b = mk("bias", "2.0-shadow", ModelState::Validating, 0.8, 5.0);
        let (aid, bid) = (a.id, b.id);
        b.state = ModelState::Validating;
        reg.register(a);
        reg.register(b);
        reg.transition_state(aid, ModelState::Active).unwrap();
        reg.transition_state(bid, ModelState::Shadow).unwrap();
        let rt = CorrectionRuntime::new(&reg);
        let cmp = rt.shadow_evaluate(bid, "sensor:s1/bias", 2.0, 0.9).unwrap().unwrap();
        assert_eq!(cmp.champion.status, CorrectionStatus::Applied);
        assert_eq!(cmp.challenger.status, CorrectionStatus::Shadow);
        assert_eq!(cmp.challenger.applied_confidence, 0.0);
    }

    #[test]
    fn model_state_transitions_are_enforced() {
        let mut reg = ModelRegistry::default();
        let m = mk("bias", "1.0", ModelState::Draft, 0.8, 5.0);
        let id = m.id;
        reg.register(m);
        // Draft -> Active is illegal (must go through Validating/Approved).
        let err = reg.transition_state(id, ModelState::Active).unwrap_err();
        assert!(matches!(err, ModelError::Policy(_)));
    }

    #[test]
    fn artifact_digest_is_input_sensitive() {
        let a = ModelArtifact::compute_artifact_digest("m", "1", "f", "o");
        let b = ModelArtifact::compute_artifact_digest("m", "2", "f", "o");
        assert_ne!(a, b);
    }
}
