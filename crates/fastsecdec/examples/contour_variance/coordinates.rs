//! Shared audit of actual native transformed coordinates and weights.
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct CoordinateRange {
    pub(crate) sector: u64,
    pub(crate) shift: u64,
    pub(crate) start: u64,
    pub(crate) count: u64,
    pub(crate) dimension: usize,
    pub(crate) digest: String,
}

pub(crate) struct RangeHasher {
    pub(crate) range: CoordinateRange,
    hash: blake3::Hasher,
}
impl RangeHasher {
    pub(crate) fn new(sector: u64, shift: u64, start: u64, dimension: usize) -> Self {
        let mut hash = blake3::Hasher::new();
        hash.update(b"fastsecdec-contour-coordinate-audit-v1");
        for value in [sector, shift, start, dimension as u64] {
            hash.update(&value.to_le_bytes());
        }
        Self {
            range: CoordinateRange {
                sector,
                shift,
                start,
                count: 0,
                dimension,
                digest: String::new(),
            },
            hash,
        }
    }
    pub(crate) fn push(&mut self, index: u64, point: &[f64], weight: f64) {
        self.hash.update(&index.to_le_bytes());
        for coordinate in point.iter().chain(std::iter::once(&weight)) {
            self.hash.update(&coordinate.to_bits().to_le_bytes());
        }
        self.range.count += 1;
    }
    pub(crate) fn finish(mut self) -> CoordinateRange {
        self.hash.update(&self.range.count.to_le_bytes());
        self.range.digest = self.hash.finalize().to_hex().to_string();
        self.range
    }
}
