use super::*;
use crate::{
    contour::{ContourMode, ContourSettings, ContourValidation},
    generation::GenerationMode,
    kernel::{EvaluatorBackend, KernelLoadOptions, indexed::ProgramArchiveReader},
    parametric::{FactorRole, FactorSemantics, ParametricDomain, ParametricTerm, PolynomialFactor},
    status::CoefficientComponent,
};
use std::io::Cursor;
use symbolica::{atom::Atom, parse, symbol};

fn input() -> ParametricIntegrand {
    ParametricIntegrand::new(
        vec![symbol!("family_session::x")],
        symbol!("family_session::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("2+3𝑖"),
            vec![parse!("-1+family_session::eps")],
            vec![
                PolynomialFactor::new(
                    parse!("1+family_session::x"),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap()
}
fn family() -> RecipeFamily {
    RecipeFamily::new(
        [ProgramRecipe::FixedV1, ProgramRecipe::UndeformedV1],
        ProgramRecipe::UndeformedV1,
    )
    .unwrap()
}
fn bind(kernels: &mut KernelSet) {
    let mut settings = ContourSettings::default();
    if kernels.program_recipe() == ProgramRecipe::FixedV1 {
        settings.deformation = ContourMode::Fixed { lambda: 0.2 };
    }
    settings.validation.policy = ContourValidation::Off;
    kernels
        .bind_parameters_with_contour(&BTreeMap::new(), &settings)
        .unwrap();
}
fn integral(kernels: &mut KernelSet) -> Vec<f64> {
    bind(kernels);
    let mut total = kernels.exact_coefficients().to_vec();
    for sector in kernels.sectors_mut() {
        let mut values = vec![0.; total.len()];
        for i in 0..512 {
            sector
                .evaluate(&[(i as f64 + 0.5) / 512.], &mut values)
                .unwrap();
            for (sum, value) in total.iter_mut().zip(&values) {
                *sum += value / 512.;
            }
        }
    }
    total
}

#[test]
fn paused_family_shares_sources_and_preserves_only_requested_resident_recipe() {
    for mode in [GenerationMode::Symbolic, GenerationMode::NumericalDual] {
        let staging = tempfile::tempdir().unwrap();
        let mut session = RecipeFamilySession::new(
            input(),
            GenerationOptions {
                mode,
                ..Default::default()
            },
            family(),
            staging.path().to_owned(),
            Cursor::new(Vec::new()),
        )
        .with_resident_recipe(Some(ProgramRecipe::FixedV1))
        .unwrap()
        .with_evaluator(
            Default::default(),
            CompilationSettings {
                backend: EvaluatorBackend::Eager,
                ..Default::default()
            },
        )
        .unwrap();
        let mut previous = 0;
        let mut live_seconds = 0.;
        while !session.is_complete() {
            let state = session
                .step(64, |snapshot| {
                    assert!(snapshot.generation.elapsed_seconds.is_finite());
                    assert!(snapshot.generation.elapsed_seconds >= live_seconds);
                    live_seconds = snapshot.generation.elapsed_seconds;
                    ControlFlow::Break(())
                })
                .unwrap();
            assert_eq!(session.snapshot().completed_units, previous + 1);
            previous += 1;
            assert!(matches!(
                state,
                GenerationSessionState::Paused | GenerationSessionState::Complete
            ));
            if !session.is_complete() {
                assert!(session.take_result().is_none());
            }
        }
        let snapshot = session.snapshot().clone();
        assert!(live_seconds > 0.);
        assert!(snapshot.generation.elapsed_seconds >= live_seconds);
        assert_eq!(snapshot.completed_recipes, 2);
        assert_eq!(snapshot.prepared_sources, 1);
        let output = session.take_result().unwrap();
        assert!(session.take_result().is_none());
        assert_eq!(output.family.default_recipe(), ProgramRecipe::UndeformedV1);
        let mut resident = output.resident.unwrap();
        assert_eq!(resident.program_recipe(), ProgramRecipe::FixedV1);
        assert_eq!(
            resident.content_id(),
            output
                .catalogue
                .recipe(ProgramRecipe::FixedV1)
                .unwrap()
                .content_id
        );
        let selected_bytes = resident.to_bytes().unwrap();
        let retained = ProgramArchiveReader::from_reader(
            Cursor::new(&selected_bytes),
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(retained.catalogue().recipes.len(), 1);
        assert_eq!(
            retained.catalogue().recipes[0].recipe,
            ProgramRecipe::FixedV1
        );
        let mut restored = KernelSet::from_bytes_with_options(
            &selected_bytes,
            KernelLoadOptions { validate: true },
        )
        .unwrap();
        assert_eq!(restored.content_id(), resident.content_id());
        let resident_value = integral(&mut resident);
        assert_eq!(resident_value, integral(&mut restored));
        let mut full =
            ProgramArchiveReader::from_reader(output.writer, KernelLoadOptions { validate: true })
                .unwrap();
        for recipe in family().recipes() {
            let mut kernels = full.select(*recipe).unwrap().load_all().unwrap();
            let value = integral(&mut kernels);
            for ((order, component), value) in
                kernels.orders().iter().zip(kernels.components()).zip(value)
            {
                let scale = if *component == CoefficientComponent::Real {
                    2.
                } else {
                    3.
                };
                let expected = if *order == -1 {
                    scale
                } else {
                    assert_eq!(*order, 0);
                    -scale * 2f64.ln()
                };
                assert!(
                    (value - expected).abs() < 2e-5,
                    "{mode:?}/{recipe:?}: {value} vs {expected}"
                );
            }
        }
    }
}

#[test]
fn no_resident_family_keeps_storage_and_publication_caller_owned() {
    let staging = tempfile::tempdir().unwrap();
    let writer = tempfile::tempfile().unwrap();
    let mut session = RecipeFamilySession::new(
        input(),
        Default::default(),
        family(),
        staging.path().to_owned(),
        writer,
    )
    .with_evaluator(
        Default::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(session.step(0, |_| ControlFlow::Continue(())).is_err());
    while !session.is_complete() {
        session.step(1, |_| ControlFlow::Continue(())).unwrap();
    }
    let output = session.take_result().unwrap();
    assert!(output.resident.is_none());
    assert_eq!(output.catalogue.recipes.len(), 2);
    assert!(
        staging.path().read_dir().unwrap().next().is_some(),
        "caller still owns staged records"
    );
    let reader =
        ProgramArchiveReader::from_reader(output.writer, KernelLoadOptions { validate: true })
            .unwrap();
    assert_eq!(reader.catalogue().content_id, output.catalogue.content_id);
}

#[test]
fn empty_input_still_publishes_complete_native_exact_records() {
    let empty = ParametricIntegrand::new(
        vec![symbol!("family_empty::x")],
        symbol!("family_empty::eps"),
        ParametricDomain::UnitCube,
        vec![],
    )
    .unwrap();
    let staging = tempfile::tempdir().unwrap();
    let mut session = RecipeFamilySession::new(
        empty,
        Default::default(),
        family(),
        staging.path().to_owned(),
        Cursor::new(Vec::new()),
    )
    .with_resident_recipe(Some(ProgramRecipe::UndeformedV1))
    .unwrap()
    .with_evaluator(
        Default::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap();
    while !session.is_complete() {
        session.step(1, |_| ControlFlow::Continue(())).unwrap();
    }
    assert_eq!(session.snapshot().prepared_sources, 0);
    let output = session.take_result().unwrap();
    assert!(output.resident.unwrap().sectors().is_empty());
    assert!(
        output
            .catalogue
            .recipes
            .iter()
            .all(|recipe| recipe.sector_count() == 0 && !recipe.records.is_empty())
    );
}

struct BrokenWriter;
impl Write for BrokenWriter {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("injected disk full"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Seek for BrokenWriter {
    fn seek(&mut self, _: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}
#[test]
fn failed_storage_never_yields_or_resumes_a_partial_family() {
    let staging = tempfile::tempdir().unwrap();
    let mut session = RecipeFamilySession::new(
        input(),
        Default::default(),
        family(),
        staging.path().to_owned(),
        BrokenWriter,
    );
    assert!(
        session
            .step(1, |_| ControlFlow::Continue(()))
            .unwrap_err()
            .to_string()
            .contains("injected disk full")
    );
    assert!(session.take_result().is_none());
    assert!(
        session
            .step(1, |_| ControlFlow::Continue(()))
            .unwrap_err()
            .to_string()
            .contains("failed session")
    );
}

struct LimitedWriter {
    bytes: Cursor<Vec<u8>>,
    remaining: usize,
}
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::Error::other("injected partial-record disk full"));
        }
        let count = bytes.len().min(self.remaining);
        self.remaining -= count;
        self.bytes.write(&bytes[..count])
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Seek for LimitedWriter {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        self.bytes.seek(position)
    }
}
#[test]
fn partial_record_write_is_terminal_and_never_replayed_into_the_archive() {
    let staging = tempfile::tempdir().unwrap();
    let writer = LimitedWriter {
        bytes: Cursor::new(Vec::new()),
        remaining: 128,
    };
    let mut session = RecipeFamilySession::new(
        input(),
        Default::default(),
        family(),
        staging.path().to_owned(),
        writer,
    )
    .with_evaluator(
        Default::default(),
        CompilationSettings {
            backend: EvaluatorBackend::Eager,
            ..Default::default()
        },
    )
    .unwrap();
    let error = loop {
        match session.step(1, |_| ControlFlow::Continue(())) {
            Ok(_) => assert!(session.take_result().is_none()),
            Err(error) => break error,
        }
    };
    assert!(error.to_string().contains("partial-record disk full"));
    assert!(
        session.snapshot().completed_units > 1,
        "header and preparation succeeded"
    );
    assert_eq!(session.snapshot().persisted_units, 0);
    assert!(session.take_result().is_none());
    assert!(session.step(1, |_| ControlFlow::Continue(())).is_err());
}
