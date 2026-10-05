"""Exact native model bindings for the optional fixed-point ggHH input.

The expensive sector-generation lifecycle is a separately bounded notebook gate.
"""

import json
from pathlib import Path
import sys

import pytest
from symbolica import E, Replacement
from symbolica.community import hepkit as hep

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase import gghh

pytestmark = pytest.mark.skipif(
    sys.platform == "emscripten" or getattr(hep, "fastsecdec", None) is None,
    reason="requires native experimental-fastsecdec wheel and ggHH inputs",
)


def test_bound_model_contains_exact_masses_and_closed_native_couplings():
    card_text = (gghh.ASSETS / "parameters.json").read_text()
    model = hep.Model(gghh.ASSETS / "model.json").with_parameter_card(hep.ParameterCard.from_json(card_text))
    values = gghh._scalar_values(model, json.loads(card_text))
    assert set(values) == {p.symbol for p in model.parameters} | {c.symbol for c in model.couplings}
    for name, expected in [("MT", "345/2"), ("MH", "125"), ("ymt", "345/2"), ("WT", "0"), ("WH", "0")]:
        assert values[model.parameter(name).symbol] == E(expected)
    rules = [Replacement(symbol, value) for symbol, value in values.items()]
    for coupling in model.couplings:
        assert coupling.expression.replace_multiple(rules) == values[coupling.symbol]
    assert all(not value.contains(symbol) for value in values.values() for symbol in values)
    assert any(values[coupling.symbol] != E("0") for coupling in model.couplings)


def test_explicit_internal_card_override_remains_authoritative():
    card_text = (gghh.ASSETS / "parameters.json").read_text()
    card = hep.ParameterCard.from_json(card_text)
    card.set("aEW", 0.125, 0.0)
    model = hep.Model(gghh.ASSETS / "model.json").with_parameter_card(card)
    restriction = dict(json.loads(card_text), aEW=[0.125, 0.0])
    assert model.parameter("aEW").nature == hep.ParameterNature.INTERNAL
    values = gghh._scalar_values(model, restriction)
    assert values[model.parameter("aEW").symbol] == E("1/8")
