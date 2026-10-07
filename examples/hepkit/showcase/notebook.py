"""Notebook presentation and explicit caller actions, independent of numerics."""
from .state import RunState
from . import generation, integration, sectors
from .presentation import panel
from time import perf_counter
from html import escape


class Study:
    def __init__(self, kind="gghh"):
        if kind not in {"gghh", "examples"}:
            raise ValueError("Unknown notebook study")
        self.kind = kind
        self.catalogue = None
        self.run = RunState()
        self.seen = {key: 0 for key in ("build", "generate", "inspect", "qmc", "havana", "pause", "resume", "export")}
        self.last_tick = None
        self.inspection = None
        self.inspection_request = None
        self.expression_viewer = None
        self.prepared_panel = None
        self.prepared_owner = None
        self.numerator_viewers = {}
        self.numerator_content = {}
        self.work_budget_seconds = 0.05
        self.work_unit_limit = 2048
        self.observation_interval = 1.0
        self.last_publication = -float("inf")
        self.revision = 0
        self.view_revisions = {name: 0 for name in ("prepared", "inspection", "artifact")}
        self.pending_views = set()
        self.prepared_downloads = {}

    def _repr_html_(self):
        title = "gg → HH · native diagram study" if self.kind == "gghh" else "FastSecDec · native integral study"
        return f'<div style="padding:1rem;border:1px solid #7583a0;border-radius:12px"><b>{title}</b><br>Single caller · eager Symbolica evaluator · explicit scientific actions</div>'

    def close_inspection(self):
        if self.expression_viewer is not None:
            self.expression_viewer.close()
            self.expression_viewer = None
        self.inspection = None

    def close_numerators(self):
        for viewer in self.numerator_viewers.values():
            viewer.close()
        self.numerator_viewers.clear()
        self.numerator_content.clear()
        self.prepared_owner = self.prepared_panel = None

    def invalidate_views(self, *names):
        for name in names:
            self.view_revisions[name] += 1
            self.pending_views.add(name)

    def take_view_updates(self):
        updates = {name: self.view_revisions[name] for name in self.pending_views}
        self.pending_views.clear()
        return updates

    def build(self, count, mo):
        if count != self.seen["build"]:
            self.seen["build"] = count
            if self.run.work_active:
                self.run.message = "Pause the calculation before rebuilding diagrams."
            else:
                from .gghh import catalogue
                def progress(event):
                    mo.output.replace(panel(mo, "Building native diagram catalogue", mo.vstack([
                        mo.md(f"**{event.stage}** · {event.completed} / {event.total if event.total is not None else '…'}"),
                        mo.md("One and two loops · QED² · Higgs, gluon, top · initial and final symmetries"),
                    ])))
                    return True
                self.catalogue = catalogue(progress=progress)
        return self.catalogue

    def choices(self):
        if self.kind == "examples":
            return {"Massive triangle": "triangle", "Massless box": "box",
                    "Rank-two box numerator": "rank_two_box", "Coupled two-loop sunset": "sunset"}
        if self.catalogue is None:
            return {"Build diagrams first": None}
        return {f"{d.loop_count} loop · {d.name} · {i}": d.id
                for i, d in enumerate(self.catalogue.diagrams)}

    def default_choice(self):
        options = self.choices()
        if self.catalogue is None:
            return next(iter(options))
        identity = self.catalogue.default_diagram.id
        return next(label for label, value in options.items() if value == identity)

    def diagram_view(self, mo, identity):
        if self.kind != "gghh" or self.catalogue is None or identity is None:
            return mo.md("Build the native catalogue, then choose a diagram." if self.kind == "gghh" else "Generate prepares the selected scalar example.")
        diagram = self.catalogue.selected(identity)
        return mo.vstack([
            mo.md(f"**{diagram.name}** · {diagram.loop_count} loop(s) · native ID `{diagram.id}`"),
            mo.as_html(diagram),
        ])

    def dispatch(self, actions, tick, selection, settings, mo):
        from . import science
        changed = {key for key, value in actions.items() if value != self.seen[key]}
        self.seen.update(actions)
        tick_changed = tick is not None and tick != self.last_tick
        self.last_tick = tick
        run = self.run
        previous_phase = run.phase
        was_generating = run.generation_active
        if changed.intersection({"generate", "qmc", "havana", "resume"}):
            self.prepared_downloads.clear()
        if "pause" in changed:
            run.pause()
        elif "resume" in changed:
            run.resume()
        elif "generate" in changed:
            if run.work_active:
                run.message = "Pause the active calculation before choosing another input."
                return self.publish()
            self.inspection_request = None
            if self.kind == "gghh":
                if self.catalogue is None or selection is None:
                    run.message = "Build the diagram catalogue first."
                else:
                    from .gghh import prepare
                    configuration = {"example": "gghh", "diagram_id": selection, "max_order": 0}
                    run.start_generation(lambda: prepare(selected=selection, source=self.catalogue), configuration, science.generation,
                                         display=lambda: mo.output.replace(self.monitor(mo)))
            else:
                from .inputs import prepare
                configuration = {"example": selection, "mass": 1.0, "s": -1.0, "t": -1.0, "max_order": 0}
                run.start_generation(lambda: prepare(configuration), configuration, science.generation,
                                         display=lambda: mo.output.replace(self.monitor(mo)))
            self.invalidate_views("prepared", "inspection", "artifact")
        elif "qmc" in changed or "havana" in changed:
            if run.kernels is None:
                run.message = "Generate the selected input before starting integration."
            elif "qmc" in changed and int(settings["points"]) < 1024:
                run.message = "The selected native Kuo-33002 rule requires at least 1024 points per shift. Increase Points, then select Integrate QMC."
            elif run.work_active:
                run.message = "Pause the active calculation before starting another integration."
            else:
                if run.session is not None:
                    run.new_integration()
                    if run.session is not None:
                        # A failed report/checkpoint handoff retains the old
                        # owner and point; do not bind a new point around it.
                        return self.publish()
                try:
                    run.kernels, point = science.bind(run.generated, run.kernels, run.prepared, settings)
                    self.invalidate_views("artifact")
                    run.parameter_point = point
                    if self.kind == "gghh":
                        run.configuration["physical_point"] = {
                            key: float(settings[key]) for key in ("sqrt_s", "higgs_mass", "top_mass", "cos_theta")
                        }
                except (Exception, KeyboardInterrupt) as error:
                    run.fail(error)
                    return self.publish()
                method = "havana_discrete_mc" if "havana" in changed else "qmc"
                common = {"method": method, "seed": int(settings["seed"])}
                allocation = ({"pilot_points": 64, "pilot_batches": 2,
                               "points_per_batch": int(settings["points"]), "batches": int(settings["replicas"]),
                               "bins": 16, "minimum_probability_density": 0.01,
                               "maximum_sector_probability_ratio": 100.0}
                              if method == "havana_discrete_mc" else
                              {"points": int(settings["points"]), "shifts": int(settings["replicas"]),
                               "package_points": 4096, "rule": "kuo_33002", "periodization": "korobov3"})
                run.integrate(science.integration, {**common, **allocation})
        elif "inspect" in changed:
            if run.generated is None or run.kernels is None:
                run.message = "Finish generation before inspecting the numerical sectors."
            else:
                # Widgets must be created AND displayed by the dedicated cell:
                # Marimo disposes comms belonging to a cell whenever it reruns.
                self.inspection_request = (int(settings["sector"]), int(settings.get("epsilon_order", 0)))
                self.invalidate_views("inspection")
        elif "export" in changed:
            self.prepare_downloads()
        elif tick_changed:
            self.advance_budget(science)
        if was_generating and not run.generation_active:
            self.invalidate_views("artifact")
        if changed or previous_phase != run.phase or perf_counter() - self.last_publication >= self.observation_interval:
            if run.active:
                try:
                    run.capture_observation()
                    run.record_estimate()
                except (Exception, KeyboardInterrupt) as error:
                    run.fail(error)
            return self.publish()
        return None

    def publish(self):
        self.last_publication = perf_counter()
        self.revision += 1
        return self.revision

    def advance_budget(self, science):
        """Spend a short caller budget, then return control to the frontend."""
        deadline = perf_counter() + self.work_budget_seconds
        for _ in range(self.work_unit_limit):
            if self.run.generation_active:
                self.run.advance_generation()
            elif self.run.active:
                self.run.advance(science.step, observe=False)
                # The explicit Havana action authorizes pilot then production.
                # Native freeze discards the pilot estimator before continuing.
                if self.run.phase == "pilot_ready":
                    self.run.pilot_action(freeze=True)
            else:
                break
            if not self.run.work_active or perf_counter() >= deadline:
                break

    def monitor(self, mo):
        run = self.run
        content = [mo.callout(mo.md(f"{run.error}\n\n{run.message}") if run.error else run.message,
                              kind="danger" if run.error else "info")]
        if run.generation_session is not None or run.phase == "preparing":
            content.append(panel(mo, "Generation · retained native units", generation.generation_view(mo, run), expanded=run.generation_active or run.phase == "generation_paused"))
        if run.snapshot is not None:
            content.append(panel(mo, "Integration · accepted native work", integration.result_view(mo, run)))
            if run.live_observation is not None:
                content.append(mo.as_html(run.live_observation))
        if run.checkpoint_warning:
            content.append(mo.callout(run.checkpoint_warning, kind="warn"))
        return mo.vstack(content)

    def prepared_view(self, mo):
        # This runs only in the stable prepared-input cell. If that cell is
        # intentionally rerun, create new widgets instead of stale cached comms.
        self.close_numerators()
        prepared = self.run.prepared
        if prepared is None:
            return mo.md("")
        raw = getattr(prepared, "raw_numerator", None)
        simplified = getattr(prepared, "simplified_numerator", None)
        items = {}
        if raw is not None:
            self.numerator_viewers["raw"] = raw.paged(page_size=25)
            items["Raw numerator · native tensor expression"] = mo.as_html(self.numerator_viewers["raw"])
        if simplified is not None:
            from symbolica.community.spenso import TensorExpression
            self.numerator_viewers["simplified"] = TensorExpression(simplified).paged(page_size=25)
            items["Simplified projected numerator · native Symbolica expression"] = mo.as_html(self.numerator_viewers["simplified"])
        self.prepared_owner = prepared
        # Marimo's accordion unmounts closed content. Native Pager responds by
        # disposing its final widget view, so cached markup cannot reopen it.
        # HTML details only hides descendants: both widget DOM trees stay
        # mounted, and their native owners/UI wrappers remain retained here.
        self.numerator_content = items
        panels = [mo.Html(
            '<details style="border:1px solid var(--gray-5);border-radius:10px;padding:0.8rem">'
            '<summary style="cursor:pointer;font-weight:600">' + escape(title) + '</summary>'
            '<div style="padding-top:0.8rem">' + content.text + '</div></details>'
        ) for title, content in items.items()]
        content = panels or [mo.as_html(prepared.diagram)]
        if hasattr(prepared, "gram_legend"):
            content.insert(0, mo.accordion({"Kinematic symbols · momentum and polarization products": mo.vstack([
                mo.md("`dot_i_j` is a runtime Minkowski scalar product, with metric (+, −, −, −). "
                      "Its vectors are listed below using this diagram's native external-leg routing. "
                      "The symbols stay unbound during generation; Integrate binds them from the chosen "
                      "energy, Higgs mass, scattering angle and (+,+) helicities."),
                sectors._static_table(mo, list(prepared.gram_legend())),
            ])}))
        self.prepared_panel = mo.vstack([mo.md(f"**Prepared input:** {prepared.name}"), *content])
        return self.prepared_panel

    def inspection_view(self, mo):
        """Create native pagers only in their stable inspection display cell."""
        self.close_inspection()
        if self.inspection_request is None:
            return mo.md("Select Inspect after generation to open the evaluator overview and sector detail.")
        try:
            index, order = self.inspection_request
            coefficient_index = 0
            if self.run.generated.sectors:
                coefficient_index = sectors.coefficient_index(self.run.generated, index, order)
                self.expression_viewer = sectors.expression_viewer(self.run.generated, index, coefficient_index)
            self.inspection = sectors.inspection(mo, self.run.generated, self.run.kernels, index,
                coefficient_index=coefficient_index, viewer=self.expression_viewer)
        except Exception as error:
            self.close_inspection()
            self.inspection = mo.callout(f"Inspection failed: {error}", kind="danger")
        return self.inspection

    def prepare_downloads(self):
        """Explicit native serialization on the notebook caller, never a download worker."""
        if self.run.work_active:
            self.run.message = "Pause the calculation before preparing downloads."
            return
        if self.run.kernels is None:
            self.run.message = "Finish generation before preparing downloads."
            return
        try:
            from .report import report_bytes
            artifact = self.run.kernels.to_bytes()
            report = report_bytes(self.run)
            self.prepared_downloads = {"artifact": artifact, "report": report}
            self.run.message = "Downloads are ready. Their bytes describe this retained artifact and current allocation."
        except (Exception, KeyboardInterrupt) as error:
            self.run.message = f"Could not prepare downloads: {type(error).__name__}: {error}"

    def result_view(self, mo):
        run = self.run
        native = mo.as_html(run.observation or run.snapshot) if run.snapshot is not None else mo.md("No integration has run.")
        previous = integration.previous_result_view(mo, run)
        downloads = []
        if self.prepared_downloads and not run.work_active:
            downloads.append(mo.download(self.prepared_downloads["artifact"], filename="fastsecdec-kernels.dat", label="Evaluator artifact"))
            downloads.append(mo.download(self.prepared_downloads["report"], filename="fastsecdec-report.json", label="Full run report"))
        if run.previous_report_bytes is not None:
            downloads.append(mo.download(run.previous_report_bytes, filename="fastsecdec-previous-report.json", label="Previous run report"))
        if run.checkpoint_bytes is not None and not run.work_active:
            downloads.append(mo.download(run.checkpoint_bytes, filename="fastsecdec-checkpoint.json", label="Accepted checkpoint"))
        point = mo.accordion({"Bound evaluator parameter point": sectors._static_table(mo, [
            {"Native symbol": sectors._formula(mo, symbol), "Value": value}
            for symbol, value in run.parameter_point.items()
        ])}) if run.parameter_point else mo.md("")
        return mo.vstack([native, point, previous, mo.hstack(downloads, justify="start") if downloads else mo.md("")])
