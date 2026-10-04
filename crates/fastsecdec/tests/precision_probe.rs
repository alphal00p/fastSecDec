//! Reproducible diagnostics, intentionally excluded from ordinary unit tests.
use fastsecdec::{
    Atom, Kinematics, Model,
    generation::{GenerationOptions, GenerationProgress, generate},
    input::GraphIntegral,
    parametric::ParametricIntegrand,
};
use feynkit_graph::symbols;
use numerica::numerical_integration::qmc::{Korobov3, QmcPlan, Rank1Rule};
use std::{
    collections::BTreeMap,
    ops::ControlFlow,
    sync::Arc,
    time::{Duration, Instant},
};
use symbolica::{parse, symbol};

#[test]
#[ignore = "native double-box generation and boundary-conditioning performance probe"]
fn double_box_boundary_precision_probe() {
    let model =
        Arc::new(Model::from_json(include_str!("../../../examples/models/scalar.json")).unwrap());
    let momenta = (0..4)
        .map(|i| symbols::external_momentum().call(i))
        .collect::<Vec<_>>();
    let gram = [
        [0, -1, 2, -1],
        [-1, 0, -1, 2],
        [2, -1, 0, -1],
        [-1, 2, -1, 0],
    ];
    let mut kinematics = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_momenta(momenta.clone())
        .unwrap();
    for i in 0..4 {
        for j in i..4 {
            kinematics = kinematics
                .with_scalar_product(&momenta[i], &momenta[j], Atom::num((gram[i][j], 2)))
                .unwrap();
        }
    }
    let graph = GraphIntegral::from_dot(
        model,
        include_str!("../../../examples/graphs/double_box.dot"),
        &kinematics,
    )
    .unwrap()
    .with_scalar_values(&BTreeMap::from([(symbol!("UFO::mt"), Atom::Zero)]))
    .unwrap();
    let input = ParametricIntegrand::from_graph(
        &graph,
        (0..7).map(|i| symbol!(format!("x{i}"))).collect(),
        symbol!("eps"),
        parse!("4-2*eps"),
    )
    .unwrap();
    let started = Instant::now();
    let mut last = started;
    let generated = generate(&input, &GenerationOptions::default(), |status| {
        let early = match status {
            GenerationProgress::Factorization { sector, .. }
            | GenerationProgress::Subtraction { sector, .. }
            | GenerationProgress::LaurentExpansion { sector, .. } => *sector < 3,
            _ => false,
        };
        if early || last.elapsed() > Duration::from_secs(5) {
            eprintln!("{status:?}, {:.1}s", started.elapsed().as_secs_f64());
            last = Instant::now();
        }
        ControlFlow::Continue(())
    })
    .unwrap();
    eprintln!(
        "generation: {} sectors, orders {:?}, {:.3}s",
        generated.sectors().len(),
        generated.orders(),
        started.elapsed().as_secs_f64()
    );
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(
        evidence.join("double-box-generated.fsd"),
        generated.to_kernel_bytes(Default::default()).unwrap(),
    )
    .unwrap();
    let compiled = Instant::now();
    let mut kernels = generated.compile().unwrap();
    eprintln!("compile {:.3}s", compiled.elapsed().as_secs_f64());
    std::fs::write(
        evidence.join("double-box-compiled.fsd"),
        kernels.to_bytes().unwrap(),
    )
    .unwrap();
    let started = Instant::now();
    let (mut points, mut checked, mut rescued) = (0, 0, 0);
    for (id, kernel) in kernels.sectors_mut().iter_mut().enumerate() {
        let plan =
            QmcPlan::new(Rank1Rule::kuo(1024, kernel.dimension()).unwrap(), 1, 481, 0).unwrap();
        let mut point = vec![0.0; kernel.dimension()];
        let mut output = vec![0.0; kernel.output_count()];
        for index in 0..64 {
            plan.point(index * 13, &mut point).unwrap();
            Korobov3::transform_in_place(&mut point).unwrap();
            let report = kernel
                .evaluate_with_diagnostics(&point, &mut output)
                .unwrap();
            points += 1;
            checked += usize::from(report.checked);
            rescued += usize::from(report.rescued);
        }
        if id % 25 == 0 {
            eprintln!(
                "sector {id}, points {points}, checked {checked}, MPFR {rescued}, {:.1}s",
                started.elapsed().as_secs_f64()
            );
        }
    }
    eprintln!(
        "precision probe: {points} evaluations, {checked} conditioning checks, {rescued} MPFR rescues, {:.3}s",
        started.elapsed().as_secs_f64()
    );
}
