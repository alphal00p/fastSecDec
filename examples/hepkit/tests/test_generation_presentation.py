"""A stopped generation must never look like a successful zero integral."""
from pathlib import Path
from types import SimpleNamespace
import sys

import marimo as mo

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.generation import generation_view
from showcase.notebook import Study


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
