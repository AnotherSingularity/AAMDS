//! Synthetic sensor adapter — deterministic, seedable, closed-form generator.
//!
//! Emits `RawObservation`s along a linear ground track. Used by
//! simulation, replay, and the operator interface's developer profile.

use aeon_contracts::coords::CoordinateReference;
use aeon_contracts::ids::{AdapterId, ObservationId, SensorId, SourceSystemId};
use aeon_contracts::observation::{QualityIndicators, RawMeasurement, RawObservation};
use aeon_contracts::provenance::Integrity;
use aeon_contracts::time_kind::TimeQuality;
use aeon_contracts::unknown::Known;
use aeon_contracts::version::observation_schema;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::adapter::{AdapterDiagnostic, AdapterError, SensorAdapter};
use crate::capability::AdapterCapability;

#[derive(Debug, Clone)]
pub struct SyntheticConfig {
    pub sensor_id: String,
    pub start_lat_deg: f64,
    pub start_lon_deg: f64,
    pub start_alt_m: f64,
    pub v_north_ms: f64,
    pub v_east_ms: f64,
    pub v_up_ms: f64,
    pub max_observations: u32,
    pub tick_seconds: f64,
    pub start_time: OffsetDateTime,
}

#[derive(Debug)]
pub struct SyntheticAdapter {
    cfg: SyntheticConfig,
    connected: bool,
    tick: u32,
    forwarded: u64,
    rejected: u64,
    last_seq: Option<u64>,
    seed: u64,
}

impl SyntheticAdapter {
    pub fn new(cfg: SyntheticConfig) -> Self {
        Self {
            cfg,
            connected: false,
            tick: 0,
            forwarded: 0,
            rejected: 0,
            last_seq: None,
            seed: 0xA1E0_1EEF,
        }
    }
}

impl SensorAdapter for SyntheticAdapter {
    fn capability(&self) -> AdapterCapability {
        AdapterCapability {
            name: "aeon-synthetic".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            supported_coordinate_frames: vec![CoordinateReference::Wgs84Geodetic],
            supplies_velocity: true,
            supplies_classification: true,
            max_observations_per_second: (1.0 / self.cfg.tick_seconds.max(0.001)) as u32,
            supports_backpressure: true,
            supports_reconnect: true,
            supports_signed_source: true,
        }
    }

    fn validate_configuration(&self) -> Result<(), AdapterError> {
        if self.cfg.tick_seconds <= 0.0 {
            return Err(AdapterError::InvalidConfiguration(
                "tick_seconds must be > 0".into(),
            ));
        }
        if self.cfg.max_observations == 0 {
            return Err(AdapterError::InvalidConfiguration(
                "max_observations must be > 0".into(),
            ));
        }
        if self.cfg.sensor_id.is_empty() {
            return Err(AdapterError::InvalidConfiguration(
                "sensor_id must be non-empty".into(),
            ));
        }
        Ok(())
    }

    fn connect(&mut self) -> Result<(), AdapterError> {
        self.connected = true;
        Ok(())
    }

    fn next_observation(&mut self) -> Result<Option<RawObservation>, AdapterError> {
        if !self.connected {
            return Err(AdapterError::Connect("adapter not connected".into()));
        }
        if self.tick >= self.cfg.max_observations {
            return Ok(None);
        }
        let dt = self.cfg.tick_seconds * self.tick as f64;
        let lat = self.cfg.start_lat_deg + self.cfg.v_north_ms * dt / 111_320.0;
        let lon = self.cfg.start_lon_deg
            + self.cfg.v_east_ms * dt
                / (111_320.0 * self.cfg.start_lat_deg.to_radians().cos().abs().max(1e-6));
        let alt = self.cfg.start_alt_m + self.cfg.v_up_ms * dt;
        let ts = self.cfg.start_time + time::Duration::seconds_f64(dt);

        // Deterministic UUID from tick + seed so replay is bit-identical.
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&self.seed.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.tick.to_be_bytes());
        bytes[12..].copy_from_slice(&self.tick.to_be_bytes());
        let obs_id = ObservationId(Uuid::from_bytes(bytes));

        let seq = self.tick as u64 + 1;
        self.last_seq = Some(seq);
        self.forwarded += 1;
        self.tick += 1;

        Ok(Some(RawObservation {
            schema_version: observation_schema(),
            observation_id: obs_id,
            source_system_id: SourceSystemId::new("synthetic"),
            sensor_id: SensorId::new(self.cfg.sensor_id.clone()),
            adapter_id: AdapterId::new("aeon-synthetic"),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            source_timestamp: ts,
            receive_timestamp: ts,
            sequence_number: seq,
            coordinate_reference: CoordinateReference::Wgs84Geodetic,
            measurement: RawMeasurement::Position {
                x: lon,
                y: lat,
                z: alt,
            },
            measurement_uncertainty: Known::Known {
                value: aeon_contracts::uncertainty::PositionUncertainty {
                    sigma_east_m: 5.0,
                    sigma_north_m: 5.0,
                    sigma_up_m: 20.0,
                },
            },
            velocity_uncertainty: Known::Unknown,
            classification_claims: vec!["synthetic_air_object".into()],
            quality_indicators: QualityIndicators {
                time_quality: TimeQuality::Disciplined,
                declared_snr_db: Known::Unknown,
                declared_confidence: Known::Unknown,
            },
            integrity: Integrity::Verified,
            raw_source_blob_digest: None,
        }))
    }

    fn diagnostic(&self) -> AdapterDiagnostic {
        AdapterDiagnostic {
            connected: self.connected,
            observations_forwarded: self.forwarded,
            observations_rejected: self.rejected,
            last_error: None,
            last_sequence: self.last_seq,
        }
    }

    fn shutdown(&mut self) -> Result<(), AdapterError> {
        self.connected = false;
        Ok(())
    }
}
