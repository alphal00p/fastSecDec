"""The complete notebook runs without adjacent project modules or fixtures."""

import ast
import inspect
import json
from pathlib import Path
import shutil
import subprocess
import sys
import textwrap

import pytest


NOTEBOOK = Path(__file__).resolve().parents[1] / "gghh_complete.py"


def test_complete_notebook_keeps_preparation_and_presentation_fixes_in_sync():
    """The standalone workflow must carry the same fixes as its maintained source."""
    class NormalizeDocstrings(ast.NodeTransformer):
        def visit_FunctionDef(self, node):
            if ast.get_docstring(node, clean=False) is not None:
                node.body[0].value.value = inspect.cleandoc(node.body[0].value.value)
            return self.generic_visit(node)

        visit_ClassDef = visit_FunctionDef

    complete = NormalizeDocstrings().visit(ast.parse(NOTEBOOK.read_text()))
    for filename, names in {
        "inputs.py": {"ShowcaseInput"},
        "gghh.py": {"GGHHInput", "prepare", "external_data", "scalar_dot", "tensor_vector"},
        "notebook.py": {"monitor", "prepared_view"},
        "science.py": {"generation"},
        "generation.py": {"generation_view", "timing_rows"},
    }.items():
        shared = NormalizeDocstrings().visit(ast.parse((NOTEBOOK.parent / "showcase" / filename).read_text()))
        for name in names:
            def definition(tree):
                return next(node for node in ast.walk(tree)
                            if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name == name)
            assert ast.dump(definition(complete)) == ast.dump(definition(shared)), (filename, name)


def test_complete_notebook_has_only_library_or_standard_imports():
    tree = ast.parse(NOTEBOOK.read_text())
    allowed = sys.stdlib_module_names | {"marimo", "symbolica"}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            assert all(alias.name.split(".")[0] in allowed for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            assert node.level == 0, "The complete notebook must not import local helpers"
            assert node.module.split(".")[0] in allowed
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            assert node.func.id not in {"exec", "eval", "__import__"}
            if node.func.id == "import_module":
                assert len(node.args) == 1 and isinstance(node.args[0], ast.Constant)
                assert node.args[0].value in {"micropip", "pyodide.http"}
        elif isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name):
            assert (node.value.id, node.attr) not in {("sys", "path"), ("sys", "modules")}
    assert "fixtures/" not in NOTEBOOK.read_text()


def test_complete_notebook_retains_explicit_actions_and_native_citations():
    tree = ast.parse(NOTEBOOK.read_text())
    labels = {
        keyword.value.value
        for node in ast.walk(tree) if isinstance(node, ast.Call)
        for keyword in node.keywords
        if keyword.arg == "label" and isinstance(keyword.value, ast.Constant)
    }
    assert {"Build diagrams", "Generate sectors", "Inspect", "Integrate QMC",
            "Integrate Havana", "Pause", "Resume", "Prepare downloads"} <= labels
    calls = [node.func for node in ast.walk(tree) if isinstance(node, ast.Call)]
    attributes = {node.attr for node in calls if isinstance(node, ast.Attribute)}
    assert {"standard_model", "generate_diagrams", "simplify_algebra",
            "generation_session", "mc_session", "session", "with_parameters"} <= attributes
    assert any(isinstance(node, ast.Name) and node.id == "get_citations" for node in calls)
    assert "fastsecdec-references.bib" in NOTEBOOK.read_text()


def test_copy_only_notebook_starts_without_scientific_work(tmp_path):
    pytest.importorskip("marimo")
    pytest.importorskip("symbolica.community.hepkit.sector_decomposition")
    copied = tmp_path / NOTEBOOK.name
    shutil.copyfile(NOTEBOOK, copied)
    # -I ignores PYTHONPATH, the invoking repository and the temporary working
    # directory as import roots. Only this .py file is copied; no helper package,
    # model card, DOT input or fixtures accompany it.
    probe = textwrap.dedent("""
        import importlib.util
        import json
        import sys

        operations = []
        scientific_names = {
            "standard_model", "generate_diagrams", "simplify_algebra",
            "sector_decompose", "generation_session", "qmc_session", "mc_session",
        }
        def record(frame, event, argument):
            if event == "c_call":
                name = getattr(argument, "__name__", "")
            elif event == "call":
                name = frame.f_code.co_name
            else:
                return
            if name in scientific_names:
                operations.append(name)

        spec = importlib.util.spec_from_file_location("relocated_gghh", sys.argv[1])
        notebook = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = notebook
        sys.setprofile(record)
        try:
            spec.loader.exec_module(notebook)
            outputs, definitions = notebook.app.run()
        finally:
            sys.setprofile(None)
        from symbolica.community import hepkit
        from symbolica.community.hepkit import sector_decomposition
        study = definitions["study"]
        assert study.catalogue is None
        assert study.run.phase == "draft"
        assert not study.run.work_active
        assert study.run.prepared is None and study.run.generated is None
        assert study.run.generation_session is None and study.run.session is None
        assert study.run.kernels is None and study.run.snapshot is None
        assert not operations, operations
        assert all(count == 0 for count in study.seen.values())
        assert not any(name == "showcase" or name.startswith("showcase.") for name in sys.modules)
        assert definitions["refresh"] is None
        assert definitions["generated"] is None and definitions["artifact"] is None
        assert len(outputs) > 0
        print("STANDALONE_NOTEBOOK=" + json.dumps({
            "phase": study.run.phase,
            "scientific_calls": operations,
            "native_model": hasattr(hepkit.Model, "standard_model"),
            "native_integral": hasattr(sector_decomposition, "Integral"),
        }))
    """)
    completed = subprocess.run(
        [sys.executable, "-I", "-c", probe, str(copied)],
        cwd=tmp_path, capture_output=True, text=True, timeout=60,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    result = next(line.removeprefix("STANDALONE_NOTEBOOK=")
                  for line in completed.stdout.splitlines()
                  if line.startswith("STANDALONE_NOTEBOOK="))
    assert json.loads(result) == {
        "phase": "draft", "scientific_calls": [],
        "native_model": True, "native_integral": True,
    }


def test_compiled_relocated_notebook_generates_and_binds_runtime_points(tmp_path):
    """Exercise retained class methods after Marimo has renamed/compiled cells."""
    pytest.importorskip("marimo")
    pytest.importorskip("symbolica.community.hepkit.sector_decomposition")
    copied = tmp_path / NOTEBOOK.name
    shutil.copyfile(NOTEBOOK, copied)
    probe = textwrap.dedent("""
        import math
        import runpy
        import sys

        app = runpy.run_path(sys.argv[1], run_name="notebook_check")["app"]
        _, notebook = app.run()
        study, mo, science = notebook["study"], notebook["mo"], notebook["science"]
        assert study.run.phase == "draft" and not study.run.work_active
        assert study.catalogue is None and study.run.prepared is None
        source = study.build(1, mo)
        box = next(diagram for diagram in source.diagrams
                   if diagram.loop_count == 1 and len(diagram.internal_edges) == 4
                   and all(abs(edge.particle.pdg_code) == 6 for edge in diagram.internal_edges))
        actions = dict(study.seen, generate=1)
        study.dispatch(actions, None, box.id, {}, mo)
        assert study.run.error is None, study.run.error
        owner = study.run.generation_session
        assert owner.mode == "symbolic" and owner.subtraction == "taylor"
        assert study.run.session is None
        actions["pause"] = 1
        study.dispatch(actions, None, box.id, {}, mo)
        assert study.run.phase == "generation_paused"
        assert study.run.generation_session is owner and not study.run.work_active
        actions["resume"] = 1
        study.dispatch(actions, None, box.id, {}, mo)
        for tick in range(1000):
            if not study.run.work_active:
                break
            study.dispatch(actions, tick, box.id, {}, mo)
            assert study.run.error is None, study.run.error
        assert study.run.phase == "ready" and owner.complete
        assert study.run.generation_session is owner and study.run.session is None
        assert study.run.generated.mode == "symbolic"
        assert study.run.kernels.backend == "symbolica_interpreter"
        assert all(stat.symjit_ir_bytes is None for stat in study.run.kernels.sector_statistics)
        prepared = study.run.prepared
        first = {"sqrt_s": 300, "higgs_mass": 125, "top_mass": 172.5, "cos_theta": 0.8}
        second = {"sqrt_s": 400, "higgs_mass": 125, "top_mass": 170, "cos_theta": 0.4}
        gram_first, gram_second = prepared.runtime_point(first), prepared.runtime_point(second)
        assert set(gram_first) == set(gram_second) == {symbol for _, _, symbol in prepared.gram_symbols}
        assert gram_first != gram_second
        assert all(math.isfinite(value) for point in (gram_first, gram_second) for value in point.values())
        template = study.run.kernels.to_bytes()
        bound_first, values_first = science.bind(study.run.generated, study.run.kernels, prepared, first)
        bound_second, values_second = science.bind(study.run.generated, study.run.kernels, prepared, second)
        assert values_first != values_second
        assert set(values_first) == set(values_second) == set(study.run.kernels.runtime_parameters)
        assert bound_first is not study.run.kernels and bound_second is not study.run.kernels
        assert study.run.kernels.to_bytes() == template and study.run.session is None
        print("COMPILED_NOTEBOOK_RUNTIME_POINTS=ok")
    """)
    completed = subprocess.run(
        [sys.executable, "-I", "-c", probe, str(copied)], cwd=tmp_path,
        capture_output=True, text=True, timeout=120,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    assert "COMPILED_NOTEBOOK_RUNTIME_POINTS=ok" in completed.stdout
