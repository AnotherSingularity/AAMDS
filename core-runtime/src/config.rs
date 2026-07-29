//! Typed, versioned configuration.
//!
//! Configuration is validated *before* activation. Activation records the
//! actor, timestamp, and the configuration's canonical digest. Previous
//! versions are preserved so a rollback is a matter of re-activating a
//! stored `ConfigurationVersion`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub runtime_id: String,
    pub build_id: String,
    pub max_ingest_queue: u32,
    pub max_relay_queue: u32,
    pub freshness_seconds: u32,
    pub clock_drift_tolerance_ms: u32,
    pub scope_boundary_scan_outbound: bool,
    pub allow_unsigned_sources: bool,
}

impl RuntimeConfig {
    pub fn digest_hex(&self) -> String {
        let canonical = serde_json::to_vec(self).expect("config canonicalisation");
        hex::encode(Sha256::digest(&canonical))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("max_ingest_queue must be > 0")]
    ZeroIngestQueue,
    #[error("max_relay_queue must be > 0")]
    ZeroRelayQueue,
    #[error("freshness_seconds must be > 0")]
    ZeroFreshness,
    #[error("scope-boundary outbound scan cannot be disabled in a production config")]
    OutboundScanDisabled,
    #[error("build_id must be non-empty")]
    EmptyBuildId,
    #[error("runtime_id must be non-empty")]
    EmptyRuntimeId,
    #[error("allow_unsigned_sources=true is not a production default")]
    UnsignedSourcesAllowed,
}

impl RuntimeConfig {
    /// Validate the configuration. `production` mode enforces the fail-closed
    /// defaults from `docs/architecture/FAILURE_MODEL.md`.
    pub fn validate(&self, production: bool) -> Result<(), Vec<ConfigError>> {
        let mut errs = Vec::new();
        if self.max_ingest_queue == 0 { errs.push(ConfigError::ZeroIngestQueue); }
        if self.max_relay_queue == 0  { errs.push(ConfigError::ZeroRelayQueue); }
        if self.freshness_seconds == 0 { errs.push(ConfigError::ZeroFreshness); }
        if self.runtime_id.is_empty() { errs.push(ConfigError::EmptyRuntimeId); }
        if self.build_id.is_empty()   { errs.push(ConfigError::EmptyBuildId); }
        if production {
            if !self.scope_boundary_scan_outbound {
                errs.push(ConfigError::OutboundScanDisabled);
            }
            if self.allow_unsigned_sources {
                errs.push(ConfigError::UnsignedSourcesAllowed);
            }
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_cfg() -> RuntimeConfig {
        RuntimeConfig {
            runtime_id: "test".into(),
            build_id: "b1".into(),
            max_ingest_queue: 100,
            max_relay_queue: 100,
            freshness_seconds: 30,
            clock_drift_tolerance_ms: 500,
            scope_boundary_scan_outbound: true,
            allow_unsigned_sources: false,
        }
    }

    #[test]
    fn valid_config_passes_production_validation() {
        assert!(ok_cfg().validate(true).is_ok());
    }

    #[test]
    fn zero_ingest_queue_is_rejected() {
        let mut c = ok_cfg();
        c.max_ingest_queue = 0;
        let e = c.validate(true).unwrap_err();
        assert!(matches!(e.first(), Some(ConfigError::ZeroIngestQueue)));
    }

    #[test]
    fn production_requires_outbound_scan_and_signed_sources() {
        let mut c = ok_cfg();
        c.scope_boundary_scan_outbound = false;
        c.allow_unsigned_sources = true;
        let e = c.validate(true).unwrap_err();
        assert!(e.iter().any(|x| matches!(x, ConfigError::OutboundScanDisabled)));
        assert!(e.iter().any(|x| matches!(x, ConfigError::UnsignedSourcesAllowed)));
    }

    #[test]
    fn non_production_is_permissive_but_still_rejects_zero_queues() {
        let mut c = ok_cfg();
        c.allow_unsigned_sources = true;
        c.scope_boundary_scan_outbound = false;
        assert!(c.validate(false).is_ok());
        c.max_ingest_queue = 0;
        assert!(c.validate(false).is_err());
    }

    #[test]
    fn digest_is_stable_and_change_sensitive() {
        let a = ok_cfg();
        let mut b = a.clone();
        assert_eq!(a.digest_hex(), b.digest_hex());
        b.max_ingest_queue = 200;
        assert_ne!(a.digest_hex(), b.digest_hex());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let bad = r#"{"runtime_id":"x","build_id":"b","max_ingest_queue":1,
            "max_relay_queue":1,"freshness_seconds":1,"clock_drift_tolerance_ms":1,
            "scope_boundary_scan_outbound":true,"allow_unsigned_sources":false,
            "extra_field":true}"#;
        let r: Result<RuntimeConfig, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }
}
