use super::*;
use std::collections::BTreeSet;
use symbolica::{poly::PolyVariable, symbol};

#[derive(Clone, Debug)]
pub struct SupportFrameMap {
    source: Arc<EtaleFrame>,
    target: Arc<EtaleFrame>,
    axes: Vec<usize>,
    checked_derivatives: usize,
}
impl SupportFrameMap {
    pub fn source(&self) -> &Arc<EtaleFrame> {
        &self.source
    }
    pub fn target(&self) -> &Arc<EtaleFrame> {
        &self.target
    }
    pub fn coordinate_indices(&self) -> &[usize] {
        &self.axes
    }
    pub fn checked_derivatives(&self) -> usize {
        self.checked_derivatives
    }
    pub fn pull(&self, p: &Poly, b: &mut Budget) -> Result<Poly> {
        self.source.local().supports(p)?;
        super::super::super::elimination::remap(p, self.target.local().ring(), b)
    }
}
#[derive(Clone, Debug)]
pub struct RefinedSupportOpen {
    parent: Arc<StrictSupportOpen>,
    child: Arc<StrictSupportOpen>,
    frame: Arc<EtaleFrame>,
    parent_map: SupportFrameMap,
    child_map: SupportFrameMap,
    full_equations: Ideal,
}
impl RefinedSupportOpen {
    pub fn parent(&self) -> &Arc<StrictSupportOpen> {
        &self.parent
    }
    pub fn child(&self) -> &Arc<StrictSupportOpen> {
        &self.child
    }
    pub fn frame(&self) -> &Arc<EtaleFrame> {
        &self.frame
    }
    pub fn parent_map(&self) -> &SupportFrameMap {
        &self.parent_map
    }
    pub fn child_map(&self) -> &SupportFrameMap {
        &self.child_map
    }
    pub fn full_equations(&self) -> &Ideal {
        &self.full_equations
    }
}
pub(super) enum FrameProduction {
    Complete(Arc<RefinedSupportOpen>),
    Empty {
        equations: Ideal,
        unit_relations: Vec<Poly>,
    },
}
fn positions(old: &Arc<Ring>, target: &Arc<Ring>, b: &mut Budget) -> Result<Vec<usize>> {
    b.reserve_slots(old.len())?;
    old.one()
        .variables()
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let j = target
                .one()
                .variables()
                .iter()
                .position(|w| v == w)
                .ok_or(Error::Invalid("common support lost symbol"))?;
            if old.is_parameter(i) != target.is_parameter(j) {
                return Err(Error::Invalid("common support parameter role conflict"));
            }
            Ok(j)
        })
        .collect()
}
fn frame_map(
    source: Arc<EtaleFrame>,
    target: Arc<EtaleFrame>,
    axes: Vec<usize>,
    b: &mut Budget,
) -> Result<SupportFrameMap> {
    let ring = target.local().ring();
    let pull = |p: &Poly, b: &mut Budget| super::super::super::elimination::remap(p, ring, b);
    for f in source
        .local()
        .ideal()
        .generators()
        .iter()
        .chain(source.local().unit_relations())
    {
        if !target.local().zero(&pull(f, b)?, b)? {
            return Err(Error::Invalid("common support full original relation map"));
        }
    }
    let checks = source
        .local()
        .ring()
        .len()
        .checked_mul(target.free_axes().len())
        .and_then(|n| n.checked_mul(source.free_axes().len().checked_add(1)?))
        .ok_or(Error::ResourceIncomplete("common support derivative work"))?;
    b.count(checks)?;
    let mut count = 0usize;
    for i in 0..source.local().ring().len() {
        if !source.local().axes().contains(&i)
            && !source.local().ring().is_parameter(i)
            && !source.local().guards().iter().any(|g| g.inverse_axis == i)
        {
            continue;
        }
        let f = source.local().ring().coordinate(i)?;
        let image = pull(&f, b)?;
        for j in 0..target.free_axes().len() {
            let mut expected = &ring.one() - &ring.one();
            for (k, free) in source.free_axes().iter().enumerate() {
                let coordinate = source.local().ring().coordinate(*free)?;
                let weight = target.derivative(j, &pull(&coordinate, b)?, b)?;
                let derivative = pull(&source.derivative(k, &f, b)?, b)?;
                expected = &expected + &b.mul(&weight, &derivative)?;
            }
            let actual = target.derivative(j, &image, b)?;
            if !target.local().zero(&(&actual - &expected), b)? {
                return Err(Error::Invalid(
                    "common support composed relative derivation",
                ));
            }
            count = count
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("common support derivative count"))?;
        }
    }
    Ok(SupportFrameMap {
        source,
        target,
        axes,
        checked_derivatives: count,
    })
}
#[derive(Clone, Debug)]
pub(crate) struct JoinedSupportFrame {
    pub(crate) frame: Arc<EtaleFrame>,
    pub(crate) parent_map: SupportFrameMap,
    pub(crate) child_map: SupportFrameMap,
    pub(crate) full_equations: Ideal,
}
pub(crate) enum JoinedFrameProduction {
    Complete(Arc<JoinedSupportFrame>),
    Empty {
        equations: Ideal,
        unit_relations: Vec<Poly>,
    },
}
pub(super) fn prepare(
    parent: Arc<StrictSupportOpen>,
    child: Arc<StrictSupportOpen>,
    base: &Arc<Ring>,
    namespace: &str,
    b: &mut Budget,
) -> Result<FrameProduction> {
    match join(
        parent.frame().clone(),
        child.frame().clone(),
        base,
        namespace,
        b,
    )? {
        JoinedFrameProduction::Empty {
            equations,
            unit_relations,
        } => Ok(FrameProduction::Empty {
            equations,
            unit_relations,
        }),
        JoinedFrameProduction::Complete(joined) => {
            Ok(FrameProduction::Complete(Arc::new(RefinedSupportOpen {
                parent,
                child,
                frame: joined.frame.clone(),
                parent_map: joined.parent_map.clone(),
                child_map: joined.child_map.clone(),
                full_equations: joined.full_equations.clone(),
            })))
        }
    }
}
pub(crate) fn join(
    parent: Arc<EtaleFrame>,
    child: Arc<EtaleFrame>,
    base: &Arc<Ring>,
    namespace: &str,
    b: &mut Budget,
) -> Result<JoinedFrameProduction> {
    let sources = [&parent, &child];
    let maximum = sources[0]
        .local()
        .ring()
        .len()
        .checked_add(sources[1].local().ring().len())
        .and_then(|v| v.checked_add(1))
        .ok_or(Error::ResourceIncomplete("common support ring size"))?;
    b.reserve_slots(maximum)?;
    let mut variables = base.one().variables().to_vec();
    for source in sources {
        let old = source.local().ring();
        if old.len() < base.len()
            || old.one().variables()[..base.len()] != base.one().variables()[..]
            || (0..base.len()).any(|i| old.is_parameter(i) != base.is_parameter(i))
            || (base.len()..old.len()).any(|i| old.is_parameter(i))
        {
            return Err(Error::Invalid("common support physical coordinate prefix"));
        }
        for v in &old.one().variables()[base.len()..] {
            if !variables.contains(v) {
                variables.push(v.clone());
            }
        }
    }
    let determinant_axis = variables.len();
    variables.push(PolyVariable::Symbol(symbol!(format!(
        "{namespace}::determinant_inverse"
    ))));
    let symbols = variables
        .iter()
        .map(|v| match v {
            PolyVariable::Symbol(s) => Ok(*s),
            _ => Err(Error::Invalid("common support coordinate symbol")),
        })
        .collect::<Result<Vec<_>>>()?;
    let ring = Arc::new(Ring::new(
        symbols,
        (0..base.len()).filter(|i| base.is_parameter(*i)).collect(),
    )?);
    let parent_axes = positions(parent.local().ring(), &ring, b)?;
    let child_axes = positions(child.local().ring(), &ring, b)?;
    let mut equations = Vec::new();
    let mut guards: Vec<Guard> = Vec::new();
    let mut axes = BTreeSet::new();
    for (source, map) in [(&parent, &parent_axes), (&child, &child_axes)] {
        b.reserve_slots(source.local().ideal().generators().len())?;
        for f in source.local().ideal().generators() {
            equations.push(super::super::super::elimination::remap(f, &ring, b)?);
        }
        for i in source.local().axes() {
            axes.insert(map[*i]);
        }
        b.reserve_slots(source.local().guards().len())?;
        for g in source.local().guards() {
            let factor = super::super::super::elimination::remap(&g.factor, &ring, b)?;
            let inverse_axis = map[g.inverse_axis];
            if let Some(old) = guards.iter().find(|q| q.inverse_axis == inverse_axis) {
                if old.factor != factor {
                    return Err(Error::Invalid("common support inverse role conflict"));
                }
            } else {
                guards.push(Guard {
                    factor,
                    inverse_axis,
                });
            }
        }
    }
    let equations = Ideal::new(ring.clone(), equations, b)?;
    b.reserve_slots(guards.len())?;
    let relations = guards
        .iter()
        .map(|g| Ok(&b.mul(&ring.coordinate(g.inverse_axis)?, &g.factor)? - &ring.one()))
        .collect::<Result<Vec<_>>>()?;
    if equations.contains(&ring.one(), &relations, b)? {
        return Ok(JoinedFrameProduction::Empty {
            equations,
            unit_relations: relations,
        });
    }
    let local = LocalizedAlgebra::new(equations.clone(), axes.into_iter().collect(), guards, b)?;
    let mut selected = Vec::new();
    b.reserve_slots(child.selected_equations().len())?;
    for i in child.selected_equations() {
        let q = super::super::super::elimination::remap(
            &child.source().ideal().generators()[*i],
            &ring,
            b,
        )?;
        selected.push(equations.generators().iter().position(|p| p == &q).ok_or(
            Error::Invalid("common support selected equation association"),
        )?);
    }
    let frame = Arc::new(
        EtaleCertificate {
            source: local,
            equations: selected.clone(),
            dependent_axes: child
                .dependent_axes()
                .iter()
                .map(|i| child_axes[*i])
                .collect(),
            free_axes: child.free_axes().iter().map(|i| child_axes[*i]).collect(),
            determinant_inverse_axis: (!selected.is_empty()).then_some(determinant_axis),
        }
        .verify(b)?,
    );
    for (a, z) in [
        (&equations, frame.local().ideal()),
        (frame.local().ideal(), &equations),
    ] {
        for f in a.generators() {
            if !z.contains(f, frame.local().unit_relations(), b)? {
                return Err(Error::Invalid("common support full reconstructed ideal"));
            }
        }
    }
    let parent_map = frame_map(parent.clone(), frame.clone(), parent_axes, b)?;
    let child_map = frame_map(child.clone(), frame.clone(), child_axes, b)?;
    Ok(JoinedFrameProduction::Complete(Arc::new(
        JoinedSupportFrame {
            frame,
            parent_map,
            child_map,
            full_equations: equations,
        },
    )))
}
