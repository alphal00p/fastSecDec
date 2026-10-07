"""Execute actual notebook/helper functions with native-owner doubles."""
import ast
from pathlib import Path
from types import SimpleNamespace

SOURCE = Path(__file__).resolve().parents[1] / "fastsecdec_showcase.py"


def cell_defining(name):
    cells = [node for node in ast.parse(SOURCE.read_text()).body if isinstance(node, ast.FunctionDef)
             and any(isinstance(child, ast.Name) and isinstance(child.ctx, ast.Store) and child.id == name
                     for child in ast.walk(node))]
    assert len(cells) == 1
    cell = cells[0]
    cell.decorator_list = []
    cell.name = "cell"
    scope = {}
    exec(compile(ast.Module(body=[cell], type_ignores=[]), str(SOURCE), "exec"), scope)
    return scope["cell"], cell


def science(namespace):
    source = SOURCE.parent / "showcase/science.py"
    tree = ast.parse(source.read_text())
    tree.body = [node for node in tree.body if not isinstance(node, ast.ImportFrom)]
    scope = {"sd": namespace}
    exec(compile(tree, str(source), "exec"), scope)
    return SimpleNamespace(generation=scope["generation"], integration=scope["integration"],
                           step=scope["step"], bind=scope["bind"], create_session=scope["integration"],
                           advance_session=scope["step"])


def generate(state, namespace, prepare, configuration):
    callbacks = science(namespace)
    state.start_generation(lambda: prepare(None), configuration, callbacks.generation)
    # Test convenience only: the actual notebook performs one unit per UI tick.
    while state.generation_active:
        state.advance_generation()
