//! Bounded, caller-driven scalar comparison. Actions never imply later stages.
#[path = "../../fastsecdec-cli/src/contour_pilot.rs"]
mod contour_pilot;
#[path = "contour_variance/coordinates.rs"]
mod coordinates;
#[path = "contour_scalar_benchmark/fixtures.rs"]
mod fixtures;
#[path = "contour_scalar_benchmark/runner.rs"]
mod runner;

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> CliResult<()> {
    runner::execute()
}
