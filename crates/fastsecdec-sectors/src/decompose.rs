use crate::{
    stages::{decompose_candidate, prepare_chart, trivial_map, validate_input},
    types::Monitor,
    *,
};
use std::ops::ControlFlow;

pub(crate) fn initial_status(chart: usize, sectors: usize) -> DecompositionProgress {
    DecompositionProgress {
        phase: DecompositionPhase::Supports,
        chart,
        completed_constraints: 0,
        total_constraints: 0,
        rays: 0,
        sectors,
    }
}

pub(crate) fn add_count(
    total: &mut usize,
    count: usize,
    resource: &'static str,
) -> Result<(), SectorError> {
    *total = total.checked_add(count).ok_or(SectorError::ResourceLimit {
        resource,
        limit: usize::MAX,
    })?;
    Ok(())
}

pub fn decompose(
    domain: ParametricDomain,
    supports: &[PolynomialSupport],
    options: &DecompositionOptions,
    mut progress: impl FnMut(&DecompositionProgress) -> ControlFlow<()>,
) -> Result<Decomposition, SectorError> {
    let dim = validate_input(domain, supports)?;
    let mut monitor = Monitor {
        callback: &mut progress,
        options,
        status: initial_status(0, 0),
    };
    let mut output = Decomposition {
        domain,
        sectors: Vec::new(),
        candidate_vertices: 0,
        geometric_vertices: 0,
    };
    let fixed_axes: Vec<_> = if domain == ParametricDomain::ProjectiveSimplex {
        (0..dim).map(Some).collect()
    } else {
        vec![None]
    };
    for fixed in fixed_axes {
        monitor.status = initial_status(fixed.unwrap_or(0), output.sectors.len());
        let supports = supports
            .iter()
            .map(|s| fixed.map_or_else(|| s.clone(), |i| s.without_axis(i)))
            .collect();
        let chart = prepare_chart(
            supports,
            fixed,
            domain != ParametricDomain::PositiveOrthant,
            &mut monitor,
        )?;
        if chart.supports[0].dimension() == 0 {
            if output.sectors.len() >= options.max_sectors {
                return Err(SectorError::ResourceLimit {
                    resource: "sectors",
                    limit: options.max_sectors,
                });
            }
            output.sectors.push(trivial_map(&chart));
            continue;
        }
        add_count(
            &mut output.candidate_vertices,
            chart.candidates.len(),
            "candidate vertices",
        )?;
        for candidate in 0..chart.candidates.len() {
            let geometric =
                decompose_candidate(&chart, candidate, &mut output.sectors, &mut monitor)?;
            add_count(
                &mut output.geometric_vertices,
                usize::from(geometric),
                "geometric vertices",
            )?;
        }
        if output.sectors.is_empty() {
            return Err(SectorError::Geometry(
                "normal fan produced no full-dimensional sectors".into(),
            ));
        }
    }
    monitor.status.phase = DecompositionPhase::Complete;
    monitor.status.sectors = output.sectors.len();
    monitor.emit()?;
    Ok(output)
}
