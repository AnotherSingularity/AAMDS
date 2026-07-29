//! Sensor adapter SDK.
//!
//! Adapter authors implement the [`SensorAdapter`] trait to make a sensor
//! source consumable by Aeon. The SDK ships with three reference adapters
//! and a conformance harness that exercises every mandatory failure mode
//! listed in section 9 of the implementation directive.

#![forbid(unsafe_code)]

pub mod adapter;
pub mod capability;
pub mod conformance;
pub mod adapters {
    pub mod synthetic;
    pub mod replay;
    pub mod file;
}

pub use adapter::{AdapterDiagnostic, AdapterError, SensorAdapter};
pub use capability::AdapterCapability;
