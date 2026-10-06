//! Export the supplied Standard-Model s-channel top/gluon double box.
//!
//! `cargo run --release -p fastsecdec --example gghh_double_box -- SM.json NEW_OUTPUT_DIR`
//! writes native input files; run the emitted card with the ordinary CLI.
//! This is one diagram with an explicit color/helicity projection, not a
//! gauge-invariant sum or a cross section.

mod color;
mod export;
mod point;
mod select;
mod source;

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
    let requested_model = Model::from_json(&model_bytes)?;
    let (raw, source) = source::physical_diagram(&requested_model)?;
    let model = raw.model_arc();
    let topology =
        select::double_box(&raw)?.ok_or("D05 is not the requested s-channel double box")?;
    let target_key = raw.canonical_key()?;
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
    let mut target_matches = Vec::new();
    for (index, diagram) in generated.diagrams.iter().enumerate() {
        if let Some(topology) = select::double_box(diagram)? {
            matches.push((index, topology));
            if diagram.canonical_key()? == target_key {
                target_matches.push(select::GeneratedMatch {
                    index,
                    name: diagram.name().to_owned(),
                    id: diagram.id().to_string(),
                });
            }
        }
    }
    if target_matches.is_empty() {
        return Err(
            "supplied D05 has no exact native colored-topology match in the generated set".into(),
        );
    }
    let elapsed = start.elapsed().as_secs_f64();
    std::fs::create_dir_all(&output)?;
    // Persist raw evidence before projection or any potentially expensive algebra.
    export::raw(
        &output,
        &model,
        &model_bytes,
        &raw,
        &process,
        &options,
        &generated.report,
    )?;
    let projected = export::projected(&raw)?;
    let point = point::Point::new(&projected)?;
    export::fixture(
        &output,
        &raw,
        &projected,
        &point,
        &topology,
        &select::Selection {
            source,
            channel_matches: matches,
            target_matches,
        },
        elapsed,
    )?;
    eprintln!(
        "selected supplied {} with exact native generated membership / {} generated diagrams in {elapsed:.3}s",
        raw.name(),
        generated.diagrams.len()
    );
    println!("Generated run.toml and point.toml in the requested output directory");
    Ok(())
}
