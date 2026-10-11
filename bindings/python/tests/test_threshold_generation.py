"""Requires a freshly linked native Community host; no mocked physics owners."""
import gc
import json
import math
from pathlib import Path
import pytest
from symbolica import E, S
from symbolica.community import hepkit as hep
from symbolica.community.hepkit import sector_decomposition as sd

ROOT = Path(__file__).resolve().parents[3]

@pytest.fixture(scope="module")
def bubble():
    model = hep.Model(ROOT / "examples/models/scalar.json")
    diagram = hep.FeynmanDiagram.from_dot(
        model, (ROOT / "examples/graphs/bubble.dot").read_text())
    eps, dim = S("hepkit_threshold_py::eps", "hepkit_threshold_py::D")
    p, k = hep.Kinematics.external_momentum(), hep.Kinematics.loop_momentum()
    kinematics = hep.Kinematics(dim, momenta=[p(1), k(0)])
    kinematics = kinematics.with_scalar_product(p(1), p(1), E("16"))
    # Explicit conventional rGamma normalization, matching native/OneLOop gate.
    gamma = E("gamma")
    normalization = gamma(1 - 2*eps) / (gamma(1 + eps)*gamma(1 - eps)**2)
    return sd.Integral(diagram, regulator=eps, dimension=4-2*eps,
                       kinematics=kinematics, model_parameters="fixed",
                       scalar_values={S("UFO::mt"): E("sqrt(3)")},
                       measure_multiplier=normalization)


def test_constructor_and_false_callbacks_are_inert(bubble, tmp_path, monkeypatch):
    monkeypatch.setenv("TMPDIR", str(tmp_path))
    owner = bubble.threshold_generation_session()
    assert not owner.prepared and not owner.complete and owner.result is None
    assert owner.snapshot().elapsed_seconds == 0
    assert owner.preparation_receipt_json() is None
    assert list(tmp_path.iterdir()) == []
    owner.prepare(observer=lambda _: False, progress=None)
    assert list(tmp_path.iterdir()) == []
    with pytest.raises(ValueError, match="prepare"):
        owner.compile_next(progress=None)
    with pytest.raises(ValueError, match="positive"):
        owner.compile_next(0, progress=None)


def test_raw_checkpoint_recovery_exception_and_no_implicit_sampling(bubble):
    owner = bubble.threshold_generation_session()
    interruption = KeyboardInterrupt("pause after durable solve")
    def interrupt(event):
        if event.detail == "Native threshold Verify":
            raise interruption
    with pytest.raises(KeyboardInterrupt) as caught:
        owner.prepare(observer=interrupt, progress=None)
    assert caught.value is interruption and owner.failed is None
    raw = json.loads(owner.preparation_progress_json())
    assert raw["receipts"]["raw"] is not None
    seen = []
    owner.prepare(observer=lambda event: seen.append(event.detail), progress=None)
    assert owner.prepared and not owner.complete and owner.result is None
    assert "Native threshold Solve" not in seen
    receipt = json.loads(owner.preparation_receipt_json())
    assert receipt["original_dimensions"] == 2
    assert receipt["integration_dimensions"] == 1
    assert receipt["cells"] == 3 and receipt["endpoint_charts"] == 6
    before = owner.snapshot().completed
    owner.compile_next(100, observer=lambda _: False, progress=None)
    assert owner.snapshot().completed == before and not owner.complete
    while not owner.complete:
        prior = owner.snapshot().completed
        owner.compile_next(1, progress=None)
        assert owner.snapshot().completed == prior + 1
    assert owner.result is owner.result
    kernels = owner.result.select("threshold")
    session = kernels.session(sd.QmcSettings(points=1024, shifts=2, seed=202610108011))
    assert session.snapshot().completed_points == 0


def test_archive_survives_session_and_restores_full_complex_bubble(bubble, tmp_path):
    owner = bubble.threshold_generation_session()
    owner.prepare(progress=None)
    while not owner.complete:
        owner.compile_next(2, progress=None)
    archive = owner.result
    assert archive.default_recipe == "threshold" and archive.recipes == ["threshold"]
    catalogue = json.loads(archive.catalogue_json())
    assert catalogue["recipes"][0]["threshold"] is not None
    path = tmp_path / "threshold.fsd"
    archive.save(path)
    del owner
    gc.collect()
    restored = sd.RecipeArchive.load(path, default_recipe="threshold")
    assert restored.content_id == archive.content_id
    kernels = restored.select("threshold")
    del restored
    path.unlink()
    gc.collect()
    qmc = kernels.session(sd.QmcSettings(points=4096, shifts=8, seed=202610108012))
    while not qmc.complete:
        qmc.step(16)
    result = qmc.snapshot().estimate
    expected = {(-1,"real"):1.,(-1,"imag"):0.,
                (0,"real"):2.-1.5*math.log(3.),(0,"imag"):math.pi/2}
    assert result.production_complete
    for order, part, value, error in zip(result.orders,result.components,result.mean,result.standard_error):
        assert abs(value - expected[(order,part)]) < 8*error + 1e-6


def test_existing_session_dispatch_and_ordinary_default(bubble):
    ordinary = bubble.generation_session()
    assert type(ordinary) is sd.GenerationSession
    ordinary.step(observer=lambda _: False)
    assert ordinary.kernels is None
    settings = sd.ThresholdSettings(gcad_limits_json='{"memory_mib":1024}')
    assert json.loads(settings.to_json())["gcad_limits"]["workers"] == 1
    with pytest.raises(ValueError, match="workers=1"):
        sd.ThresholdSettings(gcad_limits_json='{"workers":2}')
    with pytest.raises(ValueError, match="requires"):
        bubble.generation_session(threshold_settings=settings)
    for options in ({"contour": True}, {"mode": "numerical_dual"},
                    {"contour_jacobian": "dual"}):
        with pytest.raises(ValueError):
            bubble.generation_session(threshold_decomposition=True, **options)
    owner = bubble.generation_session(threshold_decomposition=True,
             subtraction="integrate_by_parts", threshold_settings=settings)
    assert type(owner) is sd.ThresholdGenerationSession
    owner.step(observer=lambda _: False, progress=None)
    assert not owner.prepared
    owner.step(progress=None)
    assert owner.prepared and owner.snapshot().completed == 0
    while not owner.complete:
        before = owner.snapshot().completed
        owner.step(progress=None)
        assert owner.snapshot().completed == before + 1
    assert owner.result.default_recipe == "threshold"
