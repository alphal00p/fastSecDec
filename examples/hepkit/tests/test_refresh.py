"""The real caller dispatch yields between native units and never starts implicitly."""
from types import SimpleNamespace as NS
import sys

from test_state import backend
from notebook_cells import science, cell_defining
from showcase.notebook import Study


class Notebook:
    def __init__(self, monkeypatch):
        self.study = Study("examples")
        # Isolate individual transitions in the lifecycle controls below.
        self.study.work_unit_limit = 1
        fs, self.prepared, self.calls = backend()
        import showcase
        recipe = science(fs)
        monkeypatch.setitem(sys.modules, "showcase.science", recipe)
        monkeypatch.setattr(showcase, "science", recipe, raising=False)
        monkeypatch.setitem(sys.modules, "showcase.inputs", NS(prepare=lambda _: self.prepared))
        self.actions = {key: 0 for key in ("generate", "inspect", "qmc", "havana", "pause", "resume")}
        self.tick = None
        self.settings = {"points": 1024, "replicas": 4, "seed": 7, "sector": 0}
        self.mo = NS(output=NS(replace=lambda value: None))
        self.study.monitor = lambda mo: None

    @property
    def run(self): return self.study.run

    def dispatch(self, action=None):
        if action == "tick": self.tick = (self.tick or 0) + 1
        elif action is not None: self.actions[action] += 1
        return self.study.dispatch(self.actions, self.tick, "triangle", self.settings, self.mo)


def test_explicit_actions_one_native_unit_per_tick_and_stale_events(monkeypatch):
    notebook = Notebook(monkeypatch)
    notebook.dispatch()
    assert not notebook.run.work_active and all(n == 0 for n in notebook.calls.values())
    notebook.dispatch("generate")
    owner = notebook.run.generation_session
    assert notebook.run.generation_active and notebook.calls["generate"] == 1
    assert notebook.calls["compile"] == notebook.calls["step"] == 0
    notebook.dispatch("pause")
    notebook.dispatch("tick")
    assert notebook.run.phase == "generation_paused" and notebook.calls["compile"] == 0
    notebook.dispatch("resume")
    assert notebook.run.generation_session is owner and notebook.calls["compile"] == 0
    notebook.dispatch("tick")
    assert notebook.run.phase == "ready" and notebook.calls["compile"] == 1
    notebook.dispatch("tick")
    assert notebook.calls["step"] == notebook.calls["session"] == 0
    notebook.dispatch("qmc")
    assert notebook.run.active and notebook.calls["step"] == 0
    notebook.dispatch("tick")
    notebook.dispatch("pause")
    notebook.dispatch("tick")
    assert notebook.calls["step"] == 1 and not notebook.run.active
    notebook.dispatch("resume")
    notebook.dispatch("tick")
    assert notebook.calls["step"] == 2


def test_refresh_only_exists_while_work_is_armed():
    clock, _ = cell_defining("refresh")
    created = []
    mo = NS(ui=NS(refresh=lambda **kw: created.append(kw) or kw))
    assert clock(lambda: False, mo) == (None,)
    assert created == []
    assert clock(lambda: True, mo)[0]["default_interval"] == "125ms"
    # Marimo's browser validator enforces a 100 ms minimum even though the
    # Python UI constructor accepts smaller intervals.
    assert created[0]["options"] == ["125ms", "250ms", "1s"]
    assert len(created) == 1


def test_keyboard_interrupt_resumes_same_retained_generation_owner(monkeypatch):
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    owner = notebook.run.generation_session
    original = owner.step
    owner.step = lambda **kw: (_ for _ in ()).throw(KeyboardInterrupt())
    notebook.dispatch("tick")
    assert notebook.run.phase == "generation_paused" and not notebook.run.work_active
    owner.step = original
    notebook.dispatch("resume")
    notebook.dispatch("tick")
    assert notebook.run.phase == "ready" and notebook.run.generation_session is owner


def test_new_graph_generation_clears_previous_allocation_provenance(monkeypatch):
    notebook = Notebook(monkeypatch)
    notebook.run.previous_configuration = {"diagram_id": "different"}
    notebook.run.previous_snapshot = object()
    notebook.dispatch("generate")
    assert notebook.run.previous_snapshot is None and notebook.run.previous_configuration is None


def test_integrate_before_generate_and_invalid_qmc_allocation_stay_idle(monkeypatch):
    notebook = Notebook(monkeypatch)
    notebook.dispatch("qmc")
    assert notebook.run.error is None and "Generate" in notebook.run.message
    notebook.dispatch("generate")
    notebook.dispatch("tick")
    notebook.settings["points"] = 64
    notebook.dispatch("qmc")
    assert notebook.run.error is None and notebook.run.session is None
    assert "1024" in notebook.run.message


def test_caller_observer_exception_retains_resumable_generation(monkeypatch):
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    owner = notebook.run.generation_session
    original = owner.step
    owner.step = lambda **kw: (_ for _ in ()).throw(ValueError("display callback"))
    notebook.dispatch("tick")
    assert notebook.run.phase == "generation_paused" and owner.failed is None
    owner.step = original
    notebook.dispatch("resume")
    assert notebook.run.error is None
    notebook.dispatch("tick")
    assert notebook.run.phase == "ready"


def test_budget_drains_fast_units_but_yields_before_scheduling_past_deadline(monkeypatch):
    import showcase.notebook as module
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    notebook.dispatch("tick")
    notebook.dispatch("qmc")
    now = [0.0]
    monkeypatch.setattr(module, "perf_counter", lambda: now[0])
    step = notebook.run.session.step
    def timed_step(**options):
        result = step(**options)
        now[0] += 0.021
        return result
    notebook.run.session.step = timed_step
    notebook.study.work_unit_limit = 128
    notebook.study.last_publication = 0
    assert notebook.dispatch("tick") is None
    assert notebook.calls["step"] == 3
    assert now[0] == 0.063  # Last atomic unit may cross the 50 ms boundary.
    notebook.dispatch("pause")
    notebook.dispatch("tick")
    assert notebook.calls["step"] == 3


def test_observation_reduction_occurs_only_when_due_or_at_explicit_boundary(monkeypatch):
    import showcase.notebook as module
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    notebook.dispatch("tick")
    notebook.dispatch("qmc")
    counts = []
    notebook.run.capture_observation = lambda: counts.append(notebook.calls["step"])
    now = [0.0]
    monkeypatch.setattr(module, "perf_counter", lambda: now[0])
    notebook.study.last_publication = 0
    notebook.dispatch("tick")
    assert counts == []
    now[0] = 1.0
    assert notebook.dispatch("tick") is not None
    assert counts == [2]
    notebook.dispatch("pause")
    assert counts == [2, 2]


def test_failed_previous_report_does_not_rebind_or_replace_retained_session(monkeypatch):
    from showcase import report
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    notebook.dispatch("tick")
    notebook.dispatch("qmc")
    notebook.dispatch("pause")
    old_session, old_kernels = notebook.run.session, notebook.run.kernels
    def failed_report(state):
        raise RuntimeError("previous report failed")
    monkeypatch.setattr(report, "report_bytes", failed_report)
    def forbidden_rebind(*args, **kwargs):
        raise AssertionError("Point must remain attached to the previous session")
    old_kernels.with_parameters = forbidden_rebind
    notebook.dispatch("havana")
    assert "previous report failed" in notebook.run.error
    assert notebook.run.session is old_session and notebook.run.kernels is old_kernels
    assert notebook.calls["session"] == 1


def test_inspect_dispatch_records_request_without_creating_widget_in_volatile_cell(monkeypatch):
    from showcase import sectors
    notebook = Notebook(monkeypatch)
    notebook.dispatch("generate")
    notebook.dispatch("tick")
    notebook.study.take_view_updates()
    def forbidden(*args, **kwargs):
        raise AssertionError("Native widget must be created by its inspection cell")
    monkeypatch.setattr(sectors, "expression_viewer", forbidden)
    monkeypatch.setattr(sectors, "inspection", forbidden)
    notebook.dispatch("inspect")
    assert notebook.study.inspection_request == (0, 0)
    assert notebook.study.expression_viewer is None
    assert set(notebook.study.take_view_updates()) == {"inspection"}
    notebook.dispatch("qmc")
    assert set(notebook.study.take_view_updates()) == {"artifact"}
    notebook.dispatch("tick")
    assert notebook.study.take_view_updates() == {}


def test_download_preparation_uses_explicit_caller_and_returns_only_bytes(monkeypatch):
    from threading import get_ident
    from showcase import report
    study = Study()
    calls = []
    study.run.kernels = NS(to_bytes=lambda: calls.append(get_ident()) or b"native-artifact")
    monkeypatch.setattr(report, "report_bytes", lambda _: b"native-report")
    study.prepare_downloads()
    assert calls == [get_ident()]
    assert study.prepared_downloads == {"artifact": b"native-artifact", "report": b"native-report"}
    study.run.active = True
    study.prepare_downloads()
    assert calls == [get_ident()] and "Pause" in study.run.message
