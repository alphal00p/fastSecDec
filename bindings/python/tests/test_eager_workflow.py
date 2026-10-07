"""Retained native work, explicit points and eager execution on every host."""
import sys
from pathlib import Path

import pytest
from symbolica.community.hepkit import sector_decomposition as sd

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs


@pytest.fixture(scope="module")
def integral():
    return sd.Integral(**inputs.massive_triangle().integral_arguments())


def finish(owner):
    while not owner.complete:
        owner.step(max_units=1)
    assert owner.failed is None
    return owner


def test_retained_generation_pauses_on_false_and_python_exception(integral):
    settings = sd.CompilationSettings(backend="eager", horner_iterations=2, cpe_rounds=10)
    owner = integral.generation_session(compilation_settings=settings)
    assert owner.generated is owner.kernels is None
    assert owner.snapshot().elapsed_seconds == 0
    owner.step()  # Parameterization is the first explicit native unit.
    seen = []

    def pause_once(snapshot):
        seen.append(snapshot)
        return len(seen) != 2

    owner.step(max_units=10000, observer=pause_once)
    assert len(seen) == 2 and not owner.complete  # False is latched for this call.
    assert owner.failed is None
    original = KeyboardInterrupt("native unit boundary")
    seen.clear()

    def interrupt(snapshot):
        seen.append(snapshot)
        if len(seen) == 2:
            raise original

    with pytest.raises(KeyboardInterrupt) as caught:
        owner.step(max_units=10000, observer=interrupt)
    assert caught.value is original and owner.failed is None
    finish(owner)
    assert owner.generated.sector_count == owner.kernels.sector_count
    assert owner.kernels.backend == "symbolica_interpreter"
    assert all(record.symjit_ir_bytes is None for record in owner.kernels.sector_statistics)
    control = integral.generate(coefficient_expansion="coefficient_series", progress=None)
    expected = control.compile(settings=settings, progress=None)
    assert owner.kernels.to_bytes() == expected.to_bytes()
    assert owner.kernels.content_id == expected.content_id
    assert owner.step().stage == "complete"


@pytest.fixture(scope="module")
def compiled(integral):
    owner = finish(integral.generation_session())
    assert owner.kernels.runtime_parameters
    assert not owner.kernels.parameters_bound
    return owner.generated, owner.kernels


def test_runtime_points_are_explicit_and_cold_eager_policy_survives(compiled):
    generated, template = compiled
    with pytest.raises(sd.FastSecDecError):
        template.session(sd.QmcSettings(points=32, shifts=2, rule="hkkn_alpha3"))
    defaults = generated.runtime_parameter_defaults
    assert set(defaults) == set(template.runtime_parameters)
    with pytest.raises(sd.FastSecDecError, match="missing"):
        template.with_parameters({})
    point = template.with_parameters(defaults)
    assert point.parameters_bound and not template.parameters_bound
    assert point.to_bytes() == template.to_bytes()
    loaded = sd.Kernels.from_bytes(point.to_bytes())
    assert loaded.backend == "symbolica_interpreter" and not loaded.parameters_bound
    rebound = loaded.with_parameters(defaults)
    assert rebound.content_id == point.content_id
    changed = dict(defaults)
    key = next(iter(changed))
    changed[key] *= 1.2
    assert template.with_parameters(changed).content_id != point.content_id


@pytest.mark.parametrize("batch_size", [1, 7, 256])
def test_batched_eager_qmc_and_havana_keep_covariance_and_resume(compiled, batch_size):
    generated, template = compiled
    kernels = template.with_parameters(generated.runtime_parameter_defaults,
        stability=sd.StabilitySettings(large_weight_threshold=None))
    qmc = kernels.session(sd.QmcSettings(points=32, shifts=3, package_points=8, rule="hkkn_alpha3"))
    assert qmc.live_observation().total.mean is None
    qmc.step(evaluation_batch_size=batch_size)
    resumed = kernels.restore(qmc.checkpoint())
    while not qmc.complete:
        qmc.step(8, evaluation_batch_size=batch_size)
    while not resumed.complete:
        resumed.step(3, evaluation_batch_size=7)
    left, right = qmc.observation(), resumed.observation()
    assert left.total.mean == pytest.approx(right.total.mean, rel=1e-14, abs=1e-15)
    assert left.total.covariance_of_mean == pytest.approx(right.total.covariance_of_mean, rel=1e-13, abs=1e-18)
    assert qmc.live_observation().source == "complete_lattices"
    assert qmc.live_observation().total.replicas == 3
    assert left.snapshot.evaluation_diagnostics.f64_timing.matrix_invocations == 0
    assert left.total.production_complete
    settings = sd.HavanaDiscreteSettings(points_per_batch=32, batches=3, bins=4, seed=23)
    mc = kernels.mc_session(settings, pilot=True)
    while not mc.complete:
        mc.step(evaluation_batch_size=batch_size)
    assert mc.observation().total is None  # Pilot preview is not accepted production.
    assert mc.live_observation().total.mean is not None
    mc.freeze_production(points_per_batch=32, batches=3)
    assert mc.live_observation().total.mean is None
    mc.step(evaluation_batch_size=batch_size)
    resumed = kernels.restore_mc(mc.checkpoint())
    assert resumed.live_observation().source == "since_resume"
    while not mc.complete:
        mc.step(evaluation_batch_size=batch_size)
    while not resumed.complete:
        resumed.step(evaluation_batch_size=7)
    assert mc.observation().total.mean == pytest.approx(resumed.observation().total.mean, rel=1e-14, abs=1e-15)
    assert mc.observation().total.covariance_of_mean == pytest.approx(resumed.observation().total.covariance_of_mean, rel=1e-12, abs=1e-18)
    assert mc.observation().replica_relation == "shared_across_sectors"


def test_settings_validate_native_contracts():
    assert sd.CompilationSettings().backend == "eager"
    assert sd.CompilationSettings(cpe_rounds=None).cpe_rounds is None
    with pytest.raises(sd.FastSecDecError, match="cores"):
        sd.CompilationSettings(cores=2)
    with pytest.raises(sd.FastSecDecError, match="nested"):
        sd.StabilitySettings(f64_distance=1e-8, double_float_distance=1e-3)
    settings = sd.StabilitySettings(large_weight_threshold=None)
    assert sd.StabilitySettings.from_json(settings.to_json()).to_json() == settings.to_json()
