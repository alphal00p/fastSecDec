//! Bounded read-only notebook views of existing native owners and getters.
//!
//! Rendering never generates, compiles, expands aliases, binds a point or samples.
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyInt, PyList, PyTuple},
};

mod estimates;

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn card(title: &str, content: String) -> String {
    format!(
        "<section class=\"fsd-native\" style=\"border:1px solid #7d8fa555;border-radius:12px;padding:1rem;margin:.4rem 0;overflow:auto;color:inherit\"><h3 style=\"margin:0 0 .8rem;color:#5785ca\">{}</h3>{content}</section>",
        escape(title)
    )
}

fn table(headers: &[&str], rows: Vec<Vec<String>>) -> String {
    let head = headers.iter().map(|s|format!("<th style=\"text-align:left;padding:.4rem .8rem;border-bottom:1px solid #7d8fa555\">{}</th>",escape(s))).collect::<String>();
    let body = rows.into_iter().map(|row|format!("<tr>{}</tr>",row.into_iter().map(|cell|format!("<td style=\"padding:.35rem .8rem;vertical-align:top;overflow-wrap:anywhere\">{cell}</td>")).collect::<String>())).collect::<String>();
    format!(
        "<table style=\"border-collapse:collapse;width:100%;font-variant-numeric:tabular-nums\"><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>"
    )
}

fn value(owner: &Bound<'_, PyAny>) -> PyResult<String> {
    if owner.is_none() {
        return Ok("<span style=\"opacity:.6\">Unavailable</span>".into());
    }
    // Only Symbolica expressions have this bounded native formatting protocol.
    if owner.is_instance_of::<symbolica::api::python::PythonExpression>() {
        let kwargs = PyDict::new(owner.py());
        kwargs.set_item("max_terms", 12)?;
        kwargs.set_item("max_line_length", 80)?;
        kwargs.set_item("show_namespaces", true)?;
        return owner
            .call_method("formatted", (), Some(&kwargs))?
            .call_method0("_repr_html_")?
            .extract();
    }
    if owner.is_instance_of::<PyList>() || owner.is_instance_of::<PyTuple>() {
        let count = owner.len()?;
        let mut values = Vec::new();
        for item in owner.try_iter()?.take(8) {
            values.push(value(&item?)?);
        }
        if count > 8 {
            values.push(format!(
                "<span style=\"opacity:.6\">… {} more</span>",
                count - 8
            ));
        }
        return Ok(values.join(" · "));
    }
    if owner.is_instance_of::<PyInt>() || owner.is_instance_of::<PyBool>() {
        return Ok(escape(&owner.str()?.extract::<String>()?));
    }
    if let Ok(number) = owner.extract::<f64>() {
        return Ok(if !number.is_finite() {
            escape(&number.to_string())
        } else if number != 0.0 && (number.abs() < 1e-4 || number.abs() >= 1e7) {
            format!("{number:.6e}")
        } else {
            format!("{number:.7}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        });
    }
    if owner.hasattr("_repr_html_")? {
        return owner.call_method0("_repr_html_")?.extract();
    }
    Ok(escape(&owner.str()?.extract::<String>()?))
}

fn facts(owner: &Bound<'_, PyAny>, fields: &[(&str, &str)]) -> PyResult<String> {
    let mut rows = Vec::new();
    for (label, attribute) in fields {
        let (count, attribute) = attribute
            .strip_prefix('#')
            .map_or((false, *attribute), |name| (true, name));
        let field = owner.getattr(attribute)?;
        let text = if field.is_none() && attribute == "cpe_rounds" {
            "Unlimited".to_string()
        } else if field.is_none() && attribute == "failed" {
            "None".to_string()
        } else if count {
            field.len()?.to_string()
        } else {
            value(&field)?
        };
        rows.push(vec![escape(label), text]);
    }
    Ok(table(&["Property", "Native value"], rows))
}

macro_rules! facts_view {
    ($owner:path, $title:literal, [$($label:literal => $attribute:literal),* $(,)?]) => {
        #[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[pymethods]
        impl $owner {
            fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
                Ok(card($title,facts(slf.as_any(),&[$(($label,$attribute)),*])?))
            }
        }
    };
}

facts_view!(crate::input::PyIntegral,"Native integral",["Spacetime dimension"=>"dimension","Regulator"=>"regulator","Explicit edge powers"=>"powers"]);
facts_view!(crate::generation::PyGeneratedIntegral,"Generated integral",["Generation mode"=>"mode","Subtraction"=>"subtraction","Numerical sectors"=>"sector_count","Laurent orders"=>"orders","Runtime inputs"=>"#runtime_parameters","Exact contribution"=>"exact_coefficients"]);
facts_view!(crate::generation_session::PyGenerationSession,"Retained generation",["Generation mode"=>"mode","Subtraction"=>"subtraction","Complete"=>"complete","Failure"=>"failed"]);
facts_view!(crate::kernels::PyKernels,"Native evaluators",["Backend"=>"backend","Sectors"=>"sector_count","Laurent orders"=>"orders","Components"=>"components","Runtime inputs"=>"runtime_parameters","Point bound"=>"parameters_bound","Compiler settings"=>"compilation_settings"]);
facts_view!(crate::session::PyQmcSettings,"Shifted-lattice allocation",["Points per lattice"=>"points","Independent shifts"=>"shifts","Seed"=>"seed","Package points"=>"package_points","Rule"=>"rule","Periodization"=>"periodization"]);
facts_view!(crate::session::PyQmcSession,"Retained QMC session",["Allocation complete"=>"complete","Settings"=>"settings"]);
facts_view!(crate::mc::PyHavanaDiscreteSettings,"Havana allocation",["Global points per batch"=>"points_per_batch","Global batches"=>"batches","Seed"=>"seed","Bins"=>"bins","Minimum density"=>"minimum_probability_density","Maximum sector ratio"=>"maximum_sector_probability_ratio"]);
facts_view!(crate::mc::PyHavanaDiscreteSession,"Retained Havana session",["Stage"=>"stage","Allocation complete"=>"complete","Checkpoint available"=>"checkpoint_available","Settings"=>"settings"]);

facts_view!(crate::status::generation::PyGenerationSnapshot,"Generation progress",["Stage"=>"stage","Completed units"=>"completed","Planned units"=>"total","Sectors"=>"sectors","Kernels"=>"kernels","Elapsed seconds"=>"elapsed_seconds","Activity"=>"detail"]);
facts_view!(crate::status::generation::PyGenerationTimings,"Native generation timings",["Input seconds"=>"input_seconds","Parametrization seconds"=>"parametrization_seconds","Domain metadata seconds"=>"domain_seconds","Geometry seconds"=>"geometry_seconds","Mapping seconds"=>"mapping_seconds","Equivalent sectors seconds"=>"symmetry_seconds","Coefficient expansion seconds"=>"coefficient_expansion_seconds","Compilation seconds"=>"compilation_seconds","Total seconds"=>"total_seconds"]);
facts_view!(crate::status::generation::PyCoefficientExpansionSnapshot,"Coefficient expansion",["Sector"=>"sector","Stage"=>"stage","Requested method"=>"requested_method","Effective method"=>"effective_method","Expansion pass"=>"attempt","Relative depth"=>"relative_width"]);
facts_view!(crate::status::generation::PyCoefficientRequestCounts,"Native request counters",["Source bodies"=>"source_bodies","Distinct requests"=>"unique_requests","Cached partials"=>"cached_partials","Aliases"=>"aliases","Interleaved"=>"interleaved_requests","Fallback"=>"fallback_requests"]);
facts_view!(crate::status::integration::PySectorSnapshot,"Sector coverage",["Sector ID"=>"id","Coordinates"=>"dimension","Accepted points"=>"completed_points","Planned points"=>"planned_points","Complete replicas"=>"complete_replicas","Planned replicas"=>"planned_replicas","Worker seconds"=>"worker_seconds"]);
facts_view!(crate::status::diagnostics::PyEvaluationDiagnostics,"Native evaluation diagnostics",["Assessed points"=>"evaluations","f64"=>"f64_points","DoubleFloat"=>"double_float_points","Arbitrary precision"=>"arbitrary_points","Unstable"=>"unstable_points","Cutoff zero"=>"cutoff_zero_points","Rescues"=>"rescues","Failures"=>"failures"]);
facts_view!(crate::status::diagnostics::PyEvaluatorTiming,"Evaluator timing",["Evaluated rows"=>"calls","Nanoseconds"=>"nanoseconds","Native matrix calls"=>"matrix_invocations","Matrix rows"=>"matrix_points"]);
facts_view!(crate::status::allocation::PyDiscreteSectorAllocation,"Native sector proposal",["Selection probability"=>"probability","Global points per batch"=>"points_per_batch"]);

facts_view!(crate::inspection::metadata::PyGenerationMetadata,"Retained generation metadata",["Source charts"=>"#charts","Domain"=>"domain"]);
facts_view!(crate::inspection::metadata::PyChart,"Retained source chart",["Source chart ID"=>"source_index","Representative chart"=>"representative","Representative permutation"=>"representative_permutation","Kernel sector ID"=>"kernel_sector","Coordinates"=>"coordinates"]);
facts_view!(crate::inspection::domain::PyDomainAssessment,"Declared integration domain",["Domain"=>"domain","Branch policy"=>"branch_policy","Coordinates"=>"parameters"]);
facts_view!(crate::inspection::domain::PyFactorAssessment,"Retained input factor",["Term"=>"term_index","Factor"=>"factor_index","Polynomial"=>"polynomial","Exponent"=>"exponent"]);
facts_view!(crate::inspection::geometry::PySectorMap,"Native sector geometry",["Source dimension"=>"source_dimension","Target dimension"=>"dimension","Fixed parameter"=>"fixed_parameter","Determinant"=>"determinant","Exponent matrix"=>"exponent_matrix","Jacobian powers"=>"jacobian_powers"]);
facts_view!(crate::inspection::sector::PyGeneratedSector,"Generated numerical sector",["Sector ID"=>"index","Generation mode"=>"generation_mode","Coordinates"=>"dimension","Parameters"=>"parameters","Laurent coefficients"=>"coefficient_count","Stored aliases"=>"alias_counts","Conditioning provenance"=>"conditioning_basis","Cancellation degree"=>"cancellation_degree"]);
facts_view!(crate::inspection::coefficient::PyCompactCoefficient,"Compact Laurent coefficient",["Epsilon order"=>"order","Generation mode"=>"generation_mode","Native root"=>"root","Stored aliases"=>"alias_count"]);
facts_view!(crate::inspection::pre_subtraction::PyPreSubtractionMetadata,"Pre-subtraction metadata",["Version"=>"version","Regulator"=>"regulator","Mapped terms"=>"#terms"]);
facts_view!(crate::inspection::pre_subtraction::PyPreSubtractionTerm,"Retained pre-subtraction term",["Prefactor"=>"prefactor","Endpoint powers"=>"powers","Regular body basis"=>"regular_expression_basis","Regular body storage (bytes)"=>"regular_expression_bytes"]);
facts_view!(crate::inspection::pre_subtraction::PyEndpointPower,"Native endpoint power",["Coordinate"=>"parameter","Exponent"=>"exponent","Constant part"=>"constant","Epsilon slope"=>"slope","Taylor coefficients required"=>"subtraction_count"]);
facts_view!(crate::inspection::statistics::PyEvaluatorStatistics,"Shared native evaluator",["Backend"=>"backend","Arithmetic"=>"arithmetic","Inputs"=>"inputs","Native outputs"=>"outputs","Exact program bytes"=>"exact_program_bytes","Operations after native optimization"=>"operations"]);
facts_view!(crate::inspection::statistics::PyEvaluatorOperations,"Native operation counts",["Additions"=>"additions","Multiplications"=>"multiplications","Inversions"=>"inversions","Functions"=>"function_calls"]);

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::inspection::geometry::PyCoordinateMap {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        let mut rows = Vec::new();
        for pair in owner
            .getattr("source_parameters")?
            .try_iter()?
            .zip(owner.getattr("images")?.try_iter()?)
        {
            rows.push(vec![value(&pair.0?)?, value(&pair.1?)?]);
        }
        let mut body = facts(
            owner,
            &[
                ("Source domain", "source_domain"),
                ("Gauge-fixed parameter", "projective_fixed_parameter"),
                ("Positive measure", "measure_jacobian"),
            ],
        )?;
        body.push_str(&table(&["Source coordinate", "Native image"], rows));
        body.push_str("<p style=\"opacity:.7\">Density pullback before endpoint subtraction. The measure is positive; projective coordinates are gauge-fixed.</p>");
        Ok(card("Native coordinate map", body))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::settings::PyCompilationSettings {
    fn _repr_html_(&self) -> String {
        let settings = self.inner;
        let rows = [
            ("Backend", format!("{:?}", settings.backend)),
            ("Horner iterations", settings.horner_iterations.to_string()),
            (
                "CPE rounds",
                settings
                    .cpe_rounds
                    .map_or_else(|| "Unlimited".into(), |n| n.to_string()),
            ),
            ("Native cores", settings.cores.to_string()),
            (
                "Horner variable limit",
                settings.max_horner_scheme_variables.to_string(),
            ),
            (
                "Common-pair cache entries",
                settings.max_common_pair_cache_entries.to_string(),
            ),
            (
                "Common-pair distance (stored native option)",
                settings.max_common_pair_distance.to_string(),
            ),
            (
                "Direct translation",
                settings.direct_translation.to_string(),
            ),
            ("Verbose native logging", settings.verbose.to_string()),
        ]
        .into_iter()
        .map(|(label, value)| vec![escape(label), escape(&value)])
        .collect();
        card("Evaluator compilation", table(&["Setting", "Value"], rows))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::settings::PyStabilitySettings {
    fn _repr_html_(&self) -> String {
        let settings = &self.inner;
        if settings.mode == fastsecdec::kernel::StabilityMode::Validated {
            return card("Numerical stability", "<p>Native validated replay policy. Distance thresholds and the distance cutoff are inactive.</p>".into());
        }
        let rows = settings
            .levels
            .iter()
            .map(|level| {
                vec![
                    escape(&format!("{:?}", level.precision)),
                    level.minimum_effective_distance.to_string(),
                    level
                        .escalate_for_large_weight_threshold
                        .map_or_else(|| "Disabled".into(), |v| v.to_string()),
                ]
            })
            .collect();
        card(
            "Numerical stability",
            format!(
                "<p>Policy: <b>{:?}</b> · explicit zero cutoff: {}</p>{}",
                settings.mode,
                settings
                    .unstable_cutoff
                    .map_or_else(|| "Disabled".into(), |v| v.to_string()),
                table(
                    &[
                        "Native precision",
                        "Minimum effective distance",
                        "Previous-maximum fraction"
                    ],
                    rows
                )
            ),
        )
    }
}
