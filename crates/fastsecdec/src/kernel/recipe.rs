//! Explicit recipe ownership for native v10 programs. Legacy payloads retain
//! their exact v9 codec and acquire no dynamic capability from parameter names.
use super::{KernelError, KernelSet, ProgramRecipe};
use crate::contour::functions::dynamic::RootProgram;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use symbolica::{atom::AtomCore, domains::rational::Rational};

mod certificates;
mod check_program;
mod check_source;
mod stored;
mod stored_v2;
pub(crate) use check_program::DynamicCheckProgram;
pub(crate) use check_source::{CoefficientInput, DynamicCheckOutput, DynamicCheckSource};
pub(crate) use stored::SavedProgramDescriptor;
pub(crate) use stored_v2::SavedProgramDescriptorV2;
#[cfg(test)]
mod tests;

fn invalid(reason: impl std::fmt::Display) -> KernelError {
    KernelError::Artifact(format!("native recipe descriptor: {reason}"))
}

/// A positive residual proof uses the existing exact native polynomial
/// coefficients, without substituting a numerical lower floor.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositiveFactorProof {
    NonnegativeCoefficients { lower_bound: Rational },
}

/// Full-sector structural data. These counts and helper arity are retained
/// unchanged when subtraction restricts coordinates to a face.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DynamicChartRecipe {
    pub chart_index: usize,
    pub dimension: usize,
    pub causal_orders: Vec<u32>,
    pub positive_orders: Vec<Vec<u32>>,
    pub positive_proofs: Vec<PositiveFactorProof>,
    pub maximum_even_order: u32,
    pub coefficient_count: usize,
    pub regularity: Rational,
    pub helper_digest: String,
}

impl DynamicChartRecipe {
    pub(crate) fn from_envelope(
        chart_index: usize,
        envelope: &crate::contour::dynamic::DynamicEnvelope,
        helper: &RootProgram,
    ) -> Result<Self, KernelError> {
        let value = Self {
            chart_index,
            dimension: envelope.parameters().len(),
            causal_orders: envelope
                .causal_terms()
                .iter()
                .map(|term| term.order())
                .collect(),
            positive_orders: envelope
                .positive_factors()
                .iter()
                .map(|factor| factor.terms().iter().map(|term| term.order()).collect())
                .collect(),
            positive_proofs: envelope
                .positive_factors()
                .iter()
                .map(|factor| {
                    Rational::try_from(factor.certified_lower_bound().as_view())
                        .map(|lower_bound| PositiveFactorProof::NonnegativeCoefficients {
                            lower_bound,
                        })
                        .map_err(invalid)
                })
                .collect::<Result<_, _>>()?,
            maximum_even_order: envelope.maximum_even_order(),
            coefficient_count: helper.coefficient_count(),
            regularity: Rational::try_from(envelope.regularity().as_view()).map_err(invalid)?,
            helper_digest: helper.digest().into(),
        };
        value.validate(helper)?;
        Ok(value)
    }

    fn validate(&self, helper: &RootProgram) -> Result<(), KernelError> {
        let orders_valid = |orders: &[u32], minimum, parity| {
            orders
                .iter()
                .all(|order| *order >= minimum && order % 2 == parity)
                && orders.windows(2).all(|pair| pair[0] < pair[1])
        };
        if (self.dimension == 0
            && (!self.causal_orders.is_empty()
                || self.positive_orders.iter().any(|orders| !orders.is_empty())))
            || !orders_valid(&self.causal_orders, 3, 1)
            || self
                .positive_orders
                .iter()
                .any(|orders| !orders_valid(orders, 2, 0))
            || self.positive_orders.len() != self.positive_proofs.len()
            || self.positive_proofs.iter().any(|proof| match proof {
                PositiveFactorProof::NonnegativeCoefficients { lower_bound } => *lower_bound <= 0,
            })
            || self.regularity <= 0
            || self.coefficient_count != helper.coefficient_count()
            || self.coefficient_count.checked_mul(2) != Some(self.maximum_even_order as usize)
            || self.helper_digest != helper.digest()
        {
            return Err(invalid(
                "invalid full-sector dynamic structure or helper association",
            ));
        }
        let required_order = self
            .causal_orders
            .iter()
            .map(|order| (order - 1).checked_mul(2))
            .chain(
                self.positive_orders
                    .iter()
                    .flatten()
                    .map(|order| order.checked_mul(2)),
            )
            .try_fold(2, |largest, order| order.map(|order| largest.max(order)))
            .ok_or_else(|| invalid("dynamic structural order overflow"))?;
        if self.maximum_even_order != required_order {
            return Err(invalid(
                "dynamic dense schema differs from the full-sector envelope orders",
            ));
        }
        Ok(())
    }
}

/// One selected mathematical recipe owns only its native helper programs.
/// Numerical workspaces are constructed by native evaluator restoration and
/// are independently cloned; this descriptor holds no process-global scratch.
#[derive(Clone)]
pub struct NativeProgramDescriptor {
    recipe: ProgramRecipe,
    charts: Vec<DynamicChartRecipe>,
    helpers: Vec<RootProgram>,
    exact_helpers: Vec<String>,
    certificates: Option<std::sync::Arc<[DynamicCheckProgram]>>,
}

impl std::fmt::Debug for NativeProgramDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeProgramDescriptor")
            .field("recipe", &self.recipe)
            .field("charts", &self.charts)
            .field("exact_helpers", &self.exact_helpers)
            .field(
                "certificate_count",
                &self.certificates.as_ref().map(|value| value.len()),
            )
            .field(
                "helpers",
                &self
                    .helpers
                    .iter()
                    .map(RootProgram::digest)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl NativeProgramDescriptor {
    pub(crate) fn root_helper(&self, digest: &str) -> Option<&RootProgram> {
        self.helpers.iter().find(|helper| helper.digest() == digest)
    }

    pub(crate) fn enter_optional(
        owner: Option<&Self>,
    ) -> crate::contour::functions::dynamic::ProgramPreparation {
        match owner {
            Some(owner) => owner.enter(),
            None => crate::contour::functions::dynamic::ProgramScope::default().enter(),
        }
    }

    pub(crate) fn enter(&self) -> crate::contour::functions::dynamic::ProgramPreparation {
        crate::contour::functions::dynamic::ProgramScope::new(&self.helpers).enter()
    }

    pub fn recipe(&self) -> ProgramRecipe {
        self.recipe
    }
    pub fn charts(&self) -> &[DynamicChartRecipe] {
        &self.charts
    }

    pub(crate) fn static_recipe(recipe: ProgramRecipe) -> Result<Self, KernelError> {
        if recipe.is_dynamic() {
            return Err(invalid(
                "dynamic recipes require their explicit native helper and chart descriptors",
            ));
        }
        Ok(Self {
            recipe,
            charts: Vec::new(),
            helpers: Vec::new(),
            exact_helpers: Vec::new(),
            certificates: None,
        })
    }

    pub(crate) fn dynamic(
        recipe: ProgramRecipe,
        mut charts: Vec<DynamicChartRecipe>,
        helpers: Vec<RootProgram>,
    ) -> Result<Self, KernelError> {
        if !recipe.is_dynamic() {
            return Err(invalid("dynamic descriptor has a non-dynamic recipe"));
        }
        charts.sort_by_key(|chart| chart.chart_index);
        let mut unique = BTreeMap::new();
        for helper in helpers {
            unique.entry(helper.digest().to_owned()).or_insert(helper);
        }
        let value = Self {
            recipe,
            charts,
            helpers: unique.into_values().collect(),
            exact_helpers: Vec::new(),
            certificates: None,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), KernelError> {
        self.validate_certificates()?;
        if !self.recipe.is_dynamic() {
            return if self.charts.is_empty()
                && self.helpers.is_empty()
                && self.exact_helpers.is_empty()
            {
                Ok(())
            } else {
                Err(invalid("static recipe contains dynamic owners"))
            };
        }
        if self
            .charts
            .windows(2)
            .any(|pair| pair[0].chart_index >= pair[1].chart_index)
        {
            return Err(invalid("duplicate or unordered dynamic chart identity"));
        }
        let mut used = BTreeSet::new();
        if self.exact_helpers.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(invalid("duplicate or unordered exact helper identities"));
        }
        for digest in &self.exact_helpers {
            let helper = self
                .helpers
                .iter()
                .find(|helper| helper.digest() == digest)
                .ok_or_else(|| invalid("missing exact-offset dynamic root helper"))?;
            used.insert(helper.digest());
        }
        for chart in &self.charts {
            let helper = self
                .helpers
                .iter()
                .find(|helper| helper.digest() == chart.helper_digest)
                .ok_or_else(|| invalid("missing dynamic root helper"))?;
            chart.validate(helper)?;
            used.insert(helper.digest());
        }
        for certificate in self.certificates().into_iter().flatten() {
            used.insert(certificate.structure.helper_digest.as_str());
        }
        if used.len() != self.helpers.len() {
            return Err(invalid("unreferenced or duplicate dynamic root helper"));
        }
        Ok(())
    }

    pub(crate) fn for_payload(
        &self,
        sources: &[usize],
        exact: &[symbolica::atom::Atom],
        metadata: Option<&crate::generation::GenerationMetadata>,
    ) -> Result<Self, KernelError> {
        self.for_payload_with_requests(sources, exact, metadata, &[])
    }

    pub(crate) fn for_payload_with_requests(
        &self,
        sources: &[usize],
        exact: &[symbolica::atom::Atom],
        metadata: Option<&crate::generation::GenerationMetadata>,
        exact_requests: &[crate::contour::functions::dynamic::requests::ExactRequest],
    ) -> Result<Self, KernelError> {
        if sources.iter().copied().collect::<BTreeSet<_>>().len() != sources.len() {
            return Err(invalid("duplicate requested source chart"));
        }
        let symbols = exact
            .iter()
            .flat_map(|atom| atom.get_all_symbols(true))
            .collect::<BTreeSet<_>>();
        let mut exact_helpers = self
            .helpers
            .iter()
            .filter(|helper| symbols.contains(&helper.tag()))
            .map(|helper| helper.digest().to_owned())
            .collect::<Vec<_>>();
        exact_helpers.sort();
        let mut charts = Vec::new();
        for (local, source) in sources.iter().enumerate() {
            if let Some(chart) = self
                .charts
                .iter()
                .find(|chart| chart.chart_index == *source)
            {
                let mut chart = chart.clone();
                chart.chart_index = local;
                charts.push(chart);
            } else if self.recipe.is_dynamic()
                && !metadata.is_some_and(|metadata| {
                    metadata.charts().iter().any(|chart| {
                        chart.source_index() == *source
                            && chart.source_index() != chart.representative()
                            && sources.contains(&chart.representative())
                            && self
                                .charts
                                .iter()
                                .any(|descriptor| descriptor.chart_index == chart.representative())
                    })
                })
            {
                return Err(invalid(
                    "selected source lacks its dynamic chart descriptor",
                ));
            }
        }
        let certificates = self.select_certificates(sources, exact, exact_requests)?;
        let helpers = self
            .helpers
            .iter()
            .filter(|helper| {
                charts
                    .iter()
                    .any(|chart| chart.helper_digest == helper.digest())
                    || exact_helpers.iter().any(|digest| digest == helper.digest())
                    || certificates
                        .as_deref()
                        .into_iter()
                        .flatten()
                        .any(|certificate| certificate.structure.helper_digest == helper.digest())
            })
            .cloned()
            .collect();
        let value = Self {
            recipe: self.recipe,
            charts,
            helpers,
            exact_helpers,
            certificates,
        };
        value.validate()?;
        Ok(value)
    }

    /// Associate generation-owned helpers with the actual retained chart set.
    /// Symmetry copies keep inspectable maps but share the representative's
    /// executable recipe and checker, just as in the fixed construction.
    pub(crate) fn validate_generation(
        &self,
        metadata: Option<&crate::generation::GenerationMetadata>,
        runtime: &[symbolica::atom::Symbol],
    ) -> Result<(), KernelError> {
        self.validate()?;
        self.validate_certificate_runtime(runtime)?;
        self.recipe.validate_runtime_schema(
            &runtime
                .iter()
                .map(|symbol| symbol.get_name().to_owned())
                .collect::<Vec<_>>(),
        )?;
        if !self.recipe.is_dynamic() {
            return Ok(());
        }
        let representatives = metadata
            .into_iter()
            .flat_map(|metadata| metadata.charts())
            .filter(|chart| chart.source_index() == chart.representative())
            .collect::<Vec<_>>();
        if representatives.len() != self.charts.len() {
            return Err(invalid(
                "dynamic descriptors do not cover the retained representative charts",
            ));
        }
        for descriptor in &self.charts {
            let chart = representatives
                .iter()
                .find(|chart| chart.source_index() == descriptor.chart_index)
                .ok_or_else(|| {
                    invalid("dynamic descriptor references an unknown representative chart")
                })?;
            let contour = chart
                .contour()
                .ok_or_else(|| invalid("dynamic descriptor references an undeformed chart"))?;
            if descriptor.dimension != chart.coordinates().target_parameters().len()
                || descriptor.dimension != contour.images().len()
                || descriptor.positive_orders.len() != contour.positive_polynomials().len()
            {
                return Err(invalid(
                    "dynamic descriptor dimensions or positive factors differ from its chart",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn admit_runtime(&self) -> Result<(), KernelError> {
        if self.recipe.is_dynamic() && self.certificates.is_none() {
            return Err(invalid(
                "dynamic runtime requires saved v11 certificates; regenerate this artifact",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_sources(
        &self,
        sources: &[std::sync::Arc<DynamicCheckSource>],
        metadata: Option<&crate::generation::GenerationMetadata>,
    ) -> Result<(), KernelError> {
        if sources.len() != self.charts.len() {
            return Err(invalid("missing or surplus dynamic check sources"));
        }
        let mut seen = BTreeSet::new();
        for source in sources {
            let chart = self
                .charts
                .iter()
                .find(|chart| chart.chart_index == source.chart_index)
                .ok_or_else(|| invalid("check source refers to an unknown dynamic chart"))?;
            if !seen.insert(source.chart_index) {
                return Err(invalid("duplicate dynamic check source"));
            }
            source.validate_for(chart)?;
            if source.recipe != self.recipe {
                return Err(invalid(
                    "check source belongs to a different mathematical recipe",
                ));
            }
            let helper = self
                .helpers
                .iter()
                .find(|helper| helper.digest() == chart.helper_digest)
                .ok_or_else(|| invalid("check source helper is missing"))?;
            let strength = source
                .full_strength
                .as_fun_view()
                .ok_or_else(|| invalid("check source strength is not a callback"))?;
            if strength.get(1) != symbolica::atom::Atom::var(helper.tag()).as_view() {
                return Err(invalid(
                    "check source strength belongs to a different helper",
                ));
            }
            if let Some(metadata) = metadata {
                let actual = metadata
                    .charts()
                    .iter()
                    .find(|chart| chart.source_index() == source.chart_index)
                    .ok_or_else(|| invalid("check source has no retained chart"))?;
                if actual.coordinates().target_parameters() != source.parameters {
                    return Err(invalid(
                        "check source coordinates differ from its retained chart",
                    ));
                }
                let contour = actual
                    .contour()
                    .ok_or_else(|| invalid("check source has no retained contour map"))?;
                let factors = check_source::FactorIdentity::new(
                    contour.causal_polynomial(),
                    contour.positive_polynomials(),
                )
                .map_err(invalid)?;
                if factors != source.factors {
                    return Err(invalid(
                        "check source causal/positive factors differ from its retained contour map",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn remap_charts(&mut self, sources: &[usize]) -> Result<(), KernelError> {
        for chart in &mut self.charts {
            chart.chart_index = *sources
                .get(chart.chart_index)
                .ok_or_else(|| invalid("invalid local dynamic chart index"))?;
        }
        self.remap_certificates(sources)?;
        self.validate()
    }

    pub(crate) fn merge(&mut self, other: Self) -> Result<(), KernelError> {
        if self.recipe != other.recipe {
            return Err(invalid("cannot merge different native recipes"));
        }
        self.merge_certificates(other.certificates.clone())?;
        self.charts.extend(other.charts);
        self.exact_helpers.extend(other.exact_helpers);
        self.exact_helpers.sort();
        self.exact_helpers.dedup();
        self.charts.sort_by_key(|chart| chart.chart_index);
        for helper in other.helpers {
            if !self
                .helpers
                .iter()
                .any(|existing| existing.digest() == helper.digest())
            {
                self.helpers.push(helper);
            }
        }
        self.helpers
            .sort_by(|left, right| left.digest().cmp(right.digest()));
        self.validate()
    }
}

impl KernelSet {
    pub(crate) fn attach_program_descriptor(
        &mut self,
        descriptor: Option<&std::sync::Arc<NativeProgramDescriptor>>,
    ) -> Result<(), KernelError> {
        if let Some(descriptor) = descriptor {
            descriptor.validate_generation(self.metadata.as_ref(), &self.runtime_parameters)?;
            self.program_descriptor = Some(descriptor.as_ref().clone());
        }
        Ok(())
    }
    pub fn program_descriptor(&self) -> Option<&NativeProgramDescriptor> {
        self.program_descriptor.as_ref()
    }

    /// Explicit native v10 identity, or the established legacy v9 classifier.
    pub fn program_recipe(&self) -> ProgramRecipe {
        self.program_descriptor.as_ref().map_or_else(
            || {
                if self
                    .runtime_parameters
                    .contains(&crate::contour::lambda_symbol())
                {
                    ProgramRecipe::FixedV1
                } else {
                    ProgramRecipe::UndeformedV1
                }
            },
            NativeProgramDescriptor::recipe,
        )
    }

    /// Declare the already-compiled fixed/undeformed recipe explicitly, using
    /// native v10 storage. This does not alter its mathematical expressions.
    /// Dynamic recipes require generation-owned helper/check descriptors.
    pub fn declare_program_recipe(&mut self, recipe: ProgramRecipe) -> Result<(), KernelError> {
        if self.template_content_id.is_some() || self.program_recipe() != recipe {
            return Err(invalid(
                "explicit recipe differs from the unbound compiled program",
            ));
        }
        self.program_descriptor = Some(NativeProgramDescriptor::static_recipe(recipe)?);
        self.portable_artifact = None;
        self.initialize_artifact()
    }
}
