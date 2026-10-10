//! Native Ward cards and exact equivalence to pre-contraction projector replacement.
//! Reuses HEPKit routing, FourMomentum dot products and Gaussian parameterization.
use crate::Result;
use fastsecdec::{
    Atom, AtomCore, FeynmanDiagram, Kinematics, Model, ParameterCard,
    input::GraphIntegral,
    parametric::{FamilyPreparationPolicy, ParametricIntegrand},
};
use feynkit_graph::symbols;
use feynkit_kinematics::FourMomentum;
use numerica::domains::{float::Complex, rational::Rational};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::Arc,
};
use symbolica::{function, parser::ParseSettings, symbol};
fn atom(s: &str) -> Result<Atom> {
    Ok(Atom::parse(s, "feynkit_graph", ParseSettings::default())?)
}
fn momentum(v: &[Atom]) -> Result<FourMomentum<Atom>> {
    let [t, x, y, z] = v else {
        return Err("native point requires four components per vector".into());
    };
    Ok(FourMomentum::from_args(
        t.clone(),
        x.clone(),
        y.clone(),
        z.clone(),
    ))
}
fn kin(point: &Value) -> Result<Kinematics> {
    let mut k = Kinematics::in_dimension(&Atom::var(symbol!("feynkit_graph::D")))?;
    for p in point["products"].as_array().ok_or("products")? {
        k = k.with_scalar_product(
            &atom(p["left"].as_str().ok_or("left")?)?,
            &atom(p["right"].as_str().ok_or("right")?)?,
            atom(p["value"].as_str().ok_or("value")?)?,
        )?;
    }
    Ok(k)
}
fn density(g: &GraphIntegral) -> Result<(Atom, usize)> {
    let eps = symbol!("feynkit_graph::eps");
    let (p, _) = ParametricIntegrand::from_graph_prepared(
        g,
        (0..g.powers().len())
            .map(|i| symbol!(&format!("ward_proof::x{i}")))
            .collect(),
        eps,
        Atom::num(4) - Atom::num(2) * Atom::var(eps),
        FamilyPreparationPolicy::SingleTerm { max_states: 32 },
    )?;
    let sum = p
        .terms()
        .iter()
        .map(|t| {
            let mut a = t.prefactor().clone();
            for (x, power) in p.parameters().iter().zip(t.monomial_powers()) {
                a *= Atom::var(*x).pow(power);
            }
            for f in t.factors() {
                a *= f.polynomial().pow(f.exponent());
            }
            a
        })
        .sum::<Atom>();
    Ok((sum, p.terms().len()))
}
pub fn run(input: &Path, output: &Path) -> Result<()> {
    if output.exists() {
        return Err("Ward preparation requires a fresh output directory".into());
    }
    std::fs::create_dir_all(output)?;
    let _ = [
        spenso::vector_symbol!("gghh::eps1"),
        spenso::vector_symbol!("gghh::eps2"),
    ];
    let manifest: Value = serde_json::from_slice(&std::fs::read(input.join("manifest.json"))?)?;
    let runtime_names = [
        "p0p0", "p0p1", "p0p2", "p0eps1", "p0eps2", "p1p1", "p1p2", "p1eps1", "p1eps2", "p2p2",
        "p2eps1", "p2eps2", "eps1eps1", "eps1eps2", "eps2eps2",
    ];
    let mut rows = vec![];
    if manifest["diagrams"].as_array().is_none_or(|x| x.len() != 8) {
        return Err("Ward preparation requires the complete eight-diagram catalogue".into());
    }
    for ward in 1..=2 {
        let dest = output.join(format!("ward{ward}"));
        std::fs::create_dir(&dest)?;
        let mut m = manifest.clone();
        m["ward"] = json!(ward);
        for (i, d) in m["diagrams"].as_array_mut().unwrap().iter_mut().enumerate() {
            d["seed"] = json!(178139_u64 + i as u64 * 104729 + (ward - 1) * 1000003);
        }
        std::fs::write(dest.join("manifest.json"), serde_json::to_vec_pretty(&m)?)?;
    }
    for d in manifest["diagrams"].as_array().ok_or("diagrams")? {
        let directory = d["directory"].as_str().ok_or("directory")?;
        let source = input.join(directory);
        let original: Value =
            serde_json::from_slice(&std::fs::read(source.join("point-exact.json"))?)?;
        if original["external_index_by_coordinate"] != json!([0, 1, 2, 3]) {
            return Err(
                "native P(0),P(1),P(2),P(3) must identify the original external ordering".into(),
            );
        }
        let mut model = Model::from_json(&std::fs::read_to_string(source.join("model.json"))?)?;
        let card =
            ParameterCard::from_json(&std::fs::read_to_string(source.join("parameters.json"))?)?;
        model.apply_parameter_card(&card)?;
        let values = model.scalar_bindings(Some(&card), &BTreeMap::new())?;
        let diagram = FeynmanDiagram::from_dot(
            Arc::new(model),
            &std::fs::read_to_string(source.join("graph.dot"))?,
        )?;
        let auxiliaries = original["auxiliary_momenta"]
            .as_array()
            .ok_or("missing auxiliary momenta")?
            .iter()
            .map(|v| atom(v.as_str().ok_or("invalid auxiliary name")?))
            .collect::<Result<Vec<_>>>()?;
        if auxiliaries.len() != 2 {
            return Err("expected the two original polarization vectors".into());
        }
        let physical = (0..4)
            .map(|i| {
                let v = original["physical_momenta"][i.to_string()]
                    .as_array()
                    .ok_or("missing physical momentum components")?
                    .iter()
                    .map(|v| atom(v.as_str().ok_or("invalid exact component")?))
                    .collect::<Result<Vec<_>>>()?;
                momentum(&v)
            })
            .collect::<Result<Vec<_>>>()?;
        let pol = original["polarization_components"]
            .as_array()
            .ok_or("missing polarization components")?
            .iter()
            .map(|v| {
                let a = v
                    .as_array()
                    .ok_or("invalid polarization component array")?
                    .iter()
                    .map(|z| {
                        Ok(Atom::num(Rational::try_from(
                            z[0].as_f64().ok_or("invalid real component")?,
                        )?) + Atom::i()
                            * Atom::num(Rational::try_from(
                                z[1].as_f64().ok_or("invalid imaginary component")?,
                            )?))
                    })
                    .collect::<Result<Vec<_>>>()?;
                momentum(&a)
            })
            .collect::<Result<Vec<_>>>()?;
        if pol.len() != 2 {
            return Err("expected two native wavefunctions".into());
        }
        let names = (0..3)
            .map(|i| symbols::external_momentum().call(i))
            .chain(auxiliaries.iter().cloned())
            .collect::<Vec<_>>();
        let pairs = names
            .iter()
            .enumerate()
            .flat_map(|(i, a)| names[i..].iter().map(move |b| (a, b)))
            .collect::<Vec<_>>();
        let products = original["products"]
            .as_array()
            .ok_or("missing native Gram products")?;
        if products.len() != runtime_names.len() {
            return Err("unexpected native external Gram layout".into());
        }
        for (product, (left, right)) in products.iter().zip(pairs) {
            if atom(product["left"].as_str().ok_or("product left")?)? != *left
                || atom(product["right"].as_str().ok_or("product right")?)? != *right
            {
                return Err("native Gram ordering differs from the exported runtime layout".into());
            }
        }
        for ward in 0..2 {
            let mut vectors: BTreeMap<Atom, FourMomentum<Atom>> = BTreeMap::new();
            for (i, p) in physical.iter().enumerate().take(3) {
                vectors.insert(symbols::external_momentum().call(i), p.clone());
            }
            for i in 0..2 {
                vectors.insert(
                    auxiliaries[i].clone(),
                    if i == ward {
                        physical[i].clone()
                    } else {
                        pol[i].clone()
                    },
                );
            }
            let mut point = original.clone();
            point["ward_replaced_incoming_index"] = json!(ward);
            // Keep the diagnostic vector components consistent with the new
            // Gram values; the source physical momenta themselves never change.
            point["polarization_components"][ward] = json!(
                physical[ward]
                    .components()
                    .iter()
                    .map(|a| {
                        a.evaluate(&HashMap::<Atom, Complex<f64>>::new())
                            .map(|z| [z.re, z.im])
                    })
                    .collect::<std::result::Result<Vec<_>, _>>()?
            );
            for p in point["products"].as_array_mut().unwrap() {
                let left = atom(p["left"].as_str().unwrap())?;
                let right = atom(p["right"].as_str().unwrap())?;
                p["value"] = json!(
                    vectors[&left]
                        .dot(&vectors[&right])
                        .expand()
                        .to_canonical_string()
                );
            }
            let mut card = std::fs::read_to_string(source.join("point.toml"))?;
            for (p, name) in point["products"]
                .as_array()
                .unwrap()
                .iter()
                .zip(runtime_names)
            {
                if matches!(name, "p0p0" | "p1p1") {
                    assert!(atom(p["value"].as_str().unwrap())?.is_zero());
                    continue;
                }
                let v = atom(p["value"].as_str().unwrap())?
                    .evaluate(&HashMap::<Atom, Complex<f64>>::new())?;
                assert!(v.re.is_finite() && v.im == 0.);
                let prefix = format!("{name} = ");
                if card
                    .lines()
                    .filter(|line| line.starts_with(&prefix))
                    .count()
                    != 1
                {
                    return Err(
                        format!("expected exactly one native runtime binding for {name}").into(),
                    );
                }
                let lines = card
                    .lines()
                    .map(|line| {
                        if line.starts_with(&prefix) {
                            format!("{name} = {:?}", v.re)
                        } else {
                            line.to_owned()
                        }
                    })
                    .collect::<Vec<_>>();
                card = lines.join("\n") + "\n";
            }
            let dest = output.join(format!("ward{}", ward + 1)).join(directory);
            std::fs::create_dir_all(&dest)?;
            std::fs::write(dest.join("point.toml"), card)?;
            std::fs::write(
                dest.join("point-exact.json"),
                serde_json::to_vec_pretty(&point)?,
            )?;
            let generic = GraphIntegral::new_with_scalar_values(
                Arc::new(diagram.clone()),
                &kin(&point)?,
                &values,
            )?
            .with_auxiliary_external_momenta(&auxiliaries)?;
            let slots = symbol!("ward_proof::slots___");
            let polarization = auxiliaries[ward].as_view().get_symbol().unwrap();
            let projector = diagram
                .projector()
                .replace(function!(polarization, slots))
                .with(function!(symbols::external_momentum(), ward, slots));
            assert!(projector != *diagram.projector() && !projector.contains_symbol(polarization));
            let mut remaining = auxiliaries.clone();
            let _ = remaining.remove(ward);
            let direct = GraphIntegral::new_with_scalar_values(
                Arc::new(diagram.clone().with_projector(projector)),
                &kin(&original)?,
                &values,
            )?
            .with_auxiliary_external_momenta(&remaining)?;
            let (a, na) = density(&generic)?;
            let (b, nb) = density(&direct)?;
            let difference = (&a - &b).expand().cancel();
            let equal = difference.is_zero();
            eprintln!(
                "{} ward{}: native terms {na}/{nb}, exact equality {equal}",
                d["name"],
                ward + 1
            );
            rows.push(json!({"diagram":d["name"],"ward":ward+1,"generic_terms":na,"direct_terms":nb,"native_exact_equal":equal}));
            if !equal {
                std::fs::write(
                    dest.join("difference.txt"),
                    difference.to_canonical_string(),
                )?;
                return Err("native Ward equivalence failed".into());
            }
        }
    }
    std::fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(&json!({"rows":rows,"native_exact_equal":true}))?,
    )?;
    Ok(())
}
