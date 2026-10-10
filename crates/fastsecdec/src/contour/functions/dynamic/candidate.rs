//! Exact native centres for optional certificates. In particular a double-float
//! is the exact sum of both binary limbs, not a rounded 106-bit conversion.
use super::numeric::Number;
use symbolica::{
    domains::{
        float::{DoubleFloat, ErrorPropagatingFloat, Float},
        rational::Rational,
    },
    evaluate::EvaluationDomain,
};

pub(crate) trait CandidateNumber: Number {
    fn exact_centre(&self) -> Result<Rational, String>;
}
impl CandidateNumber for f64 {
    fn exact_centre(&self) -> Result<Rational, String> {
        Rational::try_from(*self).map_err(|_| "nonfinite f64 contour candidate".into())
    }
}
impl CandidateNumber for DoubleFloat {
    fn exact_centre(&self) -> Result<Rational, String> {
        let pair = self.into_inner();
        Ok(pair.hi().exact_centre()? + pair.lo().exact_centre()?)
    }
}
impl CandidateNumber for Float {
    fn exact_centre(&self) -> Result<Rational, String> {
        self.try_to_rational()
            .ok_or_else(|| "nonfinite arbitrary-precision contour candidate".into())
    }
}
impl<T: CandidateNumber> CandidateNumber for ErrorPropagatingFloat<T>
where
    Self: EvaluationDomain,
{
    fn exact_centre(&self) -> Result<Rational, String> {
        self.get_num().exact_centre()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_candidate_conversion_preserves_both_double_float_limbs() {
        let value = DoubleFloat::from(1.) + DoubleFloat::from(1e-300);
        let exact = value.exact_centre().unwrap();
        assert_eq!(
            exact,
            Rational::from(1) + Rational::try_from(1e-300).unwrap()
        );
        assert_ne!(
            exact,
            exact.to_multi_prec_float(106).try_to_rational().unwrap()
        );
        let tracked = ErrorPropagatingFloat::new_with_accuracy(Float::with_val(192, 0.3), 40.);
        assert_eq!(
            tracked.exact_centre().unwrap(),
            tracked.get_num().try_to_rational().unwrap()
        );
        assert!(f64::NAN.exact_centre().is_err());
        assert!(Float::with_val(192, f64::INFINITY).exact_centre().is_err());
    }
}
