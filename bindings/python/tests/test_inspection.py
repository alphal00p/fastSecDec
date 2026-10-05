"""Retained native geometry, compact expressions and owner lifetimes; no sampling."""

import gc
from pathlib import Path
import sys

import pytest
from symbolica import E, Expression
from symbolica.community import hepkit as hep

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "examples/hepkit"))
from showcase import inputs

fs = getattr(hep, "fastsecdec", None)
pytestmark = pytest.mark.skipif(fs is None, reason="requires experimental-fastsecdec wheel")


@pytest.fixture(scope="module")
def prepared():
    return inputs.massive_triangle()


@pytest.fixture(scope="module")
def generated(prepared):
    return fs.Integral(**prepared.integral_arguments()).generate(
        1, coefficient_expansion="native_named"
    )


def test_all_sector_views_preserve_native_chart_and_representative_indices(generated):
    sectors = generated.sectors
    charts = generated.metadata.charts
    assert len(sectors) == generated.sector_count > 0
    assert [sector.index for sector in sectors] == list(range(len(sectors)))
    assert [chart.source_index for chart in charts] == list(range(len(charts)))
    associated = set()
    for chart in charts:
        representative = charts[chart.representative]
        assert representative.representative == representative.source_index
        assert chart.kernel_sector == representative.kernel_sector
        assert sorted(chart.representative_permutation) == list(range(chart.geometry.dimension))
        if chart.kernel_sector is not None:
            associated.add(chart.kernel_sector)
            sector = sectors[chart.kernel_sector]
            assert representative.coordinates.target_parameters == sector.parameters
            assert representative.geometry.exponent_matrix == sector.map.exponent_matrix
            assert representative.geometry.determinant == sector.map.determinant
    assert associated == set(range(len(sectors)))
    for sector in sectors:
        assert sector.dimension == len(sector.parameters) == 2
        assert sector.coefficient_count == len(generated.orders)
        assert len(sector.alias_counts) == sector.coefficient_count
        assert sector.conditioning_basis in {"retained_remainders", "mapped_endpoint_bound"}
        with pytest.raises(AttributeError):
            sector.index = 100


def test_coordinate_images_and_positive_measure_preserve_exact_geometry(generated):
    domain = generated.metadata.domain
    assert domain.domain == "projective_simplex"
    assert domain.branch_policy == "no_threshold_real"
    assert not domain.caller_asserted and not domain.relies_on_assertion
    assert domain.factors
    for factor in domain.factors:
        assert factor.term_index >= 0 and factor.factor_index >= 0
        assert isinstance(factor.polynomial, Expression)
        assert isinstance(factor.exponent, Expression)
        assert factor.certificate in {
            "positive_coefficients", "negative_coefficients_integer_power"
        }
    for chart in generated.metadata.charts:
        coordinates, geometry = chart.coordinates, chart.geometry
        assert coordinates.source_parameters == domain.parameters
        assert len(coordinates.images) == geometry.source_dimension
        assert len(coordinates.target_parameters) == geometry.dimension
        assert coordinates.source_domain == domain.domain
        assert coordinates.projective_fixed_parameter == geometry.fixed_parameter
        assert isinstance(geometry.determinant, int) and geometry.determinant > 0
        assert all(isinstance(power, int) for row in geometry.exponent_matrix for power in row)
        assert all(isinstance(power, int) for power in geometry.jacobian_powers)
        # Small exact native Symbolica checks: the two independent retained views
        # must describe the same scientific map, including chart association.
        for image, powers in zip(coordinates.images, geometry.exponent_matrix, strict=True):
            expected = E("1")
            for parameter, power in zip(coordinates.target_parameters, powers, strict=True):
                expected *= parameter**power
            assert image == expected
        measure = E(str(geometry.determinant))
        for parameter, power in zip(coordinates.target_parameters, geometry.jacobian_powers, strict=True):
            measure *= parameter**power
        assert coordinates.measure_jacobian == measure
        fixed = coordinates.projective_fixed_parameter
        assert fixed is not None and coordinates.images[fixed] == E("1")
        assert sum(coordinates.images, E("0")) != E("1")  # Gauge-fixed, not normalized simplex.


def test_compact_roots_and_definitions_are_native_and_immutable(generated):
    saw_alias = False
    for sector in generated.sectors:
        coefficients = sector.aliased_coefficients
        assert [coefficient.order for coefficient in coefficients] == generated.orders
        assert [coefficient.alias_count for coefficient in coefficients] == sector.alias_counts
        for coefficient in coefficients:
            assert isinstance(coefficient.root, Expression)
            aliases = coefficient.aliases
            assert len(aliases) == coefficient.alias_count
            assert len({alias for alias, _ in aliases}) == len(aliases)
            assert all(isinstance(alias, Expression) and isinstance(body, Expression)
                       for alias, body in aliases)
            assert coefficient.aliases == aliases  # Stable native Atom order.
            saw_alias |= bool(aliases)
            with pytest.raises(AttributeError):
                coefficient.root = E("0")
    assert saw_alias, "Named generation must exercise real retained alias transport"
    # No public inspection path silently restores the complete expression.
    assert not hasattr(generated.sectors[0], "coefficients")


def test_selected_views_outlive_all_parent_python_variables(prepared):
    value = fs.Integral(**prepared.integral_arguments()).generate(
        0, coefficient_expansion="native_named"
    )
    sector = value.sectors[0]
    coefficient = sector.aliased_coefficients[0]
    metadata = value.metadata
    chart = metadata.charts[0]
    coordinates, geometry = chart.coordinates, chart.geometry
    domain = metadata.domain
    factor = domain.factors[0]
    retained = (coefficient.root, coefficient.aliases, coordinates.images,
                coordinates.measure_jacobian, geometry.exponent_matrix,
                factor.polynomial, factor.exponent, chart.kernel_sector, sector.dimension)
    del value, metadata
    gc.collect()
    assert (coefficient.root, coefficient.aliases, coordinates.images,
            coordinates.measure_jacobian, geometry.exponent_matrix,
            factor.polynomial, factor.exponent, chart.kernel_sector, sector.dimension) == retained
    del sector, chart, domain
    gc.collect()
    assert coefficient.root == retained[0] and coefficient.aliases == retained[1]
    assert coordinates.images == retained[2] and geometry.exponent_matrix == retained[4]
    assert factor.polynomial == retained[5]


def test_generated_complex_orders_are_distinct_from_compiled_components(prepared):
    arguments = prepared.integral_arguments()
    arguments["measure_multiplier"] = 2 + 3 * Expression.I
    value = fs.Integral(**arguments).generate(0, coefficient_expansion="native_named")
    assert value.orders == [0]
    assert all([coefficient.order for coefficient in sector.aliased_coefficients] == [0]
               for sector in value.sectors)
    kernels = value.compile()
    assert kernels.sector_count == value.sector_count
    assert kernels.orders == [0, 0] and kernels.components == ["real", "imag"]
    # No integration session is created, even when compiled layout is inspected.


def test_truncated_charts_retain_none_without_inventing_per_chart_zeros(prepared):
    arguments = prepared.integral_arguments()
    arguments["measure_multiplier"] = prepared.regulator**2
    value = fs.Integral(**arguments).generate(0)
    assert value.sectors == [] and value.sector_count == 0
    assert value.metadata.charts
    assert all(chart.kernel_sector is None for chart in value.metadata.charts)
    assert value.exact_coefficients == [E("0")]
    assert all(not hasattr(chart, "exact_coefficients") for chart in value.metadata.charts)
