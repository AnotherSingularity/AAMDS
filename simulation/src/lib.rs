//! Simulation and deterministic replay.
//!
//! Given the same synthetic-adapter configuration, the same normalization
//! version, and the same track policy, this crate produces a bit-identical
//! sequence of `TrackUpdate`s across runs. The determinism harness runs
//! the pipeline twice and diffs a canonical trace digest.
//!
//! Replay never opens outbound network sockets. The runtime's `replay_mode`
//! flag (surfaced via the operator API) is expected to be `true` during a
//! replay session so the operator UI can mark it clearly.

#![forbid(unsafe_code)]

pub mod determinism;
pub mod pipeline;
pub mod scenarios;

pub use pipeline::{run_pipeline, PipelineOutcome};
