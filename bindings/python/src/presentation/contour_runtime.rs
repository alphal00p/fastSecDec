//! Bounded, inert presentation of copied native operational reports.
use super::{card, escape, facts, table, value};
use pyo3::prelude::*;

fn work_table(report: &Bound<'_, PyAny>) -> PyResult<String> {
    let mut rows = Vec::with_capacity(5);
    for (label, attribute) in [
        ("Evaluation", "evaluation"),
        ("Conditioning", "conditioning"),
        ("Preparation", "preparation"),
        ("Exact contributions", "exact"),
        ("Validation pilot", "pilot"),
    ] {
        let work = report.getattr(attribute)?;
        let mut row = vec![escape(label)];
        for field in [
            "callback_calls",
            "solver_calls",
            "solver_iterations",
            "closed_form_calls",
            "callback_failures",
            "maximum_bits",
        ] {
            row.push(value(&work.getattr(field)?)?);
        }
        rows.push(row);
    }
    Ok(table(
        &[
            "Work category",
            "Callbacks",
            "Root solves",
            "Iterations",
            "Closed forms",
            "Callback failures",
            "Maximum bits",
        ],
        rows,
    ))
}

fn details(label: &str, content: &str) -> String {
    format!(
        "<details style=\"margin:.6rem 0\"><summary style=\"cursor:pointer;color:#5785ca\">{}</summary>{content}</details>",
        escape(label)
    )
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::contour::PyContourRuntimeDiagnostics {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let mut body = String::from(
            "<p style=\"opacity:.75\">Observed execution work, including retries and discarded batches. Counts are separate from accepted samples and causal validation.</p>",
        );
        for (label, attribute) in [("Production", "production"), ("Adaptation", "adaptation")] {
            let report = slf.getattr(attribute)?;
            body.push_str(&format!("<h4>{}</h4>", escape(label)));
            body.push_str(&work_table(&report)?);
            body.push_str(&details(
                "Strength, displacement and solver details",
                &value(&report.getattr("evaluation")?)?,
            ));
        }
        Ok(card("Contour work by integration phase", body))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::contour::PyContourRuntimeReport {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let mut body = work_table(slf.as_any())?;
        body.push_str(&details(
            "Evaluation strength, displacement and solver details",
            &value(&slf.getattr("evaluation")?)?,
        ));
        body.push_str("<p style=\"opacity:.75\">Operational observations include retries and discarded work; they are not accepted sample counts.</p>");
        Ok(card("Native contour observations", body))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::diagnostics::PyEvaluationDiagnostics {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let mut body = facts(
            slf.as_any(),
            &[
                ("Assessed points", "evaluations"),
                ("f64", "f64_points"),
                ("DoubleFloat", "double_float_points"),
                ("Arbitrary precision", "arbitrary_points"),
                ("Unstable", "unstable_points"),
                ("Cutoff zero", "cutoff_zero_points"),
                ("Rescues", "rescues"),
                ("Failures", "failures"),
            ],
        )?;
        let runtime = slf.getattr("contour_runtime")?;
        if runtime.is_none() {
            body.push_str("<p style=\"opacity:.65\">Contour work observations: not recorded.</p>");
        } else {
            body.push_str(&details("Contour work observations", &value(&runtime)?));
        }
        Ok(card("Native evaluation diagnostics", body))
    }
}

facts_view!(crate::contour::PyContourRuntimeWork,"Contour evaluation work",["Callbacks"=>"callback_calls","Callback failures"=>"callback_failures","Root solves"=>"solver_calls","Root failures"=>"solver_failures","Total root iterations"=>"solver_iterations","Maximum root iterations"=>"maximum_solver_iterations","Closed-form solutions"=>"closed_form_calls","Maximum precision (bits)"=>"maximum_bits","Strength λ"=>"strength","Normalized displacement"=>"normalized_displacement","Physical displacement"=>"physical_displacement"]);
facts_view!(crate::contour::PyContourDiagnosticRange,"Observed range (approximate centres)",["Available observations"=>"count","Unavailable observations"=>"unavailable","Minimum"=>"minimum","Maximum"=>"maximum"]);
