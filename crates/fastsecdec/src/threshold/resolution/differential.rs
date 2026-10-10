use super::{Budget, Error, EtaleFrame, Ideal};
type Result<T> = std::result::Result<T, Error>;

/// Shared full inclusive relative derivative ideal. All derivatives, including
/// inverse guards and inert parameter axes, belong to the checked native frame.
pub(crate) fn relative_differential_ideal(
    frame: &EtaleFrame,
    ideal: &Ideal,
    budget: &mut Budget,
) -> Result<Ideal> {
    if ideal.ring() != frame.local().ring() {
        return Err(Error::Invalid("relative derivative ideal ring"));
    }
    let count = ideal
        .generators()
        .len()
        .checked_mul(
            frame
                .free_axes()
                .len()
                .checked_add(1)
                .ok_or(Error::ResourceIncomplete("derivative axes"))?,
        )
        .ok_or(Error::ResourceIncomplete("derivative ideal count"))?;
    budget.reserve_slots(count)?;
    let mut next = ideal.generators().to_vec();
    for f in ideal.generators() {
        frame.local().supports(f)?;
        for index in 0..frame.free_axes().len() {
            next.push(frame.derivative(index, f, budget)?);
        }
    }
    Ideal::new(ideal.ring().clone(), next, budget)
}
