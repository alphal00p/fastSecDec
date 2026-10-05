"""Read-only native sector inspection; no JSON parsing or coefficient expansion."""

from html import escape
from .presentation import table, epsilon_label


def overview(mo, generated):
    if generated is None:
        return mo.md("Generate first to inspect the decomposition.")
    if not hasattr(generated, "sectors") or not hasattr(generated, "metadata"):
        return mo.callout("This wheel lacks the native sector-inspection API. Install the updated experimental bindings.", kind="warn")
    charts = generated.metadata.charts
    rows = []
    for sector in generated.sectors:
        rows.append({
            "sector": sector.index, "dimension": sector.dimension,
            "generated orders": ", ".join(epsilon_label(order) for order in generated.orders),
            "source charts": sum(chart.kernel_sector == sector.index for chart in charts),
            "stored aliases per coefficient": ", ".join(str(count) for count in sector.alias_counts),
            "conditioning basis": sector.conditioning_basis.replace("_", " "),
        })
    no_kernel = sum(chart.kernel_sector is None for chart in charts)
    domain = generated.metadata.domain
    certificates = [{"term": factor.term_index, "factor": factor.factor_index,
                     "native certificate": factor.certificate.replace("_", " ")}
                    for factor in domain.factors]
    return mo.vstack([
        mo.md(f"**{len(rows)} numerical sectors** · {len(charts)} retained charts · branch: {domain.branch_policy.replace('_', ' ')}"),
        mo.md(f"Native domain: **{domain.domain.replace('_', ' ')}** · caller assertion: **{'yes' if domain.caller_asserted else 'no'}** · admission relies on assertion: **{'yes' if domain.relies_on_assertion else 'no'}**"),
        mo.accordion({f"Domain certificates · {len(certificates)} factors": table(mo, certificates)}),
        table(mo, rows),
        mo.md(f"{no_kernel} chart(s) retain no numerical kernel at the requested orders. This may mean exact, cancelled or truncated contributions; the native metadata does not classify them further.") if no_kernel else mo.md(""),
        mo.md("Alias counts describe stored definitions for each coefficient; shared definitions may repeat. They are not a count of unique algebraic complexity."),
    ])


def _geometry(mo, geometry):
    matrix = geometry.exponent_matrix
    rows = [{"source coordinate": i, **{f"target {j}": str(value) for j, value in enumerate(row)}} for i, row in enumerate(matrix)]
    return mo.vstack([
        table(mo, [{"source dimension": geometry.source_dimension, "target dimension": geometry.dimension,
                    "fixed parameter": str(geometry.fixed_parameter), "determinant": str(geometry.determinant),
                    "Jacobian powers": str(geometry.jacobian_powers), "factor valuations": str(geometry.factor_valuations)}]),
        table(mo, rows),
    ])


def _preview(value):
    # Reuse Symbolica's bounded native printer; this does not expand aliases.
    text = str(value.formatted(max_terms=12, max_line_length=80, show_namespaces=True))
    return text if len(text) <= 2000 else text[:2000] + " … [display preview clipped]"


def detail(mo, generated, index, coefficient_index=0, alias_page=0, chart_index=0):
    """Inspect one coefficient page and one chart after an explicit action."""
    native_sectors = generated.sectors
    if not 0 <= index < len(native_sectors):
        raise ValueError("Choose an existing generated sector")
    sector = native_sectors[index]
    coefficients = sector.aliased_coefficients
    if not 0 <= coefficient_index < len(coefficients):
        raise ValueError(f"Coefficient index must be between 0 and {len(coefficients)-1}")
    coefficient = coefficients[coefficient_index]
    definitions = coefficient.aliases
    page_count = max(1, (len(definitions) + 19) // 20)
    if not 0 <= alias_page < page_count:
        raise ValueError(f"Alias page must be between 0 and {page_count-1}")
    selected = definitions[alias_page * 20:(alias_page + 1) * 20]
    aliases = [{"alias": str(key), "native definition preview": _preview(value)} for key, value in selected]
    coefficient_view = mo.vstack([
        mo.md(f"**{epsilon_label(coefficient.order)}** · coefficient index {coefficient_index} · {coefficient.alias_count} stored definitions"),
        mo.Html('<pre style="white-space:pre-wrap;overflow:auto;max-height:18rem">' + escape(_preview(coefficient.root)) + '</pre>'),
        mo.md(f"Alias page **{alias_page} / {page_count-1}** · at most 20 definitions. Native formatting shows at most 12 terms and 2,000 characters per preview; the full expressions remain in the native owner."),
        table(mo, aliases),
    ])
    matching = [chart for chart in generated.metadata.charts if chart.kernel_sector == sector.index]
    chart_view = mo.md("No source chart points to this numerical sector.")
    if matching:
        if not 0 <= chart_index < len(matching):
            raise ValueError(f"Chart index must be between 0 and {len(matching)-1}")
        chart = matching[chart_index]
        coordinates = chart.coordinates
        if len(coordinates.source_parameters) != len(coordinates.images):
            raise ValueError("Native coordinate-map shape mismatch")
        images = [{"source parameter": str(source), "native image preview": _preview(image)}
                  for source, image in zip(coordinates.source_parameters, coordinates.images)]
        chart_view = mo.vstack([
            mo.md(f"**Source chart {chart.source_index}** · chart selection {chart_index} / {len(matching)-1}"),
            table(mo, [{"representative": chart.representative,
                        "representative permutation": str(chart.representative_permutation),
                        "source domain": coordinates.source_domain,
                        "gauge-fixed parameter": str(coordinates.projective_fixed_parameter)}]),
            table(mo, images),
            mo.md("Positive real measure Jacobian:"),
            mo.Html('<pre style="white-space:pre-wrap;overflow:auto">' + escape(_preview(coordinates.measure_jacobian)) + '</pre>'),
            _geometry(mo, chart.geometry),
        ])
    return mo.accordion({f"Sector {sector.index} · {sector.dimension} dimensions": mo.vstack([
        mo.md("Maps describe the density pullback **before endpoint subtraction**. Generated coefficients may be complex; compiled results can split real and imaginary components."),
        table(mo, [{"parameters": ", ".join(str(value) for value in sector.parameters),
                    "conditioning basis": sector.conditioning_basis,
                    "cancellation degree": sector.cancellation_degree,
                    "conditioning rows": str(sector.cancellation_terms)}]),
        mo.accordion({"Selected compact coefficient": coefficient_view,
                      "Selected source chart": chart_view,
                      "Sector geometry": _geometry(mo, sector.map)}),
    ])})
