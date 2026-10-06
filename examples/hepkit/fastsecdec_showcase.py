import marimo

__generated_with = "0.24.2"
app = marimo.App(width="medium", app_title="FastSecDec · from graph to Laurent vector")


@app.cell(hide_code=True)
def _():
    import marimo as mo
    return (mo,)


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    <div style="font-size:.78rem;letter-spacing:.16em;color:#7c699d">HEPKIT / FASTSECDEC</div>

    # From a Feynman graph to a Laurent vector

    Choose a native diagram, set its kinematics, and watch sector
    decomposition turn it into a numerical expansion in $\epsilon$.
    HEPKit and FastSecDec provide the native calculation. Every numerical value
    below comes from your explicit calculation. **Generate** prepares sectors and kernels; **Integrate** starts sampling separately.

    $D=4-2\epsilon$, with measure
    $\prod_\ell d^Dk_\ell/(i\pi^{D/2})$. Native graph weights and the scalar
    numerator enter **once**. There are no implicit Euler-gamma, scale or
    $4\pi$ factors.
    """)
    return


@app.cell(hide_code=True)
async def _(mo):
    import hashlib
    import io
    import json
    from pathlib import Path
    import sys
    import zipfile

    # Native: explicitly use the checked-out examples/hepkit asset directory.
    # Browser: explicitly fetch, verify and mount the exported assets before
    # importing builders. A browser filesystem is never assumed to contain them.
    notebook_location = mo.notebook_location()
    browser_runtime = sys.platform == "emscripten"
    interrupt_isolated = False
    if browser_runtime:
        import micropip
        from js import globalThis

        interrupt_isolated = bool(globalThis.crossOriginIsolated)
        from pyodide.http import pyfetch

        _base = str(notebook_location).rstrip("/")
        _manifest_response = await pyfetch(f"{_base}/public/fastsecdec/manifest.json")
        if not _manifest_response.ok:
            raise RuntimeError("Missing FastSecDec asset manifest. Use the showcase export helper.")
        _manifest = await _manifest_response.json()
        asset_root = Path("/fastsecdec-showcase")
        asset_root.mkdir(exist_ok=True)
        for _kind in ("wheel", "assets"):
            _entry = _manifest[_kind]
            _response = await pyfetch(f"{_base}/public/fastsecdec/{_entry['filename']}")
            if not _response.ok:
                raise RuntimeError(f"Unable to fetch {_kind} from the explicit asset bundle")
            _data = await _response.bytes()
            if hashlib.sha256(_data).hexdigest() != _entry["sha256"]:
                raise RuntimeError(f"FastSecDec {_kind} hash does not match the manifest")
            if _kind == "wheel":
                _wheel_path = asset_root / _entry["filename"]
                _wheel_path.write_bytes(_data)
                await micropip.install(f"emfs:{_wheel_path}")
            else:
                with zipfile.ZipFile(io.BytesIO(_data)) as _archive:
                    if set(_archive.namelist()) != set(_entry["files"]):
                        raise RuntimeError("Asset archive differs from its explicit file manifest")
                    for _name in _archive.namelist():
                        _relative = Path(_name)
                        if _relative.is_absolute() or ".." in _relative.parts:
                            raise RuntimeError("Invalid asset archive path")
                        _destination = asset_root / _relative
                        _destination.parent.mkdir(parents=True, exist_ok=True)
                        _destination.write_bytes(_archive.read(_name))
        asset_description = "Verified browser bundle, mounted at /fastsecdec-showcase"
    else:
        if notebook_location is None:
            raise RuntimeError("Open this notebook with marimo so its asset location is explicit.")
        asset_root = Path(notebook_location)
        asset_description = f"Native assets: {asset_root}"
    if not (asset_root / "showcase/inputs.py").is_file() or not (asset_root / "fixtures/fastsecdec/scalar.json").is_file():
        raise RuntimeError(f"Incomplete FastSecDec assets at {asset_root}")
    sys.path.insert(0, str(asset_root))
    from showcase import inputs as builders
    from showcase import state as workflow, generation, integration, sectors, presentation, report
    from showcase import gghh as gghh_builder
    from symbolica.community import hepkit as hep
    fs = getattr(hep, "fastsecdec", None)
    return asset_description, browser_runtime, builders, fs, gghh_builder, interrupt_isolated, workflow, generation, integration, sectors, presentation, report


@app.cell(hide_code=True)
def _(mo, presentation):
    _choices = dict(presentation.EXAMPLES)
    _choices["gg → HH · extended run"] = "gghh"
    problem = mo.ui.dropdown(_choices, value="Massive triangle", label="Integral", allow_select_none=False)
    mo.vstack([mo.md("## 1 · Choose the input"), problem])
    return (problem,)


@app.cell(hide_code=True)
def _(browser_runtime, mo, problem):
    physics_controls = None
    allocation_controls = None
    if problem.value == "gghh":
        _input_panel = mo.vstack([
            mo.md(r"""### $gg\to HH$ · fixed physical point
One HEPKit-generated top double box with an internal gluon and $(+,+)$ helicities. This single projected diagram is not the full gauge-invariant amplitude."""),
            mo.hstack([
                mo.stat(label="Energy √s", value="300 GeV"), mo.stat(label="Higgs mass", value="125 GeV"),
                mo.stat(label="Top mass", value="172.5 GeV"), mo.stat(label="cos θ", value="4/5"),
            ], widths="equal", wrap=True),
            mo.accordion({"Conventions and allocation": mo.md(r"""
            Native HEPKit algebra closes the unnormalized color projection $\delta_{ab}$;
            shared GammaLoop wavefunctions supply the incoming helicities. Internal
            algebra retains $D=4-2\epsilon$ and external states are four-dimensional.
            Feynman gauge, generated weights and couplings, no spin/color average.

            The ordinary domain guard remains active. Generate prepares through the
            finite coefficient. Integrate uses 1,024 points × 8 shifts per sector,
            Kuo 33002/Korobov-3, seed 20261005, in 1,024-point caller steps.
            This fixed allocation need not meet the 0.1% target. Browser generation
            and interpreted integration can be substantially slower than native execution.
            """)}),
            mo.callout("Optional extended browser run: one CPU, with portable interpreted integration. Generate may take several minutes; completion and browser cost are not yet validated. No calculation begins until you select Generate.", kind="warn") if browser_runtime else mo.md(""),
        ])
    else:
        _fields = {"s": mo.ui.number(value=-1.0, step=0.1, label="s < 0")}
        if problem.value == "triangle":
            _fields["mass"] = mo.ui.number(start=0.0, value=1.0, step=0.1, label="Mass m")
        if problem.value in {"box", "rank_two_box"}:
            _fields["t"] = mo.ui.number(value=-1.0, step=0.1, label="t < 0")
        _fields["max_order"] = mo.ui.dropdown({"Finite term · ε⁰": 0, "Through ε¹": 1}, value="Through ε¹", label="Highest requested order")
        physics_controls = mo.ui.batch(mo.Html('<div style="display:flex;gap:1.5rem;flex-wrap:wrap">' + ''.join('<div>{' + name + '}</div>' for name in _fields) + '</div>'), _fields)
        allocation_controls = mo.ui.batch(mo.Html('''
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:1rem">
        <div>{points}</div><div>{shifts}</div><div>{package_points}</div><div>{rule}</div><div>{seed}</div>
        </div>'''), {
            "points": mo.ui.dropdown({"1,024": 1024, "4,096": 4096, "16,384": 16384}, value="1,024", label="Points per sector and shift"),
            "shifts": mo.ui.dropdown({"4": 4, "8": 8, "16": 16}, value="8", label="Independent shifts"),
            "package_points": mo.ui.dropdown({"256": 256, "1,024": 1024}, value="1,024", label="Points per caller step"),
            "rule": mo.ui.dropdown({"Kuo 33002": "kuo_33002", "Kuo 38005": "kuo_38005", "Kuo 39101": "kuo_39101", "HKKN α=3": "hkkn_alpha3"}, value="Kuo 33002", label="Published lattice"),
            "seed": mo.ui.number(start=0, stop=2**32 - 1, step=1, value=20261005, label="Seed"),
        })
        _input_panel = mo.vstack([physics_controls, mo.accordion({"Integration settings · fixed allocation": allocation_controls}),
                   mo.md("Editing these controls does not change an existing generated input or result. Select Generate to bind new physics; select Integrate separately to start sampling.")])
    _input_panel
    return allocation_controls, physics_controls


@app.cell(hide_code=True)
def _(allocation_controls, physics_controls, problem):
    if problem.value == "gghh":
        draft = {"example": "gghh", "max_order": 0, "points": 1024, "shifts": 8,
                 "seed": 20261005, "package_points": 1024, "rule": "kuo_33002"}
    else:
        draft = {"example": problem.value, **physics_controls.value, **allocation_controls.value}
    return (draft,)


@app.cell(hide_code=True)
def _(browser_runtime, interrupt_isolated, mo, workflow):
    run_state = workflow.RunState(message="Choose the input, then Generate. No scientific work starts automatically.")
    generate_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Generate", kind="success")
    integrate_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Integrate", kind="success")
    cancel_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Cancel", kind="danger")
    resume_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Resume checkpoint")
    refresh = mo.ui.refresh(options=["250ms", "1s", "5s"], default_interval="250ms", label="Caller step / refresh")
    mo.vstack([
        mo.md("## 2 · Generate → inspect → integrate"),
        mo.hstack([generate_button, integrate_button, cancel_button, resume_button, refresh], justify="start", wrap=True),
        mo.md("Generate includes native compilation and stops before sampling. Integrate starts a fixed allocation from ready kernels. Cancel saves accepted coverage; Resume restores it."),
        mo.accordion({"Execution and cancellation": mo.vstack([
            mo.md("Cancel pauses integration between packages and saves accepted coverage. In marimo's editor, Stop (interrupt) / Ctrl-I (Cmd-I on macOS) requests KeyboardInterrupt. Generation checks it at native callback boundaries; integration checks within packages every 256 points. Long algebra operations between checks can delay interruption. An interrupted package does not enter accepted coverage. Only explicit Integrate or Resume enables further packages; no allocation grows automatically."),
            mo.callout(
                "Browser interrupt is available in this isolated editor. Use Stop (interrupt) or Ctrl-I (Cmd-I on macOS); long native algebra calls may delay the response."
                if interrupt_isolated and mo.app_meta().mode == "edit" else
                "This view or host has no active KeyboardInterrupt control. Cancel is processed between integration packages. For browser interruption, export with --mode edit and use the documented isolated server. Reloading the page discards unsaved in-memory work.",
                kind="info" if interrupt_isolated and mo.app_meta().mode == "edit" else "warn",
            ) if browser_runtime else mo.md(""),
        ])}),
    ])
    return cancel_button, generate_button, integrate_button, refresh, resume_button, run_state


@app.cell(hide_code=True)
def _(builders, cancel_button, draft, fs, generate_button, generation, gghh_builder, integrate_button, integration, mo, presentation, refresh, resume_button, run_state):
    _actions = {"generate": generate_button.value, "integrate": integrate_button.value,
                "cancel": cancel_button.value, "resume": resume_button.value, "tick": refresh.value}
    _changed = {key for key, value in _actions.items() if value != run_state.seen[key]}
    run_state.seen.update(_actions)
    if "cancel" in _changed:
        run_state.cancel()
    elif "generate" in _changed:
        _validation = None if draft["example"] == "gghh" else presentation.validate_configuration(draft)
        if _validation:
            run_state.message = _validation
        elif fs is None:
            run_state.message = "Install a wheel with the experimental FastSecDec API."
        else:
            _configuration = dict(draft)
            _prepare = (lambda observer: gghh_builder.prepare(observer=observer)) if draft["example"] == "gghh" else (lambda observer: builders.prepare(_configuration))
            run_state.generate(fs, _prepare, _configuration,
                display=lambda event: mo.output.replace(generation.generation_view(mo, run_state)),
                input_display=lambda event: mo.output.replace(mo.vstack([
                    mo.md("**HEPKit diagram generation**"), presentation.table(mo, [{"native stage": event.stage, "completed": event.completed, "total": event.total}]),
                ])))
            if run_state.prepared is not None:
                try:
                    run_state.drawing = mo.Html(f'<div style="max-width:480px;margin:auto">{run_state.prepared.diagram.render()}</div>')
                except Exception as _drawing_error:
                    run_state.drawing = mo.callout(f"Native graph rendering is unavailable: {_drawing_error}", kind="warn")
    elif "integrate" in _changed:
        # Draft physics never replaces generated provenance. The fixed ggHH
        # allocation also stays independent of another currently edited input.
        _settings = None
        if run_state.configuration is not None and run_state.configuration["example"] != "gghh" and draft["example"] == run_state.configuration["example"]:
            _settings = {key: draft[key] for key in ("points", "shifts", "seed", "package_points", "rule")}
        run_state.integrate(fs, _settings)
    elif "resume" in _changed:
        run_state.resume()
    elif "tick" in _changed:
        run_state.advance()
    _content = [mo.callout(run_state.message, kind="danger" if run_state.error else "info")]
    if run_state.error:
        _content.append(mo.md(f"`{run_state.error}`"))
    if run_state.checkpoint_warning:
        _content.append(mo.callout(run_state.checkpoint_warning, kind="warn"))
    if run_state.configuration is not None:
        _c = run_state.configuration
        _content.append(mo.md(f"**Generated input:** {_c['example']} · requested $\\epsilon^{{{_c['max_order']}}}$. Draft edits do not alter this native owner."))
    _content.extend([generation.generation_view(mo, run_state), integration.result_view(mo, run_state)])
    mo.output.replace(mo.vstack(_content))
    run_revision = (run_state.phase, run_state.snapshot, len(run_state.events), run_state.generated)
    return (run_revision,)


@app.cell(hide_code=True)
def _(generation, mo, run_revision, run_state, sectors):
    run_revision
    mo.vstack([
        mo.md("## 3 · Native input and generated sectors"),
        generation.input_view(mo, run_state),
        sectors.overview(mo, run_state.generated),
    ])
    return


@app.cell(hide_code=True)
def _(mo):
    sector_index = mo.ui.number(start=0, value=0, step=1, label="Sector index")
    coefficient_index = mo.ui.number(start=0, value=0, step=1, label="Coefficient index")
    alias_page = mo.ui.number(start=0, value=0, step=1, label="Alias page")
    chart_index = mo.ui.number(start=0, value=0, step=1, label="Chart index")
    inspect_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Inspect sector")
    numerator_button = mo.ui.button(value=0, on_click=lambda n: n+1, label="Inspect weighted numerator")
    mo.vstack([mo.hstack([sector_index, coefficient_index, alias_page, chart_index], justify="start", wrap=True), mo.hstack([inspect_button, numerator_button], justify="start", wrap=True)])
    return alias_page, chart_index, coefficient_index, inspect_button, numerator_button, sector_index


@app.cell(hide_code=True)
def _(alias_page, chart_index, coefficient_index, inspect_button, mo, numerator_button, run_revision, run_state, sector_index, sectors):
    run_revision
    _actions = {"inspect": inspect_button.value, "numerator": numerator_button.value}
    _changed = {key for key, value in _actions.items() if value != run_state.seen[key]}
    run_state.seen.update(_actions)
    if "inspect" in _changed:
        try:
            if run_state.generated is None:
                raise ValueError("Generate an input first")
            run_state.inspected_sector = sectors.detail(mo, run_state.generated, int(sector_index.value), int(coefficient_index.value), int(alias_page.value), int(chart_index.value))
        except Exception as _inspection_error:
            run_state.inspected_sector = mo.callout(f"Sector inspection failed: {_inspection_error}", kind="warn")
    elif "numerator" in _changed:
        try:
            if run_state.prepared is None:
                raise ValueError("Generate an input first")
            run_state.numerator_view = run_state.prepared.scalar_numerator()
        except Exception as _numerator_error:
            run_state.numerator_view = mo.callout(f"Native numerator inspection failed: {_numerator_error}", kind="warn")
    mo.vstack([item for item in (run_state.inspected_sector, run_state.numerator_view) if item is not None])
    return


@app.cell(hide_code=True)
def _(asset_description, mo, report, run_revision, run_state):
    run_revision
    _downloads = []
    if run_state.configuration is not None and not run_state.active:
        _downloads.append(mo.download(lambda: report.report_bytes(run_state), filename="fastsecdec-report.json", label="Download run report"))
    if run_state.kernels is not None and not run_state.active:
        _downloads.append(mo.download(lambda: run_state.kernels.to_bytes(), filename="fastsecdec-kernels.bin", label="Download native kernels"))
    if run_state.checkpoint_bytes is not None and not run_state.active:
        _downloads.append(mo.download(run_state.checkpoint_bytes, filename="fastsecdec-checkpoint.json", label="Download checkpoint"))
    mo.vstack([
        mo.hstack(_downloads, justify="start") if _downloads else mo.md(""),
        mo.accordion({"Reading the result · provenance and limits": mo.md(f"""
        Full signed Laurent vectors and real/imaginary covariance come from native
        snapshots. Missing uncertainty stays missing. The highest requested order
        target is 0.1%; complete production is required. Completing an allocation
        does not guarantee that target. Successive history observations share samples.

        Generated sector coefficients may be complex, while compiled estimates split
        their real and imaginary components. Native backend labels distinguish O2
        kernels from the portable interpreter. Interactive active time includes
        refresh waits; native worker time is separate. No responsiveness guarantee
        or prepared numerical output is supplied.

        **Assets:** {asset_description}. gg → HH is an optional extended run;
        browser completion and cost remain unvalidated. See `README.md` for
        source/build provenance, explicit browser asset mounting and interruption.
        """)}),
    ])
    return


if __name__ == "__main__":
    app.run()
