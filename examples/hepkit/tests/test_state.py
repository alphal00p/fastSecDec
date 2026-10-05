"""Caller lifecycle controls, using test doubles without scientific evaluation."""
from pathlib import Path
from types import SimpleNamespace
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase.state import RunState
import showcase.state as state_module

CONFIG = {"example": "triangle", "mass": 1, "s": -1, "max_order": 0,
          "points": 1024, "shifts": 8, "seed": 7, "package_points": 1024,
          "rule": "kuo_33002"}


def backend():
    calls = {name: 0 for name in ("generate", "compile", "session", "step")}
    snapshot = SimpleNamespace(estimate=None)
    class Session:
        complete = False
        def snapshot(self): return snapshot
        def checkpoint(self): return b"accepted-checkpoint"
        def step(self, max_packages):
            calls["step"] += 1
            return snapshot
    class Kernels:
        def session(self, settings):
            calls["session"] += 1
            return Session()
        def restore(self, checkpoint):
            assert checkpoint == b"accepted-checkpoint"
            return Session()
    class Generated:
        def compile(self, observer):
            calls["compile"] += 1
            return Kernels()
    class Integral:
        def __init__(self, **arguments): pass
        def generate(self, max_order, observer, **arguments):
            calls["generate"] += 1
            return Generated()
    fs = SimpleNamespace(Integral=Integral, QmcSettings=lambda **kw: kw)
    prepared = SimpleNamespace(integral_arguments=lambda: {})
    return fs, prepared, calls


def test_explicit_generate_stops_before_session_and_sampling():
    fs, prepared, calls = backend()
    state = RunState()
    state.advance()
    assert all(value == 0 for value in calls.values())
    state.generate(fs, lambda observer: prepared, CONFIG)
    assert state.generated is not None and state.kernels is not None
    assert state.session is None and state.snapshot is None and not state.active
    assert calls == {"generate": 1, "compile": 1, "session": 0, "step": 0}
    state.advance()
    assert calls["step"] == 0
    state.integrate(fs)
    assert calls["compile"] == 1 and calls["session"] == 1 and calls["step"] == 0
    state.advance()
    assert calls["step"] == 1


def test_draft_edits_cannot_replace_generated_physics():
    fs, prepared, calls = backend()
    draft = dict(CONFIG)
    state = RunState()
    state.generate(fs, lambda observer: prepared, draft)
    draft["mass"] = 99
    state.integrate(fs, {"mass": 99})
    assert state.configuration["mass"] == 1
    assert calls["session"] == 0
    assert "allocation settings only" in state.error
    assert state.generated is not None and state.kernels is not None


def test_cancel_prevents_timer_steps_and_resume_is_explicit():
    fs, prepared, calls = backend()
    state = RunState()
    state.generate(fs, lambda observer: prepared, CONFIG)
    state.integrate(fs)
    state.advance()
    state.cancel()
    state.advance()
    assert calls["step"] == 1 and state.phase == "paused"
    state.resume()
    assert state.active and calls["step"] == 1
    state.advance()
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
    state.generate(None, failed, CONFIG)
    assert state.preparation_seconds == 0.0
    assert state.prepared is None and state.generated is None
    assert "preparing" in state.message


def test_compilation_failure_keeps_generated_native_owner():
    generated = SimpleNamespace(compile=lambda **kw: (_ for _ in ()).throw(ValueError("compile stopped")))
    fs = SimpleNamespace(Integral=lambda **kw: SimpleNamespace(generate=lambda *args, **kw: generated))
    prepared = SimpleNamespace(integral_arguments=lambda: {})
    state = RunState()
    state.generate(fs, lambda observer: prepared, CONFIG)
    assert state.generated is generated
    assert state.kernels is None and state.session is None
    assert "compiling" in state.message


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
