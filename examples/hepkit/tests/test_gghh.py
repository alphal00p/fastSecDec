"""Native complete gg→HH catalogue and symbolic physical-point preparation."""
from pathlib import Path
import sys
import pytest
from symbolica import E
from symbolica.community import hepkit as hep

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from showcase import gghh

pytestmark = pytest.mark.skipif(
    not hasattr(hep, "sector_decomposition"), reason="requires current HEPKit FastSecDec bindings",
)


@pytest.fixture(scope="module")
def catalogue():
    return gghh.catalogue(progress=None)


def test_model_point_uses_native_card_and_preserves_zero_width_structure():
    model, values = gghh.standard_model()
    for name, expected in [("MT", "345/2"), ("MH", "125"), ("ymt", "345/2"), ("WT", "0"), ("WH", "0")]:
        assert values[model.parameter(name).symbol] == E(expected)
        assert model.parameter(name).value == complex(float(E(expected)))


def test_catalogue_keeps_all_native_loop_orders_and_default_first_one_loop(catalogue):
    assert catalogue.result.report.completed
    assert {diagram.loop_count for diagram in catalogue.diagrams} == {1, 2}
    assert catalogue.default_diagram.id == next(d.id for d in catalogue.diagrams if d.loop_count == 1)
    for diagram in catalogue.diagrams:
        assert all(abs(edge.particle.pdg_code) in {6, 21, 25} for edge in diagram.internal_edges)
    # Literal-all includes a one-loop topology with a Higgs bridge. Do not
    # silently narrow the allowed theory to ttg/ttH box vertices.
    assert any(d.loop_count == 1 and any(e.particle.pdg_code == 25 for e in d.internal_edges)
               for d in catalogue.diagrams)


def test_box_preparation_retains_native_gram_symbols_for_later_point_binding(catalogue):
    box = next(d for d in catalogue.diagrams if d.loop_count == 1
               and len(d.internal_edges) == 4
               and all(abs(e.particle.pdg_code) == 6 for e in d.internal_edges))
    value = gghh.prepare(selected=box.id, source=catalogue)
    arguments = value.integral_arguments()
    assert arguments["model_parameters"] == "runtime"
    assert set(arguments["scalar_values"]) == {value.model.parameter(name).symbol for name in ("WT", "WH")}
    assert arguments["runtime_parameters"] and value.raw_diagram.id == box.id
    # Admission checks the native Real attribute; symbolic names alone do not
    # establish that the runtime f64 input schema is real.
    hep.sector_decomposition.Integral(**arguments)
    assert value.raw_numerator is not None and value.simplified_numerator != E("0")
    # Polarization normalization remains a runtime Gram input too. Native f64
    # helicity components must not leave giant exact-binary rational constants.
    assert any(i == 3 and j == 4 for i, j, _ in value.gram_symbols)
    assert "316912650057057350374175801344" not in str(value.simplified_numerator)
    first = value.runtime_point({"sqrt_s": 300, "higgs_mass": 125, "cos_theta": 0.8})
    alternate = value.runtime_point({"sqrt_s": 320, "higgs_mass": 125, "cos_theta": 0.4})
    assert set(first) == set(alternate) == set(arguments["runtime_parameters"])
    assert first != alternate
    # Special angle zeros remain runtime values, never structural omissions.
    symmetric = value.runtime_point({"sqrt_s": 300, "higgs_mass": 125, "cos_theta": 0})
    assert set(symmetric) == set(first)
    legend = value.gram_legend()
    assert {row["Runtime symbol"] for row in legend} == {
        str(symbol.formatted(show_namespaces=True)) for symbol in arguments["runtime_parameters"]
    }
    assert all("leg " in row["Left vector"] and "leg " in row["Right vector"] for row in legend)
    assert any("eps1" in row["Left vector"] and "eps2" in row["Right vector"] for row in legend)


@pytest.mark.parametrize("identity", [
    "6586fc41a2a00087ef7be79f59b61224",  # User-reported graph: two triple-gluon vertices.
    "bf45cfca79c449b3c03e49aebf39b9dc",  # Distinct routing with the same tensor obstruction.
])
def test_triple_gluon_numerators_are_contracted_before_native_parametrization(catalogue, identity):
    from showcase import science

    diagram = catalogue.selected(identity)
    triple_gluon_vertices = [
        vertex for vertex in diagram.vertices
        if catalogue.model.vertex_rule(vertex.interaction).particles == ["g", "g", "g"]
    ]
    assert diagram.loop_count == 2 and len(triple_gluon_vertices) == 2
    prepared = gghh.prepare(selected=identity, source=catalogue)
    assert prepared.simplified_numerator != E("0")
    owner = science.generation(prepared, {"max_order": 0})
    # A tensor may be structurally scalar while retaining indexed contractions
    # across sums. The previous minimal policy failed on this first native unit.
    snapshot = owner.step(max_units=1)
    assert owner.failed is None
    assert snapshot.stage == "parametrization"
    assert snapshot.timings.parametrization_seconds > 0
    # Continue into genuine nonempty sector geometry, without expensive full
    # compilation or an integration in this regression test.
    for _ in range(3):
        snapshot = owner.step(max_units=1)
        if snapshot.stage == "geometry":
            break
    assert owner.failed is None
    assert snapshot.stage == "geometry" and snapshot.completed > 0


def test_box_numerator_pagers_are_native_cached_and_released(catalogue):
    import marimo as mo
    from showcase.notebook import Study
    box = next(d for d in catalogue.diagrams if d.loop_count == 1
               and len(d.internal_edges) == 4
               and all(abs(e.particle.pdg_code) == 6 for e in d.internal_edges))
    study = Study()
    study.run.prepared = gghh.prepare(selected=box.id, source=catalogue)
    original = study.run.prepared.simplified_numerator
    panel = study.prepared_view(mo)
    assert set(study.numerator_viewers) == {"raw", "simplified"}
    # Browser <details> keeps both descendants mounted on collapse. Marimo's
    # accordion unmounts content and closes the native widget's final view.
    assert panel.text.count("<details ") == 2
    assert panel.text.count("<marimo-anywidget ") == 2
    assert all("marimo-accordion" not in content.text
               for content in study.numerator_content.values())
    assert len(study.numerator_content) == 2
    study.publish()  # Monitor publication never recreates the widget cell.
    assert study.prepared_panel is panel
    assert study.run.prepared.simplified_numerator == original
    viewers = list(study.numerator_viewers.values())
    assert all(viewer._widget is not None and len(viewer._cache) <= 3 for viewer in viewers)
    # An intentional rerun of the widget-owning cell must use fresh models;
    # Marimo has already disposed the old cell's communication channels.
    assert study.prepared_view(mo) is not panel
    assert all(viewer._closed and viewer._source is None for viewer in viewers)
    viewers = list(study.numerator_viewers.values())
    study.close_numerators()
    assert all(viewer._closed and viewer._source is None for viewer in viewers)


def test_box_eager_schema_retains_model_leaves_and_polarization_normalization(catalogue):
    from showcase import science
    box = next(d for d in catalogue.diagrams if d.loop_count == 1
               and len(d.internal_edges) == 4
               and all(abs(e.particle.pdg_code) == 6 for e in d.internal_edges))
    prepared = gghh.prepare(selected=box.id, source=catalogue)
    owner = science.generation(prepared, {"max_order": 0})
    while not owner.complete:
        owner.step(max_units=1)
    names = {str(symbol.formatted(show_namespaces=True)) for symbol in owner.kernels.runtime_parameters}
    assert {"model::MT", "model::aS", "model::ymt", "gghh_kinematics::dot_3_4"} <= names
    assert owner.kernels.backend == "symbolica_interpreter"
    assert all(stat.symjit_ir_bytes is None for stat in owner.kernels.sector_statistics)
