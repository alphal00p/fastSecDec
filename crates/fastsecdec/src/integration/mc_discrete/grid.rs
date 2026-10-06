use crate::integration::{IntegrationError, Result};
use numerica::numerical_integration::{DiscreteGrid, Grid};

/// Copy FastSecDec's sector proposal, clearing only unprocessed observations.
///
/// Numerica 3.0.1 discards the returned sample-free clone of each discrete
/// child. Use its public accumulators and continuous-grid operation directly.
/// Settings construct one discrete selector with continuous children; restore
/// reconstructs that shape from settings and stores only proposal partitions.
pub(super) fn clone_without_samples(grid: &DiscreteGrid<f64>) -> Result<DiscreteGrid<f64>> {
    let mut fresh = grid.clone();
    fresh.accumulator.clear_samples();
    for bin in &mut fresh.bins {
        bin.accumulator.clear_samples();
        let Some(Grid::Continuous(child)) = &mut bin.sub_grid else {
            return Err(IntegrationError::Invalid(
                "sector sampling requires one continuous grid per discrete bin".into(),
            ));
        };
        *child = child.clone_without_samples();
    }
    Ok(fresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use numerica::numerical_integration::{ContinuousGrid, MonteCarloRng, Sample};

    #[test]
    fn pending_training_is_cleared_without_changing_proposal_or_rng_samples() {
        let child = ContinuousGrid::<f64>::new(2, 4, 4, None, false).unwrap();
        let mut original =
            DiscreteGrid::new(vec![Some(Grid::Continuous(child))], 100.0, false).unwrap();
        let mut rng = MonteCarloRng::new(71, 0);
        let mut sample = Sample::new();
        for _ in 0..7 {
            original.sample(&mut rng, &mut sample);
            original.add_training_sample(&sample, 2.0).unwrap();
        }
        let before = serde_json::to_value(&original).unwrap();
        let mut fresh = clone_without_samples(&original).unwrap();
        assert_eq!(before["accumulator"]["new_samples"], 7);
        let cleared = serde_json::to_value(&fresh).unwrap();
        assert_eq!(cleared["accumulator"]["new_samples"], 0);
        assert_eq!(cleared["bins"][0]["accumulator"]["new_samples"], 0);
        let Some(Grid::Continuous(old_child)) = &original.bins[0].sub_grid else {
            panic!("continuous fixture")
        };
        let Some(Grid::Continuous(new_child)) = &fresh.bins[0].sub_grid else {
            panic!("continuous fixture")
        };
        assert_eq!(
            serde_json::to_value(&old_child.accumulator).unwrap()["new_samples"],
            7
        );
        assert_eq!(
            serde_json::to_value(&new_child.accumulator).unwrap()["new_samples"],
            0
        );
        assert_eq!(
            serde_json::to_value(new_child).unwrap(),
            serde_json::to_value(old_child.clone_without_samples()).unwrap()
        );
        let mut replay = rng.clone();
        let mut old_sample = Sample::new();
        let mut new_sample = Sample::new();
        for _ in 0..16 {
            original.sample(&mut rng, &mut old_sample);
            fresh.sample(&mut replay, &mut new_sample);
            assert_eq!(
                serde_json::to_value(&old_sample).unwrap(),
                serde_json::to_value(&new_sample).unwrap()
            );
        }
        assert_eq!(rng.export(), replay.export());
        assert_eq!(serde_json::to_value(&original).unwrap(), before);
        fresh.add_training_sample(&new_sample, 3.0).unwrap();
        original.merge(&fresh).unwrap();
        assert_eq!(
            serde_json::to_value(&original.accumulator).unwrap()["new_samples"],
            8
        );
        let Some(Grid::Continuous(child)) = &original.bins[0].sub_grid else {
            panic!("continuous fixture")
        };
        assert_eq!(
            serde_json::to_value(&child.accumulator).unwrap()["new_samples"],
            8
        );
    }

    #[test]
    fn unexpected_native_grid_shapes_are_rejected_without_retaining_training() {
        let child = ContinuousGrid::<f64>::new(1, 4, 4, None, false).unwrap();
        let nested =
            DiscreteGrid::new(vec![Some(Grid::Continuous(child.clone()))], 100.0, false).unwrap();
        for child in [
            None,
            Some(Grid::Discrete(nested)),
            Some(Grid::Uniform(vec![2], child)),
        ] {
            let grid = DiscreteGrid::new(vec![child], 100.0, false).unwrap();
            assert!(clone_without_samples(&grid).is_err());
        }
    }
}
