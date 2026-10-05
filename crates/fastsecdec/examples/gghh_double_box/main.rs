//! Generate one genuine Standard-Model top/gluon double-box contribution.
//!
//! `cargo run -p fastsecdec --example gghh_double_box -- SM.json NEW_OUTPUT_DIR`
//! writes native input files; run the emitted card with the ordinary CLI.
//! This is one diagram with an explicit color/helicity projection, not a
//! gauge-invariant sum or a cross section.

mod color;
mod export;
mod point;
mod select;

use std::{path::PathBuf, sync::Arc, time::Instant};

use feynkit_generator::{GenerationOptions, NumeratorGrouping, Process};
use feynkit_model::Model;
use symbolica::atom::Atom;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let model_path = PathBuf::from(args.next().ok_or("expected SM.json NEW_OUTPUT_DIR")?);
    let output = PathBuf::from(args.next().ok_or("expected NEW_OUTPUT_DIR")?);
    if args.next().is_some() || output.exists() {
        return Err("supply exactly two arguments and a fresh output directory".into());
    }
    let model_bytes = std::fs::read_to_string(model_path)?;
    let mut model = Model::from_json(&model_bytes)?;
    model.apply_parameter_card(&export::parameter_card(&model)?)?;
    let model = Arc::new(model);
    let allowed = select::interactions(&model)?;
    // PDG selectors avoid relying on a particular UFO's spelling of H or tbar.
    let process =
        Process::new([21_i64, 21], [25_i64, 25]).with_filters(vec![], Some(allowed), vec![]);
    let options = GenerationOptions::default()
        .with_loop_count(2, 2)?
        .max_vertices(6)
        .threads(1)
        .numerator_grouping(NumeratorGrouping::None)
        // Keep the generated amputated tensor. Attach the chosen external
        // helicities only after selecting and archiving the raw diagram.
        .projector(Atom::one());
    let start = Instant::now();
    let generated = process.generate_diagrams(Arc::clone(&model), &options)?;
    if !generated.report.completed {
        return Err("native diagram generation did not complete".into());
    }
    generated.validate_groups()?;
    let mut matches = Vec::new();
    for (index, diagram) in generated.diagrams.iter().enumerate() {
        if let Some(topology) = select::double_box(diagram)? {
            matches.push((index, topology));
        }
    }
    let (selected, topology) = matches
        .first()
        .ok_or("no top-hexagon/gluon double box found")?;
    let raw = &generated.diagrams[*selected];
    let elapsed = start.elapsed().as_secs_f64();
    std::fs::create_dir_all(&output)?;
    // Persist raw evidence before projection or any potentially expensive algebra.
    export::raw(
        &output,
        &model,
        &model_bytes,
        raw,
        &process,
        &options,
        &generated.report,
    )?;
    let projected = export::projected(raw)?;
    let point = point::Point::new(&projected)?;
    export::fixture(
        &output, raw, &projected, &point, topology, &matches, elapsed,
    )?;
    eprintln!(
        "selected {}: {} matching / {} generated diagrams in {elapsed:.3}s",
        raw.name(),
        matches.len(),
        generated.diagrams.len()
    );
    println!("{}", output.join("run.toml").display());
    Ok(())
}
