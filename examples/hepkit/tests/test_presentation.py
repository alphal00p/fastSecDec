"""Human-only notebook rounding leaves native data available at full precision."""
from decimal import getcontext
import math
from pathlib import Path
import sys
from types import SimpleNamespace

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.integration import estimate_table, vector_rows
from showcase.presentation import compact_count, scientific, table, uncertainty


@pytest.mark.parametrize("mean,error,expected", [
    (0.1234, 0.0056, "+1.234(56) ·10⁻¹"),
    (0.123, 12434.43, "+1(120000) ·10⁻¹"),
    (1.234, 5.6, "+1.2(56) ·10⁰"),
    (-33.71328010619019, 0.34632108662346217, "−3.371(35) ·10¹"),
    (9.99999, 0.001, "+1.00000(10) ·10¹"),
    (0.0, 0.0, "+0(0) ·10⁰"),
    (-0.0, 0.0, "+0(0) ·10⁰"),
    (0.0, 0.0056, "+0.0(56) ·10⁻³"),
    (1e-280, 2.34e-283, "+1.0000(23) ·10⁻²⁸⁰"),
    (1.25, 0.0, "+1.25(0) ·10⁰"),
])
def test_normalized_last_digit_uncertainty(mean, error, expected):
    assert uncertainty(mean, error) == expected


@pytest.mark.parametrize("error", [None, -1.0, math.nan, math.inf])
def test_missing_or_invalid_uncertainty_never_becomes_zero(error):
    assert uncertainty(-1.0, error) == "−1 ·10⁰ (σ unavailable)"


def test_extreme_finite_numbers_and_decimal_context_are_preserved():
    context = getcontext()
    original_precision = context.prec
    try:
        context.prec = 3
        assert uncertainty(0.1234, 0.0056) == "+1.234(56) ·10⁻¹"
        for mean, error in [(5e-324, 1e308), (1e308, 5e-324)]:
            text = uncertainty(mean, error)
            assert len(text) < 700
            assert text.count("(") == text.count(")") == 1
            assert "e+" not in text and "e-" not in text
            assert "unavailable" not in text
        assert context.prec == 3
    finally:
        context.prec = original_precision
    assert scientific(-0.0) == "+0 ·10⁰"
    assert scientific(5e-324).endswith(" ·10⁻³²⁴")
    assert uncertainty(math.nan, 1.0) == "unavailable"


@pytest.mark.parametrize("count,expected", [
    (0, "0"), (999, "999"), (1000, "1.000 K"),
    (9999, "9.999 K"), (10000, "10.00 K"),
    (999949, "999.9 K"), (999950, "1.000 M"),
    (999950000, "1.000 B"), (2**64 - 1, "18450000000 B"),
])
def test_compact_counts_match_cli_boundaries(count, expected):
    assert compact_count(count) == expected


def test_display_adapters_leave_raw_vectors_and_counts_unchanged():
    estimate = SimpleNamespace(orders=[-1, 0], components=["real", "imag"],
        mean=[0.1234, -33.71328010619019],
        standard_error=[0.0056, 0.34632108662346217])
    native_rows = vector_rows(estimate)
    mo = SimpleNamespace(ui=SimpleNamespace(table=lambda rows, **kwargs: (rows, kwargs)))
    rows, _ = estimate_table(mo, estimate)
    assert rows[0]["estimate ± 1σ"] == "+1.234(56) ·10⁻¹"
    assert rows[1]["estimate ± 1σ"] == "−3.371(35) ·10¹"
    assert vector_rows(estimate) == native_rows
    counts = [{"sector": 1000, "accepted points": 1048576, "planned points": "as allocated"}]
    displayed_rows, options = table(mo, counts)
    assert displayed_rows is counts
    assert counts[0]["accepted points"] == 1048576
    assert options["format_mapping"]["accepted points"](1048576) == "1.049 M"
    assert options["format_mapping"]["planned points"]("as allocated") == "as allocated"
    assert "sector" not in options["format_mapping"]
