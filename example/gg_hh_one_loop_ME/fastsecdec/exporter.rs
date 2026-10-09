//! Reuse the native graph generator, color contraction and external-state owner.
use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Instant};

use feynkit_generator::{GenerationFilter, GenerationOptions, NumeratorGrouping, Process};
use feynkit_model::Model;
use numerica::domains::float::Complex;
use serde_json::json;
use symbolica::atom::{Atom, AtomCore};

use crate::{Result, export, point};

pub fn run() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let output = PathBuf::from(args.next().ok_or("expected a fresh output directory")?);
    let sqrt_s = match args.next() {
        None => 300,
        Some(option) if option == "--sqrt-s" => args
            .next()
            .ok_or("--sqrt-s requires an integer energy in GeV")?
            .to_str()
            .ok_or("--sqrt-s requires an integer energy in GeV")?
            .parse::<u32>()?,
        Some(_) => return Err("usage: gghh_one_loop_me OUTPUT [--sqrt-s INTEGER_GEV]".into()),
    };
    if args.next().is_some() || output.exists() || sqrt_s <= 250 {
        return Err("supply a fresh output directory and sqrt(s) above 250 GeV".into());
    }
    let mut model = Model::standard_model();
    let mut parameters = export::parameter_card(&model)?;
    for (name, value) in [
        ("aS", 0.118),
        ("Gf", 1.16639e-5),
        ("aEWM1", 132.507),
        ("MZ", 91.188),
    ] {
        parameters.insert(name.into(), Complex::new(value, 0.0));
    }
    for particle in model.particles() {
        let name = &model.parameter_by_id(particle.width)?.name;
        if parameters.contains_key(name) {
            parameters.insert(name.clone(), Complex::new(0.0, 0.0));
        }
    }
    for name in ["ymb", "ymc", "yms", "ymu", "ymd"] {
        if parameters.contains_key(name) {
            parameters.insert(name.into(), Complex::new(0.0, 0.0));
        }
    }
    model.apply_parameter_card(&parameters)?;
    let veto = model
        .particles()
        .iter()
        .filter(|particle| ![6, 21, 25].contains(&particle.pdg_code.abs()))
        .map(|particle| particle.pdg_code.into())
        .collect();
    let process = Process::new([21_i64, 21], [25_i64, 25]).with_filters(veto, None, vec![]);
    // Keep all labeled external permutations. Native symmetry/fermion factors
    // remain on each generated diagram; no hand-counted multiplicity is added.
    let options = GenerationOptions::default()
        .with_loop_count(1, 1)?
        .threads(1)
        .symmetrize_initial(false)
        .symmetrize_final(false)
        .allow_zero_flow_edges(true)
        .filter_zero_color(true)
        .with_graph_filter(GenerationFilter::CouplingOrders(BTreeMap::from([
            ("QCD".into(), (2, Some(2))),
            ("QED".into(), (2, Some(2))),
        ])))
        .numerator_grouping(NumeratorGrouping::None)
        .projector(Atom::one());
    let model = Arc::new(model);
    let started = Instant::now();
    let generated = process.generate_diagrams(Arc::clone(&model), &options)?;
    if !generated.report.completed {
        return Err("native diagram generation did not complete".into());
    }
    generated.validate_groups()?;
    eprintln!(
        "native generator completed {} one-loop diagrams in {:.3}s",
        generated.diagrams.len(),
        started.elapsed().as_secs_f64()
    );
    std::fs::create_dir_all(&output)?;
    let mut entries = Vec::new();
    let mut physical_point = None;
    for (index, raw) in generated.diagrams.iter().enumerate() {
        if raw.loop_momentum_basis().loop_edges.len() != 1 {
            return Err("generated catalogue contains a non-one-loop diagram".into());
        }
        let mut internal_pdgs = Vec::new();
        for (_, _, edge) in raw.edges() {
            if edge.external.is_none() && !edge.is_dummy {
                internal_pdgs.push(raw.model().particle_by_id(edge.particle)?.pdg_code.abs());
            }
        }
        internal_pdgs.sort_unstable();
        let kind = match internal_pdgs.as_slice() {
            [6, 6, 6, 25] => "triangle",
            [6, 6, 6, 6] => "box",
            _ => {
                return Err(format!(
                    "unexpected full top-loop topology {}: {internal_pdgs:?}",
                    raw.name()
                )
                .into());
            }
        };
        let directory = format!("diagram_{index:02}_{kind}");
        let path = output.join(&directory);
        std::fs::create_dir(&path)?;
        std::fs::write(path.join("raw-diagram.json"), raw.to_json()?)?;
        let projected = export::projected(raw)?;
        let projected = projected
            .clone()
            .with_overall_factor(projected.overall_factor() * Atom::num((1, 8)));
        let point = if sqrt_s == 300 {
            point::Point::new(&projected)?
        } else {
            point::Point::with_sqrt_s(&projected, sqrt_s)?
        };
        let shared_point = shared_point(&point, sqrt_s)?;
        if physical_point
            .as_ref()
            .is_some_and(|previous| previous != &shared_point)
        {
            return Err("diagram catalogue disagrees on the shared physical point".into());
        }
        physical_point = Some(shared_point);
        export::fixture(&path, &projected, &point)?;
        let mut exact = serde_json::to_value(&point)?;
        exact["auxiliary_momenta"] = json!(point::auxiliaries().map(|a| a.to_canonical_string()));
        std::fs::write(
            path.join("point-exact.json"),
            serde_json::to_vec_pretty(&exact)?,
        )?;
        let card = std::fs::read_to_string(path.join("run.toml"))?
            .replace(
                "Generated by the native gghh_double_box example.",
                "Generated by the native gghh_one_loop_me example.",
            )
            .replace(
                "One diagram; unnormalized delta_ab contraction",
                "One term in the complete top-loop amplitude; delta_ab/8 projection",
            )
            .replace(
                "measure_multiplier = \"1\"",
                "measure_multiplier = \"gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)\"",
            );
        let card = format!(
            "{card}\n# Extract the native loop-independent Higgs propagator exactly once.\n[generation.family_preparation.SingleTerm]\nmax_states = 32\n"
        );
        std::fs::write(path.join("run.toml"), card)?;
        entries.push(json!({
            "name":raw.name(), "id":raw.id().to_string(), "directory":directory, "kind":kind,
            "internal_pdgs":internal_pdgs,
            "raw_overall_factor":raw.overall_factor().to_canonical_string(),
            "projected_overall_factor":projected.overall_factor().to_canonical_string(),
            "seed":78139_u64 + index as u64 * 104729,
        }));
        eprintln!("exported {} {kind}: {}", index, raw.name());
    }
    if entries.iter().filter(|x| x["kind"] == "triangle").count() != 2
        || entries.iter().filter(|x| x["kind"] == "box").count() != 6
    {
        return Err("expected the complete two-triangle/six-box labeled top-loop catalogue".into());
    }
    let manifest = json!({
        "format_version":1, "scope":"complete one-loop top-quark contribution to gg -> hh, incoming ++",
        "model_fingerprint":model.fingerprint().to_string(), "process":process, "generation_options":options,
        "generation_report":generated.report, "diagrams":entries,
        "normalization":{
            "color_projector":"delta_ab/8", "amplitude_definition":"M^{ab}_{++}=delta^{ab} A_{++}",
            "native_measure":"d^D k/(i*pi^(D/2))",
            "measure_multiplier":"gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)",
            "mu_squared":1.0, "physical_finite_multiplier":"1/(16*pi^2)",
            "raw_delta_projection_multiplier":8, "color_summed_helicity_squared_multiplier":8,
            "initial_spin_average":false,"initial_color_average":false,
            "extra_R2_added":false,"dimension":"4-2*eps"
        },
        "point":{"sqrt_s":f64::from(sqrt_s),"MH":125.0,"MT":172.5,"ymt":172.5,"cos_theta":0.8,"aS":0.118,"Gf":1.16639e-5,"aEWM1":132.507,"MZ":91.188,"widths":0.0},
    });
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    std::fs::write(
        output.join("physical-point.json"),
        serde_json::to_vec_pretty(&physical_point.ok_or("missing physical point")?)?,
    )?;
    Ok(())
}

/// All three reference methods consume momenta from the same native point.
/// The established JSON supplies only shared model/convention descriptions.
fn shared_point(point: &point::Point, sqrt_s: u32) -> Result<serde_json::Value> {
    if point.physical_momenta.keys().copied().collect::<Vec<_>>() != [0, 1, 2, 3] {
        return Err("expected incoming indices 0,1 and outgoing indices 2,3".into());
    }
    let momenta = point
        .physical_momenta
        .values()
        .map(|vector| {
            vector
                .iter()
                .map(|x| point::real_value(x))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    let mut result: serde_json::Value = serde_json::from_str(include_str!("../point.json"))?;
    result["sqrt_s_GeV"] = json!(f64::from(sqrt_s));
    result["physical_momenta_GeV"] = json!(momenta);
    result["exact_outgoing_spatial_components"] = json!(&point.physical_momenta[&2][1..]);
    Ok(result)
}
