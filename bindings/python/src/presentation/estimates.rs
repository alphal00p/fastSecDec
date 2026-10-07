//! Rich views copy native estimates; no statistical reduction is implemented here.
use super::{card, escape, facts, table, value};
use pyo3::prelude::*;

fn number(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else {
        format!("{value:.6e}")
    }
}

fn coefficients(owner: &Bound<'_, PyAny>, schema: Option<&Bound<'_, PyAny>>) -> PyResult<String> {
    let means = owner.getattr("mean")?.extract::<Option<Vec<f64>>>()?;
    let errors = owner
        .getattr("standard_error")?
        .extract::<Option<Vec<f64>>>()?;
    let Some(means) = means else {
        return Ok("<p style=\"opacity:.7\">Waiting for native mean coverage; no zero estimate is substituted.</p>".into());
    };
    let labels = if let Some(schema) = schema {
        let orders = schema.getattr("orders")?.extract::<Vec<i32>>()?;
        let components = schema.getattr("components")?.extract::<Vec<String>>()?;
        orders
            .into_iter()
            .zip(components)
            .map(|(order, component)| format!("ε order {order} · {component}"))
            .collect::<Vec<_>>()
    } else {
        (0..means.len())
            .map(|i| format!("Native output {i}"))
            .collect()
    };
    let rows = means
        .iter()
        .enumerate()
        .map(|(i, mean)| {
            vec![
                escape(labels.get(i).map_or("Unspecified output", String::as_str)),
                number(*mean),
                errors
                    .as_ref()
                    .and_then(|v| v.get(i))
                    .map_or_else(|| "Unavailable".into(), |v| number(*v)),
            ]
        })
        .collect();
    Ok(table(&["Coefficient", "Mean", "Standard error"], rows))
}

fn estimate(owner: &Bound<'_, PyAny>) -> PyResult<String> {
    if owner.is_none() {
        return Ok(
            "<p style=\"opacity:.7\">No native estimate is available at this coverage.</p>".into(),
        );
    }
    let mut body = coefficients(owner, Some(owner))?;
    let dimension = owner.getattr("orders")?.len()?;
    let covariance = owner.getattr("covariance_of_mean")?.extract::<Vec<f64>>()?;
    if dimension > 0 && dimension <= 12 {
        let columns = (0..dimension)
            .map(|i| format!("Output {i}"))
            .collect::<Vec<_>>();
        let headers = columns.iter().map(String::as_str).collect::<Vec<_>>();
        let rows = covariance
            .chunks(dimension)
            .map(|row| row.iter().map(|v| number(*v)).collect())
            .collect();
        body.push_str(&format!(
            "<details><summary>Full native covariance of the mean</summary>{}</details>",
            table(&headers, rows)
        ));
    } else if dimension > 12 {
        body.push_str(&format!("<p>Full covariance retained: {dimension} × {dimension}; available through covariance_of_mean.</p>"));
    }
    Ok(body)
}

fn snapshot_facts(owner: &Bound<'_, PyAny>) -> PyResult<String> {
    facts(
        owner,
        &[
            ("Method", "method"),
            ("Stage", "stage"),
            ("Accepted points", "completed_points"),
            ("Planned points", "planned_points"),
            ("Uncertainty", "uncertainty"),
            ("Worker seconds", "worker_seconds"),
        ],
    )
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::integration::PyVectorEstimate {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        Ok(card("Native Laurent vector", estimate(slf.as_any())?))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::integration::PyIntegrationSnapshot {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        Ok(card(
            "Integration snapshot",
            snapshot_facts(owner)? + estimate(&owner.getattr("estimate")?)?.as_str(),
        ))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::observation::PyIntegrationObservation {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        let mut body = snapshot_facts(&owner.getattr("snapshot")?)?;
        body.push_str("<h4>Full integral · native accepted estimate</h4>");
        body.push_str(&estimate(&owner.getattr("total")?)?);
        let sectors = owner.getattr("sectors")?;
        let mut rows = Vec::new();
        for sector in sectors.try_iter()?.take(10) {
            let sector = sector?;
            rows.push(vec![
                value(&sector.getattr("id")?)?,
                value(&sector.getattr("completed_points")?)?,
                value(&sector.getattr("used_replicas")?)?,
                estimate(&sector.getattr("estimate")?)?,
            ]);
        }
        body.push_str(&format!(
            "<details><summary>Sector contributions · showing {} of {}</summary>{}</details>",
            rows.len(),
            sectors.len()?,
            table(
                &[
                    "Sector ID",
                    "Accepted points",
                    "Used replicas",
                    "Native vector"
                ],
                rows
            )
        ));
        body.push_str("<p style=\"opacity:.7\">The full total uses native covariance and exact offsets. Sector errors must not be added independently.</p>");
        Ok(card("Integration result", body))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::observation::PySectorContribution {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        Ok(card(
            "Native sector contribution",
            facts(
                owner,
                &[
                    ("Sector ID", "id"),
                    ("Accepted points", "completed_points"),
                    ("Used points", "used_points"),
                    ("Used replicas", "used_replicas"),
                    ("Uncertainty", "uncertainty"),
                ],
            )? + estimate(&owner.getattr("estimate")?)?.as_str(),
        ))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::observation::PyLiveObservation {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        let total = owner.getattr("total")?;
        let mut body = facts(owner, &[("Source", "source"), ("Stage", "stage")])?;
        body.push_str(&facts(
            &total,
            &[
                ("Native status", "status"),
                ("Observed points", "points"),
                ("Complete replicas", "replicas"),
            ],
        )?);
        body.push_str(&coefficients(&total, Some(owner))?);
        body.push_str("<p style=\"opacity:.7\">Provisional observation only; never checkpointed or used for stopping. QMC means require complete lattices, and errors require independent shifts.</p>");
        Ok(card("Live numerical observation", body))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::observation::PyLiveEstimate {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        let owner = slf.as_any();
        Ok(card(
            "Native provisional vector",
            facts(
                owner,
                &[
                    ("Status", "status"),
                    ("Points", "points"),
                    ("Replicas", "replicas"),
                ],
            )? + coefficients(owner, None)?.as_str(),
        ))
    }
}

#[cfg_attr(feature = "python_stubgen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl crate::status::observation::PyLiveSector {
    fn _repr_html_(slf: &Bound<'_, Self>) -> PyResult<String> {
        Ok(card(
            "Native provisional sector",
            facts(
                slf.as_any(),
                &[("Sector ID", "id"), ("Estimate", "estimate")],
            )?,
        ))
    }
}
