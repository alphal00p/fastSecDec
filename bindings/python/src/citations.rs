//! Usage records for Symbolica's existing cumulative bibliography collector.
//!
//! Method references are verified against the primary papers and the geometric
//! cone/triangulation and Taylor-subtraction implementations. They acknowledge
//! mathematical foundations, not a dependency on the authors' software.

use std::sync::atomic::{AtomicU8, Ordering};

use symbolica::api::python::Citation;

const SECTOR_DECOMPOSITION: u8 = 1;
const THRESHOLD_DECOMPOSITION: u8 = 2;
static METHODS_USED: AtomicU8 = AtomicU8::new(0);

/// Called when native generation starts, or a generated kernel artifact loads.
pub(crate) fn mark_generation() {
    METHODS_USED.fetch_or(SECTOR_DECOMPOSITION, Ordering::Relaxed);
}

pub(crate) fn mark_threshold_generation() {
    METHODS_USED.fetch_or(
        SECTOR_DECOMPOSITION | THRESHOLD_DECOMPOSITION,
        Ordering::Relaxed,
    );
}

/// Saved native recipes carry their method identity without replaying generation.
pub(crate) fn mark_kernels(kernels: &fastsecdec::kernel::KernelSet) {
    if kernels.threshold_metadata().is_some() {
        mark_threshold_generation();
    } else {
        mark_generation();
    }
}

/// Return references for FastSecDec computations used in this process.
/// Importing the module alone does not count as use; querying never resets it.
/// The community host merges these native records with the other HEPKit owners.
pub fn get_citations() -> Vec<Citation> {
    let used = METHODS_USED.load(Ordering::Relaxed);
    if used == 0 {
        return Vec::new();
    }
    let mut references = vec![
        Citation {
            id: "https://github.com/alphal00p/fastSecDec".into(),
            url: "https://github.com/alphal00p/fastSecDec".into(),
            reference: "FastSecDec contributors. FastSecDec: Rust sector decomposition and numerical integration. https://github.com/alphal00p/fastSecDec.".into(),
            bibtex: include_str!("../../../citations/fastsecdec.bib").trim().into(),
            reasons: vec!["Native FastSecDec generation or kernel loading was used.".into()],
            description: "Software repository citation; no publication DOI has been assigned here.".into(),
            relevance: None,
        },
        Citation {
            id: "doi:10.1016/j.cpc.2017.09.015".into(),
            url: "https://doi.org/10.1016/j.cpc.2017.09.015".into(),
            reference: "S. Borowka, G. Heinrich, S. Jahn, S. P. Jones, M. Kerner, J. Schlenk and T. Zirke. pySecDec: a toolbox for the numerical evaluation of multi-scale integrals. Computer Physics Communications 222 (2018), 313. doi:10.1016/j.cpc.2017.09.015.".into(),
            bibtex: include_str!("../../../citations/pysecdec.bib").trim().into(),
            reasons: vec!["Reference sector-decomposition implementation that informed FastSecDec's development and scientific cross-checks.".into()],
            description: "Acknowledgement of pySecDec's scientific and software contribution; FastSecDec implements its own Rust generation pipeline.".into(),
            relevance: None,
        },
        Citation {
            id: "doi:10.1016/j.cpc.2010.04.001".into(),
            url: "https://doi.org/10.1016/j.cpc.2010.04.001".into(),
            reference: "T. Kaneko and T. Ueda. A geometric method of sector decomposition. Computer Physics Communications 181 (2010), 1352–1361. doi:10.1016/j.cpc.2010.04.001.".into(),
            bibtex: include_str!("../../../citations/kaneko-ueda.bib").trim().into(),
            reasons: vec!["Geometric sector decomposition using polynomial exponent supports, normal cones and simplicial monomial maps.".into()],
            description: "Mathematical foundation of FastSecDec's native geometric sector construction.".into(),
            relevance: None,
        },
        Citation {
            id: "doi:10.1016/S0550-3213(00)00429-6".into(),
            url: "https://doi.org/10.1016/S0550-3213(00)00429-6".into(),
            reference: "T. Binoth and G. Heinrich. An automatized algorithm to compute infrared divergent multi-loop integrals. Nuclear Physics B 585 (2000), 741–759. doi:10.1016/S0550-3213(00)00429-6.".into(),
            bibtex: include_str!("../../../citations/binoth-heinrich.bib").trim().into(),
            reasons: vec!["Endpoint Taylor subtraction and Laurent coefficients of dimensionally regulated sector integrals.".into()],
            description: "Methodological foundation for separating endpoint poles from finite parameter integrals.".into(),
            relevance: None,
        },
    ];
    if used & THRESHOLD_DECOMPOSITION != 0 {
        references.extend([
            Citation {
                id: "arxiv:2506.24073".into(),
                url: "https://arxiv.org/abs/2506.24073".into(),
                reference: "S. Jones, A. Olsson and T. Stone. Positive Integrands from Feynman Integrals in the Minkowski Regime. arXiv:2506.24073 (2025).".into(),
                bibtex: include_str!("../../../citations/jones-olsson-stone-2025.bib").trim().into(),
                reasons: vec!["Threshold decomposition partitions causal Feynman-parameter densities into sign-definite regions with continued complex phases.".into()],
                description: "Methodological reference; complex numerators need not yield positive amplitude integrands.".into(),
                relevance: None,
            },
            Citation {
                id: "arxiv:2603.05444".into(),
                url: "https://arxiv.org/abs/2603.05444".into(),
                reference: "S. P. Jones, A. Olsson and T. Stone. Accelerating Feynman Integral Evaluation by Avoiding Contour Deformation. arXiv:2603.05444 (2026).".into(),
                bibtex: include_str!("../../../citations/jones-olsson-stone-2026.bib").trim().into(),
                reasons: vec!["The threshold recipe uses verified generic cylindrical algebraic decomposition to separate causal signs.".into()],
                description: "Reference for the GCAD construction; general algebraic endpoint resolution is separate FastSecDec work.".into(),
                relevance: None,
            },
            Citation {
                id: "https://github.com/alphal00p/symgcad".into(),
                url: "https://github.com/alphal00p/symgcad".into(),
                reference: "symGCAD contributors. symGCAD: Generic Cylindrical Algebraic Decomposition. https://github.com/alphal00p/symgcad.".into(),
                bibtex: include_str!("../../../citations/symgcad.bib").trim().into(),
                reasons: vec!["Native threshold preparation uses symGCAD solve and independent verification, or loads a saved recipe from that preparation.".into()],
                description: "Software repository citation; no publication DOI is assigned here.".into(),
                relevance: None,
            },
        ]);
    }
    references
}
