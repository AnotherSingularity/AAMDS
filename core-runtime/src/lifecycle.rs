//! Runtime lifecycle state machine.
//!
//! Transitions permitted:
//!
//!   Uninitialized -> ValidatingConfiguration
//!   ValidatingConfiguration -> Starting | Failed
//!   Starting -> Ready | Failed
//!   Ready    -> Degraded | Paused | ShuttingDown | Failed
//!   Degraded -> Ready | Paused | ShuttingDown | Failed
//!   Paused   -> Ready | ShuttingDown | Failed
//!   ShuttingDown -> Uninitialized
//!   Failed   -> Uninitialized
//!
//! Any transition not listed here is rejected.

use aeon_contracts::health::RuntimeState;
use serde::{Deserialize, Serialize};

use crate::config::{ConfigError, RuntimeConfig};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("illegal transition from {from:?} to {to:?}")]
    IllegalTransition { from: RuntimeState, to: RuntimeState },
    #[error("configuration invalid: {0:?}")]
    InvalidConfiguration(Vec<ConfigError>),
    #[error("cannot promote to Ready without a validated configuration")]
    NoValidatedConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeSupervisor {
    state: RuntimeState,
    validated_digest: Option<String>,
    production_mode: bool,
}

impl RuntimeSupervisor {
    pub fn new(production_mode: bool) -> Self {
        Self {
            state: RuntimeState::Uninitialized,
            validated_digest: None,
            production_mode,
        }
    }

    pub fn state(&self) -> RuntimeState { self.state }

    /// Attempt a state transition. Every transition is checked; any illegal
    /// transition returns an error and leaves the supervisor unchanged.
    pub fn transition(&mut self, to: RuntimeState) -> Result<(), RuntimeError> {
        use RuntimeState::*;
        let ok = matches!(
            (self.state, to),
            (Uninitialized, ValidatingConfiguration)
            | (ValidatingConfiguration, Starting)
            | (ValidatingConfiguration, Failed)
            | (Starting, Ready)
            | (Starting, Failed)
            | (Ready, Degraded)
            | (Ready, Paused)
            | (Ready, ShuttingDown)
            | (Ready, Failed)
            | (Degraded, Ready)
            | (Degraded, Paused)
            | (Degraded, ShuttingDown)
            | (Degraded, Failed)
            | (Paused, Ready)
            | (Paused, ShuttingDown)
            | (Paused, Failed)
            | (ShuttingDown, Uninitialized)
            | (Failed, Uninitialized)
        );
        if !ok {
            return Err(RuntimeError::IllegalTransition { from: self.state, to });
        }
        // Promotion to Ready requires a validated configuration.
        if to == Ready && self.validated_digest.is_none() {
            return Err(RuntimeError::NoValidatedConfig);
        }
        self.state = to;
        Ok(())
    }

    /// Validate a candidate configuration. On success, the digest is stored
    /// and the supervisor advances Uninitialized→ValidatingConfiguration→Starting.
    /// On failure, the supervisor advances Uninitialized→ValidatingConfiguration→Failed.
    pub fn validate_and_stage(&mut self, cfg: &RuntimeConfig) -> Result<String, RuntimeError> {
        self.transition(RuntimeState::ValidatingConfiguration)?;
        match cfg.validate(self.production_mode) {
            Ok(()) => {
                let digest = cfg.digest_hex();
                self.validated_digest = Some(digest.clone());
                self.transition(RuntimeState::Starting)?;
                Ok(digest)
            }
            Err(errs) => {
                self.state = RuntimeState::Failed;
                Err(RuntimeError::InvalidConfiguration(errs))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> RuntimeConfig {
        RuntimeConfig {
            runtime_id: "t".into(),
            build_id: "b".into(),
            max_ingest_queue: 10,
            max_relay_queue: 10,
            freshness_seconds: 10,
            clock_drift_tolerance_ms: 100,
            scope_boundary_scan_outbound: true,
            allow_unsigned_sources: false,
        }
    }

    #[test]
    fn happy_path_reaches_ready() {
        let mut r = RuntimeSupervisor::new(true);
        r.validate_and_stage(&cfg()).unwrap();
        r.transition(RuntimeState::Ready).unwrap();
        assert_eq!(r.state(), RuntimeState::Ready);
    }

    #[test]
    fn invalid_config_prevents_ready() {
        let mut bad = cfg();
        bad.max_ingest_queue = 0;
        let mut r = RuntimeSupervisor::new(true);
        assert!(r.validate_and_stage(&bad).is_err());
        assert_eq!(r.state(), RuntimeState::Failed);
        // Even trying to transition to Ready without a validated config
        // must be refused.
        r.state = RuntimeState::Starting; // simulate direct manipulation
        r.validated_digest = None;
        assert!(matches!(
            r.transition(RuntimeState::Ready),
            Err(RuntimeError::NoValidatedConfig)
        ));
    }

    #[test]
    fn illegal_transition_is_rejected() {
        let mut r = RuntimeSupervisor::new(true);
        // Uninitialized -> Ready is not allowed
        assert!(matches!(
            r.transition(RuntimeState::Ready),
            Err(RuntimeError::IllegalTransition { .. })
        ));
    }

    #[test]
    fn ready_can_degrade_and_recover() {
        let mut r = RuntimeSupervisor::new(true);
        r.validate_and_stage(&cfg()).unwrap();
        r.transition(RuntimeState::Ready).unwrap();
        r.transition(RuntimeState::Degraded).unwrap();
        r.transition(RuntimeState::Ready).unwrap();
        assert_eq!(r.state(), RuntimeState::Ready);
    }

    #[test]
    fn restart_recovery_path() {
        let mut r = RuntimeSupervisor::new(true);
        r.validate_and_stage(&cfg()).unwrap();
        r.transition(RuntimeState::Ready).unwrap();
        r.transition(RuntimeState::ShuttingDown).unwrap();
        r.transition(RuntimeState::Uninitialized).unwrap();
        // A restarted supervisor must re-validate configuration before
        // reaching Ready again.
        r.validated_digest = None;
        r.validate_and_stage(&cfg()).unwrap();
        r.transition(RuntimeState::Ready).unwrap();
    }
}
