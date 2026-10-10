//! Borrowed presentation of retained native records, without rebuilding maps.
use super::GenerationMetadata;
use std::fmt;
use symbolica::atom::{Atom, AtomCore};

pub struct MetadataView<'a> {
    metadata: &'a GenerationMetadata,
    expressions: bool,
}

impl GenerationMetadata {
    pub fn display(&self, expressions: bool) -> MetadataView<'_> {
        MetadataView {
            metadata: self,
            expressions,
        }
    }
}

impl fmt::Display for MetadataView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let domain = self.metadata.domain_assessment();
        writeln!(
            f,
            "Retained domain: {:?}; branch {:?}",
            domain.domain(),
            domain.branch_policy()
        )?;
        if let Some(scope) = self.metadata.source_scope() {
            writeln!(
                f,
                "Partial original integral: source charts {:?} of {}; retained chart lineage {:?}",
                scope.selection().source_sectors(),
                scope.selection().original_source_count(),
                scope.chart_source_sectors()
            )?;
        }
        if domain.branch_policy() == super::BranchPolicy::UserResponsible {
            writeln!(
                f,
                "Threshold regularity: caller responsibility; no certification performed."
            )?;
        } else {
            writeln!(
                f,
                "Historical caller assertion: {}",
                domain.caller_asserted()
            )?;
        }
        for factor in domain.factors() {
            write!(
                f,
                "  Term {} factor {}: {:?}",
                factor.term_index(),
                factor.factor_index(),
                factor.certificate()
            )?;
            if self.expressions {
                write!(
                    f,
                    "  ({})^({})",
                    factor.polynomial().to_canonical_string(),
                    factor.exponent().to_canonical_string()
                )?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Chart   Representative   Kernel   Gauge-fixed parameter")?;
        for chart in self.metadata.charts() {
            let map = chart.coordinates();
            writeln!(
                f,
                "{:<7} {:<16} {:<8} {:?}",
                chart.source_index(),
                chart.representative(),
                chart
                    .kernel_sector()
                    .map_or_else(|| "none".into(), |id| id.to_string()),
                map.projective_fixed_parameter()
            )?;
            writeln!(
                f,
                "  Representative permutation: {:?}",
                chart.representative_permutation()
            )?;
            let geometry = chart.geometry();
            writeln!(
                f,
                "  Exponent matrix: {:?}; |det| {}; measure powers {:?}",
                geometry.exponent_matrix, geometry.determinant, geometry.jacobian_powers
            )?;
            for (index, valuation) in geometry.factor_valuations.iter().enumerate() {
                writeln!(f, "  Support {index} valuation: {valuation:?}")?;
            }
            if let Some(record) = chart.pre_subtraction() {
                let minimum = record
                    .terms()
                    .iter()
                    .flat_map(|term| term.powers())
                    .map(|power| power.constant())
                    .min();
                let maximum = record
                    .terms()
                    .iter()
                    .flat_map(|term| term.powers())
                    .map(|power| power.subtraction_count())
                    .max()
                    .unwrap_or(0);
                writeln!(
                    f,
                    "  Pre-subtraction v{}: {} mapped terms; minimum endpoint constant {}; maximum Taylor count {}",
                    record.version(),
                    record.terms().len(),
                    minimum.map_or_else(|| "none".into(), ToString::to_string),
                    maximum
                )?;
                if self.expressions {
                    for (index, term) in record.terms().iter().enumerate() {
                        writeln!(
                            f,
                            "    Term {index} prefactor: {}; regular Atom storage {} bytes (body not retained)",
                            term.prefactor().to_canonical_string(),
                            term.regular_expression_bytes()
                        )?;
                        for (parameter, power) in map.target_parameters().iter().zip(term.powers())
                        {
                            writeln!(
                                f,
                                "      {}^({}); b={}, c={}, Taylor count={}",
                                Atom::var(*parameter).to_canonical_string(),
                                power.exponent().to_canonical_string(),
                                power.constant(),
                                power.slope(),
                                power.subtraction_count()
                            )?;
                        }
                    }
                }
            } else {
                writeln!(f, "  Pre-subtraction factors unavailable (older artifact).")?;
            }
            if self.expressions {
                for (source, image) in map.source_parameters().iter().zip(map.images()) {
                    writeln!(
                        f,
                        "  {} = {}",
                        Atom::var(*source).to_canonical_string(),
                        image.to_canonical_string()
                    )?;
                }
                writeln!(
                    f,
                    "  Positive real measure: {}",
                    map.measure_jacobian().to_canonical_string()
                )?;
            }
        }
        writeln!(
            f,
            "Kernel none: exact, cancelled or truncated; no per-chart exact coefficient is retained."
        )?;
        writeln!(
            f,
            "Projective maps are gauge-fixed; the measure is a positive real density factor."
        )?;
        writeln!(
            f,
            "Pre-subtraction terms precede symmetry multiplicity; Taylor requirements are not surviving pole counts."
        )?;
        write!(
            f,
            "Valuation indices follow deduplicated support order; they are not unique named U/F factors."
        )
    }
}
