//! A fixture adapter around native HEPKit routing, kinematics and Symanzik APIs.
use super::{
    Result,
    records::{self, Record, Reference},
};
use fastsecdec::{
    Atom, AtomCore, EdgeId, FeynmanDiagram, IntegralFamily, Kinematics, Model,
    input::{GraphIntegral, default_algebra_settings},
    parametric::ScalarParametricIntegral,
};
use feynkit_graph::symbols;
use feynkit_kinematics::FourMomentum;
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
};
use symbolica::{
    atom::Symbol,
    domains::{atom::AtomField, rational::Rational},
    symbol,
};

static MODEL: LazyLock<Arc<Model>> =
    LazyLock::new(|| Arc::new(Model::from_json(records::MODEL).unwrap()));

#[derive(Serialize)]
pub struct Summary {
    pub name: String,
    pub source_archive: &'static str,
    pub archive_sha256: &'static str,
    pub rar_sha256: &'static str,
    pub yaml_sha256: &'static str,
    pub yaml_record_sha256: String,
    pub loops: usize,
    pub internal_edges: usize,
    pub external_edges: usize,
    pub vertices: usize,
    pub dependent_external_edge: usize,
    pub momentum_roundoff_reconciliation_max: f64,
    pub archived_shift_residual_max: f64,
    pub archived_mass_squared_residual_max: f64,
    pub denominator_matches: usize,
    pub momentum_signs: Vec<i32>,
    pub native_u_terms: usize,
    pub native_f_terms: usize,
    pub f_positive_coefficients: usize,
    pub f_negative_coefficients: usize,
    pub u: String,
    pub f: String,
    pub measure_multiplier: String,
    pub exact_external_vectors: Vec<[String; 4]>,
    pub references: Vec<Reference>,
    pub generation_performed: bool,
}
pub struct Imported {
    pub graph: GraphIntegral,
    pub vectors: Vec<FourMomentum<Atom>>,
    pub summary: Summary,
}

pub fn real(value: &Atom) -> Result<f64> {
    Ok(value.evaluate(&HashMap::<Atom, f64>::new())?)
}
fn exact_vector(value: &[f64; 4]) -> Result<FourMomentum<Atom>> {
    let values = value
        .iter()
        .map(|v| Rational::try_from(*v).map(Atom::num))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(FourMomentum::from_args(
        values[0].clone(),
        values[1].clone(),
        values[2].clone(),
        values[3].clone(),
    ))
}
fn zero_vector() -> FourMomentum<Atom> {
    FourMomentum::from_args(Atom::Zero, Atom::Zero, Atom::Zero, Atom::Zero)
}
fn difference(a: &Atom, b: &Atom) -> bool {
    a == b || (a - b).expand().is_zero()
}
pub fn measure(loops: usize, epsilon: Atom) -> Atom {
    let pi = Atom::var(Symbol::PI);
    (Atom::i() * (Atom::num(4) * &pi).pow(epsilon) / (Atom::num(16) * pi.pow(2))).pow(loops)
}

pub fn import(record: &Record) -> Result<Imported> {
    let diagram = Arc::new(FeynmanDiagram::from_dot(Arc::clone(&MODEL), record.dot())?);
    diagram.validate()?;
    let basis = diagram.loop_momentum_basis();
    basis.validate(&diagram)?;
    let external_count = record.external_kinematics.len();
    let propagators = record
        .loop_lines
        .iter()
        .flat_map(|line| line.propagators.iter().map(move |p| (&line.signature, p)))
        .collect::<Vec<_>>();
    let shifts = record.shifts();
    if diagram.loop_count() != record.n_loops
        || basis.external_edges.len() != external_count
        || propagators.len() != shifts.len()
        || basis.dependent_externals.len() != 1
    {
        return Err("archived/native graph count mismatch".into());
    }
    if basis.external_edges != (0..external_count).map(EdgeId).collect::<Vec<_>>() {
        return Err("fixture external coordinate order changed".into());
    }
    let dependent = basis.dependent_externals[0].0;
    let original = record
        .external_kinematics
        .iter()
        .map(exact_vector)
        .collect::<Result<Vec<_>>>()?;
    let mut vectors = original.clone();
    vectors[dependent] = -original
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != dependent)
        .fold(zero_vector(), |sum, (_, p)| sum + p.clone());
    let mut conservation_residual = 0_f64;
    for (actual, archived) in vectors[dependent]
        .components()
        .into_iter()
        .zip(original[dependent].components())
    {
        conservation_residual = conservation_residual.max(real(&(actual - archived))?.abs());
    }
    if conservation_residual > 5e-14 {
        return Err("archived external momenta fail conservation at printed precision".into());
    }
    let mut kinematics = Kinematics::in_dimension(&Atom::var(symbol!("ltd::D")))?;
    for (i, p) in vectors.iter().enumerate().filter(|(i, _)| *i != dependent) {
        for (j, q) in vectors
            .iter()
            .enumerate()
            .skip(i)
            .filter(|(j, _)| *j != dependent)
        {
            kinematics = kinematics.with_scalar_product(
                &symbols::external_momentum().call(i),
                &symbols::external_momentum().call(j),
                p.dot(q),
            )?;
        }
    }
    let graph = GraphIntegral::new(Arc::clone(&diagram), &kinematics)?
        .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), record.mass())]))?;
    if !graph
        .scalar_numerator(&default_algebra_settings())?
        .is_one()
        || graph.powers().iter().any(|p| *p != 1)
        || graph.propagator_edges()
            != (external_count..external_count + propagators.len())
                .map(EdgeId)
                .collect::<Vec<_>>()
        || diagram.underlying().n_edges() != external_count + propagators.len()
        || diagram.underlying().n_nodes() + record.n_loops != propagators.len() + 1
    {
        return Err("native scalar graph, power or Linnet incidence mismatch".into());
    }
    let loops = graph.family().loop_momenta();
    let external = basis
        .external_edges
        .iter()
        .map(|edge| basis.route_expression(&symbols::momentum().call(edge.0)))
        .collect::<Vec<_>>();
    let mut expected_denominators = Vec::new();
    let mut shift_residual = 0_f64;
    let mut mass_residual = 0_f64;
    let mut signs = Vec::new();
    for (index, ((signature, archived), shift)) in propagators.iter().zip(&shifts).enumerate() {
        if signature.len() != loops.len() || shift.len() != external_count {
            return Err("archived signature dimension mismatch".into());
        }
        let momentum: Atom = loops
            .iter()
            .zip(*signature)
            .chain(external.iter().zip(shift))
            .map(|(p, coefficient)| p * Atom::num(*coefficient))
            .sum();
        let native = basis.route_expression(&symbols::momentum().call(external_count + index));
        let sign = if difference(&native, &momentum) {
            1
        } else if difference(&native, &(-&momentum)) {
            -1
        } else {
            return Err(format!(
                "{} denominator {index}: native momentum {native} differs from archived {momentum}",
                record.name
            )
            .into());
        };
        signs.push(sign);
        let expected_shift = vectors
            .iter()
            .zip(shift)
            .fold(zero_vector(), |sum, (p, c)| sum + p.clone() * Atom::num(*c));
        for (component, target) in expected_shift.components().into_iter().zip(archived.q) {
            shift_residual = shift_residual.max((real(component)? - target).abs());
        }
        mass_residual =
            mass_residual.max((real(&record.mass().pow(2))? - archived.m_squared).abs());
        let denominator = graph
            .family()
            .kinematics()
            .scalar_product(&momentum, &momentum)?
            - record.mass().pow(2);
        if !difference(&denominator, &graph.family().denominators()[index]) {
            return Err("native/archive quadratic denominator mismatch".into());
        }
        expected_denominators.push(denominator);
    }
    if shift_residual > 5e-14 || mass_residual > 5e-15 {
        return Err(
            "archive/native routing or mass reconciliation exceeds printed precision".into(),
        );
    }
    let archive_family = IntegralFamily::new(
        loops.to_vec(),
        graph.family().external_momenta().to_vec(),
        expected_denominators,
        graph.family().kinematics(),
    )?;
    let parameters = (0..propagators.len())
        .map(|i| Atom::var(symbol!(&format!("ltd::x{i}"))))
        .collect::<Vec<_>>();
    let (u, f) = graph.family().symanzik(&parameters)?;
    let (expected_u, expected_f) = archive_family.symanzik(&parameters)?;
    if !difference(&u, &expected_u) || !difference(&f, &expected_f) {
        return Err("native graph/archive-family U/F mismatch".into());
    }
    let field = AtomField {
        statistical_zero_test: false,
        ..AtomField::new()
    };
    let up = u.to_polynomial_in_vars_with_field::<u32>(&parameters, &field);
    let fp = f.to_polynomial_in_vars_with_field::<u32>(&parameters, &field);
    let mut u_terms = 0;
    let mut f_terms = 0;
    let mut f_positive = 0;
    let mut f_negative = 0;
    for term in &up {
        if term.exponents.iter().sum::<u32>() != record.n_loops as u32
            || !term.coefficient.is_positive().is_true()
        {
            return Err("native U homogeneity/positive-coefficient check failed".into());
        }
        u_terms += 1;
    }
    for term in &fp {
        if term.exponents.iter().sum::<u32>() != record.n_loops as u32 + 1 {
            return Err("native F homogeneity check failed".into());
        }
        f_positive += usize::from(term.coefficient.is_positive().is_true());
        f_negative += usize::from((-term.coefficient).is_positive().is_true());
        f_terms += 1;
    }
    if f_positive + f_negative != f_terms {
        return Err("native F retains an unresolved or sign-unknown coefficient".into());
    }
    let epsilon = Atom::var(symbol!("ltd::eps"));
    let multiplier = measure(record.n_loops, epsilon.clone());
    let graph = graph.with_measure_multiplier(multiplier.clone());
    let parametric =
        ScalarParametricIntegral::from_graph(&graph, parameters, Atom::num(4) - 2 * epsilon)?;
    if !difference(parametric.u(), &u) || !difference(parametric.f(), &f) {
        return Err("native parametric input changed U/F".into());
    }
    let summary = Summary {
        name: record.name.clone(),
        source_archive: records::ARCHIVE_URL,
        archive_sha256: records::ARCHIVE_SHA256,
        rar_sha256: records::RAR_SHA256,
        yaml_sha256: records::YAML_SHA256,
        yaml_record_sha256: record.raw_yaml_sha256.clone(),
        loops: record.n_loops,
        internal_edges: propagators.len(),
        external_edges: external_count,
        vertices: diagram.underlying().n_nodes(),
        dependent_external_edge: dependent,
        momentum_roundoff_reconciliation_max: conservation_residual,
        archived_shift_residual_max: shift_residual,
        archived_mass_squared_residual_max: mass_residual,
        denominator_matches: propagators.len(),
        momentum_signs: signs,
        native_u_terms: u_terms,
        native_f_terms: f_terms,
        f_positive_coefficients: f_positive,
        f_negative_coefficients: f_negative,
        u: u.to_canonical_string(),
        f: f.to_canonical_string(),
        measure_multiplier: multiplier.to_canonical_string(),
        exact_external_vectors: vectors
            .iter()
            .map(|p| p.components().map(Atom::to_canonical_string))
            .collect(),
        references: record.references(),
        generation_performed: false,
    };
    Ok(Imported {
        graph,
        vectors,
        summary,
    })
}
