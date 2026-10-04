//! Thin adapters over Numerica's exact scalar and matrix APIs.
use crate::SectorError;
use numerica::{
    domains::{
        integer::Integer,
        rational::{Q, Rational},
    },
    tensors::matrix::Matrix,
};

pub(crate) type IntVector = Vec<Integer>;

pub(crate) fn dot(a: &[Integer], b: &[Integer]) -> Integer {
    a.iter()
        .zip(b)
        .fold(Integer::from(0), |s, (x, y)| s + x * y)
}

pub(crate) fn primitive(mut v: IntVector) -> IntVector {
    let gcd = v.iter().fold(Integer::from(0), |g, x| g.gcd(x));
    if !gcd.is_zero() {
        for x in &mut v {
            *x = &*x / &gcd;
        }
    }
    v
}

pub(crate) fn integer_ray(v: Vec<Rational>) -> IntVector {
    let lcm = v
        .iter()
        .fold(Integer::from(1), |d, x| d.lcm(x.denominator_ref()));
    primitive(
        v.iter()
            .map(|x| x.numerator_ref() * (&lcm / x.denominator_ref()))
            .collect(),
    )
}

pub(crate) fn matrix(rows: &[IntVector]) -> Result<Matrix<Q>, SectorError> {
    Matrix::from_nested_vec(
        rows.iter()
            .map(|r| r.iter().cloned().map(Rational::from).collect())
            .collect(),
        Q,
    )
    .map_err(|e| SectorError::Geometry(e.to_string()))
}

pub(crate) fn rank(rows: &[IntVector]) -> usize {
    if rows.is_empty() || rows[0].is_empty() {
        0
    } else {
        matrix(rows)
            .expect("rectangular internal geometry matrix")
            .rank()
    }
}

pub(crate) fn determinant(rows: &[IntVector]) -> Result<Integer, SectorError> {
    if rows.is_empty() {
        return Ok(1.into());
    }
    let det = matrix(rows)?
        .det()
        .map_err(|e| SectorError::Geometry(e.to_string()))?;
    if det.denominator_ref() != &Integer::from(1) {
        return Err(SectorError::Geometry(
            "integer matrix has noninteger determinant".into(),
        ));
    }
    Ok(det.numerator_ref().abs())
}
