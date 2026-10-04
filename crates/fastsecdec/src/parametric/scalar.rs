use symbolica::atom::{Atom, AtomCore, AtomView};
use symbolica::transcendental::TranscendentalFunctions;

use crate::{
    Error, Result,
    input::{GraphIntegral, default_algebra_settings},
};

/// A scalar projective Feynman-parameter integral on `sum(parameters) = 1`.
///
/// Its density is `prefactor * prod(x_i^(powers_i-1)) * U^u_exponent *
/// F^f_exponent`. Powers use `q^2-m^2+i0`; the loop measure is
/// `prod_l d^D k_l/(i*pi^(D/2))`, without implicit scale or Euler-gamma factors.
#[derive(Clone, Debug)]
pub struct ScalarParametricIntegral {
    pub(crate) parameters: Vec<Atom>,
    pub(crate) powers: Vec<u32>,
    pub(crate) u: Atom,
    pub(crate) f: Atom,
    pub(crate) u_exponent: Atom,
    pub(crate) f_exponent: Atom,
    pub(crate) prefactor: Atom,
    pub(crate) dimension: Atom,
    pub(crate) loop_count: usize,
}

impl ScalarParametricIntegral {
    pub fn parameters(&self) -> &[Atom] {
        &self.parameters
    }
    pub fn powers(&self) -> &[u32] {
        &self.powers
    }
    pub fn u(&self) -> &Atom {
        &self.u
    }
    pub fn f(&self) -> &Atom {
        &self.f
    }
    pub fn u_exponent(&self) -> &Atom {
        &self.u_exponent
    }
    pub fn f_exponent(&self) -> &Atom {
        &self.f_exponent
    }
    pub fn prefactor(&self) -> &Atom {
        &self.prefactor
    }
    pub fn dimension(&self) -> &Atom {
        &self.dimension
    }
    pub fn loop_count(&self) -> usize {
        self.loop_count
    }

    /// Construct U/F through HEPKit. `dimension` can be `4-2*eps`; tensor
    /// contractions occur in the graph kinematics' symbolic dimension first.
    /// Loop-dependent scalar numerators need the numerator-parameterization
    /// stage; this entry point rejects them instead of dropping their dependence.
    pub fn from_graph(
        integral: &GraphIntegral,
        parameters: Vec<Atom>,
        dimension: Atom,
    ) -> Result<Self> {
        if parameters.len() != integral.powers().len() {
            return Err(Error::ParameterCount {
                expected: integral.powers().len(),
                actual: parameters.len(),
            });
        }
        let family = integral.family();
        let numerator =
            integral.scalar_numerator(&default_algebra_settings())? * integral.measure_multiplier();
        if let Some(parameter) = parameters.iter().find(|parameter| {
            numerator.contains(parameter.as_view()) || dimension.contains(parameter.as_view())
        }) {
            return Err(Error::ParameterCollision(parameter.clone()));
        }
        if numerator
            .get_all_symbols(true)
            .contains(&feynkit_graph::symbols::loop_momentum())
            || family
                .loop_momenta()
                .iter()
                .any(|momentum| numerator.contains(momentum.as_view()))
        {
            return Err(Error::LoopNumerator);
        }
        let (u, f) = family.symanzik(&parameters)?;
        if u.is_zero() {
            return Err(Error::SingularLoopForm);
        }
        let total_power = integral.powers().iter().map(|p| u64::from(*p)).sum::<u64>();
        let loops = family.loop_momenta().len();
        let gamma_argument = (Atom::num(total_power) - &dimension * Atom::num(loops) / 2).expand();
        let denominator = integral
            .powers()
            .iter()
            .fold(Atom::one(), |product, power| {
                product * Atom::num(*power).gamma()
            });
        let symbolic_dimension = family.kinematics().dimension().to_symbolic();
        let scalar_weight = match symbolic_dimension.as_view() {
            AtomView::Var(_) => numerator
                .replace(symbolic_dimension)
                .with(dimension.to_pattern()),
            _ if symbolic_dimension == dimension => numerator,
            _ => return Err(Error::ConcreteDimensionMismatch),
        };
        let prefactor = Atom::num(if total_power % 2 == 0 { 1 } else { -1 })
            * gamma_argument.gamma()
            / denominator
            * scalar_weight;
        Ok(Self {
            parameters,
            powers: integral.powers().to_vec(),
            u,
            f,
            u_exponent: Atom::num(total_power) - &dimension * Atom::num(loops + 1) / 2,
            f_exponent: -gamma_argument,
            prefactor,
            dimension,
            loop_count: loops,
        })
    }

    /// Materialize the projective density when an exact identity or direct
    /// evaluator is needed. Production generation can retain the factorization.
    pub fn density(&self) -> Atom {
        let monomial = self
            .parameters
            .iter()
            .zip(&self.powers)
            .fold(Atom::one(), |product, (parameter, power)| {
                product * parameter.pow(Atom::num(power - 1))
            });
        &self.prefactor * monomial * self.u.pow(&self.u_exponent) * self.f.pow(&self.f_exponent)
    }
}
