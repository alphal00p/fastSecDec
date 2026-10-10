"""Native IntegralFamily ownership shares all existing generation/session paths."""
import gc
import math
from pathlib import Path
import sys

import pytest
from symbolica import E, S
from symbolica.community import hepkit as hep

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs
from _fixtures import fixed_arguments

sd = hep.sector_decomposition


def settings():
    return sd.CompilationSettings(backend="eager", horner_iterations=0)


def finish(owner):
    while not owner.complete:
        owner.step(32)
    assert owner.failed is None
    return owner.result


def tadpole():
    d, k, p, mass, eps = S("retained_family::D", "retained_family::k",
                          "retained_family::p", "retained_family::m2",
                          "retained_family::eps")
    kin = hep.Kinematics(d, momenta=[k, p]).with_scalar_product(p, p, E("-1"))
    family = hep.IntegralFamily([k], [p], [kin.scalar_product(k, k) - mass],
                                kinematics=kin)
    return family, eps, mass


def test_family_constructor_retains_signed_slots_and_defers_native_admission(tmp_path, monkeypatch):
    family, eps, mass = tadpole()
    completed = family.complete()
    assert len(completed.denominators) == 2
    monkeypatch.setenv("TMPDIR", str(tmp_path))
    owner = sd.Integral.from_family(
        completed, regulator=eps, powers=(3, -2), numerator=E("1"),
        dimension=2 - 2 * eps, scalar_values={mass: E("1")})
    assert owner.input_kind == "family"
    assert owner.powers == [(0, 3), (1, -2)]
    session = owner.generation_family_session(
        ["off", "polynomial", "sign_aware"], compilation_settings=settings())
    assert session.result is None and session.snapshot().completed_units == 0
    assert session.snapshot().prepared_sources == 0
    assert list(tmp_path.iterdir()) == []
    session.step(100, observer=lambda _: False)
    assert session.snapshot().completed_units == 0
    assert list(tmp_path.iterdir()) == []
    # Native power admission runs at explicit generation, not object display.
    invalid = sd.Integral.from_family(family, regulator=eps, powers=[0], numerator=E("1"))
    assert invalid.powers == [(0, 0)]
    with pytest.raises(sd.FastSecDecError) as caught:
        invalid.generate(progress=None)
    assert caught.value.stage == "input"


@pytest.mark.parametrize("auxiliary_power", [0, -2])
def test_signed_family_projection_and_weight_are_applied_once(auxiliary_power):
    family, eps, mass = tadpole()
    completed = family.complete()
    numerator = E("1") + family.kinematics.scalar_product(
        family.loop_momenta[0], family.external_momenta[0]) ** 2
    arguments = dict(regulator=eps, dimension=2 - 2 * eps,
                     scalar_values={mass: E("1")}, measure_multiplier=E("3+2i"))
    retained = sd.Integral.from_family(
        completed, powers=[3, auxiliary_power], numerator=numerator, **arguments)
    projected = sd.Integral.from_family(
        family, powers=[3],
        numerator=numerator * completed.denominators[1] ** (-auxiliary_power),
        **arguments)
    actual = retained.generate(coefficient_expansion="coefficient_series", progress=None)
    expected = projected.generate(coefficient_expansion="coefficient_series", progress=None)
    assert len(actual.metadata.domain.parameters) == 1
    assert actual.compile(settings=settings()).to_bytes() == expected.compile(settings=settings()).to_bytes()
    # The legacy synchronous route returns the same generated owner; contour=True
    # deliberately remains a fixed singleton, not an implicitly compiled archive.
    legacy = sd.sector_decompose(
        completed, powers=[3, auxiliary_power], numerator=numerator,
        coefficient_expansion="coefficient_series", progress=None, **arguments)
    assert isinstance(legacy, sd.GeneratedIntegral)
    assert legacy.compile(settings=settings()).to_bytes() == expected.compile(settings=settings()).to_bytes()


def test_graph_and_retained_family_keep_identical_native_physics():
    value = inputs.massive_triangle()
    family = value.diagram.propagator_family(kinematics=value.kinematics)
    graph_arguments = fixed_arguments(value)
    graph_arguments["measure_multiplier"] = E("3")
    graph = sd.Integral(**graph_arguments)
    retained = sd.Integral.from_family(
        family, regulator=value.regulator, dimension=value.dimension,
        powers=[1] * len(family.denominators), numerator=value.scalar_numerator(),
        scalar_values=value.scalar_values, measure_multiplier=E("3"))
    assert graph.input_kind == "graph"
    assert all(power > 0 for _, power in graph.powers)
    for contour in (False, True):
        left = graph.generate(contour=contour, progress=None)
        right = retained.generate(contour=contour, progress=None)
        assert left.compile(settings=settings()).to_bytes() == right.compile(settings=settings()).to_bytes()
        assert right.contour_capable == contour
    session = retained.generation_session(compilation_settings=settings())
    assert session.snapshot().completed == 0
    while not session.complete:
        session.step(32)
    assert session.kernels.content_id == retained.generate(
        coefficient_expansion="coefficient_series", progress=None).compile(settings=settings()).content_id


def estimate(kernels):
    session = kernels.session(sd.QmcSettings(points=1024, shifts=4, seed=38173,
                                            package_points=128))
    while not session.complete:
        session.step(16)
    result = session.snapshot().estimate
    assert result.production_complete
    return result


@pytest.mark.parametrize("mode", ["symbolic", "numerical_dual"])
def test_retained_family_dynamic_archive_pilots_and_complex_reference(mode):
    value = inputs.massive_triangle()
    family = value.diagram.propagator_family(kinematics=value.kinematics)
    integral = sd.Integral.from_family(
        family, regulator=value.regulator, dimension=value.dimension,
        powers=[1] * len(family.denominators),
        numerator=(1 + E("1i")) * value.scalar_numerator(),
        measure_multiplier=E("3"), scalar_values=value.scalar_values)
    owner = integral.generation_family_session(
        ["polynomial", "sign_aware"], default_recipe="polynomial",
        resident_recipe="sign_aware", mode=mode, compilation_settings=settings())
    archive = finish(owner)
    assert archive.default_recipe == "polynomial"
    assert archive.resident_recipe == "sign_aware"
    assert archive.select("sign_aware") is archive.select("sign_aware")
    restored = sd.RecipeArchive.from_bytes(archive.to_bytes())
    del owner, integral, archive, family
    gc.collect()
    reference = -6 * math.asinh(0.5) ** 2
    for recipe in ("polynomial", "sign_aware"):
        kernels = restored.select(recipe).with_parameters(
            {}, contour=sd.ContourSettings.dynamical(
                0.8, lambda_cap=0.25, construction=recipe,
                validation="pilot", pilot_points=2))
        with pytest.raises(sd.FastSecDecError):
            kernels.session(sd.QmcSettings(points=32, shifts=2, rule="hkkn_alpha3"))
        for chart in kernels.contour_validation_charts:
            for x in (0.3, 0.6):
                kernels.validate_contour_point(chart.chart_index, [x] * chart.dimension)
        assert kernels.finish_contour_pilot().pilot_complete
        result = estimate(kernels)
        assert result.orders == [0, 0] and result.components == ["real", "imag"]
        assert len(result.covariance_of_mean) == 4
        for mean, error in zip(result.mean, result.standard_error):
            assert error < 2e-4
            assert abs(mean - reference) <= max(8 * error, 2e-5)


def test_family_runtime_inputs_and_native_exact_normalization():
    family, eps, mass = tadpole()
    integral = sd.Integral.from_family(
        family, regulator=eps, powers=[2], numerator=E("3"),
        dimension=2 - 2 * eps, runtime_parameters=[mass])
    owner = integral.generation_family_session(
        ["polynomial", "sign_aware"], default_recipe="polynomial",
        compilation_settings=settings())
    archive = finish(owner)
    for recipe in ("polynomial", "sign_aware"):
        template = archive.select(recipe)
        assert template.runtime_parameters == [mass]
        saved_program = template.to_bytes()
        source_identity = template.content_id
        source_layout = (template.orders, template.components, template.sector_count)
        source_charts = [(chart.chart_index, chart.dimension)
                         for chart in template.contour_validation_charts]
        bound_identities = []
        for value in (1., 2.):
            kernels = template.with_parameters(
                {mass: value}, contour=sd.ContourSettings.dynamical(
                    0.8, construction=recipe, validation="off"))
            bound_identities.append(kernels.content_id)
            assert kernels.runtime_parameters == [mass]
            assert (kernels.orders, kernels.components, kernels.sector_count) == source_layout
            assert [(chart.chart_index, chart.dimension)
                    for chart in kernels.contour_validation_charts] == source_charts
            # Native D=2 tadpole J2=1/m^2 in d^Dk/(i*pi^(D/2)).
            exact = dict(zip(zip(kernels.orders, kernels.components), kernels.exact_coefficients))
            assert exact[(0, "real")] == 3. / value
            assert all(coefficient == 0. for (order, part), coefficient in exact.items()
                       if (order, part) != (0, "real"))
        assert bound_identities[0] != bound_identities[1]
        assert template.content_id == source_identity
        assert template.to_bytes() == saved_program


def test_dynamic_settings_card_displays_complete_prescription():
    settings = sd.ContourSettings.dynamical(
        0.7, lambda_cap=0.125, displacement_cap=2., construction="polynomial")
    html = settings._repr_html_()
    for label in ("Dynamic construction", "Safety fraction S", "Strength cap L",
                  "Displacement cap R", "polynomial", "0.7", "0.125"):
        assert label in html
