//! Append-only persistence for the Aeon information layer.
//!
//! The store is intentionally simple and deterministic:
//!
//!   * SQLite is the reference implementation. A Postgres-compatible
//!     back-end can be added by implementing [`EventStore`] against the
//!     production dialect.
//!   * Every event carries an integrity digest chained against the digest
//!     of the previous event. Rewriting or deleting a past event breaks
//!     the chain and fails [`EventStore::verify_integrity`].
//!   * Rejected observations are stored, never silently dropped.
//!   * Historical rows are append-only. Configuration and model versions
//!     are inserted, not updated.
//!
//! This crate does not perform any correlation, normalisation, or
//! networking — it is a durable event log with a verified hash chain.

#![forbid(unsafe_code)]

pub mod migrations;
pub mod store;

pub use store::{EventStore, StoreError, StoredEvent};
