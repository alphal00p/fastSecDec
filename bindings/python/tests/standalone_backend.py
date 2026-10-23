"""A linked backend works through owner methods without canonical package aliases."""

import builtins
import sys

# The Rust host supplies these modules directly.
core = globals()["core"]
hep = globals()["hep"]
sd = globals()["sd"]

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
