"""Construction/packaging contracts without importing the native extension."""
import ast
from pathlib import Path
from types import SimpleNamespace as NS

ROOT = Path(__file__).resolve().parents[1]


def function(path, name, scope):
    tree = ast.parse(path.read_text())
    node = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == name)
    exec(compile(ast.Module(body=[node], type_ignores=[]), str(path), "exec"), scope)
    return scope[name]


def test_complete_catalogue_native_options_without_vertex_or_topology_filter():
    received = []
    diagrams = [NS(loop_count=2), NS(loop_count=1)]
    result = NS(report=NS(completed=True), diagrams=diagrams)
    native = NS(generate_diagrams=lambda **kw: received.append(kw) or result)
    scope = {"standard_model": lambda: ("model", {}), "process": lambda model: native,
             "E": lambda value: value, "Catalogue": lambda *args: args}
    call = function(ROOT / "showcase/gghh.py", "catalogue", scope)
    assert received == []
    assert call(progress=None)[-1] is result
    assert received == [{"loops": (1, 2), "coupling_orders": {"QED": 2}, "threads": 1,
        "symmetrize_initial": True, "symmetrize_final": True, "allow_zero_flow_edges": True,
        "maximum_bridges": None, "self_energy": None, "tadpoles": None,
        "zero_snails": None, "numerator_grouping": None, "projector": "1", "progress": None}]
    kwargs = []
    model = NS(process=lambda *args, **kw: kwargs.append((args, kw)))
    function(ROOT / "showcase/gghh.py", "process", {})(model)
    assert kwargs == [(([21, 21], [25, 25]), {"particle_selection": [6, 21, 25]})]


def test_notebooks_use_same_explicit_workflow_without_disabled_scientific_cells():
    for filename in ("gghh.py", "fastsecdec_showcase.py"):
        source = (ROOT / filename).read_text()
        ast.parse(source)
        assert 'disabled=True' not in source
        assert 'study.dispatch(' in source and 'study.build(' in source
        assert 'if get_active() else None' in source
        for label in ("Generate sectors", "Inspect", "Integrate QMC", "Integrate Havana", "Pause", "Resume"):
            assert f'label="{label}"' in source
        assert '.sector_decompose(' not in source and '.compile(' not in source


def test_export_packages_current_shared_modules_for_both_notebooks():
    source = (ROOT / "export.py").read_text()
    assert 'files = sorted((here / "showcase").glob("*.py"))' in source
    assert 'mode = args.mode or "run"' in source
    assert '--no-execute' in source
    assert 'fixtures/gghh' not in source and '__marimo__' not in source


def test_native_marimo_registry_does_not_probe_browser_only_modules():
    from marimo._ast.compiler import compile_cell
    for filename in ("gghh.py", "fastsecdec_showcase.py"):
        tree = ast.parse((ROOT / filename).read_text())
        bootstrap = next(node for node in tree.body if isinstance(node, ast.AsyncFunctionDef))
        code = "\n".join(ast.unparse(node) for node in bootstrap.body if not isinstance(node, ast.Return))
        cell = compile_cell(code, cell_id="boot")
        assert "micropip" not in cell.imported_namespaces
        assert "pyodide" not in cell.imported_namespaces
        assert "importlib" in cell.imported_namespaces


def test_widget_cells_are_owned_by_stable_scientific_revisions():
    for filename in ("gghh.py", "fastsecdec_showcase.py"):
        tree = ast.parse((ROOT / filename).read_text())
        for method, getter in (("prepared_view", "get_prepared_revision"),
                               ("inspection_view", "get_inspection_revision")):
            cell = next(node for node in tree.body if isinstance(node, ast.FunctionDef)
                        and any(isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute)
                                and call.func.attr == method for call in ast.walk(node)))
            inputs = {argument.arg for argument in cell.args.args}
            assert getter in inputs
            assert not inputs.intersection({"get_revision", "refresh", "revision"})
        source = (ROOT / filename).read_text()
        assert 'label="Prepare downloads"' in source
