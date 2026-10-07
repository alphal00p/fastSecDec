"""Formatting only; native values and uncertainty are never recomputed."""

from decimal import Decimal, ROUND_HALF_EVEN, localcontext
import math

_SUPERSCRIPTS = str.maketrans("-0123456789", "⁻⁰¹²³⁴⁵⁶⁷⁸⁹")
_COUNT_COLUMNS = {
    "evaluations", "conditioning_checks", "rescues", "failures",
    "weighted_checks", "additional_replays",
}


def scientific(value):
    """Signed, normalized human notation; never used for native calculations."""
    if not math.isfinite(value):
        return "unavailable"
    mantissa, exponent = format(abs(value), ".6e").split("e")
    mantissa = mantissa.rstrip("0").rstrip(".")
    sign = "−" if value < 0 else "+"
    return f"{sign}{mantissa} ·10{str(int(exponent)).translate(_SUPERSCRIPTS)}"


def uncertainty(value, error):
    """Two significant error digits, in units of the last shown central digit.

    The Python owner has no public mean/error formatter. Native float formatting
    rounds the supplied error; Decimal only places its display digits safely
    across the finite f64 exponent range. It does not estimate an uncertainty.
    """
    if not math.isfinite(value):
        return "unavailable"
    if error is None or not math.isfinite(error) or error < 0:
        return f"{scientific(value)} (σ unavailable)"
    central = Decimal(str(abs(value)))
    exponent = central.adjusted() if value else (Decimal(str(error)).adjusted() if error else 0)
    sign = "−" if value < 0 else "+"
    with localcontext() as context:
        # Exact binary64 decimal expansions and the largest relative exponent
        # separation fit here; the caller's Decimal context is never modified.
        context.prec = 1200
        if error == 0:
            mantissa = format(central.scaleb(-exponent), "f")
            if "." in mantissa:
                mantissa = mantissa.rstrip("0").rstrip(".")
            body = f"{mantissa}(0)"
        else:
            rounded_error = Decimal(format(error, ".1e"))
            decimals = max(1 - (rounded_error.adjusted() - exponent), 0)
            mean = Decimal.from_float(float(abs(value))).scaleb(-exponent)
            rounded_mean = mean.quantize(Decimal(1).scaleb(-decimals), rounding=ROUND_HALF_EVEN)
            if rounded_mean >= 10:
                exponent += 1
                decimals = max(1 - (rounded_error.adjusted() - exponent), 0)
                mean = mean.scaleb(-1)
                rounded_mean = mean.quantize(Decimal(1).scaleb(-decimals), rounding=ROUND_HALF_EVEN)
            digits = rounded_error.scaleb(decimals - exponent)
            body = f"{rounded_mean:.{decimals}f}({digits:.0f})"
    return f"{sign}{body} ·10{str(exponent).translate(_SUPERSCRIPTS)}"


def compact_count(count):
    """Four significant decimal digits, with base-1000 K/M/B suffixes."""
    if not isinstance(count, int) or isinstance(count, bool) or count < 0:
        raise ValueError("A count must be a nonnegative integer")
    if count < 1000:
        return str(count)
    scale = 10 ** max(len(str(count)) - 4, 0)
    rounded = (count + scale // 2) // scale * scale
    unit, suffix = ((1_000_000_000, "B") if rounded >= 1_000_000_000 else
                    (1_000_000, "M") if rounded >= 1_000_000 else (1000, "K"))
    whole = rounded // unit
    digits = max(4 - len(str(whole)), 0)
    if not digits:
        return f"{whole} {suffix}"
    fraction = (rounded % unit) // (unit // 10 ** digits)
    return f"{whole}.{fraction:0{digits}d} {suffix}"

EXAMPLES = {
    "Massive triangle": "triangle",
    "Massless box": "box",
    "Rank-two box numerator": "rank_two_box",
    "Coupled two-loop sunset": "sunset",
}

# User-selected allocations; these never run or change an existing native owner.
QMC_PRESETS = {
    "quick": {"points": 1024, "shifts": 8, "seed": 20261005,
              "package_points": 1024, "rule": "kuo_33002", "periodization": "korobov3"},
    "gghh_accuracy": {"points": 32768, "shifts": 16, "seed": 20261007,
                      "package_points": 1024, "rule": "hkkn_alpha3", "periodization": "korobov3"},
}

def validate_configuration(value):
    if value is None:
        return None
    # marimo form validation receives raw frontend dropdown selections; the
    # submitted form.value is converted by the original UI elements afterwards.
    kind = value["example"]
    if isinstance(kind, list):
        kind = EXAMPLES.get(kind[0]) if kind else None
    if not math.isfinite(value["s"]) or value["s"] >= 0:
        return "Choose a finite negative s for this Euclidean example."
    if kind in {"box", "rank_two_box"} and (not math.isfinite(value["t"]) or value["t"] >= 0):
        return "Choose a finite negative t for the box."
    if kind == "triangle" and (not math.isfinite(value["mass"]) or value["mass"] <= 0):
        return "The triangle mass must be finite and positive."
    return None

def epsilon_label(order):
    return "ε" + str(order).translate(_SUPERSCRIPTS)

def table(mo, rows, *, scientific_values=False):
    if not rows:
        return mo.md("No native observations yet.")
    float_format = scientific if scientific_values else lambda value: f"{value:.8g}"
    formats = {key: (lambda value: "—" if value is None else float_format(value))
               for key in rows[0] if any(isinstance(row.get(key), float) for row in rows)}
    for key in rows[0]:
        if key in _COUNT_COLUMNS or any(word in key.split() for word in ("points", "batches", "shifts")):
            formats[key] = lambda value: ("—" if value is None else compact_count(value)
                                          if isinstance(value, int) and not isinstance(value, bool) else str(value))
    if "worker seconds" in rows[0]:
        formats["worker seconds"] = lambda value: "—" if value is None else f"{value:.3g}"
    if "relative error" in rows[0]:
        formats["relative error"] = lambda value: "—" if value is None else f"{100 * value:.4g}%"
    return mo.ui.table(rows, selection=None, show_column_summaries=False,
                       show_data_types=False, pagination=len(rows) > 12,
                       page_size=12, format_mapping=formats)


def panel(mo, title, content, *, expanded=True):
    """An open, styled monitor with Marimo 0.24-compatible containers."""
    if not expanded:
        return mo.accordion({title: content})
    return mo.vstack([mo.md(f"### {title}"), content]).style({
        "border": "1px solid var(--gray-5)", "border-radius": "12px",
        "padding": "1rem", "background": "var(--gray-1)",
    })
