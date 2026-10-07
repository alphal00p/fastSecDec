"""Caller lifecycle controls, using test doubles without scientific evaluation."""
from pathlib import Path
from types import SimpleNamespace
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.state import RunState
import showcase.state as state_module
from notebook_cells import generate, science

CONFIG = {"example": "triangle", "mass": 1, "s": -1, "max_order": 0,
          "points": 1024, "shifts": 8, "seed": 7, "package_points": 1024,
          "rule": "kuo_33002"}


def prepared_input(diagram):
    owners = dict(diagram=diagram, kinematics=object(), regulator=object(), dimension=object())
    return SimpleNamespace(**owners, integral_arguments=lambda: dict(owners))


def backend():
    calls = {name: 0 for name in ("generate", "compile", "session", "step")}
    snapshot = SimpleNamespace(estimate=None)
    class Session:
        complete = False
        def snapshot(self): return snapshot
        def checkpoint(self): return b"accepted-checkpoint"
        def step(self, max_packages, evaluation_batch_size):
            assert max_packages == 1 and evaluation_batch_size == 256
            calls["step"] += 1
            return snapshot
    class Kernels:
        runtime_parameters = []
        def with_parameters(self, values, stability):
            assert values == {}
            return self
        def session(self, settings):
            calls["session"] += 1
            return Session()
        def restore(self, checkpoint):
            assert checkpoint == b"accepted-checkpoint"
            return Session()
    class Generated:
        runtime_parameter_defaults = {}
        def compile(self, observer):
            calls["compile"] += 1
            return Kernels()
    class Diagram:
        def sector_decompose(self, *, max_order, observer, **arguments):
            calls["generate"] += 1
            return Generated()
    class Generation:
        complete = False
        failed = None
        generated = kernels = None
        def __init__(self, diagram): self.diagram = diagram
        def step(self, max_units, observer):
            assert max_units == 1
            try:
                if self.generated is None:
                    self.generated = self.diagram.sector_decompose(max_order=0, observer=observer)
                else:
                    self.kernels = self.generated.compile(observer=observer)
                    self.complete = True
            except Exception as error:
                self.failed = str(error)
                raise
    class Integral:
        def __init__(self, **arguments): self.diagram = arguments["diagram"]
        def generation_session(self, **options):
            assert options["compilation_settings"]["backend"] == "eager"
            return Generation(self.diagram)
    fs = SimpleNamespace(QmcSettings=lambda **kw: kw, Integral=Integral,
                         CompilationSettings=lambda **kw: kw, StabilitySettings=lambda **kw: kw)
    diagram = Diagram()
    prepared = prepared_input(diagram)
    return fs, prepared, calls


def test_explicit_generate_stops_before_session_and_sampling():
    fs, prepared, calls = backend()
    state = RunState()
    state.advance(science(fs).advance_session)
    assert all(value == 0 for value in calls.values())
    generate(state, fs, lambda observer: prepared, CONFIG)
    assert state.generated is not None and state.kernels is not None
    assert state.session is None and state.snapshot is None and not state.active
    assert calls == {"generate": 1, "compile": 1, "session": 0, "step": 0}
    state.advance(science(fs).advance_session)
    assert calls["step"] == 0
    state.integrate(science(fs).create_session)
    assert calls["compile"] == 1 and calls["session"] == 1 and calls["step"] == 0
    state.advance(science(fs).advance_session)
    assert calls["step"] == 1


def test_draft_edits_cannot_replace_generated_physics():
    fs, prepared, calls = backend()
    draft = dict(CONFIG)
    state = RunState()
    generate(state, fs, lambda observer: prepared, draft)
    draft["mass"] = 99
    state.integrate(science(fs).create_session, {"mass": 99})
    assert state.configuration["mass"] == 1
    assert calls["session"] == 0
    assert "allocation settings only" in state.error
    assert state.generated is not None and state.kernels is not None


def test_cancel_prevents_timer_steps_and_resume_is_explicit():
    fs, prepared, calls = backend()
    state = RunState()
    generate(state, fs, lambda observer: prepared, CONFIG)
    state.integrate(science(fs).create_session)
    state.advance(science(fs).advance_session)
    state.cancel()
    state.advance(science(fs).advance_session)
    assert calls["step"] == 1 and state.phase == "paused"
    state.resume()
    assert state.active and calls["step"] == 1
    state.advance(science(fs).advance_session)
    assert calls["step"] == 2


def test_original_error_survives_checkpoint_capture_failure():
    class BrokenSession:
        def snapshot(self): raise ValueError("secondary snapshot")
        def checkpoint(self): raise ValueError("secondary checkpoint")
    class NativeFailure(RuntimeError): stage = "evaluation"
    state = RunState(session=BrokenSession(), phase="integrating", checkpoint_bytes=b"older accepted checkpoint")
    state.fail(NativeFailure("original numerical failure"))
    assert state.error == "NativeFailure [evaluation]: original numerical failure"
    assert "secondary snapshot" in state.checkpoint_warning
    assert "secondary checkpoint" in state.checkpoint_warning
    assert state.checkpoint_bytes == b"older accepted checkpoint"
    assert not state.active


def test_new_preparation_failure_does_not_reuse_old_timing(monkeypatch):
    monkeypatch.setattr(state_module, "perf_counter", lambda: 5.0)
    state = RunState(preparation_seconds=99.0)
    def failed(observer): raise ValueError("bad new input")
    generate(state, None, failed, CONFIG)
    assert state.preparation_seconds == 0.0
    assert state.prepared is None and state.generated is None
    assert "preparing" in state.message


def test_compilation_failure_keeps_generated_native_owner():
    fs, prepared, calls = backend()
    class CompilationFailure(ValueError): stage = "compilation"
    generated = SimpleNamespace(compile=lambda **kw: (_ for _ in ()).throw(CompilationFailure("compile stopped")))
    prepared.diagram.sector_decompose = lambda **kw: generated
    state = RunState()
    generate(state, fs, lambda observer: prepared, CONFIG)
    assert state.generated is generated
    assert state.kernels is None and state.session is None
    assert "compilation" in state.error and not state.generation_active


def test_cancel_storage_failure_keeps_session_and_prior_checkpoint():
    class StorageFailure(RuntimeError): stage = "checkpoint"
    snapshot = object()
    class Session:
        def snapshot(self): return snapshot
        def checkpoint(self): raise StorageFailure("cannot save accepted prefix")
    session = Session()
    state = RunState(session=session, kernels=object(), active=True, phase="integrating",
                     snapshot=snapshot, checkpoint_bytes=b"prior accepted checkpoint")
    state.cancel()
    assert state.session is session and state.snapshot is snapshot
    assert state.checkpoint_bytes == b"prior accepted checkpoint"
    assert state.error == "StorageFailure [checkpoint]: cannot save accepted prefix"
    assert state.checkpoint_warning and not state.active
    state.resume()
    assert state.session is session and not state.active
    assert "older checkpoint" in state.message


def test_accuracy_preset_stays_idle_until_integrate_and_binds_all_native_settings():
    from showcase.presentation import QMC_PRESETS
    fs, prepared, calls = backend()
    received = []
    fs.QmcSettings = lambda **settings: received.append(settings) or settings
    state = RunState()
    configuration = {"example": "gghh", "max_order": 0, **QMC_PRESETS["quick"]}
    generate(state, fs, lambda observer: prepared, configuration)
    accuracy = dict(QMC_PRESETS["gghh_accuracy"])
    assert state.configuration["points"] == 1024
    assert state.session is None and calls["step"] == 0
    state.advance(science(fs).advance_session)
    assert calls["step"] == 0
    state.integrate(science(fs).create_session, accuracy)
    assert received == [accuracy]
    assert state.configuration["example"] == "gghh" and state.configuration["max_order"] == 0
    assert calls == {"generate": 1, "compile": 1, "session": 1, "step": 0}


def test_explicit_periodization_is_forwarded_without_replacing_generated_physics():
    fs, prepared, calls = backend()
    received = []
    fs.QmcSettings = lambda **settings: received.append(settings) or settings
    state = RunState()
    generate(state, fs, lambda observer: prepared, CONFIG)
    state.integrate(science(fs).create_session, {"periodization": "none", "rule": "hkkn_alpha3"})
    assert received[0]["periodization"] == "none"
    assert received[0]["rule"] == "hkkn_alpha3"
    assert state.configuration["mass"] == 1
    assert calls["generate"] == calls["compile"] == 1


def test_new_integration_preserves_compiled_owners_and_prior_report_without_sampling():
    import json
    fs, prepared, calls = backend()
    state = RunState()
    generate(state, fs, lambda observer: prepared, CONFIG)
    state.integrate(science(fs).create_session)
    old_session = state.session
    state.new_integration()
    assert state.session is old_session and state.active  # Must cancel first.
    state.cancel()
    state.generated.orders = [0]
    for key, value in dict(content_id="bound-kernel", backend="native_o2", orders=[0],
                           components=["real"], sector_count=1).items():
        setattr(state.kernels, key, value)
    for key, value in dict(method="democratic_qmc", stage="production", completed_points=512,
                           planned_points=8192, complete_sectors=0, worker_seconds=1.0,
                           uncertainty="waiting_for_coverage", uncertainty_detail=None,
                           stop_reason=None, stop_detail=None, sectors=[], evaluation_diagnostics=None).items():
        setattr(state.snapshot, key, value)
    owners = (state.prepared, state.generated, state.kernels)
    state.new_integration()
    assert (state.prepared, state.generated, state.kernels) == owners
    assert state.session is None and state.snapshot is None and not state.active
    previous = state.previous_report_bytes
    assert json.loads(previous)["snapshot"]["completed_points"] == 512
    assert state.previous_snapshot.completed_points == 512
    state.advance(science(fs).advance_session)
    assert calls == {"generate": 1, "compile": 1, "session": 1, "step": 0}
    state.integrate(science(fs).create_session, {"points": 4096})
    assert calls == {"generate": 1, "compile": 1, "session": 2, "step": 0}
    assert state.previous_report_bytes == previous
    assert state.configuration["mass"] == 1 and state.configuration["points"] == 4096
