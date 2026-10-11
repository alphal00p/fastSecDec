"""Existing HEPKit entrypoints; requires a matching native Community host."""
import gc
import math
from pathlib import Path
import pytest
from symbolica import E, S
from symbolica.community import hepkit as hep
from symbolica.community.hepkit import sector_decomposition as sd

ROOT = Path(__file__).resolve().parents[3]

@pytest.fixture(scope="module")
def source():
    model = hep.Model(ROOT / "examples/models/scalar.json")
    diagram = hep.FeynmanDiagram.from_dot(
        model, (ROOT / "examples/graphs/bubble.dot").read_text())
    eps, dim = S("threshold_prepared_py::eps", "threshold_prepared_py::D")
    p, k = hep.Kinematics.external_momentum(), hep.Kinematics.loop_momentum()
    kinematics = hep.Kinematics(dim, momenta=[p(1), k(0)])
    kinematics = kinematics.with_scalar_product(p(1), p(1), E("16"))
    gamma = E("gamma")
    arguments = dict(regulator=eps, dimension=4-2*eps, kinematics=kinematics,
        model_parameters="fixed", scalar_values={S("UFO::mt"): E("sqrt(3)")},
        measure_multiplier=gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)**2),
        threshold_decomposition=True, coefficient_expansion="coefficient_series",
        subtraction="integrate_by_parts", progress=None)
    return diagram, arguments


def test_preparation_cancellation_and_retained_owner(source):
    diagram, arguments = source
    with pytest.raises(sd.CancelledError) as cancelled:
        diagram.sector_decompose(**arguments, observer=lambda _: False)
    assert cancelled.value.stage == "generation"
    marker = RuntimeError("original callback error")
    def fail(_):
        raise marker
    with pytest.raises(RuntimeError) as caught:
        sd.sector_decompose(diagram, **arguments, observer=fail)
    assert caught.value is marker
    prepared = diagram.sector_decompose(**arguments)
    assert type(prepared) is sd.PreparedThreshold
    assert prepared.snapshot().kernels == 0 and prepared.archive is None
    assert prepared.compilation_settings.horner_iterations == 10
    work = prepared.generation_session()
    assert work is prepared.generation_session() and work.prepared
    with pytest.raises(sd.CancelledError) as cancelled:
        prepared.compile(observer=lambda _: False, progress=None)
    assert cancelled.value.stage == "compilation" and work.snapshot().completed == 0
    with pytest.raises(RuntimeError):
        work.compile_next(observer=lambda _: prepared.snapshot(), progress=None)
    assert work.failed is None
    work.compile_next(progress=None)
    assert work.snapshot().completed == 1
    kernels = prepared.compile(progress=None)
    assert prepared.complete and prepared.compile(progress=None) is kernels
    assert prepared.archive.default_recipe == "threshold"


def test_free_diagram_family_full_complex_oracle(source):
    diagram, arguments = source
    family = diagram.propagator_family(kinematics=arguments["kinematics"])
    family_args = dict(arguments, powers=[1]*len(family.denominators), numerator=E("1"))
    family_args.pop("model_parameters")
    owners = [sd.sector_decompose(diagram, **arguments),
              diagram.sector_decompose(**arguments),
              family.sector_decompose(**family_args)]
    assert all(o.snapshot().kernels == 0 and not o.complete for o in owners)
    kernels = [o.compile(progress=None) for o in owners]
    del owners
    gc.collect()
    expected = {(-1, "real"): 1., (-1, "imag"): 0.,
                (0, "real"): 2-1.5*math.log(3), (0, "imag"): math.pi/2}
    vectors = []
    for native in kernels:
        session = native.session(sd.QmcSettings(points=4096, shifts=8, seed=202610111001))
        assert session.snapshot().completed_points == 0
        while not session.complete:
            session.step(16)
        estimate = session.snapshot().estimate
        assert estimate.production_complete
        for order, part, value, error in zip(estimate.orders, estimate.components,
                                            estimate.mean, estimate.standard_error):
            assert abs(value-expected[(order, part)]) < 8*error + 1e-6
        vectors.append(list(estimate.mean))
    assert all(abs(a-b) < 2e-12 for row in vectors[1:] for a, b in zip(vectors[0], row))
