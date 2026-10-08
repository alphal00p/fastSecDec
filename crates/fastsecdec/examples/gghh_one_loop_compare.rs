//! Reproducible comparison of the three saved gg -> hh helicity calculations.
#[path = "../../../example/gg_hh_one_loop_ME/compare.rs"]
mod comparison;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    comparison::run()
}
