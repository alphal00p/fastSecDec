"""Installed native generation with the shared HEPKit presenter and callbacks."""
import inspect
from pathlib import Path
import sys
from types import SimpleNamespace

import pytest
from symbolica import E
from symbolica.community import hepkit as hep
from symbolica.community.hepkit import sector_decomposition as sd

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs


@pytest.fixture(scope="module")
def integral():
    return sd.Integral(**inputs.massive_triangle().integral_arguments())


class Presenter:
    """Only the optional display is simulated; all scientific owners are native."""
    def __init__(self, *, notebook=True, update_error=None, exit_error=None):
        self.notebook = notebook
        self.update_error = update_error
        self.exit_error = exit_error
        self.events = []
        self.module = SimpleNamespace(
            running_in_notebook=self.detect,
            status=SimpleNamespace(spinner=lambda *a, **k: self.open("spinner", *a, **k),
                                   progress_bar=lambda *a, **k: self.open("bar", *a, **k)),
            output=SimpleNamespace(_output=SimpleNamespace(flush=self.flush)),
        )

    def detect(self):
        self.events.append(("detect", self.notebook))
        return self.notebook

    def open(self, kind, *args, **kwargs):
        owner = self
        self.events.append(("open", kind, args, kwargs))

        class Context:
            def __enter__(self):
                owner.events.append(("enter", kind))
                return self

            def update(self, *args, **kwargs):
                owner.events.append(("update", kind, args, kwargs))
                if owner.update_error is not None:
                    raise owner.update_error

            def __exit__(self, typ, value, traceback):
                owner.events.append(("exit", kind, typ, value))
                if owner.exit_error is not None:
                    raise owner.exit_error

        return Context()

    def flush(self):
        self.events.append(("flush",))


def test_public_progress_defaults_are_discoverable(integral):
    generated = integral.generate(progress=None)
    value = inputs.massive_triangle()
    for call in (sd.sector_decompose, integral.generate, generated.compile,
                 value.diagram.sector_decompose,
                 value.diagram.propagator_family(kinematics=value.kinematics).sector_decompose):
        assert inspect.signature(call).parameters["progress"].default == "auto"


def test_none_and_legacy_observer_do_not_create_duplicate_display(integral, monkeypatch):
    presenter = Presenter()
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    silent = integral.generate(progress=None)
    silent_kernels = silent.compile(progress=None)
    seen = []
    observed = integral.generate(observer=seen.append)
    observed_kernels = observed.compile(observer=seen.append)
    assert presenter.events == []  # Even automatic detection is suppressed.
    assert seen[0].stage == "parametrization" and seen[-1].stage == "complete"
    assert silent_kernels.to_bytes() == observed_kernels.to_bytes()


def test_auto_is_quiet_outside_a_running_notebook(integral, monkeypatch):
    presenter = Presenter(notebook=False)
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    integral.generate().compile()
    assert presenter.events and all(event == ("detect", False) for event in presenter.events)
    monkeypatch.delitem(sys.modules, "marimo")
    integral.generate().compile()
    assert "marimo" not in sys.modules  # Optional dependency is never imported.


def test_auto_presenter_finishes_generation_and_compilation_without_sampling(integral, monkeypatch):
    presenter = Presenter()
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    generated = integral.generate()
    assert generated.snapshot().kernels == 0
    kernels = generated.compile()
    assert kernels.snapshot().stage == "complete"
    assert kernels.snapshot().kernels == generated.sector_count
    opens = [event for event in presenter.events if event[0] == "open"]
    assert {event[1] for event in opens} == {"spinner", "bar"}
    assert all(event[3].get("total", 1) > 0 for event in opens)
    assert sum(event[0] == "enter" for event in presenter.events) == sum(
        event[0] == "exit" for event in presenter.events)
    assert any(event[0] == "flush" for event in presenter.events)
    assert all(event[2:] == (None, None) for event in presenter.events if event[0] == "exit")
    session = kernels.session(sd.QmcSettings(points=1024, shifts=2))
    assert session.snapshot().completed_points == 0


def test_both_callbacks_receive_full_native_events_in_order(integral, monkeypatch):
    presenter = Presenter()
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    seen = []

    def collect(label):
        def callback(event):
            assert isinstance(event, sd.GenerationSnapshot)
            seen.append((label, event.stage, event.completed, event.total,
                         event.sectors, event.kernels, event.elapsed_seconds))
            return None if label == "observer" else True
        return callback

    generated = integral.generate(observer=collect("observer"), progress=collect("progress"))
    generated.compile(observer=collect("observer"), progress=collect("progress"))
    assert len(seen) > 4 and len(seen) % 2 == 0
    for observer, progress in zip(seen[::2], seen[1::2]):
        assert observer[0] == "observer" and progress[0] == "progress"
        assert observer[1:] == progress[1:]
    assert seen[0][1] == "parametrization" and seen[-1][1] == "complete"
    assert presenter.events == []


@pytest.mark.parametrize("owner", ["free_diagram", "diagram", "family", "legacy"])
def test_progress_false_cancels_before_native_parametrization(owner, integral):
    value = inputs.massive_triangle()
    kwargs = value.integral_arguments()
    kwargs.pop("diagram")
    family_kwargs = kwargs.copy()
    family_kwargs.pop("powers")
    family = value.diagram.propagator_family(kinematics=value.kinematics)
    calls = {
        "free_diagram": lambda callback: sd.sector_decompose(value.diagram, progress=callback, **kwargs),
        "diagram": lambda callback: value.diagram.sector_decompose(progress=callback, **kwargs),
        "family": lambda callback: family.sector_decompose(powers=[1, 1, 1], numerator=E("1"), progress=callback, **family_kwargs),
        "legacy": lambda callback: integral.generate(progress=callback),
    }
    seen = []

    def cancel(event):
        seen.append(event)
        return False

    with pytest.raises(sd.CancelledError) as caught:
        calls[owner](cancel)
    assert caught.value.stage == "generation"
    assert len(seen) == 1 and seen[0].stage == "parametrization"
    assert seen[0].sectors == seen[0].kernels == 0


def test_compilation_cancel_does_not_mutate_generated_owner(integral):
    generated = integral.generate(progress=None)
    seen = []

    def cancel(event):
        seen.append(event)
        return False

    with pytest.raises(sd.CancelledError) as caught:
        generated.compile(progress=cancel)
    assert caught.value.stage == "compilation"
    assert seen and seen[0].stage == "compilation"
    assert generated.snapshot().kernels == 0
    assert generated.compile(progress=None).sector_count == generated.sector_count


def test_observer_false_short_circuits_progress_at_same_event(integral):
    progress_seen = []
    with pytest.raises(sd.CancelledError):
        integral.generate(observer=lambda _: False, progress=progress_seen.append)
    assert progress_seen == []


def test_native_failure_closes_auto_display_without_replacing_error(monkeypatch):
    presenter = Presenter(exit_error=ValueError("secondary cleanup error"))
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    value = inputs.massive_triangle()
    family = value.diagram.propagator_family(kinematics=value.kinematics)
    with pytest.raises(sd.FastSecDecError) as caught:
        family.sector_decompose(regulator=value.regulator, powers=[0, 0, 0], numerator=E("1"))
    assert caught.value.stage == "input"
    exits = [event for event in presenter.events if event[0] == "exit"]
    assert len(exits) == 1 and exits[0][3] is caught.value


@pytest.mark.parametrize("compile_first", [False, True])
def test_callback_exception_identity_and_order_are_preserved(integral, compile_first):
    call = integral.generate(progress=None).compile if compile_first else integral.generate
    error = LookupError("callback original object")
    observed = []

    def fail(event):
        raise error

    with pytest.raises(LookupError) as caught:
        call(observer=observed.append, progress=fail)
    assert caught.value is error and len(observed) == 1
    progress_seen = []
    with pytest.raises(LookupError) as caught:
        call(observer=fail, progress=progress_seen.append)
    assert caught.value is error and progress_seen == []


@pytest.mark.parametrize("error", [RuntimeError("display update failed"), KeyboardInterrupt("stop")])
def test_presenter_cleanup_preserves_original_error(integral, monkeypatch, error):
    presenter = Presenter(update_error=error, exit_error=ValueError("secondary cleanup error"))
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    with pytest.raises(type(error)) as caught:
        integral.generate()
    assert caught.value is error
    exits = [event for event in presenter.events if event[0] == "exit"]
    assert exits and exits[-1][2:] == (type(error), error)


def test_invalid_progress_is_rejected_without_native_events_or_display(integral, monkeypatch):
    presenter = Presenter()
    monkeypatch.setitem(sys.modules, "marimo", presenter.module)
    generated = integral.generate(progress=None)
    for call in (integral.generate, generated.compile):
        for invalid in (True, "enabled", 3, object()):
            seen = []
            with pytest.raises(TypeError, match="progress must"):
                call(progress=invalid, observer=seen.append)
            assert seen == []
    assert presenter.events == []
