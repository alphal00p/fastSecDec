//! Cargo entry point; reproducible reference implementation lives with its inputs.
#[path = "../../../example/gg_hh_one_loop_ME/hepkit/reference.rs"]
mod reference;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    reference::main()
}
