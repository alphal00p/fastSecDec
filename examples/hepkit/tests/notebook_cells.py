"""Execute selected real notebook cells with supplied native-owner doubles."""
import ast
from pathlib import Path
from types import SimpleNamespace

SOURCE = Path(__file__).resolve().parents[1] / "fastsecdec_showcase.py"


def cell_defining(name):
    tree = ast.parse(SOURCE.read_text())
    cells = [node for node in tree.body if isinstance(node, ast.FunctionDef)
             and any((isinstance(child, ast.Name) and isinstance(child.ctx, ast.Store)
                      and child.id == name)
                     or (isinstance(child, ast.FunctionDef) and child.name == name)
                     for child in ast.walk(node))]
    assert len(cells) == 1
    cell = cells[0]
    cell.decorator_list = []
    cell.name = "cell"
    scope = {}
    exec(compile(ast.Module(body=[cell], type_ignores=[]), str(SOURCE), "exec"), scope)
    return scope["cell"], cell


def science(namespace):
    generation, _ = cell_defining("decompose_input")
    sampling, _ = cell_defining("create_session")
    presentation = SimpleNamespace(show_code=lambda: None)
    compile_sectors, decompose_input = generation(presentation)
    advance_session, create_session = sampling(presentation, namespace)
    return SimpleNamespace(compile_sectors=compile_sectors, decompose_input=decompose_input,
                           advance_session=advance_session, create_session=create_session)


def generate(state, namespace, prepare, configuration, **options):
    callbacks = science(namespace)
    return state.generate(callbacks.decompose_input, prepare, configuration,
                          compile=callbacks.compile_sectors, **options)
