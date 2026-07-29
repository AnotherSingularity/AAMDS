//! Canonical, versioned domain contracts for the Aeon Air Defense
//! Information Layer.
//!
//! Every material output of the system serializes through the types in this
//! crate. The types are intentionally strict: unknown states are represented
//! explicitly, never as silent defaults, and provenance is required on
//! anything that leaves a subsystem boundary.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod alert;
pub mod audit;
pub mod coords;
pub mod health;
pub mod ids;
pub mod observation;
pub mod prohibited;
pub mod provenance;
pub mod relay;
pub mod time_kind;
pub mod track;
pub mod uncertainty;
pub mod unknown;
pub mod version;

pub use ids::*;
pub use unknown::Known;
pub use version::SchemaVersion;
