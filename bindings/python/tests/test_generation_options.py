"""Native generation selectors are explicit, inert and retained for inspection."""
from pathlib import Path
import sys

import pytest
from symbolica.community.hepkit import sector_decomposition as sd

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs
from _fixtures import fixed_arguments


@pytest.fixture(scope="module")
def prepared():
    value = inputs.massive_triangle()
    return value, sd.Integral(**fixed_arguments(value))


@pytest.mark.parametrize("mode", ["symbolic", "numerical_dual"])
@pytest.mark.parametrize("subtraction", ["taylor", "integrate_by_parts"])
def test_generation_session_options_are_inert_and_inspectable(prepared, mode, subtraction):
    owner = prepared[1].generation_session(mode=mode, subtraction=subtraction)
    assert owner.mode == mode
    assert owner.subtraction == subtraction
    assert owner.generated is owner.kernels is None
    assert owner.snapshot().elapsed_seconds == 0
    assert not owner.complete and owner.failed is None
    rendered = owner._repr_html_()
    assert mode in rendered and subtraction in rendered
    assert owner.generated is owner.kernels is None


@pytest.mark.parametrize("argument,value", [
    ("mode", "numerical"), ("mode", "Symbolic"),
    ("subtraction", "ibp"), ("subtraction", "Taylor"),
])
def test_invalid_generation_choices_fail_before_observer(prepared, argument, value):
    events = []
    with pytest.raises(ValueError, match=argument):
        prepared[1].generate(**{argument: value}, observer=events.append, progress=None)
    assert events == []
    with pytest.raises(ValueError, match=argument):
        prepared[1].generation_session(**{argument: value})
    arguments = fixed_arguments(prepared[0])
    diagram = arguments.pop("diagram")
    with pytest.raises(ValueError, match=argument):
        sd.sector_decompose(diagram, **arguments, **{argument: value},
                            observer=events.append, progress=None)
    assert events == []


def test_symbolic_taylor_remains_default(prepared):
    owner = prepared[1].generation_session()
    assert owner.mode == "symbolic" and owner.subtraction == "taylor"
    generated = prepared[1].generate(progress=None)
    assert generated.mode == "symbolic" and generated.subtraction == "taylor"


@pytest.mark.parametrize("mode", ["symbolic", "numerical_dual"])
@pytest.mark.parametrize("subtraction", ["taylor", "integrate_by_parts"])
def test_generation_choices_reach_sync_and_retained_native_owners(prepared, mode, subtraction):
    integral = prepared[1]
    settings = sd.CompilationSettings(backend="eager", horner_iterations=0)
    generated = integral.generate(mode=mode, subtraction=subtraction,
                                  coefficient_expansion="coefficient_series", progress=None)
    assert generated.mode == mode and generated.subtraction == subtraction
    for sector in generated.sectors:
        assert sector.generation_mode in ("symbolic", "numerical_dual")
        for coefficient in sector.aliased_coefficients:
            assert coefficient.generation_mode == sector.generation_mode
            # Explicit inspection asks the native owner for the real expression.
            # Passive root/alias views above do not materialize deferred bodies.
            assert coefficient.expression() is not None
    owner = integral.generation_session(mode=mode, subtraction=subtraction,
                                        compilation_settings=settings)
    while not owner.complete:
        owner.step(max_units=1)
    assert owner.failed is None
    assert owner.generated.mode == mode and owner.generated.subtraction == subtraction
    assert owner.kernels.to_bytes() == generated.compile(settings=settings, progress=None).to_bytes()
