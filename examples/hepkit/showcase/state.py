"""Explicit caller-owned actions; no work runs merely by creating this state."""

from __future__ import annotations
from dataclasses import dataclass, field
from time import perf_counter
from .integration import highest_order_rows


@dataclass
class RunState:
    prepared: object = None
    generated: object = None
    generation_session: object = None
    generation_active: bool = False
    generation_display: object = None
    generation_last_display: float = -float("inf")
    kernels: object = None
    parameter_point: dict = field(default_factory=dict)
    session: object = None
    snapshot: object = None
    observation: object = None
    live_observation: object = None
    configuration: dict | None = None
    events: list = field(default_factory=list)
    input_events: list = field(default_factory=list)
    history: list = field(default_factory=list)
    phase: str = "draft"
    active: bool = False
    active_started: float | None = None
    active_seconds: float = 0.0
    preparation_seconds: float = 0.0
    pilot_seconds: float = 0.0
    message: str = "Submit inputs, then select Generate. Integration starts separately."
    error: str | None = None
    checkpoint_warning: str | None = None
    checkpoint_bytes: bytes | None = None
    previous_report_bytes: bytes | None = None
    previous_snapshot: object = None
    previous_configuration: dict | None = None
    previous_phase: str | None = None
    inspected_sector: object = None
    numerator_view: object = None
    drawing: object = None
    seen: dict = field(default_factory=lambda: {"generate": 0, "integrate": 0, "new": 0, "cancel": 0, "resume": 0, "adapt": 0, "freeze": 0, "inspect": 0, "numerator": 0, "tick": ""})

    @property
    def is_mc(self):
        return self.configuration is not None and self.configuration.get("method", "qmc") == "havana_discrete_mc"

    @property
    def pilot(self):
        return self.is_mc and self.session is not None and self.session.stage == "pilot"

    def save_checkpoint(self):
        if not getattr(self.session, "checkpoint_available", True):
            self.checkpoint_bytes = None
        else:
            self.checkpoint_bytes = self.session.checkpoint()

    @property
    def integration_wall_seconds(self):
        return self.active_seconds + (perf_counter() - self.active_started if self.active_started is not None else 0.0)

    def stop_clock(self):
        self.active_seconds = self.integration_wall_seconds
        self.active_started = None

    def observe_generation(self, event, display=None):
        self.events.append(event)
        now = perf_counter()
        if display is not None and (now - self.generation_last_display >= 1.0 or event.stage == "complete"):
            display(event)
            self.generation_last_display = now
        return True

    @property
    def work_active(self):
        return self.active or self.generation_active

    def start_generation(self, prepare, configuration, create_session, *, display=None):
        """Construct a retained native generation owner; sampling stays absent."""
        if self.work_active:
            self.message = "Pause the active calculation before choosing another input."
            return
        self.prepared = self.generated = self.kernels = self.session = self.snapshot = None
        self.generation_session = None
        self.observation = self.live_observation = None
        self.parameter_point.clear()
        self.previous_report_bytes = self.previous_snapshot = None
        self.previous_configuration = self.previous_phase = None
        self.preparation_seconds = 0.0
        self.configuration = dict(configuration)
        self.events.clear()
        self.history.clear()
        self.input_events.clear()
        self.inspected_sector = self.numerator_view = self.drawing = None
        self.checkpoint_bytes = None
        self.error = self.checkpoint_warning = None
        self.active_seconds = self.pilot_seconds = 0.0
        self.active_started = None
        self.phase = "preparing"
        self.message = "Preparing the selected native diagram. Pause is serviced at the next native unit boundary."
        self.generation_display = display
        self.generation_last_display = -float("inf")
        if display is not None:
            display()
        started = perf_counter()
        try:
            self.prepared = prepare()
            self.preparation_seconds = perf_counter() - started
            self.generation_session = create_session(self.prepared, configuration)
            self.generation_active = not self.generation_session.complete
            self.phase = "generating"
            self.message = "Generating sectors and eager evaluators. Pause retains this native owner."
            if self.generation_active:
                self.advance_generation()
            else:
                self.generated = self.generation_session.generated
                self.kernels = self.generation_session.kernels
                self.phase = "ready"
                self.message = "Native generation is already complete. No points sampled."
        except (Exception, KeyboardInterrupt) as error:
            self.preparation_seconds = self.preparation_seconds or perf_counter() - started
            self.generation_active = False
            self.fail(error)

    def advance_generation(self, *, display=True):
        if not self.generation_active:
            return
        try:
            self.generation_session.step(max_units=1, observer=lambda event: self.observe_generation(
                event, (lambda _: self.generation_display()) if display and self.generation_display is not None else None))
            self.generated = self.generation_session.generated
            self.kernels = self.generation_session.kernels
            if self.generation_session.complete:
                self.generation_active = False
                self.phase = "ready"
                self.message = "Sectors and eager evaluators ready. Inspect them, then integrate explicitly. No points sampled."
        except KeyboardInterrupt:
            self.generation_active = False
            self.phase = "generation_paused"
            self.message = "Generation paused; Resume continues the retained native owner."
        except Exception as error:
            self.generated = self.generation_session.generated
            self.kernels = self.generation_session.kernels
            self.generation_active = False
            if self.generation_session.failed is None:
                self.phase = "generation_paused"
                self.error = f"{type(error).__name__}: {error}"
                self.message = "Caller/presentation interruption; the native unit is retained. Resume retries the next unit."
            else:
                self.fail(error)

    def pause(self):
        if self.generation_active:
            self.generation_active = False
            self.phase = "generation_paused"
            self.message = "Generation paused between native units. Resume continues the same owner."
        else:
            self.cancel()

    def integrate(self, create_session, settings=None):
        """Start a fresh native session only from already prepared kernels."""
        if self.kernels is None:
            self.message = "Generate and compile the input before integration."
            return
        if self.session is not None:
            self.message = "This allocation already exists. Resume it, or choose New integration after stopping to reuse these kernels with different settings."
            return
        try:
            configuration = dict(self.configuration)
            if settings is not None:
                allowed = {"method", "points", "shifts", "seed", "package_points", "rule", "periodization",
                           "pilot_points", "pilot_batches", "points_per_batch", "batches", "bins",
                           "minimum_probability_density", "maximum_sector_probability_ratio"}
                if set(settings) - allowed:
                    raise ValueError("Integrate can change allocation settings only; Generate binds the physics input")
                configuration.update(settings)
            if configuration.get("method", "qmc") not in {"qmc", "havana_discrete_mc"}:
                raise ValueError("Choose QMC or Havana discrete Monte Carlo")
            inactive_keys = ({"points", "shifts", "package_points", "rule", "periodization"}
                             if configuration.get("method", "qmc") == "havana_discrete_mc" else
                             {"pilot_points", "pilot_batches", "points_per_batch", "batches", "bins",
                              "minimum_probability_density", "maximum_sector_probability_ratio"})
            for key in inactive_keys:
                configuration.pop(key, None)
            self.session = create_session(self.kernels, configuration)
            self.configuration = configuration
            self.snapshot = self.session.snapshot()
            self.capture_observation()
            self.active = not self.session.complete
            self.active_started = perf_counter() if self.active else None
            self.phase = ("pilot" if self.pilot else "integrating") if self.active else ("pilot_ready" if self.pilot else "complete")
            self.error = None
            self.message = ("Havana pilot. Production starts after this pilot; its samples stay excluded." if self.pilot else "Integrating with short caller work budgets; native observations refresh once per second.") if self.active else "The native allocation is exact and already complete."
            if not self.active:
                self.save_checkpoint()
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def new_integration(self):
        """Release only inactive sampling state; retain the generated native owners."""
        if self.active:
            self.message = "Cancel the active allocation before choosing New integration."
            return
        if self.kernels is None:
            self.message = "Generate and compile an input first."
            return
        if self.session is not None:
            try:
                from .report import report_bytes
                self.previous_report_bytes = report_bytes(self)
                self.previous_snapshot = self.snapshot
                self.previous_configuration = dict(self.configuration)
                self.previous_phase = self.phase
            except (Exception, KeyboardInterrupt) as error:
                self.fail(error)
                return
        self.session = self.snapshot = None
        self.observation = self.live_observation = None
        self.history.clear()
        self.checkpoint_bytes = None
        self.error = self.checkpoint_warning = None
        self.active_seconds = self.pilot_seconds = 0.0
        self.active_started = None
        self.phase = "ready"
        self.message = "Same generated input and compiled kernels retained. Choose a method/allocation, then Integrate explicitly. The previous allocation report remains downloadable."

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
                self.capture_observation()
            except (Exception, KeyboardInterrupt) as secondary:
                warnings.append(f"Snapshot capture failed: {type(secondary).__name__}: {secondary}")
            try:
                self.save_checkpoint()
            except (Exception, KeyboardInterrupt) as secondary:
                warnings.append(f"Checkpoint capture failed: {type(secondary).__name__}: {secondary}")
            self.checkpoint_warning = " ".join(warnings) or None

    def cancel(self):
        if self.session is None or not self.active:
            return
        self.stop_clock()
        self.active = False
        try:
            self.save_checkpoint()
            self.snapshot = self.session.snapshot()
            self.capture_observation()
            self.phase = "paused"
            self.message = "Pilot paused in memory; Resume continues this native session. Downloadable checkpoints become available after freezing production." if self.pilot else "Cancelled by the caller between packages. Accepted coverage is saved."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def resume(self):
        if self.phase == "generation_paused" and self.generation_session is not None:
            self.generation_active = True
            self.phase = "generating"
            self.error = None
            self.message = "Resuming native generation and eager compilation."
            return
        if self.active or self.kernels is None:
            return
        if self.pilot:
            try:
                self.snapshot = self.session.snapshot()
                self.capture_observation()
                self.active = not self.session.complete
                self.active_started = perf_counter() if self.active else None
                self.phase = "pilot" if self.active else "pilot_ready"
                self.error = None
                self.message = "Resumed the retained native pilot in memory." if self.active else "Pilot complete. Adapt another pilot or freeze production explicitly."
            except (Exception, KeyboardInterrupt) as error:
                self.fail(error)
            return
        if self.checkpoint_bytes is None:
            return
        if self.checkpoint_warning:
            self.message = "Checkpoint capture failed. The native session is retained; an older checkpoint will not be resumed silently."
            return
        try:
            restore = self.kernels.restore_mc if self.is_mc else self.kernels.restore
            self.session = restore(self.checkpoint_bytes)
            self.snapshot = self.session.snapshot()
            self.capture_observation()
            self.active = not self.session.complete
            self.active_started = perf_counter() if self.active else None
            self.error = None
            self.phase = "integrating" if self.active else "complete"
            self.message = "Resumed from the native checkpoint." if self.active else "The saved allocation is already complete."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def pilot_action(self, freeze=False):
        """Explicit native adaptation/freeze; pilot estimates never enter production."""
        if self.active or not self.pilot or not self.session.complete:
            self.message = "Complete and stop a Havana pilot before adapting or freezing production."
            return
        try:
            elapsed = self.integration_wall_seconds
            if freeze:
                self.snapshot = self.session.freeze_production(
                    points_per_batch=self.configuration["points_per_batch"], batches=self.configuration["batches"],
                )
            else:
                self.snapshot = self.session.adapt_pilot()
            self.capture_observation()
            self.pilot_seconds += elapsed
            self.history.clear()
            self.checkpoint_bytes = None
            self.active_seconds = 0.0
            self.active = not self.session.complete
            self.active_started = perf_counter() if self.active else None
            self.phase = "pilot" if self.pilot else "integrating"
            self.error = self.checkpoint_warning = None
            self.message = "Another native pilot epoch started." if self.pilot else "Production grids frozen; pilot estimates discarded. Sampling in short caller work budgets."
            if not self.active:
                self.phase = "pilot_ready" if self.pilot else "complete"
                self.save_checkpoint()
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)

    def capture_observation(self):
        # Read-only native views; these never enter checkpoints or convergence.
        self.observation = self.session.observation() if hasattr(self.session, "observation") else None
        self.live_observation = self.session.live_observation() if hasattr(self.session, "live_observation") else None

    def record_estimate(self):
        if self.snapshot.estimate is not None:
            for row in highest_order_rows(self.snapshot.estimate):
                self.history.append({"accepted points": self.snapshot.completed_points, **row})

    def advance(self, step, *, observe=True):
        """Only an explicit Integrate or Resume action can arm these steps."""
        if not self.active:
            return
        try:
            self.snapshot = step(self.session, self.configuration.get("method", "qmc"))
            if observe or self.session.complete:
                self.capture_observation()
                self.record_estimate()
            if self.session.complete:
                self.stop_clock()
                self.active = False
                self.phase = "pilot_ready" if self.pilot else "complete"
                self.save_checkpoint()
                self.message = "Pilot complete. Adapt another pilot or Freeze production explicitly; pilot values are not production estimates." if self.pilot else "Planned allocation complete. Check the accuracy target separately."
        except (Exception, KeyboardInterrupt) as error:
            self.fail(error)
