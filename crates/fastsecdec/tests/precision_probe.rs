//! Reproducible diagnostics, intentionally excluded from ordinary unit tests.
use fastsecdec::{
    Atom, Kinematics, Model,
    generation::{GenerationOptions, GenerationProgress, SubtractionStrategy, generate},
    input::GraphIntegral,
    kernel::KernelSet,
    parametric::ParametricIntegrand,
};
use fastsecdec_qmc::{Korobov3, QmcPlan, Rank1Rule};
use feynkit_graph::symbols;
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
    let strategy =
        std::env::var("FASTSECDEC_GENERATION_PROBE_STRATEGY").unwrap_or_else(|_| "taylor".into());
    let subtraction = match strategy.as_str() {
        "taylor" => SubtractionStrategy::Taylor,
        "ibp" => SubtractionStrategy::IntegrateByParts,
        other => panic!("unknown probe strategy {other}"),
    };
    let label = format!("{strategy}-factored");
    let mut phases = BTreeMap::<String, f64>::new();
    let started = Instant::now();
    let mut last = started;
    let generated = generate(
        &input,
        &GenerationOptions {
            subtraction,
            ..Default::default()
        },
        |status| {
            if let GenerationProgress::PhaseTiming { phase, seconds } = status {
                *phases.entry(format!("{phase:?}")).or_default() += seconds;
            }
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
        },
    )
    .unwrap();
    let generation_seconds = started.elapsed().as_secs_f64();
    eprintln!(
        "generation: {} sectors, orders {:?}, {:.3}s",
        generated.sectors().len(),
        generated.orders(),
        generation_seconds
    );
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    std::fs::create_dir_all(&evidence).unwrap();
    let bytes = generated.to_kernel_bytes(Default::default()).unwrap();
    std::fs::write(evidence.join(format!("double-box-{label}.fsd")), &bytes).unwrap();
    let compiled = Instant::now();
    let mut kernels = generated.compile().unwrap();
    let compile_seconds = compiled.elapsed().as_secs_f64();
    eprintln!(
        "compile {compile_seconds:.3}s, artifact {} bytes, phases {phases:?}",
        bytes.len()
    );
    std::fs::write(evidence.join(format!("double-box-generation-{label}.json")), serde_json::to_vec(&serde_json::json!({
        "content_id": kernels.content_id(), "strategy": strategy, "sectors": generated.sectors().len(), "orders": generated.orders(),
        "generation_seconds": generation_seconds, "compile_seconds": compile_seconds, "phases": phases, "artifact_bytes": bytes.len()
    })).unwrap()).unwrap();
    measure(&mut kernels, &label);
}

#[test]
#[ignore = "saved double-box numeric precision performance probe"]
fn saved_double_box_precision_probe() {
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    let started = Instant::now();
    let mut kernels =
        KernelSet::from_bytes(&std::fs::read(evidence.join("double-box-generated.fsd")).unwrap())
            .unwrap();
    eprintln!(
        "artifact load/compile {:.3}s",
        started.elapsed().as_secs_f64()
    );
    measure(
        &mut kernels,
        &std::env::var("FASTSECDEC_PRECISION_PROBE_LABEL").unwrap_or_else(|_| "numeric".into()),
    );
}

fn measure(kernels: &mut KernelSet, label: &str) {
    let started = Instant::now();
    let (mut points, mut checked, mut rescued) = (0, 0, 0);
    let mut values = Vec::new();
    let mut precisions = BTreeMap::<u32, usize>::new();
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
            *precisions.entry(report.bits).or_default() += 1;
            values.extend_from_slice(&output);
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
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/probes");
    std::fs::write(evidence.join(format!("double-box-precision-{label}.json")), serde_json::to_vec(&serde_json::json!({
        "content_id": kernels.content_id(), "points": points, "checked": checked, "rescued": rescued,
        "seconds": started.elapsed().as_secs_f64(), "precision_counts": precisions, "values": values,
        "point_rule": "kuo1024-shift481-index13k-korobov3-64-per-sector"
    })).unwrap()).unwrap();
}
