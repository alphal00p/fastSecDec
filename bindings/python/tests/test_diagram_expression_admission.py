"""Tensor interfaces and source labels survive the native diagram-copy boundary."""

from pathlib import Path

import pytest
from symbolica import E, S
from symbolica.community import hepkit as hep
from symbolica.community.tensor import TensorExpression


sd = getattr(hep, "sector_decomposition", None)
pytestmark = pytest.mark.skipif(sd is None, reason="requires sector_decomposition wheel")
ROOT = Path(__file__).resolve().parents[3]
VECTOR = "gammalooprs::P(1,spenso::mink(4,gammalooprs::hedge(1,1)))"


def bubble(annotations=""):
    model = hep.Model(ROOT / "examples/models/massless_phi3.json")
    source = (ROOT / "examples/graphs/bubble.dot").read_text()
    source = source.replace("digraph bubble {", "digraph bubble {\n" + annotations)
    return hep.FeynmanDiagram.from_dot(model, source)


@pytest.mark.parametrize("role", ["numerator", "projector"])
@pytest.mark.parametrize("overall", [0, 1])
def test_ranked_zero_keeps_free_indices_until_native_parametrization(role, overall):
    original = bubble()
    before = original.to_json()
    vector = TensorExpression(E(VECTOR), intern="indices")
    zero = 0 * vector
    assert isinstance(zero, TensorExpression) and zero.rank == 1
    assert zero.to_expression() == E("0")

    copied = sd.with_diagram_expressions(
        original, **{role: zero}, overall_factor=E(str(overall)),
    )
    retained = getattr(copied, role + "_expression")()
    assert retained == zero
    assert retained.rank == 1
    assert original.to_json() == before

    dimension, eps = S("diagram_admission::D", "diagram_admission::eps")
    p = hep.Kinematics.external_momentum()
    kin = hep.Kinematics(dimension).with_scalar_product(p(1), p(1), E("-1"))
    integral = sd.Integral(copied, kin, regulator=eps, model_parameters="fixed")
    with pytest.raises(sd.FastSecDecError, match="retains free tensor indices") as caught:
        integral.generate(progress=None)
    assert caught.value.stage == "parametrization"


@pytest.mark.parametrize("role", ["numerator", "projector"])
def test_raw_expression_contracts_with_shared_native_hedge_source_label(role):
    other = "projector" if role == "numerator" else "numerator"
    # Compact DOT stores graph-level num as a prefactor; the numerator belongs
    # to a vertex so this exercises native numerator/projector port matching.
    annotations = (
        f'graph [projector="{VECTOR}", overall_factor="3"];'
        if role == "numerator"
        else f'graph [overall_factor="3"]; a [num="{VECTOR}"];'
    )
    original = bubble(annotations)
    before = original.to_json()
    copied = sd.with_diagram_expressions(
        original, **{role: E(VECTOR)}, overall_factor=E("5"),
    )
    assert getattr(copied, role + "_expression")() == getattr(original, other + "_expression")()
    projected = copied.numerator_expression() * copied.projector_expression()
    assert projected.rank == 0
    scalar = projected.contract().to_dots().to_expression()
    p = hep.Kinematics.external_momentum()
    kin = hep.Kinematics(E("4"))
    assert scalar == kin.scalar_product(p(1), p(1))
    assert copied.overall_factor_expression() == E("5")
    assert original.to_json() == before


@pytest.mark.parametrize("role", ["numerator", "projector", "overall_factor"])
def test_diagram_copy_still_requires_symbolica_expressions(role):
    with pytest.raises(TypeError):
        sd.with_diagram_expressions(bubble(), **{role: 1})


def test_overall_factor_keeps_scalar_expression_semantics():
    scalar = TensorExpression(E("7"))
    copied = sd.with_diagram_expressions(bubble(), overall_factor=scalar)
    assert copied.overall_factor_expression() == E("7")
    assert sd.with_diagram_expressions(copied).to_json() == copied.to_json()
