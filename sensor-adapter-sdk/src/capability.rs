//! Adapter capability declaration.
//!
//! An adapter declares up-front what it can supply. Normalization uses this
//! to decide whether to accept a given measurement kind and how to
//! interpret vendor-native payloads.

use aeon_contracts::coords::CoordinateReference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterCapability {
    pub name: String,
    pub adapter_version: String,
    pub supported_coordinate_frames: Vec<CoordinateReference>,
    pub supplies_velocity: bool,
    pub supplies_classification: bool,
    pub max_observations_per_second: u32,
    pub supports_backpressure: bool,
    pub supports_reconnect: bool,
    pub supports_signed_source: bool,
}
