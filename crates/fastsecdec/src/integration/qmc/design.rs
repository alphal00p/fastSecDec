use super::QmcDesign;
use crate::integration::{Periodization, PublishedLattice, RuleSource};
use std::fmt;

impl fmt::Display for QmcDesign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QMC design: ")?;
        match &self.settings.rule {
            RuleSource::Kuo | RuleSource::Published(PublishedLattice::Kuo33002) => {
                write!(f, "Kuo 33002")?
            }
            RuleSource::Published(PublishedLattice::Kuo38005) => write!(f, "Kuo 38005")?,
            RuleSource::Published(PublishedLattice::Kuo39101) => write!(f, "Kuo 39101")?,
            RuleSource::Published(PublishedLattice::HkknAlpha3) => write!(f, "HKKN alpha 3")?,
            RuleSource::Supplied(vector) => {
                write!(f, "supplied vector ({} components)", vector.len())?
            }
        }
        let transform = match self.settings.periodization {
            Periodization::None => "no periodization",
            Periodization::Korobov3 => "Korobov 3",
        };
        write!(f, "; {transform}; seed {}", self.settings.seed)?;
        let Some(min_points) = self.allocations.iter().map(|a| a.points).min() else {
            return write!(f, "; no stochastic sectors");
        };
        let max_points = self.allocations.iter().map(|a| a.points).max().unwrap();
        let min_shifts = self.allocations.iter().map(|a| a.shifts).min().unwrap();
        let max_shifts = self.allocations.iter().map(|a| a.shifts).max().unwrap();
        write!(
            f,
            "; {} sectors; points per shift {min_points}",
            self.allocations.len()
        )?;
        if max_points != min_points {
            write!(f, "–{max_points}")?;
        }
        write!(f, "; shifts {min_shifts}")?;
        if max_shifts != min_shifts {
            write!(f, "–{max_shifts}")?;
        }
        Ok(())
    }
}
