//! RC2 cross-process determinism harness.
//!
//! Prints the trace digest for a named scenario to stdout. Two runs of
//! this binary (separate processes, possibly on different machines with
//! the same tool versions) must print bit-identical digests.
//!
//! Usage: emit_trace_digest <scenario-name>
//! Scenarios: single_clean_track | crossing_two_tracks

use aeon_simulation::determinism::trace_digest;
use aeon_simulation::pipeline::run_pipeline;
use aeon_simulation::scenarios::{crossing_two_tracks, single_clean_track};
use time::OffsetDateTime;

fn main() -> anyhow::Result<()> {
    let name = std::env::args().nth(1).unwrap_or_else(|| "single_clean_track".into());
    let mut s = match name.as_str() {
        "single_clean_track" => single_clean_track(),
        "crossing_two_tracks" => crossing_two_tracks(),
        other => anyhow::bail!("unknown scenario: {other}"),
    };
    let far = OffsetDateTime::UNIX_EPOCH + time::Duration::days(365 * 10);
    let out = run_pipeline(&s.manifest.name, &mut s.adapter, s.policy, || far)?;
    println!("{}", trace_digest(&out));
    Ok(())
}
