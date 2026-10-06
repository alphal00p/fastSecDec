"""Exercise the installed bridge through existing native HEPKit input owners."""

import math
from pathlib import Path
import sys

import pytest
from symbolica import E, Expression
from symbolica.community import hepkit as hep


sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs
fs = getattr(hep, "sector_decomposition", None)
pytestmark = pytest.mark.skipif(fs is None, reason="requires a community wheel with FastSecDec")


def test_periodization_settings_preserve_defaults_and_roundtrip_all_native_variants():
    assert fs.QmcSettings().periodization == "korobov3"
    for periodization in ("none", "korobov2", "korobov3"):
        settings = fs.QmcSettings(periodization=periodization)
        assert settings.periodization == periodization
        assert settings.points == 4096 and settings.shifts == 64
        assert settings.package_points == 1024 and settings.rule == "kuo_33002"
    with pytest.raises(fs.FastSecDecError) as caught:
        fs.QmcSettings(periodization="korobov1")
    assert caught.value.stage == "configuration"
    assert "korobov2" in str(caught.value)


@pytest.fixture(scope="module")
def prepared():
    value = inputs.massive_triangle()
    return value, fs.Integral(**value.integral_arguments())


@pytest.fixture(scope="module")
def compiled(prepared):
    events = []
    generated = prepared[1].generate(1, observer=events.append)
    assert events and all(isinstance(event, fs.GenerationSnapshot) for event in events)
    assert generated.orders == [0, 1]
    assert generated.snapshot().stage == "compilation"
    assert generated.snapshot().kernels == 0
    kernels = generated.compile(observer=events.append)
    assert events[-1].stage == "complete"
    assert kernels.snapshot().kernels == kernels.sector_count
    expected_backend = "portable_interpreted" if sys.platform == "emscripten" else "native_o2"
    assert kernels.backend == expected_backend
    return kernels


def test_native_input_and_dimension_remain_distinct(prepared):
    value, integral = prepared
    assert integral.regulator == value.regulator
    assert integral.dimension == 4 - 2 * value.regulator
    assert len(integral.powers) == 3
    args = value.integral_arguments()
    args["scalar_values"] = {value.kinematics.dimension: E("4")}
    with pytest.raises(fs.FastSecDecError) as caught:
        fs.Integral(**args)
    assert caught.value.stage == "input"


def test_selected_subgraph_guard_is_preserved(prepared):
    value, _ = prepared
    arguments = value.integral_arguments()
    # A native physical-cut view avoids requiring the optional standalone
    # Linnet Python renderer merely to exercise the guarded Rust accessor.
    model = hep.Model.phi4()
    process = model.process(["phi", "phi"], ["phi", "phi"])
    diagram = process.generate_cross_section(loops=1).diagrams[0]
    arguments["diagram"] = diagram.cuts[0].left.subgraph
    with pytest.raises(hep.DiagramError, match="excise"):
        fs.Integral(**arguments)


def test_initial_and_complete_coverage_checkpoint_and_covariance(compiled):
    settings = fs.QmcSettings(points=1024, shifts=4, seed=19, package_points=128)
    session = compiled.session(settings)
    initial = session.snapshot()
    assert initial.uncertainty == "waiting_for_coverage"
    assert initial.estimate is None
    assert initial.completed_points == 0
    with pytest.raises(AttributeError):
        initial.completed_points = 1
    partial = session.step()
    assert partial.completed_points == 128
    assert partial.estimate is None
    saved = session.checkpoint()
    assert isinstance(saved, bytes)
    restored = compiled.restore(saved)
    assert restored.snapshot().completed_points == partial.completed_points
    events = []
    while not session.complete:
        session.step(8, observer=events.append)
    while not restored.complete:
        restored.step(3)
    result = session.snapshot()
    estimate = result.estimate
    other = restored.snapshot().estimate
    assert all(isinstance(event, fs.IntegrationSnapshot) for event in events)
    assert result.uncertainty == "available"
    assert result.completed_points == result.planned_points
    assert all(s.complete_replicas == s.planned_replicas for s in result.sectors)
    assert estimate.production_complete
    assert estimate.meets(absolute=1.0, relative=0)
    assert estimate.orders == [0, 1]
    assert estimate.components == ["real", "real"]
    assert len(estimate.covariance_of_mean) == 4
    assert estimate.mean == other.mean
    assert estimate.covariance_of_mean == other.covariance_of_mean
    assert estimate.covariance_of_mean[1] == estimate.covariance_of_mean[2]
    # Equal-mass triangle C0 with two lightlike legs, s=-1 and m=1,
    # normalized by d^D k/(i*pi^(D/2)); analytic spacelike continuation.
    exact_finite = -2 * math.asinh(0.5) ** 2
    assert abs(estimate.mean[0] - exact_finite) <= max(8 * estimate.standard_error[0], 2e-5)
    assert result.evaluation_diagnostics.evaluations == result.completed_points


def test_generation_and_compilation_cancel_without_partial_results(prepared):
    with pytest.raises(fs.CancelledError) as caught:
        prepared[1].generate(observer=lambda _: False)
    assert caught.value.stage == "generation"
    generated = prepared[1].generate()
    with pytest.raises(fs.CancelledError) as caught:
        generated.compile(observer=lambda _: False)
    assert caught.value.stage == "compilation"


def test_observer_python_errors_keep_their_type(prepared, compiled):
    class ObserverFailure(Exception):
        pass

    def fail(_):
        raise ObserverFailure("observer failed")

    with pytest.raises(ObserverFailure, match="observer failed"):
        prepared[1].generate(observer=fail)
    session = compiled.session(fs.QmcSettings(points=32, rule="hkkn_alpha3", shifts=2, package_points=8))
    with pytest.raises(ObserverFailure, match="observer failed"):
        session.step(observer=fail)
    assert session.snapshot().completed_points == 8
    assert session.snapshot().estimate is None
    assert session.snapshot().stop_reason is None
    restored = compiled.restore(session.checkpoint())
    assert restored.snapshot().completed_points == 8
    assert restored.snapshot().stop_reason is None
    assert restored.settings.rule == "hkkn_alpha3"


def test_step_cancellation_preserves_accepted_coverage_and_can_resume(compiled):
    session = compiled.session(fs.QmcSettings(points=32, rule="hkkn_alpha3", shifts=2, package_points=8))
    stopped = session.step(20, observer=lambda _: False)
    assert stopped.completed_points == 8
    assert stopped.stop_reason == "cancelled"
    assert stopped.estimate is None
    resumed = compiled.restore(session.checkpoint())
    assert resumed.step().completed_points == 16


def test_keyboard_interrupt_observer_retains_cancelled_accepted_checkpoint(compiled):
    session = compiled.session(fs.QmcSettings(points=32, rule="hkkn_alpha3", shifts=2, package_points=8))

    def interrupt(_):
        raise KeyboardInterrupt("cancel requested")

    with pytest.raises(KeyboardInterrupt, match="cancel requested"):
        session.step(8, observer=interrupt)
    stopped = session.snapshot()
    assert stopped.completed_points == 8
    assert stopped.stop_reason == "cancelled"
    assert stopped.estimate is None
    assert not session.complete
    restored = compiled.restore(session.checkpoint())
    assert restored.snapshot().stop_reason == "cancelled"
    assert restored.step().completed_points == 16


def test_native_kernel_artifact_roundtrip(compiled):
    artifact = compiled.to_bytes()
    restored = fs.Kernels.from_bytes(artifact)
    assert restored.content_id == compiled.content_id
    assert restored.to_bytes() == artifact
    assert restored.orders == compiled.orders
    assert restored.components == compiled.components
    assert restored.snapshot().stage == "complete"
    settings = fs.QmcSettings(points=32, shifts=2, rule="hkkn_alpha3", package_points=8)
    session = compiled.session(settings)
    session.step()
    assert restored.restore(session.checkpoint()).snapshot().completed_points == 8
    with pytest.raises(fs.FastSecDecError) as caught:
        fs.Kernels.from_bytes(b"invalid native artifact")
    assert caught.value.stage == "artifact"


def test_errors_do_not_become_zero_estimates(prepared, compiled):
    with pytest.raises(fs.FastSecDecError) as caught:
        fs.QmcSettings(shifts=1)
    assert caught.value.stage == "configuration"
    with pytest.raises(fs.FastSecDecError) as caught:
        compiled.restore(b"invalid checkpoint")
    assert caught.value.stage == "checkpoint"
    value, _ = prepared
    arguments = value.integral_arguments()
    arguments["measure_multiplier"] = E("10^10000")
    generated = fs.Integral(**arguments).generate()
    session = generated.compile().session(fs.QmcSettings(points=32, rule="hkkn_alpha3", shifts=2, package_points=8))
    with pytest.raises(fs.FastSecDecError) as caught:
        session.step()
    assert caught.value.stage == "integration"
    failed = session.snapshot()
    assert failed.completed_points == 0
    assert failed.estimate is None
    assert failed.stop_reason == "numerical_failure"
    assert failed.evaluation_diagnostics.failures == 1
    assert not session.complete
    # The failed package is available for retry; no accepted replay prefix survives.
    with pytest.raises(fs.FastSecDecError):
        session.step()
    assert session.snapshot().completed_points == 0
    assert session.snapshot().evaluation_diagnostics.failures == 2


def test_complex_measure_weight_is_applied_once_and_checkpoint_identity_is_bound(prepared):
    value, integral = prepared
    settings = fs.QmcSettings(points=32, rule="hkkn_alpha3", shifts=2, seed=7, package_points=64)
    baseline = integral.generate().compile()
    arguments = value.integral_arguments()
    arguments["measure_multiplier"] = 2 + 3 * Expression.I
    weighted = fs.Integral(**arguments).generate().compile()
    original_session = baseline.session(settings)
    weighted_session = weighted.session(settings)
    with pytest.raises(fs.FastSecDecError) as caught:
        weighted.restore(original_session.checkpoint())
    assert caught.value.stage == "checkpoint"
    while not original_session.complete:
        original_session.step(8)
    while not weighted_session.complete:
        weighted_session.step(8)
    original = original_session.snapshot().estimate
    result = weighted_session.snapshot().estimate
    assert result.orders == [0, 0]
    assert result.components == ["real", "imag"]
    assert result.mean == pytest.approx([2 * original.mean[0], 3 * original.mean[0]], rel=1e-12)
    variance = original.covariance_of_mean[0]
    assert result.covariance_of_mean == pytest.approx([4 * variance, 6 * variance, 6 * variance, 9 * variance], rel=1e-9, abs=1e-25)


@pytest.mark.parametrize("builder", [inputs.massless_box, inputs.rank_two_box, inputs.coupled_sunset])
def test_default_showcase_inputs_generate_and_evaluate_signed_laurent_layout(builder):
    value = builder()
    generated = fs.Integral(**value.integral_arguments()).generate(value.max_order)
    assert generated.orders[-1] == value.max_order
    assert generated.orders[0] < 0
    assert generated.sector_count > 0
    kernels = generated.compile()
    assert kernels.orders == generated.orders
    session = kernels.session(fs.QmcSettings(points=1024, shifts=2, package_points=32))
    partial = session.step()
    assert partial.completed_points == 32
    assert partial.estimate is None
    assert not session.complete
    assert partial.evaluation_diagnostics.failures == 0


def test_diagram_expression_copy_preserves_native_owner_and_other_factors(prepared):
    value, _ = prepared
    original = value.diagram
    before = original.to_json()
    copied = fs.with_diagram_expressions(
        original, numerator=E("3"), projector=E("2"), overall_factor=E("5"),
    )
    assert original.to_json() == before
    assert copied.numerator_expression().to_expression() == E("3")
    assert copied.projector_expression() == E("2")
    assert copied.overall_factor_expression() == E("5")
    assert copied.numerator_prefactor_expression() == original.numerator_prefactor_expression()
    assert copied.loop_momentum_basis.external_edges == original.loop_momentum_basis.external_edges
    assert copied.loop_count == original.loop_count
    assert fs.with_diagram_expressions(original).to_json() == before
    with pytest.raises(TypeError):
        fs.with_diagram_expressions(original, numerator=object())


def test_diagram_expression_copy_rejects_partial_views():
    model = hep.Model.phi4()
    diagram = model.process(["phi", "phi"], ["phi", "phi"]).generate_cross_section(loops=1).diagrams[0]
    with pytest.raises(hep.DiagramError, match="excise"):
        fs.with_diagram_expressions(diagram.cuts[0].left.subgraph, projector=E("1"))


def test_coefficient_expansion_validates_before_observation(prepared):
    events = []
    with pytest.raises(ValueError, match="coefficient_expansion"):
        prepared[1].generate(0, coefficient_expansion="unrecognized", observer=events.append)
    assert not events


def test_native_named_and_default_physical_preserve_triangle_coefficients(prepared):
    default = prepared[1].generate(0)
    named = prepared[1].generate(0, coefficient_expansion="native_named")
    assert named.orders == default.orders
    assert named.sector_count == default.sector_count
    assert named.exact_coefficients == default.exact_coefficients
    settings = fs.QmcSettings(points=32, shifts=2, seed=71, package_points=32, rule="hkkn_alpha3")
    estimates = []
    for generated in (default, named):
        kernels = generated.compile()
        session = kernels.session(settings)
        while not session.complete:
            session.step()
        snapshot = session.snapshot()
        assert snapshot.completed_points == snapshot.planned_points == kernels.sector_count * 64
        assert all(sector.complete_replicas == sector.planned_replicas == 2 for sector in snapshot.sectors)
        assert snapshot.estimate.production_complete
        estimates.append(snapshot.estimate)
    assert estimates[0].orders == estimates[1].orders
    assert estimates[0].components == estimates[1].components
    assert estimates[0].mean == pytest.approx(estimates[1].mean, rel=1e-12, abs=1e-14)
    assert estimates[0].covariance_of_mean == pytest.approx(estimates[1].covariance_of_mean, rel=1e-12, abs=1e-14)
