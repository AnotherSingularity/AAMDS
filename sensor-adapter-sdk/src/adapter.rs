//! The `SensorAdapter` trait.

use aeon_contracts::observation::RawObservation;
use serde::{Deserialize, Serialize};

use crate::capability::AdapterCapability;

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("configuration invalid: {0}")]
    InvalidConfiguration(String),
    #[error("connection failed: {0}")]
    Connect(String),
    #[error("malformed payload: {0}")]
    Malformed(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("unsupported schema version: {0}")]
    UnsupportedSchema(String),
    #[error("integrity failure: {0}")]
    IntegrityFailure(String),
    #[error("clock rollback detected: seen {seen_ns}, received {received_ns}")]
    ClockRollback { seen_ns: i128, received_ns: i128 },
    #[error("duplicate sequence number: {0}")]
    DuplicateSequence(u64),
    #[error("shutting down")]
    ShuttingDown,
    #[error("backpressure — queue full")]
    Backpressure,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Adapter diagnostic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterDiagnostic {
    pub connected: bool,
    pub observations_forwarded: u64,
    pub observations_rejected: u64,
    pub last_error: Option<String>,
    pub last_sequence: Option<u64>,
}

/// The blocking-style contract every adapter must implement.
///
/// Real-world adapters will typically use async I/O internally but must
/// expose a `next_observation` semantics that either returns a raw
/// observation, returns a typed error, or blocks up to a policy-defined
/// timeout. The conformance harness drives adapters through this trait.
pub trait SensorAdapter {
    fn capability(&self) -> AdapterCapability;
    fn validate_configuration(&self) -> Result<(), AdapterError>;
    fn connect(&mut self) -> Result<(), AdapterError>;
    fn next_observation(&mut self) -> Result<Option<RawObservation>, AdapterError>;
    fn diagnostic(&self) -> AdapterDiagnostic;
    fn shutdown(&mut self) -> Result<(), AdapterError>;
}
