//! Secure information relay.
//!
//! The gateway enforces the outbound-message allowlist, prohibited-content
//! scanning, destination allowlisting, signing, and anti-replay. Store-
//! and-forward and dead-lettering are implemented against a bounded
//! in-memory queue for the baseline; a durable queue lives in the
//! `persistence` crate for production deployments.
//!
//! The `allowlist` module is one of the three files exempt from the
//! scope-boundary source scanner — it explicitly encodes the *permitted*
//! surface and does not contribute prohibited tokens to any exported API.

#![forbid(unsafe_code)]

pub mod allowlist;
pub mod gateway;
pub mod policy;
pub mod signing;

pub use gateway::{RelayGateway, RelayRejectReason};
pub use policy::{DestinationPolicy, RelayPolicy};
