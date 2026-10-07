"""Read-only native sector inspection; no JSON parsing or coefficient expansion."""
from .presentation import panel

from collections import Counter
from html import escape
from .presentation import table, epsilon_label


def _monomials(chart):
    """Group native retained monomial factors, not inferred integrand limits."""
    from symbolica import E
    groups = {}
    record = getattr(chart, "pre_subtraction", None)
    if record is None:
        return []
    for term in record.terms:
        value = E("1")
        for parameter, power in zip(chart.coordinates.target_parameters, term.powers):
            value *= parameter ** power.exponent
        groups[value] = groups.get(value, 0) + 1
    return list(groups.items())


def overview(mo, generated, kernels=None):
    if generated is None:
        return mo.md("Generate first to inspect the decomposition.")
    charts = generated.metadata.charts
    statistics = getattr(kernels, "sector_statistics", ())
    chart_counts = Counter(chart.kernel_sector for chart in charts)
    indexed = {sector.index: sector for sector in generated.sectors}
    ranked = sorted(indexed, key=lambda index: (
        -statistics[index].exact_program_bytes if index < len(statistics) else 0, index))
    rows = []
    for index in ranked[:10]:
        sector = indexed[index]
        stats = statistics[index] if index < len(statistics) else None
        matching = [chart for chart in charts if chart.kernel_sector == index]
        representative = next((chart for chart in matching if chart.source_index == chart.representative), matching[0] if matching else None)
        groups = _monomials(representative) if representative is not None else []
        monomials = mo.vstack([mo.hstack([_formula(mo, value), mo.md(f"× {count} mapped term(s)")], justify="start")
                              for value, count in groups[:3]]) if groups else mo.md("Not retained")
        if len(groups) > 3:
            monomials = mo.vstack([monomials, mo.md(f"+ {len(groups)-3} more; inspect sector {index}")])
        rows.append({"Sector ID": index, "Coordinates": sector.dimension,
                     "Evaluator bytes": stats.exact_program_bytes if stats else "Not recorded",
                     "Native operations": _operation_total(stats) if stats else "Not recorded",
                     "Retained pre-subtraction monomial factors": monomials})
    sizes = [stat.exact_program_bytes for stat in statistics]
    return mo.vstack([
        mo.md(f"### {len(indexed)} numerical sectors · {len(charts)} source charts"),
        _static_table(mo, [{"Backend": getattr(kernels, "backend", "Not compiled"),
                           "Evaluator storage": f"{sum(sizes):,} bytes",
                           "Largest evaluator": f"{max(sizes, default=0):,} bytes",
                           "Orders": str(generated.orders)}]),
        mo.md("**Ten largest shared evaluators**, ordered by serialized native program size. IDs are the saved kernel indices."),
        _static_table(mo, rows),
        mo.md("Monomials precede endpoint subtraction and symmetry multiplicity; the regular body may still vanish. Operation counts follow native Horner/CPE optimization. No numerical session is created by inspection."),
        mo.md(f"{chart_counts[None]} chart(s) have no numerical kernel at these orders; this alone does not distinguish exact, cancelled or truncated terms.") if chart_counts[None] else mo.md(""),
    ])


def coefficient_index(generated, index, order):
    """Resolve a physical Laurent order from the native retained schema."""
    if not 0 <= index < len(generated.sectors):
        raise ValueError("Choose an existing generated sector")
    coefficients = generated.sectors[index].aliased_coefficients
    for position, coefficient in enumerate(coefficients):
        if coefficient.order == order:
            return position
    available = ", ".join(str(value.order) for value in coefficients)
    raise ValueError(f"Sector {index} has no epsilon order {order}; available orders: {available}")


def expression_viewer(generated, index, coefficient_index):
    """Open only the explicitly selected native coefficient's scoped pager."""
    from symbolica.community.spenso import TensorExpression
    native_sectors = generated.sectors
    if not 0 <= index < len(native_sectors):
        raise ValueError("Choose an existing generated sector")
    coefficients = native_sectors[index].aliased_coefficients
    if not 0 <= coefficient_index < len(coefficients):
        raise ValueError(f"Coefficient index must be between 0 and {len(coefficients)-1}")
    # Native alias materialization is an explicit inspection operation. The
    # native Pager bounds rendering and owns navigation/cache lifetimes.
    return TensorExpression(coefficients[coefficient_index].expression()).paged(page_size=25)


def inspection(mo, generated, kernels, index, coefficient_index=0, viewer=None):
    if not generated.sectors:
        return mo.vstack([overview(mo, generated, kernels),
                          mo.md("There are no numerical sectors at these orders. Bind the physical point and integrate to obtain the native exact contribution, if any.")])
    views = [overview(mo, generated, kernels),
             detail(mo, generated, index, coefficient_index=coefficient_index, kernels=kernels)]
    if viewer is not None:
        coefficient = generated.sectors[index].aliased_coefficients[coefficient_index]
        views.append(panel(mo, f"Sector {index} integrand · {epsilon_label(coefficient.order)}", mo.vstack([
            mo.md("The actual native Symbolica Laurent coefficient after endpoint subtraction, including the retained sector's symmetry multiplicity. The global coordinate-independent exact contribution is separate. HEPKit renders one scoped page at a time; use the viewer controls to explore the full expression."),
            mo.as_html(viewer),
        ])))
    return mo.vstack(views)


def _static_table(mo, rows):
    """Small selected-detail tables, with native rich math cells preserved."""
    if not rows:
        return mo.md("No retained entries.")
    columns = list(rows[0])

    def cell(value):
        if value is None:
            return "—"
        if isinstance(value, (str, int, float, bool)):
            return escape(str(value))
        return mo.as_html(value).text

    header = "".join(f"<th>{escape(str(key))}</th>" for key in columns)
    body = "".join("<tr>" + "".join(f"<td>{cell(row.get(key))}</td>" for key in columns) + "</tr>" for row in rows)
    return mo.Html("""<style>
        .fsd-inspection-table {width:100%;border-collapse:collapse}
        .fsd-inspection-table th,.fsd-inspection-table td {padding:.45rem .8rem;text-align:left;vertical-align:top}
        .fsd-inspection-table th {border-bottom:1px solid var(--gray-5,#e2e8f0)}
        .fsd-inspection-table .katex-display {margin:.25rem 0;text-align:left}
        </style><div style="overflow:auto"><table class="fsd-inspection-table"><thead><tr>""" + header + '</tr></thead><tbody>' + body + '</tbody></table></div>')


def _geometry(mo, geometry):
    matrix = geometry.exponent_matrix
    rows = [{"source coordinate": i, **{f"target {j}": str(value) for j, value in enumerate(row)}} for i, row in enumerate(matrix)]
    return mo.vstack([
        _static_table(mo, [{"source dimension": geometry.source_dimension, "target dimension": geometry.dimension,
                    "fixed parameter": str(geometry.fixed_parameter), "determinant": str(geometry.determinant),
                    "Jacobian powers": str(geometry.jacobian_powers), "factor valuations": str(geometry.factor_valuations)}]),
        _static_table(mo, rows),
    ])


def _preview(value):
    # Reuse Symbolica's bounded native printer; this does not expand aliases.
    text = str(value.formatted(max_terms=12, max_line_length=80, show_namespaces=True))
    return text if len(text) <= 2000 else text[:2000] + " … [display preview clipped]"


def _operation_total(stats):
    operations = stats.operations
    return sum(getattr(operations, key) for key in ("additions", "multiplications", "inversions", "function_calls"))


def _compact(value):
    return str(value.formatted(max_terms=12, max_line_length=80, show_namespaces=True))


def _formula(mo, value):
    # Symbolica supplies the bounded LaTeX; no expression reconstruction or CAS.
    # Qualified identities remain in the exact-source metadata and downloads.
    formatted = value.formatted(max_terms=12, max_line_length=80, show_namespaces=False)
    latex = formatted._repr_latex_()
    if latex and len(latex) <= 4000:
        return mo.md(latex)
    return mo.Html('<pre style="white-space:pre-wrap;overflow:auto">' + escape(_preview(value)) + '</pre>')


def _pre_subtraction(mo, chart, term_index):
    record = getattr(chart, "pre_subtraction", None)
    if record is None:
        return mo.md("Pre-subtraction factors were not retained by this artifact or wheel. They cannot be recovered from the expanded coefficients without new algebra.")
    terms = record.terms
    if not terms:
        return mo.md("This source chart had no nonzero mapped terms before subtraction.")
    if not 0 <= term_index < len(terms):
        raise ValueError(f"Mapped term index must be between 0 and {len(terms)-1}")
    term = terms[term_index]
    parameters = chart.coordinates.target_parameters
    if len(parameters) != len(term.powers):
        raise ValueError("Native pre-subtraction power dimension differs")
    powers = [{"coordinate": _formula(mo, parameter), "exponent b + c ε": _formula(mo, power.exponent),
               "b": _formula(mo, power.constant), "c": _formula(mo, power.slope),
               "Taylor coefficients required": power.subtraction_count}
              for parameter, power in zip(parameters, term.powers)]
    source = [{"coordinate": _compact(parameter), "exact exponent": _preview(power.exponent)}
              for parameter, power in zip(parameters, term.powers)]
    return mo.vstack([
        mo.md(f"**Mapped term {term_index} / {len(terms)-1}** · metadata v{record.version} · regulator {_compact(record.regulator)}"),
        mo.md("Each term has the form $P(\\epsilon)\\,\\prod_i t_i^{b_i+c_i\\epsilon}\\,R(t,\\epsilon)$. The native prefactor and powers below precede symmetry multiplicity, endpoint subtraction and Laurent expansion."),
        mo.md("**Native prefactor** $P(\\epsilon)$"),
        _formula(mo, term.prefactor),
        _static_table(mo, powers),
        mo.md(f"Regular body native Atom storage: **{term.regular_expression_bytes:,} bytes**. Its full body is not duplicated in this metadata. Native Taylor endpoint subtraction implements the corresponding plus-distribution continuation. Taylor counts are endpoint admission requirements, not a count of surviving poles or a floating-point conditioning estimate."),
        mo.accordion({"Exact native names and prefactor source": mo.vstack([
            _static_table(mo, source),
            mo.Html('<pre style="white-space:pre-wrap;overflow:auto">' + escape(_preview(term.prefactor)) + '</pre>'),
            mo.download(str(term.prefactor.formatted(show_namespaces=True)).encode(), filename=f"chart-{chart.source_index}-term-{term_index}-prefactor.txt", label="Download exact prefactor"),
        ])}),
    ])


def _statistics(mo, kernels, index):
    statistics = getattr(kernels, "sector_statistics", ())
    if index >= len(statistics):
        return mo.md("Compile with the updated bindings to retain evaluator size statistics.")
    stats = statistics[index]
    return mo.vstack([
        _static_table(mo, [{"backend": stats.backend, "arithmetic": stats.arithmetic,
                    "inputs": stats.inputs, "shared outputs": stats.outputs,
                    "exact program bytes": stats.exact_program_bytes,
                    "SymJIT application bytes": stats.symjit_ir_bytes}]),
        _static_table(mo, [{key: getattr(stats.operations, key) for key in ("additions", "multiplications", "inversions", "function_calls")}]),
        mo.md("Counts belong to the actual shared complete-vector evaluator after native Horner/CPE optimization and before backend lowering. Complex outputs split into real/imaginary components during numerical evaluation. No per-coefficient compiled cost is inferred from shared expressions."),
    ])


def detail(mo, generated, index, coefficient_index=0, alias_page=0, chart_index=0, term_index=0, kernels=None):
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
        _static_table(mo, aliases),
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
        images = [{"source parameter": _formula(mo, source), "native image": _formula(mo, image)}
                  for source, image in zip(coordinates.source_parameters, coordinates.images)]
        chart_view = mo.vstack([
            mo.md(f"**Source chart {chart.source_index}** · chart selection {chart_index} / {len(matching)-1}"),
            _static_table(mo, [{"representative": chart.representative,
                        "representative permutation": str(chart.representative_permutation),
                        "source domain": coordinates.source_domain,
                        "gauge-fixed parameter": str(coordinates.projective_fixed_parameter)}]),
            _static_table(mo, images),
            mo.md("Positive real measure Jacobian:"),
            _formula(mo, coordinates.measure_jacobian),
            _pre_subtraction(mo, chart, term_index),
            _geometry(mo, chart.geometry),
        ])
    return panel(mo, f"Sector {sector.index} · {sector.dimension} coordinates", mo.vstack([
        mo.md("Maps describe the density pullback **before endpoint subtraction**. Generated coefficients may be complex; compiled results can split real and imaginary components."),
        _static_table(mo, [{"native parameter IDs": ", ".join(_compact(value) for value in sector.parameters),
                    "conditioning basis": sector.conditioning_basis,
                    "cancellation degree": sector.cancellation_degree,
                    "conditioning rows": str(sector.cancellation_terms)}]),
        mo.accordion({"Actual evaluator size": _statistics(mo, kernels, index),
                      "Selected compact coefficient": coefficient_view,
                      "Selected source chart": chart_view,
                      "Sector geometry": _geometry(mo, sector.map)}),
    ]))
