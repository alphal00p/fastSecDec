"""The native Model owner resolves exact point data for decomposition callers."""

import json

import pytest
from symbolica import E, Expression, S
from symbolica.community import hepkit as hep


def test_inline_standard_model_point_equals_explicit_card():
    model = hep.Model.standard_model()
    before = model.to_json()
    card = model.default_parameter_card()
    overrides = {}
    for name, value in (("MT", 172.5), ("ymt", 172.5), ("WT", 0.0), ("WH", 0.0)):
        card.set(name, value, 0.0)
        overrides[model.parameter(name).symbol] = E("345/2") if value else E("0")
    values = model.scalar_bindings(overrides=overrides)
    assert values == model.scalar_bindings(card)
    assert values == model.with_parameter_card(card).scalar_bindings(card)
    assert all(type(key) is Expression and type(value) is Expression
               for key, value in values.items())
    assert values[model.parameter("MT").symbol] == E("345/2")
    assert model.to_json() == before
    # Public outputs own their atoms independently of the model and input maps.
    del model, card, overrides
    assert values[S("UFO::ymt")] == E("345/2")


def test_native_model_analytic_dependencies_override_stale_values():
    definition = json.loads(hep.Model.phi4().to_json())
    definition["parameters"].append({
        "name": "derived", "nature": "internal", "parameter_type": "complex",
        "expression": "2*mass", "value": [999.0, 0.0],
    })
    definition["couplings"][0]["expression"] = "derived*derived"
    definition["couplings"][0]["value"] = [888.0, 0.0]
    model = hep.Model.from_json(json.dumps(definition))
    mass, derived = model.parameter("mass").symbol, model.parameter("derived").symbol
    coupling = model.couplings[0].symbol
    values = model.scalar_bindings(overrides={mass: E("3+4i")})
    assert values[derived] == E("6+8i")
    assert values[coupling] == E("-28+96i")
    card = hep.ParameterCard()
    card.set("derived", 7.0, 0.0)
    values = model.scalar_bindings(card, overrides={mass: E("3")})
    assert values[derived] == E("7") and values[coupling] == E("49")
    values = model.scalar_bindings(card, overrides={derived: E("11")})
    assert values[coupling] == E("121")


def test_binding_cycles_and_non_symbol_keys_raise_without_mutation():
    model = hep.Model.phi4()
    before = model.to_json()
    a, b, outside = S("literal::a_", "literal::b_", "literal::outside")
    values = model.scalar_bindings(overrides={a: b + outside, b: E("2")})
    assert values[a] == 2 + outside
    with pytest.raises(hep.ModelError, match="cyclic|unresolved"):
        model.scalar_bindings(overrides={a: b + 1, b: a + 1})
    with pytest.raises(ValueError, match="plain symbols"):
        model.scalar_bindings(overrides={a + 1: E("2")})
    with pytest.raises(TypeError):
        model.scalar_bindings(overrides={"mass": E("2")})
    assert model.to_json() == before
