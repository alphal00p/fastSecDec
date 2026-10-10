use super::*;
use fastsecdec::{
    generation::{
        self, CoefficientExpansionMethod, CoefficientExpansionOptions, GenerationMode,
        GenerationOptions, SubtractionStrategy,
    },
    kernel::{CompilationSettings, EvaluatorBackend},
};
use std::ops::ControlFlow;

pub(super) fn prepare(out: &Path) -> CliResult<()> {
    let start = Instant::now();
    // Kite admission is deliberately first; failures do not authorize replacing it.
    for name in ["kite", "triangle", "box", "sunrise"] {
        deadline(start, 300.)?;
        let fixture = match fixtures::build(name) {
            Ok(fixture) => fixture,
            Err(error) => {
                save(
                    &out.join(format!("{name}-failure.json")),
                    &json!({"case":name,"error":error.to_string(),"admitted":false}),
                )?;
                continue;
            }
        };
        save(
            &out.join(format!("{name}.json")),
            &json!({"admission":fixture.admission,"reference":fixture.reference}),
        )?;
        event(
            out,
            json!({"case":name,"native_admission_seconds":start.elapsed().as_secs_f64()}),
        )?;
    }
    deadline(start, 300.)
}

pub(super) fn generate(
    name: &str,
    recipe: ProgramRecipe,
    jacobian: ContourJacobian,
    out: &Path,
) -> CliResult<()> {
    let start = Instant::now();
    let fixture = fixtures::build(name)?;
    let input_and_reference_seconds = start.elapsed().as_secs_f64();
    // Retain the original native graph as the fixture's input owner until construction ends.
    let _graph = &fixture.graph;
    let options = GenerationOptions {
        mode: GenerationMode::Symbolic,
        contour_jacobian: jacobian,
        program_recipe: recipe,
        max_order: 0,
        subtraction: SubtractionStrategy::IntegrateByParts,
        coefficient_expansion: CoefficientExpansionOptions {
            method: CoefficientExpansionMethod::NativeNamed,
            initial_relative_width: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let compilation = CompilationSettings {
        backend: EvaluatorBackend::Symjit,
        contour_jacobian: jacobian,
        horner_iterations: 0,
        cpe_rounds: Some(1000),
        cores: 1,
        direct_translation: true,
        ..Default::default()
    };
    let generation_settings = json!({"mode":"symbolic","contour_jacobian":jacobian,"program_recipe":recipe,"max_order":0,"subtraction":"integrate_by_parts","coefficient_expansion":options.coefficient_expansion});
    save(
        &out.join("settings.json"),
        &json!({"generation":generation_settings,"compilation":compilation,"budget_seconds":120,
            "strategy":"in-process native generation with one caller and one compiler core; no CLI serial workers"}),
    )?;
    let generation_start = Instant::now();
    let generated = generation::generate(&fixture.input, &options, |progress| {
        let _ = event(
            out,
            json!({"generation":format!("{progress:?}"),"elapsed_seconds":start.elapsed().as_secs_f64()}),
        );
        if start.elapsed().as_secs_f64() < 120. {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    })?;
    require(
        generated
            .sectors()
            .iter()
            .all(|s| s.generation_mode() == GenerationMode::Symbolic),
        "native endpoint generation mode changed",
    )?;
    deadline(start, 120.)?;
    let native_generation_seconds = generation_start.elapsed().as_secs_f64();
    let compile = Instant::now();
    let kernels = generated.compile_with_settings_parameters_and_progress(
        Default::default(),
        &[],
        compilation,
        |_| {
            if start.elapsed().as_secs_f64() < 120. {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        },
    )?;
    let compilation_seconds = compile.elapsed().as_secs_f64();
    deadline(start, 120.)?;
    let serialize = Instant::now();
    let bytes = kernels.to_bytes()?;
    let artifact = out.join("kernel.bin");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&artifact)?;
    file.write_all(&bytes)?;
    let serialization_seconds = serialize.elapsed().as_secs_f64();
    deadline(start, 120.)?;
    let saved = Saved {
        case: name.into(),
        case_index: fixture.admission.case_index,
        recipe,
        jacobian,
        artifact: Pin::new(&artifact)?,
        content_id: kernels.content_id().into(),
        endpoint_mode: "symbolic".into(),
        schema: schema(&kernels)?,
        reference: fixture.reference,
        producer: json!({"generation":generation_settings,"compilation":compilation,"admission":fixture.admission,
            "strategy":"in-process native generation with one caller and one compiler core; no CLI serial workers",
            "input_and_reference_seconds":input_and_reference_seconds,
            "native_generation_seconds":native_generation_seconds,"compilation_seconds":compilation_seconds,
            "serialization_seconds":serialization_seconds,"whole_seconds":start.elapsed().as_secs_f64(),
            "statistics":kernels.sectors().iter().map(|s|s.statistics()).collect::<Vec<_>>(),
            "exact_expressions":generated.exact_coefficients().iter().map(ToString::to_string).collect::<Vec<_>>(),
            "actual_generated_endpoint_modes":generated.sectors().iter().map(|s|s.generation_mode()).collect::<Vec<_>>()}),
    };
    save(&out.join("saved.json"), &saved)
}
