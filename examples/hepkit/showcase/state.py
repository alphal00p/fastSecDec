"""Explicit caller-owned actions; no work runs merely by creating this state."""

from __future__ import annotations
from dataclasses import dataclass, field
from time import perf_counter
from .integration import highest_order_rows


@dataclass
class RunState:
    prepared: object = None
    generated: object = None
    kernels: object = None
    session: object = None
    snapshot: object = None
    configuration: dict | None = None
    events: list = field(default_factory=list)
    input_events: list = field(default_factory=list)
    history: list = field(default_factory=list)
    phase: str = "draft"
    active: bool = False
    active_started: float | None = None
    active_seconds: float = 0.0
    preparation_seconds: float = 0.0
    message: str = "Submit inputs, then select Generate. Integration starts separately."
    error: str | None = None
    checkpoint_warning: str | None = None
    checkpoint_bytes: bytes | None = None
    inspected_sector: object = None
    numerator_view: object = None
    drawing: object = None
    seen: dict = field(default_factory=lambda: {"generate": 0, "integrate": 0, "cancel": 0, "resume": 0, "inspect": 0, "numerator": 0, "tick": ""})

    @property
    def integration_wall_seconds(self):
        return self.active_seconds + (perf_counter() - self.active_started if self.active_started is not None else 0.0)

    def stop_clock(self):
        self.active_seconds = self.integration_wall_seconds
        self.active_started = None

    def observe_generation(self, event, display=None):
        self.events.append(event)
        if display is not None:
            display(event)
        return True

    def generate(self, fs, prepare, configuration, display=None, input_display=None):
        """Explicitly prepare, generate and compile; never create a session."""
        if self.active:
            self.message = "Cancel integration before generating another input."
            return
        self.prepared = self.generated = self.kernels = self.session = self.snapshot = None
        self.configuration = dict(configuration)
        self.events.clear()
        self.input_events.clear()
        self.history.clear()
        self.inspected_sector = self.numerator_view = self.drawing = None
        self.preparation_seconds = 0.0
        self.checkpoint_bytes = None
        self.error = None
        self.checkpoint_warning = None
        self.active_seconds = 0.0
        self.active_started = None
        self.phase = "preparing"
        self.message = "Preparing the submitted native input…"
        started = perf_counter()
        try:
            def input_observer(event):
                self.input_events.append(event)
                if input_display is not None:
                    input_display(event)
                return True
            self.prepared = prepare(input_observer)
            self.preparation_seconds = perf_counter() - started
            self.phase = "generating"
            self.message = "Generating native sectors…"
            integral = fs.Integral(**self.prepared.integral_arguments())
            observer = lambda event: self.observe_generation(event, display)
            arguments = getattr(self.prepared, "generation_arguments", lambda: {})()
            self.generated = integral.generate(configuration["max_order"], observer=observer, **arguments)
            self.phase = "compiling"
            self.kernels = self.generated.compile(observer=observer)
            self.phase = "ready"
            self.message = "Generation and compilation complete. No points sampled. Inspect the sectors, then select Integrate."
        except (Exception, KeyboardInterrupt) as error:
            self.preparation_seconds = self.preparation_seconds or perf_counter() - started
            self.fail(error)

    def integrate(self, fs, settings=None):
        """Start a fresh native session only from already prepared kernels."""
        if self.kernels is None:
            self.message = "Generate and compile the input before integration."
            return
        if self.session is not None:
            self.message = "This allocation already exists. Resume its saved checkpoint, or Generate to prepare a new run."
            return
        try:
            configuration = dict(self.configuration)
            if settings is not None:
                allowed = {"points", "shifts", "seed", "package_points", "rule"}
                if set(settings) - allowed:
                    raise ValueError("Integrate can change allocation settings only; Generate binds the physics input")
                configuration.update(settings)
            native_settings = fs.QmcSettings(
                points=configuration["points"], shifts=configuration["shifts"],
                seed=configuration["seed"], package_points=configuration["package_points"],
                rule=configuration["rule"], periodization="korobov3",
            )
            self.session = self.kernels.session(native_settings)
            self.snapshot = self.session.snapshot()
            self.configuration = configuration
            self.active = not self.session.complete
            self.active_started = perf_counter() if self.active else None
            self.phase = "integrating" if self.active else "complete"
            self.error = None
            self.message = "Integrating — one native package per refresh." if self.active else "The native allocation is exact and already complete."
            if not self.active:
                self.checkpoint_bytes = self.session.checkpoint()
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def fail(self, error):
        failed_phase = self.phase
        self.stop_clock()
        self.active = False
        stage = getattr(error, "stage", failed_phase)
        if isinstance(error, KeyboardInterrupt):
            self.error = None
            self.phase = "interrupted"
            self.message = f"Caller interruption during {failed_phase}. Completed owners and accepted coverage are retained."
        else:
            self.error = f"{type(error).__name__} [{stage}]: {error}"
            self.phase = "failed"
            self.message = f"Stopped during {failed_phase}; no numerical value replaces this failure."
        if self.session is not None:
            warnings = []
            try:
                self.snapshot = self.session.snapshot()
            except (Exception, KeyboardInterrupt) as secondary:
                warnings.append(f"Snapshot capture failed: {type(secondary).__name__}: {secondary}")
            try:
                self.checkpoint_bytes = self.session.checkpoint()
            except (Exception, KeyboardInterrupt) as secondary:
                warnings.append(f"Checkpoint capture failed: {type(secondary).__name__}: {secondary}")
            self.checkpoint_warning = " ".join(warnings) or None

    def cancel(self):
        if self.session is None or not self.active:
            return
        self.stop_clock()
        self.active = False
        try:
            self.checkpoint_bytes = self.session.checkpoint()
            self.snapshot = self.session.snapshot()
            self.phase = "paused"
            self.message = "Cancelled by the caller between packages. Accepted coverage is saved."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def resume(self):
        if self.active or self.kernels is None or self.checkpoint_bytes is None:
            return
        if self.checkpoint_warning:
            self.message = "Checkpoint capture failed. The native session is retained; an older checkpoint will not be resumed silently."
            return
        try:
            self.session = self.kernels.restore(self.checkpoint_bytes)
            self.snapshot = self.session.snapshot()
            self.active = not self.session.complete
            self.active_started = perf_counter() if self.active else None
            self.error = None
            self.phase = "integrating" if self.active else "complete"
            self.message = "Resumed from the native checkpoint." if self.active else "The saved allocation is already complete."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def advance(self):
        """Only an explicit Integrate or Resume action can arm these steps."""
        if not self.active:
            return
        try:
            self.snapshot = self.session.step(max_packages=1)
            if self.snapshot.estimate is not None:
                for row in highest_order_rows(self.snapshot.estimate):
                    self.history.append({"accepted points": self.snapshot.completed_points, **row})
            if self.session.complete:
                self.stop_clock()
                self.active = False
                self.phase = "complete"
                self.checkpoint_bytes = self.session.checkpoint()
                self.message = "Planned allocation complete. Check the accuracy target separately."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)
