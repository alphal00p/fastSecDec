use super::*;
/// Complete native fiber root inventory. Real clusters are all retained;
/// nonreal roots are classified only at this exact fiber.
pub struct FiberClusters {
    source: Arc<MonicPolynomial>,
    point: Arc<GermPoint>,
    roots: Vec<Atom>,
    real: Vec<RealCluster>,
    nonreal_multiplicity: usize,
}
pub struct RealCluster {
    root: Atom,
    multiplicity: usize,
    seed: Option<Arc<FactorSeed>>,
}
impl FiberClusters {
    pub fn source(&self) -> &Arc<MonicPolynomial> {
        &self.source
    }
    pub fn point(&self) -> &Arc<GermPoint> {
        &self.point
    }
    pub fn roots(&self) -> &[Atom] {
        &self.roots
    }
    pub fn real(&self) -> &[RealCluster] {
        &self.real
    }
    pub fn nonreal_multiplicity(&self) -> usize {
        self.nonreal_multiplicity
    }
}
impl RealCluster {
    pub fn root(&self) -> &Atom {
        &self.root
    }
    pub fn multiplicity(&self) -> usize {
        self.multiplicity
    }
    pub fn seed(&self) -> Option<&Arc<FactorSeed>> {
        self.seed.as_ref()
    }
}
pub fn discover_clusters(
    source: Arc<MonicPolynomial>,
    point: Arc<GermPoint>,
    budget: &mut Budget,
) -> Result<FiberClusters> {
    if !Arc::ptr_eq(source.frame(), point.frame()) {
        return Err(Error::Invalid("cluster source/fiber frame"));
    }
    let d = source.degree();
    budget.reserve_slots(d)?;
    let variable = source.variable();
    let field = point.context().field();
    let coefficients = source
        .coefficients()
        .iter()
        .map(|p| point.evaluate(p))
        .collect::<Result<Vec<_>>>()?;
    let mut expression = Atom::num(0);
    for (i, c) in coefficients.iter().enumerate() {
        expression +=
            field.element_to_atom_simplified(c) * Atom::var(variable).pow(exact_usize(i)?);
    }
    let mut roots = Vec::with_capacity(d);
    let mut real_roots = Vec::new();
    let mut nonreal = 0usize;
    for i in 0..d {
        budget.charge(1)?;
        let root = if let Some(r) = Root::<AlgebraicExtension<Q>>::from_atom_with_variable(
            expression.as_view(),
            variable.into(),
            i,
        )
        .map_err(|_| Error::Invalid("native algebraic fiber root"))?
        {
            r.simplify()
                .map_err(|_| Error::Invalid("native algebraic root collapse"))?
                .ok_or(Error::ResourceIncomplete(
                    "native root collapse unavailable",
                ))?
        } else {
            Root::<Q>::from_atom_with_variable(expression.as_view(), variable.into(), i)
                .map_err(|_| Error::Invalid("native rational fiber root"))?
                .simplify()
                .map_err(|_| Error::Invalid("native rational root simplification"))?
        };
        let mut isolated = root
            .polynomial()
            .to_univariate_from_univariate(0)
            .root(root.index())
            .ok_or(Error::Invalid("native root isolation unavailable"))?;
        let atom = root.to_atom();
        match isolated.classify_location() {
            symbolica::poly::univariate::RootLocation::Real
            | symbolica::poly::univariate::RootLocation::Zero => real_roots.push(atom.clone()),
            _ => nonreal += 1,
        }
        roots.push(atom);
    }
    let mut context = point.context().clone();
    for a in point.original().iter().chain(&real_roots) {
        context
            .extend(a.as_view())
            .map_err(|_| Error::Invalid("native real cluster field"))?;
        if context.field().poly().degree(0) as usize > budget.limits.max_mark {
            return Err(Error::ResourceIncomplete("cluster field degree"));
        }
    }
    // Recompute original point images after completing the field, preserving all
    // embeddings. Never reuse images from an older primitive-element basis.
    let point_values = point
        .original()
        .iter()
        .map(|a| {
            context
                .image(a)
                .cloned()
                .ok_or(Error::Invalid("cluster inherited field image"))
        })
        .collect::<Result<Vec<_>>>()?;
    let field = context.field();
    let coefficients = source
        .coefficients()
        .iter()
        .map(|p| p.evaluate_with_coeff_map(|q| field.constant(q.clone()), &point_values, field))
        .collect();
    let p = UnivariatePolynomial::from_coefficients(
        field,
        coefficients,
        Arc::new(PolyVariable::from(variable)),
    );
    let values = real_roots
        .iter()
        .map(|a| {
            context
                .image(a)
                .cloned()
                .ok_or(Error::Invalid("real cluster native image"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut groups: Vec<(usize, usize)> = Vec::new();
    for (i, a) in values.iter().enumerate() {
        if let Some((_, count)) = groups.iter_mut().find(|(j, _)| values[*j] == *a) {
            *count += 1;
        } else {
            groups.push((i, 1));
        }
    }
    let mut real = Vec::new();
    for (i, multiplicity) in groups {
        let seed = if multiplicity == d {
            None
        } else {
            let linear = UnivariatePolynomial::from_coefficients(
                field,
                vec![field.neg(&values[i]), field.one()],
                Arc::new(PolyVariable::from(variable)),
            );
            let mut cluster = linear.one();
            for _ in 0..multiplicity {
                budget.charge(1)?;
                cluster = &cluster * &linear;
            }
            let complement = p
                .try_div(&cluster)
                .ok_or(Error::Invalid("native fiber cluster exact division"))?;
            if &cluster * &complement != p || field.is_zero(&cluster.resultant(&complement)) {
                return Err(Error::Invalid("native cluster coprime identity"));
            }
            let atoms = |p: &UnivariatePolynomial<AlgebraicExtension<Q>>| {
                p.coefficients()
                    .iter()
                    .map(|v| field.element_to_atom(v))
                    .collect::<Vec<_>>()
            };
            Some(FactorSeed::verify(
                source.clone(),
                point_values
                    .iter()
                    .map(|v| field.element_to_atom(v))
                    .collect(),
                atoms(&cluster),
                atoms(&complement),
                budget,
            )?)
        };
        real.push(RealCluster {
            root: real_roots[i].clone(),
            multiplicity,
            seed,
        });
    }
    Ok(FiberClusters {
        source,
        point,
        roots,
        real,
        nonreal_multiplicity: nonreal,
    })
}
