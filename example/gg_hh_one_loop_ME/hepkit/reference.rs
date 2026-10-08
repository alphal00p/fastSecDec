//! Exact HEPKit one-loop reduction and native OneLOop evaluation of the shared
//! gg -> HH diagram catalogue. No reduction or scalar-master formula is copied.
//!
//! cargo run -p fastsecdec --example gghh_one_loop_reference -- INPUT OUTPUT
//! Add `--ward1` or `--ward2` to replace one incoming polarization by momentum.

use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use fastsecdec::{
    Atom, AtomCore, Kinematics, Model, ParameterCard,
    input::{GraphIntegral, default_algebra_settings},
};
use feynkit_graph::{FeynmanDiagram, symbols};
use numerica::domains::float::Complex;
use oneloop::EvaluationBackend;
use oneloopreduce::OneLoopMasters;
use serde_json::{Value, json};
use symbolica::{function, parser::ParseSettings, symbol};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn atom(text: &str) -> Result<Atom> {
    Ok(Atom::parse(
        text,
        "feynkit_graph",
        ParseSettings::default(),
    )?)
}

fn number(expression: &Atom) -> Result<Complex<f64>> {
    let value = expression.evaluate(&HashMap::<Atom, Complex<f64>>::new())?;
    if !value.re.is_finite() || !value.im.is_finite() {
        return Err(format!("nonfinite numerical expression: {expression}").into());
    }
    Ok(value)
}

fn complex(value: Complex<f64>) -> Value {
    json!({"re": value.re, "im": value.im})
}

fn load(path: &Path, ward: Option<usize>) -> Result<GraphIntegral> {
    let mut model = Model::from_json(&std::fs::read_to_string(path.join("model.json"))?)?;
    let card = ParameterCard::from_json(&std::fs::read_to_string(path.join("parameters.json"))?)?;
    model.apply_parameter_card(&card)?;
    let values = model.scalar_bindings(Some(&card), &BTreeMap::new())?;
    let mut diagram = FeynmanDiagram::from_dot(
        Arc::new(model),
        &std::fs::read_to_string(path.join("graph.dot"))?,
    )?;
    let point: Value =
        serde_json::from_str(&std::fs::read_to_string(path.join("point-exact.json"))?)?;
    let mut auxiliary = point["auxiliary_momenta"]
        .as_array()
        .ok_or("point-exact.json requires auxiliary_momenta")?
        .iter()
        .map(|name| atom(name.as_str().ok_or("invalid auxiliary name")?))
        .collect::<Result<Vec<_>>>()?;
    if let Some(index) = ward {
        // The shared exporter verifies that P(0),P(1) are the incoming gluons.
        // Substitute in the native projector before any Gram specialization.
        let slots = symbol!("gghh_reference::slots___");
        let polarization = auxiliary[index]
            .as_view()
            .get_symbol()
            .ok_or("expected a vector name")?;
        // Native tensor vectors are functions of their slots, rather than
        // scalar occurrences of the corresponding vector-name symbol.
        let projector = diagram
            .projector()
            .replace(function!(polarization, slots))
            .with(function!(symbols::external_momentum(), index, slots));
        if projector == *diagram.projector() || projector.contains_symbol(polarization) {
            return Err("Ward substitution did not replace the native polarization vector".into());
        }
        diagram = diagram.with_projector(projector);
        let _ = auxiliary.remove(index);
    }
    let mut kinematics = Kinematics::in_dimension(&Atom::var(symbol!("feynkit_graph::D")))?;
    for product in point["products"]
        .as_array()
        .ok_or("missing exact products")?
    {
        kinematics = kinematics.with_scalar_product(
            &atom(product["left"].as_str().ok_or("missing product left")?)?,
            &atom(product["right"].as_str().ok_or("missing product right")?)?,
            atom(product["value"].as_str().ok_or("missing product value")?)?,
        )?;
    }
    Ok(
        GraphIntegral::new_with_scalar_values(Arc::new(diagram), &kinematics, &values)?
            .with_auxiliary_external_momenta(&auxiliary)?,
    )
}

/// Same composition of public owners used by the HEPKit bridge: combine masters
/// before expanding the rational D dependence, then convolve complete vectors.
fn evaluate(terms: &BTreeMap<Atom, Atom>) -> Result<([Complex<f64>; 3], Vec<Value>)> {
    let dimension = symbol!("feynkit_graph::D");
    let mut sum = [Complex::new(0.0, 0.0); 3];
    let mut rows = Vec::new();
    for (master, coefficient) in terms {
        let coefficient = coefficient.cancel();
        if coefficient.is_zero() {
            continue;
        }
        if master.contains_symbol(dimension) {
            return Err("master arguments must be independent of the dimension".into());
        }
        let expansion = coefficient.series(dimension, 4, 2)?;
        if expansion.get_trailing_exponent() < 0 {
            return Err(format!("coefficient of {master} has a D=4 pole; positive master epsilon orders are required").into());
        }
        if expansion
            .terms()
            .any(|(power, value)| !power.is_integer() && !value.is_zero())
        {
            return Err("fractional powers in a reduction coefficient".into());
        }
        let mut coefficients = [Atom::Zero, Atom::Zero, Atom::Zero];
        for (order, multiplier) in [1, -2, 4].into_iter().enumerate() {
            coefficients[order] =
                expansion.coefficient((order as i64).into()).unwrap() * Atom::num(multiplier);
        }
        let numeric_coefficients = coefficients
            .iter()
            .map(number)
            .collect::<Result<Vec<_>>>()?;
        let (family, arguments) = oneloop::master_arguments(master)?;
        let arguments = arguments.iter().map(number).collect::<Result<Vec<_>>>()?;
        let mut values = [Complex::new(0.0, 0.0); 3];
        oneloop::evaluate_with_backend(
            family,
            &arguments,
            &mut values,
            EvaluationBackend::Expression,
        )?;
        if values
            .iter()
            .any(|x| !x.re.is_finite() || !x.im.is_finite())
        {
            return Err(format!("nonfinite master value: {master}").into());
        }
        for output in 0..3 {
            for order in 0..3 - output {
                sum[output] += numeric_coefficients[order] * values[output + order];
            }
        }
        rows.push(json!({
            "master": master.to_canonical_string(),
            "coefficient_D": coefficient.to_canonical_string(),
            "coefficient_epsilon_0_1_2": coefficients.map(|x| x.to_canonical_string()),
            "master_finite_simple_double_pole": values.map(complex),
        }));
    }
    Ok((sum, rows))
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(
        args.next()
            .ok_or("expected INPUT OUTPUT [--ward1|--ward2]")?,
    );
    let output = PathBuf::from(args.next().ok_or("expected OUTPUT")?);
    let ward = match args.next().as_deref() {
        None => None,
        Some("--ward1") => Some(0),
        Some("--ward2") => Some(1),
        Some(other) => return Err(format!("unknown option {other}").into()),
    };
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    // Register native vector semantics before parsing saved tensor expressions.
    let _ = [
        spenso::vector_symbol!("gghh::eps1"),
        spenso::vector_symbol!("gghh::eps2"),
    ];
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(input.join("manifest.json"))?)?;
    let diagrams = manifest["diagrams"]
        .as_array()
        .ok_or("missing diagram catalogue")?;
    std::fs::create_dir_all(&output)?;
    std::fs::copy(
        input.join("manifest.json"),
        output.join("input-manifest.json"),
    )?;
    let started = Instant::now();
    let mut combined = BTreeMap::<Atom, Atom>::new();
    let mut contributions = Vec::new();
    let mut absolute_contribution_sum = [0.0_f64; 3];
    for diagram in diagrams {
        let name = diagram["name"].as_str().ok_or("missing diagram name")?;
        let directory = diagram["directory"]
            .as_str()
            .ok_or("missing diagram directory")?;
        eprintln!("{name}: native contraction and one-loop reduction");
        let stage = Instant::now();
        let graph = load(&input.join(directory), ward)?;
        let numerator = graph.scalar_numerator(&default_algebra_settings())?;
        let powers = graph
            .powers()
            .iter()
            .map(|p| i32::try_from(*p))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let reduction = oneloopreduce::reduce_family(graph.family(), &powers, &numerator)?;
        let mut terms = BTreeMap::<Atom, Atom>::new();
        for (coefficient, master) in reduction.terms {
            let master = OneLoopMasters.symbol_with_scale(&master, &Atom::one());
            *terms.entry(master).or_insert(Atom::Zero) += coefficient;
        }
        let (value, rows) = evaluate(&terms)?;
        for (scale, term) in absolute_contribution_sum.iter_mut().zip(value) {
            *scale += term.re.hypot(term.im);
        }
        let expression = terms
            .iter()
            .map(|(master, coefficient)| master * coefficient)
            .sum::<Atom>();
        std::fs::write(
            output.join(format!("{name}.masters.txt")),
            expression.to_canonical_string(),
        )?;
        std::fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_string_pretty(&json!({
                "diagram": name, "seconds": stage.elapsed().as_secs_f64(),
                "source_graph_blake3": blake3::hash(&std::fs::read(input.join(directory).join("graph.dot"))?).to_hex().to_string(),
                "numerator": numerator.to_canonical_string(), "terms": rows,
                "native_finite_simple_double_pole": value.map(complex),
            }))?,
        )?;
        contributions.push(json!({"diagram":name,"kind":diagram["kind"],"native_finite_simple_double_pole":value.map(complex)}));
        for (master, coefficient) in terms {
            *combined.entry(master).or_insert(Atom::Zero) += coefficient;
        }
        eprintln!(
            "{name}: native finite {} + i*{}, elapsed {:.3}s",
            value[0].re,
            value[0].im,
            stage.elapsed().as_secs_f64()
        );
    }
    let (value, rows) = evaluate(&combined)?;
    let expression = combined
        .iter()
        .map(|(master, coefficient)| master * coefficient.cancel())
        .sum::<Atom>();
    std::fs::write(
        output.join("amplitude.masters.txt"),
        expression.to_canonical_string(),
    )?;
    let physical = value[0] / (16.0 * std::f64::consts::PI.powi(2));
    let square = physical.re * physical.re + physical.im * physical.im;
    let cancellation_scale = absolute_contribution_sum
        .into_iter()
        .fold(1.0_f64, f64::max);
    let pole_residual = value[1]
        .re
        .hypot(value[1].im)
        .max(value[2].re.hypot(value[2].im));
    let poles_cancel = pole_residual <= 1e-10 * cancellation_scale;
    let ward_residual = value[0].re.hypot(value[0].im);
    let ward_pass = ward.map(|_| ward_residual <= 1e-9 * cancellation_scale);
    let result = json!({
        "method":"HEPKit native one-loop reduction and OneLOop scalar masters",
        "ward_replaced_incoming_index":ward, "diagram_count":diagrams.len(),
        "seconds":started.elapsed().as_secs_f64(), "master_terms":rows,
        "contributions":contributions,"native_finite_simple_double_pole":value.map(complex),
        "physical_color_coefficient":complex(physical),"color_summed_fixed_helicity_squared":8.0*square,
        "color_averaged_fixed_helicity_squared":square/8.0,
        "normalization":manifest["normalization"],
        "master_scale_squared":1,"master_measure_multiplier":"gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)",
        "checks": {
            "sum_absolute_diagram_finite_simple_double_pole":absolute_contribution_sum,
            "cancellation_scale":cancellation_scale,
            "pole_residual":pole_residual,"poles_cancel":poles_cancel,
            "pole_relative_tolerance":1e-10,
            "ward_finite_residual":ward.map(|_|ward_residual),"ward_pass":ward_pass,
            "ward_relative_tolerance":ward.map(|_|1e-9),
        },
    });
    std::fs::write(
        output.join("result.json"),
        serde_json::to_string_pretty(&result)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    if !poles_cancel || ward_pass == Some(false) {
        return Err(
            "full-amplitude pole cancellation or requested Ward check failed; inspect result.json"
                .into(),
        );
    }
    Ok(())
}

pub fn main() -> Result<()> {
    std::thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(|| run().map_err(|e| e.to_string()))?
        .join()
        .map_err(|_| "native reference thread panicked")?
        .map_err(Into::into)
}
