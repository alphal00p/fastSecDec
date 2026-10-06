//! Native serialization and the example's explicit external projection.

use std::{fmt::Write, path::Path};

use feynkit_generator::{GenerationOptions, GenerationReport, Process};
use feynkit_graph::{ExternalState, FeynmanDiagram, expressions::evaluate_overall_factor, symbols};
use feynkit_model::{Model, ParameterCard};
use idenso::representations::ColorAdjoint;
use linnet::half_edge::involution::HedgePair;
use numerica::domains::float::Complex;
use spenso::structure::representation::{Minkowski, RepName};
use symbolica::atom::{Atom, AtomCore};

use super::{
    Result,
    point::{self, Point},
    select::{Selection, Topology},
};

pub fn raw(
    output: &Path,
    model: &Model,
    original_model: &str,
    diagram: &FeynmanDiagram,
    process: &Process,
    options: &GenerationOptions,
    report: &GenerationReport,
) -> Result<()> {
    std::fs::write(output.join("source-diagram.dot"), super::source::DOT)?;
    std::fs::write(output.join("model.json"), model.to_json_pretty()?)?;
    std::fs::write(output.join("raw-diagram.json"), diagram.to_json()?)?;
    std::fs::write(output.join("raw-diagram.dot"), diagram.to_dot()?)?;
    for (name, atom) in [
        ("numerator", diagram.numerator()),
        ("numerator-prefactor", diagram.numerator_prefactor()),
        ("overall-factor", diagram.overall_factor()),
        ("projector", diagram.projector()),
    ] {
        std::fs::write(
            output.join(format!("raw-{name}.txt")),
            atom.to_canonical_string(),
        )?;
    }
    std::fs::write(
        output.join("generation.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "process": process, "options": options, "report": report,
            "source_model_blake3": blake3::hash(original_model.as_bytes()).to_hex().as_str(),
            "diagram_id": diagram.id().to_string(), "diagram_name": diagram.name(),
            "symmetry_factor_is_diagnostic_only": true,
        }))?,
    )?;
    Ok(())
}

pub fn projected(raw: &FeynmanDiagram) -> Result<FeynmanDiagram> {
    let mut ports = Vec::new();
    for (pair, _, edge) in raw.underlying().iter_edges() {
        let Some(external) = &edge.data.external else {
            continue;
        };
        if raw.model().particle_by_id(edge.data.particle)?.pdg_code != 21 {
            continue;
        }
        if external.state != ExternalState::Incoming {
            return Err("expected two incoming gluons".into());
        }
        let HedgePair::Unpaired { hedge, .. } = pair else {
            return Err("expected unsewn external ports".into());
        };
        // Public graph tensor labels, also used by native Amplitude::legs.
        ports.push((external.index, symbols::hedge_index().call((hedge.0, 1))));
    }
    ports.sort_by_key(|(index, _)| *index);
    if ports.len() != 2 || !raw.projector().is_one() {
        return Err("unexpected amputated external layout".into());
    }
    let lorentz = Minkowski {}.new_rep(4);
    let color = ColorAdjoint {}.new_rep(8);
    let color_projector = color.id(&ports[0].1, &ports[1].1);
    let numerator = super::color::contract(raw.numerator(), &color_projector)?;
    let mut projector = Atom::one();
    for ((_, index), name) in ports.into_iter().zip(point::auxiliaries()) {
        projector *= lorentz.vector(name.as_view(), [index]);
    }
    // The native bookkeeping owner evaluates the existing diagram weight.
    // No extra symmetry factor, color average or spin average is introduced.
    Ok(raw
        .clone()
        .with_numerator(numerator)?
        .with_projector(projector)
        .with_overall_factor(evaluate_overall_factor(raw.overall_factor().as_view())))
}

pub fn parameter_card(model: &Model) -> Result<ParameterCard> {
    let mut card = model.default_parameter_card()?;
    for (pdg, mass) in [(6, 172.5), (25, 125.0)] {
        let particle = model.particle_by_pdg(pdg)?;
        card.insert(
            model.parameter_by_id(particle.mass)?.name.clone(),
            Complex::new(mass, 0.0),
        );
        card.insert(
            model.parameter_by_id(particle.width)?.name.clone(),
            Complex::new(0.0, 0.0),
        );
    }
    // The SM UFO keeps the Yukawa mass as an independent external parameter.
    if !card.contains_key("ymt") {
        return Err("expected the Standard-Model external ymt parameter".into());
    }
    card.insert("ymt".into(), Complex::new(172.5, 0.0));
    Ok(card)
}

pub fn fixture(
    output: &Path,
    raw: &FeynmanDiagram,
    projected: &FeynmanDiagram,
    point: &Point,
    topology: &Topology,
    selection: &Selection,
    elapsed: f64,
) -> Result<()> {
    let dot = projected.to_dot()?;
    let loaded = FeynmanDiagram::from_dot(projected.model_arc(), &dot)?;
    if loaded.to_json()? != projected.to_json()? {
        return Err("native DOT round-trip changed the projected diagram payload".into());
    }
    std::fs::write(output.join("graph.dot"), dot)?;
    std::fs::write(
        output.join("color-projected-numerator.txt"),
        projected.numerator().to_canonical_string(),
    )?;
    let parameters = parameter_card(projected.model())?;
    let mut cli_model = projected.model().clone();
    cli_model.apply_parameter_card(&parameters)?;
    if cli_model.fingerprint() != projected.model().fingerprint() {
        return Err(
            "CLI parameter-card application would change the native DOT model identity".into(),
        );
    }
    parameters.write_json(output.join("parameters.json"))?;
    let auxiliary_names = point::auxiliaries().map(|name| name.to_canonical_string());
    let mut card = String::from(
        "# Generated by the native gghh_double_box example.\n# One diagram; unnormalized delta_ab contraction, incoming (+,+) helicities.\n[input]\ngraph = \"graph.dot\"\nmodel = \"model.json\"\nparameter_card = \"parameters.json\"\n\n[kinematics]\n",
    );
    writeln!(
        card,
        "auxiliary_momenta = {}\nproducts = [",
        serde_json::to_string(&auxiliary_names)?
    )?;
    let runtime_names = [
        "p0p0", "p0p1", "p0p2", "p0eps1", "p0eps2", "p1p1", "p1p2", "p1eps1", "p1eps2", "p2p2",
        "p2eps1", "p2eps2", "eps1eps1", "eps1eps2", "eps2eps2",
    ];
    if point.products.len() != runtime_names.len() {
        return Err("unexpected external Gram layout".into());
    }
    let mut runtime_point = String::from(
        "# Default physical point; supplied only at integration time.\n[parameters]\n",
    );
    for (product, name) in point.products.iter().zip(runtime_names) {
        writeln!(
            card,
            "  {{ left = {}, right = {}, symbol = {} }},",
            serde_json::to_string(&product.left)?,
            serde_json::to_string(&product.right)?,
            serde_json::to_string(name)?
        )?;
        writeln!(
            runtime_point,
            "{name} = {}",
            serde_json::to_string(&product.value)?
        )?;
    }
    card.push_str("]\n\n[integral]\ndimension = \"4-2*eps\"\nregulator = \"eps\"\nmeasure_multiplier = \"1\"\n\n[generation]\norder = 0\n\n[generation.coefficient_expansion]\nmethod = \"native_named\"\n");
    std::fs::write(output.join("run.toml"), card)?;
    std::fs::write(output.join("point.toml"), runtime_point)?;
    std::fs::write(
        output.join("provenance.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "format": "native-generated-gghh-double-box", "version": 2,
            "selected_diagram": raw.name(), "selected_diagram_id": raw.id().to_string(),
            "selection": topology,
            "source": selection.source,
            "matching_generation_indices": selection.channel_matches.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            "exact_native_generation_matches": selection.target_matches,
            "selection_rule": "supplied D05 with its labels, tensor numerator and routing; native canonical-key membership; six-top hexagon, central gluon, circuits [4,4,6], external boxes [g,g] and [H,H]",
            "generation_seconds": elapsed, "external_point": point,
            "external_dimension": 4, "internal_dimension": "4-2*eps", "helicities": [1, 1],
            "color_projection": "unnormalized delta_ab; no color or spin average",
            "overall_factor": "native evaluate_overall_factor(raw factor), included exactly once",
            "projected_numerator_is_raw": false,
            "color_reduction": {
                "owner": "Idenso SymbolicTensor::simplify_algebra with_cof_dimension_invariants",
                "convention": "native SU(3), T_F=1/2; unnormalized external delta_ab applied once",
                "symbolic_to_explicit_exact_check": true,
                "lorentz_and_dirac_reduction": "retained for ordinary native input contraction in D dimensions",
            },
            "couplings": "native SM card, mt=ymt=172.5, mH=125, widths zero",
            "gauge_invariant_sum": false, "threshold_admission": "caller responsibility; generation performs no threshold certification",
            "parametric_generation_complete": false, "numerical_integral_complete": false,
        }))?,
    )?;
    Ok(())
}
