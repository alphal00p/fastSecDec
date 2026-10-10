//! Derive native Ward bindings and compare complete numerical Ward estimates.
//! This command never generates sectors or starts integration.
#[path = "../../../example/gg_hh_one_loop_ME/fastsecdec/summarize.rs"]
mod bookkeeping;
#[path = "../../../example/gg_hh_one_loop_ME/fastsecdec/ward/prepare.rs"]
mod prepare;
#[path = "../../../example/gg_hh_one_loop_ME/fastsecdec/ward/summary.rs"]
mod summary;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, input, output] if command == "prepare" => {
            prepare::run(input.as_ref(), output.as_ref())
        }
        [command, work, reference, output] if command == "summarize" => {
            summary::run(work.as_ref(), reference.as_ref(), output.as_ref())
        }
        // Keep the unchanged physical-amplitude driver available, while sharing
        // its complete-vector summation rather than copying that implementation.
        [command, ..] if command == "amplitude" => bookkeeping::run(),
        _ => Err("usage: gghh_one_loop_ward prepare INPUTS FRESH_OUTPUT | summarize WORK WARD_REFERENCE OUTPUT.json".into()),
    }
}
