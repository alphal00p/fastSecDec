"""Thin native discrete-Havana steering, complete vectors and accepted checkpoints."""
import json
import math
from pathlib import Path
import sys

import pytest
from symbolica import Expression
from symbolica.community import hepkit as hep

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs
from _fixtures import fixed_arguments

fs = getattr(hep, "sector_decomposition", None)
pytestmark = pytest.mark.skipif(fs is None, reason="requires a community wheel with FastSecDec")


@pytest.fixture(scope="module")
def kernels():
    return fs.Integral(**fixed_arguments(inputs.massive_triangle())).generate(1).compile()


def settings(**updates):
    values = dict(points_per_batch=128, batches=4, seed=42, bins=8)
    values.update(updates)
    return fs.HavanaDiscreteSettings(**values)


def test_native_settings_and_allocation_errors(kernels):
    value = fs.HavanaDiscreteSettings()
    assert (value.points_per_batch, value.batches, value.bins, value.seed) == (4096, 64, 32, 0)
    for invalid in ({"points_per_batch": 1}, {"batches": 1}, {"bins": 0},
                    {"maximum_sector_probability_ratio": 0.5},
                    {"minimum_probability_density": float("nan")}):
        with pytest.raises(fs.FastSecDecError) as caught:
            fs.HavanaDiscreteSettings(**invalid)
        assert caught.value.stage == "configuration"
    with pytest.raises(fs.FastSecDecError):
        kernels.mc_session(settings(), sector_probabilities=[0.2] * kernels.sector_count)
    with pytest.raises(fs.FastSecDecError):
        kernels.mc_session(settings()).step(0)


def test_native_global_batch_coverage_and_checkpoint_resume(kernels):
    probabilities = [0.25, 0.75]
    assert kernels.sector_count == len(probabilities)
    session = kernels.mc_session(settings(), sector_probabilities=probabilities)
    first = session.step(4, observer=lambda _: False)
    assert first.method == "havana_discrete_mc" and first.stage == "production"
    assert first.completed_points == 128 and first.planned_points == 512
    assert sum(sector.completed_points for sector in first.sectors) == 128
    assert first.stop_reason == "cancelled" and first.estimate is None
    for sector, probability in zip(first.sectors, probabilities, strict=True):
        assert sector.planned_points is None
        assert sector.discrete_allocation.probability == probability
        assert sector.discrete_allocation.points_per_batch == 128
        assert sector.complete_replicas == 1
    restored = kernels.restore_mc(session.checkpoint())
    assert restored.checkpoint_available and restored.sector_probabilities == session.sector_probabilities
    left, right = session.step(4), restored.step(4)
    assert left.completed_points == right.completed_points == 512
    assert left.estimate.mean == right.estimate.mean
    assert left.estimate.standard_error == right.estimate.standard_error
    assert left.estimate.covariance_of_mean == right.estimate.covariance_of_mean
    assert left.estimate.production_complete and left.estimate.orders == [0, 1]
    assert sum(sector.completed_points for sector in left.sectors) == 512
    assert all(sector.complete_replicas == 4 for sector in left.sectors)


def test_pilot_pause_is_in_memory_and_freeze_excludes_pilot(kernels):
    session = kernels.mc_session(settings(), pilot=True)
    assert session.stage == "pilot" and not session.checkpoint_available
    paused = session.step(4, observer=lambda _: False)
    assert paused.completed_points == 128 and paused.uncertainty == "pilot_only"
    with pytest.raises(fs.FastSecDecError) as caught:
        session.checkpoint()
    assert caught.value.stage == "checkpoint"
    with pytest.raises(fs.FastSecDecError):
        session.freeze_production(points_per_batch=128, batches=4)
    assert session.snapshot().completed_points == 128
    assert session.step(4).completed_points == 512  # Resume the same pilot object.
    adapted = session.adapt_pilot()
    assert adapted.completed_points == 0 and adapted.uncertainty == "pilot_only"
    session.step(4)
    production = session.freeze_production(points_per_batch=256, batches=3)
    assert production.completed_points == 0 and production.planned_points == 768
    assert production.estimate is None and session.checkpoint_available
    assert session.stage == "production" and session.settings.points_per_batch == 256
    assert production.evaluation_diagnostics.evaluations == 0
    final = session.step(3)
    assert final.completed_points == 768 and final.estimate.production_complete
    assert math.isclose(sum(p for _, p in session.sector_probabilities), 1.0, abs_tol=1e-15)


def test_observer_keyboard_interrupt_keeps_accepted_batch(kernels):
    session = kernels.mc_session(settings(), pilot=True)
    def interrupt(_):
        raise KeyboardInterrupt
    with pytest.raises(KeyboardInterrupt):
        session.step(observer=interrupt)
    assert session.snapshot().completed_points == 128
    assert session.snapshot().stop_reason == "cancelled"
    assert session.step(4).completed_points == 512


def test_checkpoint_rejects_wrong_lane_and_native_identity(kernels):
    session = kernels.mc_session(settings())
    session.step()
    saved = session.checkpoint()
    with pytest.raises(fs.FastSecDecError):
        kernels.restore(saved)
    with pytest.raises(fs.FastSecDecError):
        kernels.restore_mc(kernels.session(fs.QmcSettings(points=16, shifts=2)).checkpoint())
    state = json.loads(saved)
    state["replay"] = []
    with pytest.raises(fs.FastSecDecError):
        kernels.restore_mc(json.dumps(state).encode())
    state = json.loads(saved)
    native = json.loads(bytes(state["session"]))
    native["problem"]["content_id"] = "different kernels"
    state["session"] = list(json.dumps(native).encode())
    with pytest.raises(fs.FastSecDecError):
        kernels.restore_mc(json.dumps(state).encode())


def test_complex_weight_and_native_covariance_are_preserved(kernels):
    arguments = fixed_arguments(inputs.massive_triangle())
    arguments["measure_multiplier"] = 2 + 3 * Expression.I
    complex_kernels = fs.Integral(**arguments).generate(1).compile()
    design = fs.HavanaDiscreteSettings(points_per_batch=1024, batches=8, seed=13, bins=4)
    probabilities = [0.25, 0.75]
    real = kernels.mc_session(design, sector_probabilities=probabilities).step(8).estimate
    mixed = complex_kernels.mc_session(design, sector_probabilities=probabilities).step(8).estimate
    assert mixed.orders == [0, 0, 1, 1]
    assert mixed.components == ["real", "imag", "real", "imag"]
    scales = [2, 3, 2, 3]
    for index, scale in enumerate(scales):
        assert mixed.mean[index] == pytest.approx(scale * real.mean[index // 2], rel=2e-12)
        for other, other_scale in enumerate(scales):
            expected = scale * other_scale * real.covariance_of_mean[(index // 2) * 2 + other // 2]
            assert mixed.covariance_of_mean[index * 4 + other] == pytest.approx(expected, rel=2e-10, abs=1e-16)
    # An unequal-probability native run must still retain the analytic scalar normalization.
    assert abs(real.mean[0] + 2 * math.asinh(0.5)**2) < max(8 * real.standard_error[0], 0.02)
