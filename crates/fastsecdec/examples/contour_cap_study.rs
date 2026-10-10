//! Caller-owned cap diagnostics over existing native saved kernels. Never generates sectors.
#[path = "../../fastsecdec-cli/src/contour_pilot.rs"]
mod contour_pilot;
#[path = "contour_variance/coordinates.rs"]
mod coordinates;
#[path = "contour_cap_study/runner.rs"]
mod runner;

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;
fn main() -> CliResult<()> {
    runner::execute()
}
