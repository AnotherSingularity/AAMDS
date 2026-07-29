//! Operator API — read-oriented HTTP surface for the recognized
//! operational picture.
//!
//! The API is intentionally narrow. Write operations are restricted to:
//!   * alert acknowledgment
//!   * operator annotation
//!   * authorized configuration activation (out-of-band control)
//!   * replay control
//!   * pause / resume
//!   * model activation (through the ML approval workflow)
//!
//! No firing / launch / guidance / engagement endpoint exists — that is
//! a scope-boundary constraint enforced by the prohibited-message
//! registry and mechanically checked by CI.

#![forbid(unsafe_code)]

pub mod state;
pub mod routes;

pub use routes::build_router;
pub use state::ApiState;
