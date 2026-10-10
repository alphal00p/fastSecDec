use super::{Budget, Error, Poly, Ring};
use symbolica::{atom::AtomCore, domains::atom::AtomField, tensors::matrix::Matrix};
type Result<T> = std::result::Result<T, Error>;
pub fn combinations(n: usize, k: usize, b: &mut Budget) -> Result<Vec<Vec<usize>>> {
    if k > n {
        return Ok(vec![]);
    }
    let mut count = 1usize;
    for i in 0..k {
        count = count
            .checked_mul(n - i)
            .ok_or(Error::ResourceIncomplete("minor count"))?
            / (i + 1);
    }
    b.reserve_slots(count)?;
    fn run(n: usize, k: usize, start: usize, v: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if v.len() == k {
            out.push(v.clone());
            return;
        }
        for i in start..=n - (k - v.len()) {
            v.push(i);
            run(n, k, i + 1, v, out);
            v.pop();
        }
    }
    let mut out = Vec::new();
    run(n, k, 0, &mut Vec::new(), &mut out);
    Ok(out)
}
pub fn determinant(r: &Ring, entries: Vec<Poly>, n: usize, b: &mut Budget) -> Result<Poly> {
    let count = n
        .checked_mul(n)
        .ok_or(Error::ResourceIncomplete("matrix entries"))?;
    b.reserve_slots(count)?;
    if entries.len() != count {
        return Err(Error::Invalid("matrix shape"));
    }
    for entry in &entries {
        r.check(entry)?;
        b.poly(entry)?;
    }
    if n == 0 {
        return Ok(r.one());
    }
    let max_terms = entries.iter().map(Poly::nterms).max().unwrap_or(1);
    let mut bound = 1usize;
    for i in 1..=n {
        bound = bound
            .checked_mul(i)
            .and_then(|v| v.checked_mul(max_terms))
            .ok_or(Error::ResourceIncomplete("determinant terms"))?;
        if bound > b.limits.max_terms {
            return Err(Error::ResourceIncomplete("determinant terms"));
        }
    }
    for axis in 0..r.len() {
        let degree = entries
            .chunks(n)
            .try_fold(0usize, |s, row| {
                s.checked_add(
                    row.iter()
                        .map(|p| usize::from(p.degree(axis)))
                        .max()
                        .unwrap_or(0),
                )
            })
            .ok_or(Error::ResourceIncomplete("determinant degree"))?;
        if degree > usize::from(b.limits.max_degree_per_axis) {
            return Err(Error::ResourceIncomplete("determinant degree"));
        }
    }
    let dim = u32::try_from(n).map_err(|_| Error::ResourceIncomplete("matrix dimension"))?;
    b.charge(1)?;
    let field = AtomField {
        statistical_zero_test: false,
        cancel_check_on_division: true,
        ..AtomField::new()
    };
    let p = r.atom(
        &Matrix::from_linear(
            entries.iter().map(Poly::to_expression).collect(),
            dim,
            dim,
            field,
        )
        .map_err(|_| Error::Invalid("native matrix"))?
        .det()
        .map_err(|_| Error::Invalid("native determinant"))?
        .together()
        .expand(),
    )?;
    b.poly(&p)?;
    Ok(p)
}
