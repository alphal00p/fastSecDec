"""A linked backend works through owner methods without canonical package aliases."""

import builtins
import sys

# The Rust host supplies these modules directly.
core = globals()["core"]
hep = globals()["hep"]
sd = globals()["sd"]
tensor = globals()["tensor"]

sys.modules.pop("symbolica.community.hepkit_fastsecdec_native", None)
assert not any(
    name == "symbolica" or name.startswith("symbolica.") for name in sys.modules
)
original_import = builtins.__import__


def without_symbolica(name, globals=None, locals=None, fromlist=(), level=0):
    if name == "symbolica" or name.startswith("symbolica."):
        raise AssertionError(f"backend tried to import {name}")
    return original_import(name, globals, locals, fromlist, level)


builtins.__import__ = without_symbolica
try:
    model = hep.Model.phi_3_4()
    diagram = hep.FeynmanDiagram.from_dot(
        model, 'digraph tadpole { a -> a [particle="phi", lmb_id=0]; }'
    )
    before = diagram.to_json()
    zero = tensor.TensorExpression(
        core.E("0"),
        structure=tensor.TensorStructure([tensor.Representation.mink(4)("mu")]),
    )
    copied = sd.with_diagram_expressions(diagram, numerator=zero, projector=zero)
    assert type(copied) is hep.FeynmanDiagram
    assert diagram.to_json() == before
    assert copied.numerator_expression().axes == zero.axes
    assert copied.projector_expression().axes == zero.axes
    assert copied.numerator_expression().to_expression() == core.E("0")
    scalar = sd.with_diagram_expressions(
        diagram,
        numerator=core.E("3"),
        projector=core.E("2"),
        overall_factor=core.E("5"),
    )
    assert scalar.numerator_expression().to_expression() == core.E("3")
    assert scalar.projector_expression().to_expression() == core.E("2")
    assert scalar.overall_factor_expression() == core.E("5")
    try:
        sd.with_diagram_expressions(diagram, numerator=object())
    except TypeError:
        pass
    else:
        raise AssertionError("invalid numerator was accepted")

    eps, dimension, k = core.S("eps", "D", "k")
    point = hep.Kinematics(dimension, momenta=[k])
    family = hep.IntegralFamily(
        [k], [], [point.scalar_product(k, k) - 1], kinematics=point
    )
    options = {
        "regulator": eps,
        "powers": [2],
        "numerator": core.E("1"),
        "progress": None,
    }
    direct = sd.sector_decompose(family, **options)
    owned = family.sector_decompose(**options)
    assert type(owned) is type(direct) is sd.GeneratedIntegral
    assert owned.orders == direct.orders
    assert owned.sector_count == direct.sector_count
    assert owned.exact_coefficients == direct.exact_coefficients
    assert all(isinstance(value, core.Expression) for value in owned.exact_coefficients)

    for extra in ({"unknown_option": True}, {"max_order": "invalid"}):
        failures = []
        for invoke in (
            lambda **kwargs: sd.sector_decompose(family, **kwargs),
            family.sector_decompose,
        ):
            try:
                invoke(**(options | extra))
            except TypeError as error:
                failures.append(str(error))
            else:
                raise AssertionError("invalid backend argument was accepted")
        assert failures[0] == failures[1]

    failure = ValueError("observer failure")

    def fail(snapshot):
        raise failure

    try:
        family.sector_decompose(**options, observer=fail)
    except ValueError as error:
        assert error is failure
    else:
        raise AssertionError("observer exception was swallowed")
finally:
    builtins.__import__ = original_import

assert not any(
    name == "symbolica" or name.startswith("symbolica.") for name in sys.modules
)
