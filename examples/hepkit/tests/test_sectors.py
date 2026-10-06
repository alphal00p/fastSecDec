"""Selected inspection reads retained facts; formatting never invokes science."""
from pathlib import Path
import sys
from types import SimpleNamespace as NS

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase import sectors


class Expression:
    def __init__(self, text):
        self.text = text
        self.calls = []

    def formatted(self, **options):
        self.calls.append(options)
        return self

    def __str__(self):
        return self.text

    def _repr_latex_(self):
        return "$$" + self.text + "$$"


class Display:
    def __init__(self):
        self.rows = []
        self.text = []
        self.downloads = []
        self.ui = NS(table=self.table)

    def table(self, rows, **kwargs):
        self.rows.extend(rows)
        return rows

    def md(self, value):
        self.text.append(value)
        return value

    Html = md

    def vstack(self, values):
        return values

    def accordion(self, values):
        return values

    def download(self, callback, **kwargs):
        self.downloads.append(callback)
        return kwargs


def chart():
    prefactors = [Expression("unselected"), Expression("gamma(eps)")]
    powers = [NS(exponent=Expression("-2+eps"), constant=Expression("-2"),
                 slope=Expression("1"), subtraction_count=2)]
    record = NS(version=1, regulator=Expression("eps"), terms=[
        NS(prefactor=value, powers=powers, regular_expression_bytes=123) for value in prefactors])
    return NS(source_index=4, pre_subtraction=record,
              coordinates=NS(target_parameters=[Expression("t0")]))


def test_selected_term_reads_exact_retained_requirements_without_touching_other_terms():
    display = Display()
    native = chart()
    sectors._pre_subtraction(display, native, 1)
    assert not native.pre_subtraction.terms[0].prefactor.calls
    assert any("Taylor coefficients required" in text and "<td>2</td>" in text for text in display.text)
    assert any("not a count of surviving poles" in text for text in display.text)
    assert "$$gamma(eps)$$" in display.text
    assert display.downloads[0]() == b"gamma(eps)"
    for call in native.pre_subtraction.terms[1].prefactor.calls:
        assert call["max_terms"] == 12
    assert native.pre_subtraction.terms[1].prefactor.calls[0]["show_namespaces"] is False


def test_missing_legacy_record_is_explicit_and_bad_selection_fails():
    display = Display()
    sectors._pre_subtraction(display, NS(pre_subtraction=None), 0)
    assert "not retained" in display.text[0]
    with pytest.raises(ValueError, match="Mapped term index"):
        sectors._pre_subtraction(display, chart(), 2)
    malformed = chart()
    malformed.coordinates.target_parameters = []
    with pytest.raises(ValueError, match="dimension differs"):
        sectors._pre_subtraction(display, malformed, 0)


def test_stats_preserve_native_counts_and_portable_absence_without_session_access():
    class KernelOwner:
        sector_statistics = [NS(version=1, backend="symbolica_interpreter", arithmetic="complex",
            inputs=6, outputs=2, exact_program_bytes=456, symjit_ir_bytes=None,
            operations=NS(additions=1, multiplications=2, inversions=3, function_calls=4))]

        def __getattr__(self, name):
            raise AssertionError(f"Inspection attempted non-statistics access: {name}")

    display = Display()
    owner = KernelOwner()
    sectors._statistics(display, owner, 0)
    assert sectors._operation_total(owner.sector_statistics[0]) == 10
    assert any("SymJIT application bytes" in text and "<td>—</td>" in text and "<td>2</td>" in text for text in display.text)
    assert any("multiplications" in text and "<td>2</td>" in text for text in display.text)
    assert any("before SymJIT" in text for text in display.text)


def test_static_math_cells_preserve_native_markup_and_escape_plain_source():
    import marimo as mo
    html = sectors._static_table(mo, [{"native image": mo.md("$$x^{2}$$"), "source": "<raw&name>"}]).text
    assert "marimo-tex" in html
    assert "&lt;raw&amp;name&gt;" in html
    assert "marimo-table" not in html
