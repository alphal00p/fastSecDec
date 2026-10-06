"""The code displayed in the notebook is also its sole scientific call path."""
from types import SimpleNamespace as NS

from notebook_cells import cell_defining, science
from test_refresh import Notebook


def test_visible_function_definitions_are_inert_and_render_their_actual_source():
    displays = []
    mo = NS(show_code=lambda: displays.append("current-cell-source"))
    generation, _ = cell_defining("decompose_input")
    sampling, _ = cell_defining("create_session")
    generation(mo)
    sampling(mo, None)  # A backend is not even needed until an explicit action.
    assert displays == ["current-cell-source", "current-cell-source"]


def test_native_owner_options_and_observer_pass_through_without_mutating_inputs():
    calls = []
    event = object()
    observed = []
    observer = lambda value: observed.append(value) or True
    kernels = object()
    class Generated:
        def compile(self, *, observer):
            calls.append("compile")
            observer(event)
            return kernels
    generated = Generated()
    class Diagram:
        def sector_decompose(self, **kwargs):
            calls.append(kwargs)
            kwargs["observer"](event)
            return generated
    diagram = Diagram()
    arguments = dict(diagram=diagram, kinematics=object(), regulator=object(),
                     dimension=object(), powers={}, scalar_values={object(): object()},
                     auxiliary_momenta=[object()], measure_multiplier=object())
    original = dict(arguments)
    prepared = NS(**{key: arguments[key] for key in ("diagram", "kinematics", "regulator", "dimension")},
                  integral_arguments=lambda: arguments,
                  generation_arguments=lambda: {"coefficient_expansion": "native_named"})
    callbacks = science(None)
    assert callbacks.decompose_input(prepared, 2, observer) is generated
    forwarded = calls[0]
    assert arguments == original
    assert all(forwarded[key] is value for key, value in arguments.items() if key != "diagram")
    assert "diagram" not in forwarded and "numerator" not in forwarded
    assert forwarded["max_order"] == 2 and forwarded["coefficient_expansion"] == "native_named"
    assert forwarded["observer"] is observer and observed == [event]
    assert callbacks.compile_sectors(generated, observer) is kernels
    assert observed == [event, event] and len(calls) == 2


def test_native_render_owner_reaches_marimo_without_string_coercion():
    notebook = Notebook()
    rendered = object()
    received = []
    prepared = notebook.env["builders"].prepare({})
    prepared.diagram.render = lambda: rendered
    notebook.mo.as_html = lambda owner: received.append(owner) or owner
    notebook.dispatch("generate")
    assert notebook.run.drawing is rendered and received == [rendered]
    assert notebook.calls == {"generate": 1, "compile": 1, "session": 0, "step": 0}
