use std::collections::{BTreeMap, BTreeSet};

use numerica::domains::float::{Float, Real, RealLike};

use super::*;
use crate::diagnostics::{
    AxisEndpoint, BoundaryProbe, BoundaryReport, BoundarySide, DiagnosticError, Result,
    boundary::{face_count, scaled_distances},
};

type Families = BTreeMap<(usize, Vec<AxisEndpoint>), BTreeMap<i32, usize>>;

fn validate(samples: &BoundaryReport, options: &BoundaryGrowthOptions) -> Result<Families> {
    options.threshold(samples.options.max_codimension)?;
    scaled_distances(&samples.options, samples.distance_scale)?;
    if samples.options.exponents.len() < 2 {
        return Err(DiagnosticError::Invalid(
            "growth analysis needs at least two distinct distances",
        ));
    }
    if samples.orders.is_empty() || samples.orders.len() != samples.components.len() {
        return Err(DiagnosticError::Invalid(
            "boundary report has no consistent coefficient layout",
        ));
    }
    let keys = samples
        .orders
        .iter()
        .zip(&samples.components)
        .collect::<BTreeSet<_>>();
    if keys.len() != samples.orders.len() {
        return Err(DiagnosticError::Invalid(
            "boundary coefficient keys are duplicated",
        ));
    }
    let sectors = samples
        .sectors
        .iter()
        .map(|sector| (sector.sector, sector.dimension))
        .collect::<BTreeMap<_, _>>();
    if sectors.len() != samples.sectors.len() {
        return Err(DiagnosticError::Invalid(
            "duplicate boundary sector metadata",
        ));
    }
    let mut families = Families::new();
    for (index, probe) in samples.probes.iter().enumerate() {
        let Some(&dimension) = sectors.get(&probe.sector) else {
            return Err(DiagnosticError::Invalid(
                "boundary probe has an unknown sector",
            ));
        };
        if probe.point.len() != dimension
            || probe
                .point
                .iter()
                .any(|x| !x.is_finite() || *x <= 0.0 || *x >= 1.0)
            || probe.endpoints.is_empty()
            || probe.endpoints.len() > samples.options.max_codimension
            || probe.endpoints.iter().any(|e| e.axis >= dimension)
            || probe.endpoints.windows(2).any(|p| p[0].axis >= p[1].axis)
            || !samples.options.exponents.contains(&probe.exponent)
            || !probe.distance.is_finite()
            || probe.distance <= 0.0
            || probe.distance >= 1.0
        {
            return Err(DiagnosticError::Invalid(
                "invalid boundary probe coordinates or face",
            ));
        }
        if let Some(values) = &probe.values
            && (!probe.finite
                || values.len() != samples.orders.len()
                || values.iter().any(|value| !value.is_finite()))
        {
            return Err(DiagnosticError::Invalid(
                "boundary physical values are nonfinite or have an inconsistent layout",
            ));
        }
        if families
            .entry((probe.sector, probe.endpoints.clone()))
            .or_default()
            .insert(probe.exponent, index)
            .is_some()
        {
            return Err(DiagnosticError::Invalid(
                "duplicate boundary face-distance sample",
            ));
        }
    }
    Ok(families)
}

fn log_distance_ratio(
    far: &BoundaryProbe,
    near: &BoundaryProbe,
) -> std::result::Result<Float, BoundaryGrowthUnavailable> {
    let mut logarithm = Float::with_val(128, 0);
    for (axis, (&left, &right)) in far.point.iter().zip(&near.point).enumerate() {
        let endpoint = far.endpoints.iter().find(|e| e.axis == axis);
        if let Some(endpoint) = endpoint {
            let distance = |x: f64| match endpoint.side {
                BoundarySide::Lower => x,
                BoundarySide::Upper => 1.0 - x,
            };
            let (left, right) = (distance(left), distance(right));
            if !(0.0 < right && right < left && left < 1.0) {
                return Err(BoundaryGrowthUnavailable::InvalidDistance);
            }
            logarithm += Float::with_val(128, left).log() - Float::with_val(128, right).log();
        } else if left != right {
            return Err(BoundaryGrowthUnavailable::InvalidDistance);
        }
    }
    logarithm /= Float::with_val(128, far.endpoints.len());
    let value = logarithm.to_f64();
    if !value.is_finite() || value <= 0.0 {
        Err(BoundaryGrowthUnavailable::NumericalRange)
    } else {
        Ok(logarithm)
    }
}

fn magnitude_growth(far: f64, near: f64, log_distance: &Float) -> BoundaryGrowthEstimate {
    if far == 0.0 && near == 0.0 {
        return BoundaryGrowthEstimate::BothZero;
    }
    if near == 0.0 {
        return BoundaryGrowthEstimate::DecreasesToZero;
    }
    if far == 0.0 {
        return BoundaryGrowthEstimate::EmergesFromZero;
    }
    if near <= far {
        return BoundaryGrowthEstimate::Power(0.0);
    }
    let value = ((Float::with_val(128, near).log() - Float::with_val(128, far).log())
        / log_distance)
        .to_f64();
    if value.is_finite() && value >= 0.0 {
        BoundaryGrowthEstimate::Power(value)
    } else {
        BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::NumericalRange)
    }
}

fn pair(
    samples: &BoundaryReport,
    sector: usize,
    endpoints: &[AxisEndpoint],
    component: usize,
    far_index: Option<usize>,
    near_index: Option<usize>,
    threshold: f64,
) -> BoundaryGrowthPair {
    let far = far_index.map(|index| &samples.probes[index]);
    let near = near_index.map(|index| &samples.probes[index]);
    let magnitude = |probe: Option<&BoundaryProbe>| {
        probe.and_then(|probe| probe.values.as_ref().map(|values| values[component].abs()))
    };
    let (farther_magnitude, nearer_magnitude) = (magnitude(far), magnitude(near));
    let mut mean_log_distance_ratio = None;
    let estimate = match (far, near) {
        (Some(far), Some(near)) => {
            if !far.finite || !near.finite {
                BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::EvaluationFailed)
            } else if let (Some(left), Some(right)) = (farther_magnitude, nearer_magnitude) {
                match log_distance_ratio(far, near) {
                    Ok(ratio) => {
                        mean_log_distance_ratio = Some(ratio.to_f64());
                        magnitude_growth(left, right, &ratio)
                    }
                    Err(reason) => BoundaryGrowthEstimate::Unavailable(reason),
                }
            } else {
                BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::MissingValues)
            }
        }
        _ => BoundaryGrowthEstimate::Unavailable(BoundaryGrowthUnavailable::MissingProbe),
    };
    let assessment = match estimate {
        BoundaryGrowthEstimate::Power(value) if value > threshold => BoundaryAssessment::Flagged,
        BoundaryGrowthEstimate::Power(_)
        | BoundaryGrowthEstimate::BothZero
        | BoundaryGrowthEstimate::DecreasesToZero => BoundaryAssessment::WithinThreshold,
        BoundaryGrowthEstimate::EmergesFromZero | BoundaryGrowthEstimate::Unavailable(_) => {
            BoundaryAssessment::Inconclusive
        }
    };
    BoundaryGrowthPair {
        sector,
        endpoints: endpoints.to_vec(),
        component_index: component,
        farther_probe: far_index,
        nearer_probe: near_index,
        farther_magnitude,
        nearer_magnitude,
        mean_log_distance_ratio,
        estimate,
        threshold,
        assessment,
    }
}

pub(super) fn aggregate(
    assessments: impl IntoIterator<Item = BoundaryAssessment>,
) -> BoundaryAssessment {
    let mut result = BoundaryAssessment::NotApplicable;
    for assessment in assessments {
        result = match (result, assessment) {
            (BoundaryAssessment::Flagged, _) | (_, BoundaryAssessment::Flagged) => {
                BoundaryAssessment::Flagged
            }
            (BoundaryAssessment::Inconclusive, _) | (_, BoundaryAssessment::Inconclusive) => {
                BoundaryAssessment::Inconclusive
            }
            (BoundaryAssessment::WithinThreshold, _) | (_, BoundaryAssessment::WithinThreshold) => {
                BoundaryAssessment::WithinThreshold
            }
            _ => BoundaryAssessment::NotApplicable,
        };
    }
    result
}

/// Compare adjacent requested distances for each physical numerical component.
/// Missing intermediate observations are never bridged. A policy flag is not a
/// mathematical divergence claim, and absence of a flag is not a certificate.
pub fn analyze_boundary_growth(
    samples: &BoundaryReport,
    options: &BoundaryGrowthOptions,
) -> Result<BoundaryGrowthReport> {
    let families = validate(samples, options)?;
    let mut exponents = samples.options.exponents.clone();
    exponents.sort_unstable();
    let mut pairs = Vec::new();
    for ((sector, endpoints), observations) in &families {
        let threshold = options.threshold(endpoints.len())?;
        for adjacent in exponents.windows(2) {
            let far = observations.get(&adjacent[0]).copied();
            let near = observations.get(&adjacent[1]).copied();
            if far.is_none() && near.is_none() {
                continue;
            }
            for component in 0..samples.orders.len() {
                pairs.push(pair(
                    samples, *sector, endpoints, component, far, near, threshold,
                ));
            }
        }
    }
    let sectors = samples
        .sectors
        .iter()
        .map(|sector| {
            let faces = face_count(sector.dimension, samples.options.max_codimension);
            let expected_samples =
                faces.and_then(|count| count.checked_mul(exponents.len() as u64));
            let expected_pairs = faces
                .and_then(|count| count.checked_mul((exponents.len() - 1) as u64))
                .and_then(|count| count.checked_mul(samples.orders.len() as u64));
            let observed_samples = samples
                .probes
                .iter()
                .filter(|p| p.sector == sector.sector)
                .count();
            let rows = pairs.iter().filter(|pair| pair.sector == sector.sector);
            let observed_pairs = rows.clone().count();
            let flagged_pairs = rows
                .clone()
                .filter(|p| p.assessment == BoundaryAssessment::Flagged)
                .count();
            let inconclusive_pairs = rows
                .filter(|p| p.assessment == BoundaryAssessment::Inconclusive)
                .count();
            let sampling_complete = expected_samples == Some(observed_samples as u64);
            let assessment = if flagged_pairs > 0 {
                BoundaryAssessment::Flagged
            } else if !sampling_complete
                || expected_pairs != Some(observed_pairs as u64)
                || inconclusive_pairs > 0
            {
                BoundaryAssessment::Inconclusive
            } else if observed_pairs == 0 {
                BoundaryAssessment::NotApplicable
            } else {
                BoundaryAssessment::WithinThreshold
            };
            BoundarySectorAssessment {
                sector: sector.sector,
                sampling_complete,
                expected_pairs,
                observed_pairs,
                flagged_pairs,
                inconclusive_pairs,
                assessment,
            }
        })
        .collect::<Vec<_>>();
    Ok(BoundaryGrowthReport {
        options: options.clone(),
        orders: samples.orders.clone(),
        components: samples.components.clone(),
        pairs,
        assessment: aggregate(sectors.iter().map(|s| s.assessment)),
        sectors,
    })
}
