use super::*;
use symbolica::symbol;
#[derive(Clone, Debug)]
pub struct RecursiveBlowupChart {
    pivot_source_axis: usize,
    transform: VerifiedTransform,
    target_frame: Arc<EtaleFrame>,
    target_order: OrderProduction,
}
impl RecursiveBlowupChart {
    pub fn pivot_source_axis(&self) -> usize {
        self.pivot_source_axis
    }
    pub fn transform(&self) -> &VerifiedTransform {
        &self.transform
    }
    pub fn target_frame(&self) -> &Arc<EtaleFrame> {
        &self.target_frame
    }
    pub fn target_order(&self) -> &OrderProduction {
        &self.target_order
    }
    pub fn marked_cosupport_empty(&self) -> bool {
        match &self.target_order {
            OrderProduction::UnitIdeal { .. } => true,
            OrderProduction::ContactCover(c) => {
                c.algebraic_maximum_order() < self.transform.target().mark()
            }
            _ => false,
        }
    }
}
#[derive(Clone, Debug)]
pub enum FirstBlowup {
    MarkedResolved {
        center: Arc<RecursiveCenter>,
        extension: RingExtension,
        charts: Vec<RecursiveBlowupChart>,
    },
    FurtherCyclesRequired {
        center: Arc<RecursiveCenter>,
        extension: RingExtension,
        charts: Vec<RecursiveBlowupChart>,
    },
    GeneralAdaptationRequired {
        center: Arc<RecursiveCenter>,
    },
}
/// Coordinate admission is a checked optimized case. It does not silently
/// replace a nonlinear center, nor restart without its newborn boundary.
pub fn first_coordinate_blowup(
    center: Arc<RecursiveCenter>,
    namespace: &str,
    b: &mut Budget,
) -> Result<FirstBlowup> {
    let old = center.frame.local();
    let n = center.frame.free_axes().len();
    if !old.ideal().generators().is_empty()
        || !old.guards().is_empty()
        || !center.frame.dependent_axes().is_empty()
        || n == 0
    {
        return Ok(FirstBlowup::GeneralAdaptationRequired { center });
    }
    let mut normals = Vec::new();
    for i in center.frame.free_axes() {
        if center.ideal.contains(&old.ring().coordinate(*i)?, &[], b)? {
            normals.push(*i);
        }
    }
    if normals.is_empty() {
        return Ok(FirstBlowup::GeneralAdaptationRequired { center });
    }
    let coordinate_ideal = Ideal::new(
        old.ring().clone(),
        normals
            .iter()
            .map(|i| old.ring().coordinate(*i))
            .collect::<Result<_>>()?,
        b,
    )?;
    for f in center.ideal.generators() {
        if !coordinate_ideal.contains(f, &[], b)? {
            return Ok(FirstBlowup::GeneralAdaptationRequired { center });
        }
    }
    let count = n.checked_mul(2).ok_or(Error::ResourceIncomplete(
        "recursive coordinate chart dimensions",
    ))?;
    b.reserve_slots(count)?;
    let fresh = (0..count)
        .map(|i| symbol!(format!("{namespace}::c{i}")))
        .collect::<Vec<_>>();
    let extension = RingExtension::with_symbols(old.ring().clone(), &fresh, b)?;
    let ring = extension.target();
    let start = old.ring().len();
    let source_axes = center.frame.free_axes().to_vec();
    let adapted_axes = (start..start + n).collect::<Vec<_>>();
    let target_axes = (start + n..start + count).collect::<Vec<_>>();
    let witness = vec![Rational::zero(); ring.len()];
    let source_chart = Arc::new(Chart::new(
        ring.clone(),
        source_axes.clone(),
        vec![],
        witness.clone(),
        b,
    )?);
    let adapted_chart = Arc::new(Chart::new(
        ring.clone(),
        adapted_axes.clone(),
        vec![],
        witness.clone(),
        b,
    )?);
    let target_chart = Arc::new(Chart::new(
        ring.clone(),
        target_axes.clone(),
        vec![],
        witness,
        b,
    )?);
    let coords = |axes: &[usize]| {
        axes.iter()
            .map(|i| ring.coordinate(*i))
            .collect::<Result<Vec<_>>>()
    };
    let frame = FrameCertificate {
        pullback: Map::new(
            source_chart.clone(),
            adapted_chart.clone(),
            coords(&adapted_axes)?,
            b,
        )?,
        inverse: Map::new(
            adapted_chart.clone(),
            source_chart,
            coords(&source_axes)?,
            b,
        )?,
        boundaries: vec![],
    }
    .verify(b)?;
    let marked = MarkedIdeal::new(
        extension.ideal(center.source.ideal(), b)?,
        center.source.mark(),
        b,
    )?;
    let positions = source_axes
        .iter()
        .enumerate()
        .filter_map(|(j, i)| normals.contains(i).then_some(j))
        .collect::<Vec<_>>();
    let admitted = CenterCertificate {
        frame,
        source: marked,
        normal_axes: positions.iter().map(|j| adapted_axes[*j]).collect(),
    }
    .verify(b)?;
    let mut charts = Vec::new();
    for &pivot in &positions {
        let exceptional = ring.coordinate(target_axes[pivot])?;
        let images = target_axes
            .iter()
            .enumerate()
            .map(|(j, i)| {
                let p = ring.coordinate(*i)?;
                if j != pivot && positions.contains(&j) {
                    b.mul(&exceptional, &p)
                } else {
                    Ok(p)
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let map = Map::new(adapted_chart.clone(), target_chart.clone(), images, b)?;
        let factor = b.power(&exceptional, center.source.mark())?;
        let mut quotients = Vec::new();
        for f in admitted.adapted().ideal().generators() {
            let total = map.pull(f, b)?;
            let (q, r) = total.quot_rem(&factor, false);
            if !r.is_zero() || b.mul(&factor, &q)? != total {
                return Err(Error::Invalid("recursive controlled quotient"));
            }
            quotients.push(q);
        }
        let target = MarkedIdeal::new(
            Ideal::new(ring.clone(), quotients, b)?,
            center.source.mark(),
            b,
        )?;
        let transform = ControlledTransformCertificate {
            center: admitted.clone(),
            blowup: map,
            pivot_position: pivot,
            born_divisor_id: 0,
            born_stage: 1,
            target,
            divisor_transforms: vec![],
        }
        .verify(b)?;
        let local = LocalizedAlgebra::new(
            Ideal::new(ring.clone(), vec![], b)?,
            target_axes.clone(),
            vec![],
            b,
        )?;
        let target_frame = Arc::new(
            EtaleCertificate {
                source: local,
                equations: vec![],
                dependent_axes: vec![],
                free_axes: target_axes.clone(),
                determinant_inverse_axis: None,
            }
            .verify(b)?,
        );
        let target_order = produce_ordinary_contact_cover(
            target_frame.clone(),
            Arc::new(transform.target().ideal().clone()),
            b,
        )?;
        match &target_order {
            OrderProduction::Incomplete { reason, .. } => {
                return Err(Error::ResourceIncomplete(reason));
            }
            OrderProduction::TerminalParameterLocus { .. } => {
                return Err(Error::Invalid(
                    "unexpected recursive target parameter stratum",
                ));
            }
            _ => {}
        }
        charts.push(RecursiveBlowupChart {
            pivot_source_axis: source_axes[pivot],
            transform,
            target_frame,
            target_order,
        });
    }
    Ok(
        if charts
            .iter()
            .all(RecursiveBlowupChart::marked_cosupport_empty)
        {
            FirstBlowup::MarkedResolved {
                center,
                extension,
                charts,
            }
        } else {
            FirstBlowup::FurtherCyclesRequired {
                center,
                extension,
                charts,
            }
        },
    )
}
