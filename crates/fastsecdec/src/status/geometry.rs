use serde::{Deserialize, Serialize};
use std::fmt;

/// Observation of complete native geometry, independent of integral identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryReuseStatus {
    pub reused: bool,
    /// Native geometry map count before integrand symmetry or exact extraction.
    /// This is not the eventual numerical kernel/representative count.
    pub sectors: usize,
}

impl fmt::Display for GeometryReuseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "geometry {}: {} maps",
            if self.reused { "reused" } else { "computed" },
            self.sectors
        )
    }
}
