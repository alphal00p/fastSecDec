use super::*;

pub(super) fn admit(
    owner: Arc<VerifiedDecomposition>,
    parameters: BTreeMap<Symbol, Rational>,
    unit: Symbol,
    limits: Limits,
    observer: &mut impl FnMut(Progress) -> ControlFlow<()>,
) -> Result<RegularizedFiber> {
    let request = owner.request();
    if request.domain().coordinates().len() != 1 {
        return Err(unsupported(
            "initial rational endpoint bridge requires one integration coordinate",
        ));
    }
    match request.domain().origin() {
        DomainOrigin::NativeUnitCube => {}
        DomainOrigin::AffineProjective { .. } if request.input().parameters().len() == 2 => {}
        _ => {
            return Err(unsupported(
                "closed interval coverage requires a native unit interval or proved two-variable affine projective gauge",
            ));
        }
    }
    let count = owner
        .cells()
        .len()
        .checked_mul(2)
        .filter(|n| *n <= limits.max_charts)
        .ok_or(Error::ResourceIncomplete("rational interval chart limit"))?;
    if count == 0 {
        return Err(unsupported(
            "empty generic cell inventory cannot prove unit interval coverage",
        ));
    }
    let numerator = SymbolBuilder::new(wrap_symbol!(
        "fastsecdec::threshold_regularization::numerator"
    ))
    .with_attributes(&[] as &[SymbolAttribute])
    .build()
    .map_err(|e| invalid(e.as_ref()))?;
    if !numerator.is_exportable() || !numerator.get_attributes().is_empty() {
        return Err(invalid(
            "owned numerator symbol has native hooks or attributes",
        ));
    }
    let mut intervals = Vec::new();
    for cell in owner.cells() {
        poll(Progress::Cell(cell.index()), observer)?;
        let map = CellMap::new(owner.clone(), cell.index(), vec![unit])?.linear()?;
        // Initial scope deliberately requires one common allowed parameter
        // chamber. Do not swallow arbitrary Invalid errors as nonmembership.
        let admission = map.admit_parameters(&parameters)?;
        let axis = map
            .axes()
            .last()
            .ok_or_else(|| invalid("missing interval axis"))?;
        let lower = axis
            .lower()
            .ok_or_else(|| unsupported("unbounded lower interval"))?
            .clone();
        let upper = axis
            .upper()
            .ok_or_else(|| unsupported("unbounded upper interval"))?
            .clone();
        let l = exact(&substitute(&lower, &parameters))?;
        let u = exact(&substitute(&upper, &parameters))?;
        if l >= u {
            return Err(invalid("nonpositive exact interval width"));
        }
        intervals.push((l, u, lower, upper, admission));
    }
    intervals.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut frontier = Rational::zero();
    for (l, u, ..) in &intervals {
        if l != &frontier {
            return Err(unsupported(
                "verified generic intervals do not partition the full unit interval on this fiber",
            ));
        }
        frontier = u.clone();
    }
    if frontier != Rational::one() {
        return Err(unsupported(
            "verified intervals do not end at the unit upper face",
        ));
    }
    let bodies = normalize::numerators(request, &parameters, numerator, limits)?;
    let mut charts = Vec::with_capacity(count);
    let mut strip = ConvergenceStrip::default();
    for (l, u, lower, upper, admission) in intervals {
        let width = (&upper - &lower) / Atom::num(2);
        for orientation in [1i8, -1i8] {
            let image = if orientation == 1 {
                &lower + &width * Atom::var(unit)
            } else {
                &upper - &width * Atom::var(unit)
            };
            let index = charts.len();
            poll(Progress::Chart(index), observer)?;
            let midpoint = (&l + &u) / Rational::from(2);
            let bounds = if orientation == 1 {
                (l.clone(), midpoint)
            } else {
                (midpoint, u.clone())
            };
            let mut chart = IntervalChart {
                admission: admission.clone(),
                image,
                measure: width.clone(),
                orientation,
                bounds,
                terms: Vec::new(),
            };
            chart.terms = normalize::terms(
                &chart,
                unit,
                numerator,
                &parameters,
                limits,
                index,
                observer,
            )?;
            for term in &chart.terms {
                strip.admit(
                    &substitute(&term.power, &parameters),
                    request.input().regulator(),
                )?;
            }
            charts.push(chart);
        }
    }
    let prefactors = request
        .prepared_terms()
        .iter()
        .map(|term| {
            let expression = substitute(
                &request.kinematics().specialize_exact(term.prefactor()),
                &parameters,
            );
            meromorphic::MeromorphicPrefactor::admit(
                expression,
                request.input().regulator(),
                limits.prefactors,
            )
            .map_err(prefactor_error)
        })
        .collect::<Result<Vec<_>>>()?;
    let prefactor_witness = meromorphic::MeromorphicWitness::construct(
        prefactors,
        strip.lower(),
        strip.upper(),
        limits.prefactors,
        |index| observer(Progress::PrefactorWitness(index)),
    )
    .map_err(prefactor_error)?;
    Ok(RegularizedFiber {
        owner,
        parameters,
        unit,
        numerator,
        numerator_bodies: bodies,
        charts,
        strip,
        prefactor_witness,
    })
}
