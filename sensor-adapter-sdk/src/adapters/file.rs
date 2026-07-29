//! Generic line-delimited-file adapter — a thin alias for [`super::replay::ReplayAdapter`]
//! with clock-rollback and duplicate-sequence checks enabled. Kept as a
//! separate name for clarity in configuration.

pub use super::replay::ReplayAdapter as FileAdapter;
