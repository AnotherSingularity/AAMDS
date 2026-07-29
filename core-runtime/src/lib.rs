//! Runtime supervisor for the Aeon information layer.
//!
//! Responsibilities:
//!   * Lifecycle state machine (`RuntimeState`).
//!   * Configuration validation before service activation.
//!   * Readiness / liveness reporting.
//!   * Structured logs with build + configuration identity.
//!   * Graceful shutdown and restart-recovery hooks.
//!   * Bounded queues and backpressure signalling.

#![forbid(unsafe_code)]

pub mod config;
pub mod health;
pub mod lifecycle;

pub use lifecycle::{RuntimeError, RuntimeSupervisor};
