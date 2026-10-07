use super::*;
use fastsecdec::generation::{CoefficientExpansionMethod as Method, GenerationMode as Mode};
use ratatui::{Terminal, backend::TestBackend};

fn snapshot(stage: GenerationStage) -> GenerationSnapshot {
    GenerationSnapshot {
        stage,
        completed: 3,
        total: Some(8),
        sectors: 8,
        kernels: 0,
        elapsed_seconds: 1.2,
        timings: Default::default(),
        coefficient_expansion: None,
        formula_preparation: None,
        detail: "Exact native work".into(),
    }
}

#[test]
fn nested_work_and_empty_phases_keep_honest_parent_states() {
    for (mode, method) in [
        (Mode::Symbolic, Method::NativeNamed),
        (Mode::Symbolic, Method::Physical),
        (Mode::NumericalDual, Method::NativeNamed),
    ] {
        let mut plan = Plan::default();
        assert_eq!(plan.labels(), vec!["Read run card · loading plan"]);
        plan.configure(mode, method);
        assert_eq!(plan.labels().len(), 8);
        assert_eq!(plan.state(1), State::Pending);
        plan.observe(GenerationStage::Parametrization);
        assert_eq!(plan.state(0), State::Complete);
        plan.observe(GenerationStage::Mapping);
        assert_eq!(plan.state(2), State::NotNeeded); // No geometry job occurred.
        let preparation = if mode == Mode::Symbolic {
            GenerationStage::Symmetry
        } else {
            GenerationStage::FormulaPreparation
        };
        plan.observe(preparation);
        assert_eq!(plan.state(4), State::Current);
        plan.observe(preparation); // Preparation/exact comparison can repeat.
        assert_eq!(plan.state(4), State::Current);
        for stage in [
            GenerationStage::CoefficientExpansion,
            GenerationStage::Subtraction,
            GenerationStage::Expansion,
            GenerationStage::Subtraction,
        ] {
            plan.observe(stage);
            assert_eq!(plan.state(5), State::Current);
            assert_eq!(plan.state(6), State::Pending);
        }
        plan.observe(GenerationStage::Compilation);
        plan.observe(GenerationStage::Compilation); // N/N still needs finish().
        assert_eq!(plan.state(6), State::Current);
        assert_eq!(plan.state(7), State::Pending);
        plan.saving();
        // An error here must leave save active, not call delivery complete.
        assert_eq!(plan.active_label(), "Save artifact");
        assert_eq!(plan.state(7), State::Current);
        plan.observe(GenerationStage::Compilation); // Native snapshot unchanged.
        assert_eq!(plan.state(7), State::Current);
        plan.observe(GenerationStage::Complete);
        assert_eq!(plan.state(7), State::Complete);
        assert_eq!(plan.state(2), State::NotNeeded);
    }
    let mut plan = Plan::default();
    plan.configure(Mode::NumericalDual, Method::Physical);
    plan.observe(GenerationStage::FormulaPreparation);
    plan.formula_count(0);
    assert_eq!(plan.state(4), State::NotNeeded);
    assert!(
        !plan
            .labels()
            .iter()
            .any(|label| label.contains("equivalent"))
    );
}

#[test]
fn all_planned_steps_stay_visible_with_progress_memory_and_workers() {
    for (mode, method) in [
        (Mode::Symbolic, Method::NativeNamed),
        (Mode::Symbolic, Method::Physical),
        (Mode::NumericalDual, Method::NativeNamed),
    ] {
        let mut plan = Plan::default();
        plan.configure(mode, method);
        for stage in [
            GenerationStage::Parametrization,
            GenerationStage::Geometry,
            GenerationStage::Mapping,
        ] {
            plan.observe(stage);
        }
        let active = if mode == Mode::Symbolic {
            GenerationStage::Symmetry
        } else {
            GenerationStage::FormulaPreparation
        };
        plan.observe(active);
        for (width, height) in [(40, 20), (64, 24), (80, 24), (120, 32), (160, 42)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let colors = ColorPolicy::for_stream(width != 120, width == 120);
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        &snapshot(active),
                        None,
                        &memory::Snapshot::default(),
                        &plan,
                        0,
                        colors,
                    )
                })
                .unwrap();
            let buffer = terminal.backend().buffer();
            let text = (0..height)
                .map(|y| {
                    (0..width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            for label in plan.labels() {
                assert!(
                    text.contains(label),
                    "{width}x{height}: missing {label}\n{text}"
                );
            }
            assert!(text.contains("▶"));
            assert!(text.contains("✓"));
            assert!(text.contains("○"));
            assert!(text.contains("RSS"));
            assert!(text.contains("RAM"));
            assert!(text.contains("Worker activity"));
            assert!(text.contains("Ctrl-C"));
            assert!(text.contains("ETA"));
            if colors.enabled() {
                for (marker, expected) in [("✓", TEAL), ("▶", GOLD), ("○", Color::DarkGray)] {
                    let cell = buffer
                        .content
                        .iter()
                        .find(|cell| cell.symbol() == marker)
                        .unwrap();
                    assert_eq!(cell.fg, expected);
                }
            }
        }
    }
}

#[test]
fn parsed_plan_precedes_source_io_and_observer_errors_stop_preparation() {
    use crate::input::{LoadProgress, load_observed};
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("missing.toml");
    std::fs::write(&card,"[input]\nmodel='absent-model.json'\ngraph='absent.dot'\n[generation]\nmode='numerical_dual'\n").unwrap();
    let mut stages = Vec::new();
    let result = load_observed(&card, |event| {
        match event {
            LoadProgress::Parsed(card) => {
                assert_eq!(card.generation.mode, Mode::NumericalDual);
                stages.push("parsed");
            }
            LoadProgress::Parametrization => stages.push("parametrization"),
        };
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(stages, ["parsed"]);
    let error = load_observed(&card, |_| Err("stop before opening model".into()))
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "stop before opening model");
}

#[test]
fn actual_tiny_generation_delivers_each_configured_plan_and_only_then_finishes() {
    for (mode, method) in [
        ("symbolic", "coefficient_series"),
        ("symbolic", "full_expression"),
        ("numerical_dual", "coefficient_series"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let card = dir.path().join("input.toml");
        std::fs::write(&card,format!("[direct]\ndomain='unit_cube'\nparameters=['x','y']\n[[direct.terms]]\nmonomial_powers=['0','0']\n[[direct.terms.factors]]\npolynomial='x+y'\nexponent='-1-eps'\n[generation]\nmode='{mode}'\n[generation.coefficient_expansion]\nmethod='{method}'\n")).unwrap();
        let mut dashboard = crate::display::Dashboard::new(false, false).unwrap();
        let output = dir.path().join("value.fsd");
        crate::generate::generate_with_workers(&card, &output, &mut dashboard, None, 2).unwrap();
        assert!(output.with_extension("fsd.json").exists());
        assert!(output.with_extension("fsd.dat").exists());
        for index in 0..8 {
            assert_eq!(
                dashboard.generation_plan.state(index),
                State::Complete,
                "{mode}/{method} step{index}"
            );
        }
    }
}
