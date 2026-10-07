import marimo

__generated_with = "0.24.2"
app = marimo.App(width="full", app_title="FastSecDec · graph to Laurent vector")


@app.cell(hide_code=True)
async def _():
    import marimo as mo
    import sys
    from pathlib import Path
    if sys.platform == "emscripten":
        import hashlib
        import io
        import json
        import zipfile
        from importlib import import_module
        # Browser-only packages must not enter native Marimo's static import
        # registry: its missing-module probes run outside this platform branch.
        _micropip = import_module("micropip")
        _pyfetch = import_module("pyodide.http").pyfetch
        _base = f"{str(mo.notebook_location()).rstrip('/')}/public/fastsecdec"
        _response = await _pyfetch(f"{_base}/manifest.json")
        if not _response.ok:
            raise RuntimeError("Export this notebook with the tested HEPKit Pyodide wheel.")
        _manifest = await _response.json()
        _root = Path("/fastsecdec-showcase")
        _root.mkdir(exist_ok=True)
        for _kind in ("wheel", "assets"):
            _entry = _manifest[_kind]
            _response = await _pyfetch(f"{_base}/{_entry['filename']}")
            if not _response.ok:
                raise RuntimeError(f"Missing exported {_kind}")
            _data = await _response.bytes()
            if hashlib.sha256(_data).hexdigest() != _entry["sha256"]:
                raise RuntimeError(f"Exported {_kind} hash differs")
            if _kind == "wheel":
                _wheel = _root / _entry["filename"]
                _wheel.write_bytes(_data)
                await _micropip.install(f"emfs:{_wheel}")
            else:
                with zipfile.ZipFile(io.BytesIO(_data)) as _archive:
                    if set(_archive.namelist()) != set(_entry["files"]):
                        raise RuntimeError("Asset manifest differs")
                    for _name in _archive.namelist():
                        _path = Path(_name)
                        if _path.is_absolute() or ".." in _path.parts:
                            raise RuntimeError("Invalid asset path")
                        _bytes = _archive.read(_name)
                        if hashlib.sha256(_bytes).hexdigest() != _entry["files"][_name]:
                            raise RuntimeError("Asset hash differs")
                        _destination = _root / _path
                        _destination.parent.mkdir(parents=True, exist_ok=True)
                        _destination.write_bytes(_bytes)
    else:
        _root = Path(mo.notebook_location())
    sys.path.insert(0, str(_root))
    from showcase.notebook import Study
    return Study, mo


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    # FastSecDec · graph to Laurent vector

    Native scalar triangle, massless box, rank-two box and coupled two-loop sunset examples at spacelike kinematics. The measure is $\prod_l d^Dk_l/(i\pi^{D/2})$, with $D=4-2\epsilon$ and no implicit extra normalization.

    **Build diagrams → Generate → Inspect → Integrate QMC → Integrate Havana.**
    Every calculation starts with a button. Pause retains the native owner;
    Resume continues it. Changing a selection starts no calculation.
    One caller, eager Symbolica arithmetic, no SymJIT — the same workflow on
    native Python and Pyodide.
    """)
    return


@app.cell
def _(Study):
    study = Study("examples")
    study
    return (study,)


@app.cell(hide_code=True)
def _(mo, study):
    build = mo.ui.button(value=0, on_click=lambda n: n + 1,
                         label="Build diagrams", kind="success")
    build if study.kind == "gghh" else mo.md("Choose one of the native scalar examples below.")
    return (build,)


@app.cell
def _(build, mo, study):
    catalogue = study.build(build.value, mo) if study.kind == "gghh" else None
    mo.md(f"**{len(catalogue.diagrams)} diagrams** · " + " · ".join(
        f"{sum(d.loop_count == loops for d in catalogue.diagrams)} at {loops} loop(s)"
        for loops in (1, 2))) if catalogue is not None else mo.md("")
    return (catalogue,)


@app.cell(hide_code=True)
def _(catalogue, mo, study):
    catalogue
    diagram = mo.ui.dropdown(study.choices(), value=study.default_choice(),
                             allow_select_none=False, label="Native diagram / input")
    diagram
    return (diagram,)


@app.cell
def _(diagram, mo, study):
    study.diagram_view(mo, diagram.value)
    return


@app.cell(hide_code=True)
def _(mo, study):
    generate = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Generate sectors", kind="success")
    inspect = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Inspect", kind="neutral")
    qmc = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Integrate QMC", kind="success")
    havana = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Integrate Havana", kind="success")
    pause = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Pause", kind="warn")
    resume = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Resume")
    export = mo.ui.button(value=0, on_click=lambda n: n + 1, label="Prepare downloads")
    points = mo.ui.dropdown({str(n): n for n in (64, 256, 1024, 4096, 16384)}, value="1024", label="Points per shift / global MC batch")
    replicas = mo.ui.number(start=2, stop=64, value=4, step=1, label="Shifts / global batches")
    seed = mo.ui.number(start=0, stop=2**32-1, value=20261007, step=1, label="Seed")
    sector = mo.ui.number(start=0, value=0, step=1, label="Sector ID")
    epsilon_order = mo.ui.number(value=0, step=1, label="ε order")
    sqrt_s = mo.ui.number(start=251, value=300, label="sqrt(s) [GeV]")
    higgs_mass = mo.ui.number(start=1, value=125, label="mH [GeV]")
    top_mass = mo.ui.number(start=1, value=172.5, label="mt = Yukawa mass [GeV]")
    cos_theta = mo.ui.number(start=-0.999, stop=0.999, value=0.8, step=0.1, label="cos(theta)")
    get_active, set_active = mo.state(False)
    get_revision, set_revision = mo.state(0)
    get_prepared_revision, set_prepared_revision = mo.state(0)
    get_inspection_revision, set_inspection_revision = mo.state(0)
    get_artifact_revision, set_artifact_revision = mo.state(0)
    mo.vstack([
        mo.hstack([generate, inspect, qmc, havana], justify="start", wrap=True),
        mo.hstack([pause, resume, sector, epsilon_order], justify="start", wrap=True),
        mo.accordion({"Optional exports": mo.vstack([export, mo.md("Prepare portable evaluator and run-report bytes on the notebook caller, then use the download links below.")])}),
        mo.accordion({"Runtime physical point": mo.vstack([
            mo.hstack([sqrt_s, higgs_mass, top_mass, cos_theta], justify="start", wrap=True),
            mo.md("Bound only at Integrate. Native model leaves and momentum Gram products remain evaluator parameters; changing this point reuses generated sectors."),
        ])}) if study.kind == "gghh" else mo.md(""),
        mo.accordion({"Integration allocation": mo.vstack([
            mo.hstack([points, replicas, seed], justify="start", wrap=True),
            mo.md("QMC uses Kuo-33002 (at least 1024 points), Korobov-3 and independent shifted lattices. Havana trains two 64-point pilot batches, then freezes production; pilot samples are excluded. A complete allocation is not an accuracy guarantee."),
        ])}),
    ])
    return cos_theta, epsilon_order, export, generate, get_active, get_artifact_revision, get_inspection_revision, get_prepared_revision, get_revision, havana, higgs_mass, inspect, pause, points, qmc, replicas, resume, sector, seed, set_active, set_artifact_revision, set_inspection_revision, set_prepared_revision, set_revision, sqrt_s, top_mass


@app.cell(hide_code=True)
def _(get_active, mo):
    refresh = mo.ui.refresh(options=["125ms", "250ms", "1s"], default_interval="125ms", label="Caller steps") if get_active() else None
    refresh
    return (refresh,)


@app.cell(hide_code=True)
def _(cos_theta, diagram, epsilon_order, export, generate, havana, higgs_mass, inspect, mo, pause, points, qmc, refresh, replicas, resume, sector, seed, set_active, set_artifact_revision, set_inspection_revision, set_prepared_revision, set_revision, sqrt_s, study, top_mass):
    _before = study.run.work_active
    _revision = study.dispatch(
        {"generate": generate.value, "inspect": inspect.value, "qmc": qmc.value,
         "havana": havana.value, "pause": pause.value, "resume": resume.value, "export": export.value},
        refresh.value if refresh is not None else None,
        diagram.value, {"points": points.value, "replicas": replicas.value,
                        "seed": seed.value, "sector": sector.value, "epsilon_order": epsilon_order.value,
                        "sqrt_s": sqrt_s.value, "higgs_mass": higgs_mass.value,
                        "top_mass": top_mass.value, "cos_theta": cos_theta.value}, mo,
    )
    if study.run.work_active != _before:
        set_active(study.run.work_active)
    if _revision is not None:
        set_revision(_revision)
    _views = study.take_view_updates()
    if "prepared" in _views:
        set_prepared_revision(_views["prepared"])
    if "inspection" in _views:
        set_inspection_revision(_views["inspection"])
    if "artifact" in _views:
        set_artifact_revision(_views["artifact"])
    return


@app.cell(hide_code=True)
def _(get_revision, mo, study):
    get_revision()
    study.monitor(mo)
    return


@app.cell(hide_code=True)
def _(get_prepared_revision, mo, study):
    get_prepared_revision()
    study.prepared_view(mo)
    return


@app.cell
def _(get_artifact_revision, study):
    get_artifact_revision()
    generated = study.run.generated
    generated
    return (generated,)


@app.cell
def _(get_artifact_revision, study):
    get_artifact_revision()
    artifact = study.run.kernels
    artifact
    return (artifact,)


@app.cell(hide_code=True)
def _(get_inspection_revision, mo, study):
    get_inspection_revision()
    study.inspection_view(mo)
    return


@app.cell
def _(get_revision, mo, study):
    get_revision()
    study.result_view(mo)
    return


@app.cell(hide_code=True)
def _(get_artifact_revision, mo):
    from symbolica import get_citations
    get_artifact_revision()
    citations = get_citations()
    mo.accordion({"Native citations": mo.vstack([
        *[mo.as_html(citation) for citation in citations],
        mo.download("\n\n".join(c.to_bibtex() for c in citations).encode(), filename="fastsecdec-references.bib", label="Download BibTeX"),
    ])})
    return


if __name__ == "__main__":
    app.run()
