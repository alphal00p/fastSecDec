"""A stopped generation must never look like a successful zero integral."""
from pathlib import Path
from types import SimpleNamespace
import sys

import marimo as mo

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.generation import generation_view, timing_rows
from showcase.notebook import Study


def test_formula_preparation_preserves_known_zero_and_absent_historical_timings():
    timings = SimpleNamespace(input_seconds=0.1, total_seconds=0.2)
    event = SimpleNamespace(timings=timings)
    assert timing_rows(event) == [{"native phase": "input", "seconds": 0.1},
                                  {"native phase": "total", "seconds": 0.2}]
    timings.formula_preparation_seconds = 0.0
    assert {"native phase": "formula_preparation", "seconds": 0.0} in timing_rows(event)
    event = SimpleNamespace(stage="formula_preparation", elapsed_seconds=0.2,
        completed=1, total=2, sectors=3, kernels=0, detail="Precomputing subtraction formulas",
        timings=timings, coefficient_expansion=None,
        formula_preparation=SimpleNamespace(completed=1, total=2, sectors=3, reused=1))
    study = Study()
    study.run.events = [event]
    rendered = generation_view(mo, study.run).text
    assert "Subtraction formulas" in rendered and "unique formulas" in rendered
    assert "eligible sector uses" in rendered and "shared uses" in rendered


def test_failed_preparation_explains_that_zero_counters_are_not_a_result():
    study = Study()
    study.run.phase = "failed"
    study.run.error = "FastSecDecError [parametrization]: non-polynomial numerator"
    study.run.message = "Stopped during generating; no numerical value replaces this failure."
    assert "no numerical value replaces" in study.monitor(mo).text
    assert "not a zero integral" in generation_view(mo, study.run).text


def test_failed_generation_retains_progress_but_labels_it_incomplete():
    phases = ("input", "parametrization", "domain", "geometry", "mapping", "symmetry",
              "subtraction", "laurent", "coefficient_expansion", "compilation", "total")
    event = SimpleNamespace(stage="parametrization", elapsed_seconds=0.2,
        completed=1, total=None, sectors=0, kernels=0, detail="Parametrizing input",
        timings=SimpleNamespace(**{name + "_seconds": 0.0 for name in phases}),
        coefficient_expansion=None)
    study = Study()
    study.run.events = [event]
    study.run.phase = "failed"
    rendered = generation_view(mo, study.run)
    assert "partial progress counters" in rendered.text
    assert "not a completed decomposition or a zero integral" in rendered.text
    # Native generation exposes the decomposition before compiling kernels.
    # Its presence must not hide a subsequent compilation failure.
    study.run.generated = object()
    study.run.generation_session = SimpleNamespace(complete=False)
    assert "partial progress counters" in generation_view(mo, study.run).text


def test_later_integration_failure_keeps_completed_generation_valid():
    phases = ("input", "parametrization", "domain", "geometry", "mapping", "symmetry",
              "subtraction", "laurent", "coefficient_expansion", "compilation", "total")
    event = SimpleNamespace(stage="complete", elapsed_seconds=1.0,
        completed=4, total=4, sectors=4, kernels=4, detail="Eager evaluators ready",
        timings=SimpleNamespace(**{name + "_seconds": 0.0 for name in phases}),
        coefficient_expansion=None)
    study = Study()
    study.run.generated = object()
    study.run.generation_session = SimpleNamespace(complete=True)
    study.run.kernels = object()
    study.run.events = [event]
    study.run.phase = "integrating"
    study.run.fail(ValueError("Integration failed at the chosen point"))
    assert study.run.phase == "failed"
    rendered = generation_view(mo, study.run).text
    assert "Eager evaluators ready" in rendered
    assert "Generation failed" not in rendered
    assert "partial progress counters" not in rendered
    # A retained exact generation may finish without observed progress events.
    study.run.events.clear()
    rendered = generation_view(mo, study.run).text
    assert "completed generation is retained" in rendered
    assert "Generation failed" not in rendered


def test_generation_progress_rendering_never_reenters_borrowed_session():
    phases = ("input", "parametrization", "domain", "geometry", "mapping", "symmetry",
              "subtraction", "laurent", "coefficient_expansion", "compilation", "total")
    event = SimpleNamespace(stage="complete", elapsed_seconds=1.0,
        completed=4, total=4, sectors=4, kernels=4, detail="Eager evaluators ready",
        timings=SimpleNamespace(**{name + "_seconds": 0.0 for name in phases}),
        coefficient_expansion=None)

    class BorrowCheckedSession:
        borrowed = False
        done = False
        failed = None

        def __init__(self):
            self.owners = {"generated": object(), "kernels": object()}

        def owner(self, name):
            assert not self.borrowed, "Progress presentation re-entered the native session"
            return self.owners[name] if self.done else None

        @property
        def complete(self):
            return self.owner("kernels") is not None

        @property
        def generated(self):
            return self.owner("generated")

        @property
        def kernels(self):
            return self.owner("kernels")

        def step(self, *, max_units, observer):
            assert max_units == 1
            self.borrowed = True
            try:
                observer(event)
            finally:
                self.borrowed = False
            self.done = True

    study = Study()
    study.run.generation_session = BorrowCheckedSession()
    study.run.generation_active = True
    study.run.phase = "generating"
    rendered = []
    study.run.generation_display = lambda: rendered.append(study.monitor(mo).text)
    study.run.advance_generation()
    assert rendered and "Eager evaluators ready" in rendered[0]
    assert study.run.error is None
    assert study.run.phase == "ready" and study.run.kernels is not None
