"""Thin native family ownership; requires a refreshed Community host/reexports."""
import gc
import json
import math
from pathlib import Path
import sys

import pytest
from symbolica.community.hepkit import sector_decomposition as sd

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs
from _fixtures import fixed_arguments


@pytest.fixture(scope="module")
def integral():
    return sd.Integral(**fixed_arguments(inputs.massive_triangle()))


def settings():
    return sd.CompilationSettings(backend="eager", horner_iterations=0)


def finish(owner):
    while not owner.complete:
        owner.step(32)
    assert owner.failed is None
    return owner.result


def test_constructor_and_false_observer_are_inert(integral, tmp_path, monkeypatch):
    monkeypatch.setenv("TMPDIR", str(tmp_path))
    owner = integral.generation_family_session(
        ["fixed", "off"], compilation_settings=settings())
    initial = owner.snapshot()
    assert initial.generation.elapsed_seconds == 0
    assert initial.completed_units == initial.prepared_sources == 0
    assert owner.result is None and owner.resident_recipe is None
    assert owner.recipes == ["off", "fixed"]
    assert list(tmp_path.iterdir()) == []
    owner.step(100, observer=lambda _: False)
    assert owner.result is None and owner.failed is None
    assert owner.snapshot().completed_units == 0
    assert list(tmp_path.iterdir()) == []
    with pytest.raises(AttributeError):
        initial.completed_units = 3
    with pytest.raises(ValueError, match="positive"):
        owner.step(0)


@pytest.mark.parametrize("arguments", [
    {"recipes": []},
    {"recipes": ["off", "off"]},
    {"recipes": ["fixed"]},
    {"recipes": ["off"], "resident_recipe": "fixed"},
])
def test_native_family_admission_is_not_duplicated(integral, arguments):
    with pytest.raises(sd.FastSecDecError):
        integral.generation_family_session(**arguments)
    with pytest.raises(ValueError, match="recipe"):
        integral.generation_family_session(["unknown"])


@pytest.fixture(scope="module", params=["symbolic", "numerical_dual"])
def completed(integral, request):
    owner = integral.generation_family_session(
        ["fixed", "off"], mode=request.param, default_recipe="off",
        resident_recipe="fixed", compilation_settings=settings())
    owner.step()  # Explicit parameterization/storage; no native generation unit yet.
    seen = []

    def pause(snapshot):
        seen.append(snapshot)
        return len(seen) != 2

    owner.step(10000, observer=pause)
    assert len(seen) == 2 and not owner.complete and owner.failed is None
    completed_units = owner.snapshot().completed_units
    interruption = KeyboardInterrupt("caller paused family")

    def interrupt(_):
        raise interruption

    with pytest.raises(KeyboardInterrupt) as caught:
        owner.step(10000, observer=interrupt)
    assert caught.value is interruption and owner.failed is None
    assert owner.snapshot().completed_units == completed_units
    elapsed = []

    def record(snapshot):
        elapsed.append(snapshot.generation.elapsed_seconds)

    while not owner.complete:
        owner.step(100, observer=record)
    assert len(elapsed) > 2
    assert elapsed == sorted(elapsed)  # Includes multiple native units per call.
    snapshot = owner.snapshot()
    assert snapshot.completed_recipes == snapshot.total_recipes == 2
    assert snapshot.prepared_sources > 0 and snapshot.persisted_units > 0
    assert owner.result is owner.result
    assert owner.step().generation.stage == "complete"
    return owner


def test_resident_and_archive_default_are_independent(completed):
    archive = completed.result
    assert archive.default_recipe == "off"
    assert archive.resident_recipe == "fixed"
    assert archive.recipes == ["off", "fixed"]
    assert archive.select("fixed") is archive.select("fixed")
    assert archive.select("off") is not archive.select("fixed")
    resident = archive.select("fixed")
    assert resident.snapshot().sectors == resident.sector_count
    assert resident.snapshot().kernels == resident.sector_count
    assert "Resident fixed" in resident.snapshot().detail
    catalogue = json.loads(archive.catalogue_json())
    assert catalogue["content_id"] == archive.content_id
    assert catalogue["source_identity"] == archive.source_identity
    with pytest.raises(sd.FastSecDecError, match="capability"):
        archive.select("polynomial")


def test_native_archive_save_reload_and_detached_kernel_lifetime(completed, tmp_path):
    archive = completed.result
    path = tmp_path / "family.fsd"
    path.write_bytes(b"previous unrelated bytes")
    archive.save(path)
    assert path.read_bytes() == archive.to_bytes()
    loaded = sd.RecipeArchive.load(path)
    assert loaded.default_recipe is loaded.resident_recipe is None
    assert loaded.content_id == archive.content_id
    assert loaded.recipes == archive.recipes
    selected = loaded.select("fixed")
    assert selected.content_id == archive.select("fixed").content_id
    data = loaded.to_bytes()
    del loaded
    path.unlink()
    gc.collect()
    assert selected.to_bytes() == archive.select("fixed").to_bytes()
    restored = sd.RecipeArchive.from_bytes(data, default_recipe="fixed")
    assert restored.default_recipe == "fixed" and restored.resident_recipe is None
    assert restored.select("off").content_id == archive.select("off").content_id
    with pytest.raises(sd.FastSecDecError, match="capability"):
        sd.RecipeArchive.from_bytes(data, default_recipe="sign_aware")
    with pytest.raises(sd.FastSecDecError):
        sd.RecipeArchive.from_bytes(data[:20])


def estimate(kernels):
    session = kernels.session(sd.QmcSettings(
        points=1024, shifts=4, seed=51, package_points=128))
    while not session.complete:
        session.step(16)
    result = session.snapshot().estimate
    assert result.production_complete
    return {(order, part): (value, error) for order, part, value, error in zip(
        result.orders, result.components, result.mean, result.standard_error)}


def test_fixed_off_and_native_reload_agree_with_analytic_triangle(completed):
    archive = completed.result
    fixed = archive.select("fixed").with_parameters(
        {}, contour=sd.ContourSettings.fixed(0.01, validation="off"))
    off = archive.select("off")
    off = off.with_parameters({})
    expected = -2 * math.asinh(0.5) ** 2
    for result in (estimate(off), estimate(fixed)):
        value, error = result[(0, "real")]
        assert abs(value - expected) <= max(8 * error, 2e-5)
        if (0, "imag") in result:
            value, error = result[(0, "imag")]
            assert abs(value) <= max(8 * error, 2e-5)
    restored = sd.RecipeArchive.from_bytes(archive.to_bytes()).select("fixed")
    restored = restored.with_parameters({}, contour=sd.ContourSettings.fixed(0.01, validation="off"))
    assert estimate(restored) == estimate(fixed)


def test_no_resident_is_really_none_and_result_outlives_session(integral):
    owner = integral.generation_family_session(["off"], compilation_settings=settings())
    archive = finish(owner)
    assert archive.resident_recipe is None
    del owner
    gc.collect()
    assert archive.select("off").sector_count > 0
    assert archive.to_bytes()


@pytest.mark.parametrize("mode", ["symbolic", "numerical_dual"])
def test_dynamic_family_restoration_pilot_and_policy_resume(integral, mode):
    owner = integral.generation_family_session(
        ["polynomial", "sign_aware"], mode=mode,
        default_recipe="polynomial", resident_recipe="sign_aware",
        compilation_settings=settings(),
    )
    archive = sd.RecipeArchive.from_bytes(finish(owner).to_bytes())
    expected = -2 * math.asinh(0.5) ** 2
    identities = []
    for recipe in ("polynomial", "sign_aware"):
        template = sd.Kernels.from_bytes(archive.select(recipe).to_bytes())
        assert not template.runtime_parameters  # Native contour inputs stay separate.
        configured = template.with_parameters({}, contour=sd.ContourSettings.dynamical(
            0.8, lambda_cap=0.25, construction=recipe,
            validation="pilot", pilot_points=2,
        ))
        for chart in configured.contour_validation_charts:
            for coordinate in (0.3, 0.6):
                configured.validate_contour_point(
                    chart.chart_index, [coordinate] * chart.dimension)
        assert configured.finish_contour_pilot().pilot_complete
        identities.append(configured.content_id)
        result = estimate(configured)
        for component, reference in (("real", expected), ("imag", 0.0)):
            value, error = result[(0, component)]
            assert abs(value - reference) <= max(8 * error, 2e-5)

        qmc = sd.QmcSettings(points=32, shifts=2, rule="hkkn_alpha3")
        sampled = configured.session(qmc)
        while not sampled.complete:
            sampled.step()
        unchecked = configured.with_contour_validation("off", pilot_points=2)
        assert unchecked.content_id == configured.content_id
        resumed = unchecked.restore(sampled.checkpoint())
        assert resumed.snapshot().estimate.mean == sampled.snapshot().estimate.mean
        assert resumed.contour_provenance.validation == "off"
        assert resumed.contour_provenance.pilots[0].complete

        changed = template.with_parameters({}, contour=sd.ContourSettings.dynamical(
            0.7, lambda_cap=0.25, construction=recipe, validation="off"))
        assert changed.content_id != configured.content_id
        with pytest.raises(sd.FastSecDecError):
            changed.restore(sampled.checkpoint())
    assert identities[0] != identities[1]
