"""Diagnostic export preserves native validity and full precision; no evaluation."""
import json
from pathlib import Path
import sys
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.state import RunState
from showcase.report import report_bytes


def test_no_session_report_keeps_estimate_absent():
    report = json.loads(report_bytes(RunState(phase="ready")))
    assert report["session_created"] is False
    assert report["snapshot"] is None
    assert "estimate" not in report


def test_complex_signed_vector_roundtrips_without_reading_session():
    class NoSessionAccess:
        def __getattribute__(self, name):
            raise AssertionError("The diagnostic export must not call the native session")
    vector = SimpleNamespace(
        orders=[-1, -1, 0, 0], components=["real", "imag", "real", "imag"],
        mean=[0.0, 1.4297050282815504, 0.0, -33.71328010619019],
        standard_error=[0.0, 0.014733128718976767, 0.0, 0.34632108662346217],
        covariance_of_mean=[0.0, 0.0, 0.0, 0.0, 0.0, .000217065081849938, 0.0, -.005100389448479103,
                            0.0, 0.0, 0.0, 0.0, 0.0, -.005100389448479103, 0.0, .11993829504005557],
        production_complete=True,
    )
    snapshot = SimpleNamespace(method="democratic_qmc", stage="production", completed_points=245760,
        planned_points=245760, complete_sectors=30, worker_seconds=1.0, uncertainty="available",
        uncertainty_detail=None, stop_reason="planned_work_complete", stop_detail=None,
        estimate=vector, sectors=[], evaluation_diagnostics=None)
    state = RunState(phase="complete", session=NoSessionAccess(), snapshot=snapshot)
    result = json.loads(report_bytes(state))
    assert result["session_created"] is True
    assert result["snapshot"]["estimate"] == vars(vector)
    snapshot.estimate = None
    snapshot.uncertainty = "waiting_for_coverage"
    result = json.loads(report_bytes(state))
    assert result["snapshot"]["estimate"] is None
    assert result["snapshot"]["uncertainty"] == "waiting_for_coverage"
