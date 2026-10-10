//! Immutable executable threshold provenance. Constructors are crate-private;
//! plain lineage serde never grants native continuation authority.
use super::threshold_metadata as m;
use super::{KernelError, KernelSet};
use std::{collections::BTreeMap, sync::Arc};
use symbolica::{
    atom::{Atom, Symbol},
    domains::rational::Rational,
};

#[derive(Clone)]
pub struct ThresholdMetadata {
    manifest: m::LineageManifestV1,
    atoms: Vec<Atom>,
    symbols: Vec<Symbol>,
    records: Vec<m::RecordLineageV1>,
    resident: m::ResidentLineageV1,
    /// Compact canonical semantic projection/digests, excludes transport IDs
    /// and grouping. Global source/proof replay requires external evidence.
    semantic: String,
    orders: Vec<i32>,
}
#[derive(bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub(crate) struct Saved {
    descriptor: Vec<u8>,
    atoms: Vec<Atom>,
    symbols: Vec<Symbol>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    manifest: m::LineageManifestV1,
    records: Vec<m::RecordLineageV1>,
    resident: m::ResidentLineageV1,
    semantic: String,
    orders: Vec<i32>,
}
fn invalid(e: impl std::fmt::Display) -> KernelError {
    KernelError::Artifact(format!("threshold metadata: {e}"))
}
impl ThresholdMetadata {
    pub fn lineage(&self) -> &m::LineageManifestV1 {
        &self.manifest
    }
    pub fn result_scope(&self) -> ThresholdResultScope {
        ThresholdResultScope {
            source_extent: self.manifest.preparation.extent.clone(),
            selection: self.resident.selection.clone(),
        }
    }
    pub fn resident(&self) -> &m::ResidentLineageV1 {
        &self.resident
    }
    pub fn coefficient_orders(&self) -> &[i32] {
        &self.orders
    }
    pub fn full_original_scope(&self) -> Result<bool, KernelError> {
        self.manifest
            .validate_structure(&self.tables(), m::Limits::default())
            .map_err(invalid)?
            .validate_resident_records(
                &self.manifest.descriptor_digest().map_err(invalid)?,
                &self.resident,
                &self.records,
            )
            .map_err(invalid)
    }
    /// Local saved programs do not contain the global CAD/continuation evidence.
    /// Digest/structural load validation cannot satisfy global proof replay.
    pub fn require_global_proof(&self) -> Result<(), m::Error> {
        Err(m::Error::ProofUnavailable)
    }
    pub(crate) fn tables(&self) -> m::NativeTables {
        m::NativeTables {
            expression_records: BTreeMap::from([(
                self.manifest.preparation.expressions.clone(),
                self.atoms.len(),
            )]),
            symbols: self.symbols.len(),
        }
    }
    pub(crate) fn validate_layout(
        &self,
        coordinates: &[Vec<Symbol>],
        orders: &[i32],
        runtime: &[Symbol],
    ) -> Result<(), KernelError> {
        if !runtime.is_empty() || self.orders != orders {
            return Err(invalid("fixed fiber/layout differs"));
        }
        let checked = self
            .manifest
            .validate_structure(&self.tables(), m::Limits::default())
            .map_err(invalid)?;
        self.manifest
            .validate_initial_schema_kinds()
            .map_err(invalid)?;
        checked
            .validate_resident_records(
                &self.manifest.descriptor_digest().map_err(invalid)?,
                &self.resident,
                &self.records,
            )
            .map_err(invalid)?;
        let actual = self
            .records
            .iter()
            .filter_map(|r| match &r.kind {
                m::RecordKind::Stochastic { coordinates } => Some(
                    coordinates
                        .iter()
                        .map(|i| self.symbols[i.0])
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .collect::<Vec<_>>();
        if actual != coordinates {
            return Err(invalid("native kernel/lineage coordinate association"));
        }
        for binding in &self.manifest.preparation.fiber.bindings {
            Rational::try_from(self.atoms[binding.value.0].as_view()).map_err(invalid)?;
        }
        if self
            .symbols
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != self.symbols.len()
        {
            return Err(invalid("duplicate native role symbol"));
        }
        let _: serde_json::Value = serde_json::from_str(&self.semantic).map_err(invalid)?;
        Ok(())
    }
    pub(crate) fn visit_atoms(&self, f: &mut impl FnMut(&Atom)) {
        for a in &self.atoms {
            f(a)
        }
        for s in &self.symbols {
            f(&Atom::var(*s))
        }
    }
    pub(crate) fn semantic_identity(
        &self,
        compiler_policy: &str,
        precision: &super::PrecisionPolicy,
    ) -> Result<String, KernelError> {
        let mut h = blake3::Hasher::new();
        h.update(b"fastsecdec-threshold-native-v15\0");
        h.update(compiler_policy.as_bytes());
        serde_json::to_writer(&mut h, precision)?;
        h.update(self.semantic.as_bytes());
        serde_json::to_writer(&mut h, &self.orders)?;
        // Scope is semantic; physical record grouping/transport hashes are not.
        serde_json::to_writer(&mut h, &self.resident.selection)?;
        Ok(h.finalize().to_hex().to_string())
    }
    pub(crate) fn save(&self) -> Result<Saved, KernelError> {
        Ok(Saved {
            descriptor: serde_json::to_vec(&Descriptor {
                manifest: self.manifest.clone(),
                records: self.records.clone(),
                resident: self.resident.clone(),
                semantic: self.semantic.clone(),
                orders: self.orders.clone(),
            })?,
            atoms: self.atoms.clone(),
            symbols: self.symbols.clone(),
        })
    }
    pub(crate) fn restore(saved: Saved) -> Result<Self, KernelError> {
        let d: Descriptor = serde_json::from_slice(&saved.descriptor)?;
        Ok(Self {
            manifest: d.manifest,
            records: d.records,
            resident: d.resident,
            semantic: d.semantic,
            orders: d.orders,
            atoms: saved.atoms,
            symbols: saved.symbols,
        })
    }
}
impl KernelSet {
    pub fn threshold_metadata(&self) -> Option<&ThresholdMetadata> {
        self.threshold.as_deref()
    }
    pub(crate) fn attach_threshold(
        &mut self,
        metadata: ThresholdMetadata,
    ) -> Result<(), KernelError> {
        if self.metadata.is_some()
            || self.program_descriptor.is_some()
            || !self.contour_checks.is_empty()
        {
            return Err(invalid("mixed legacy/threshold provenance"));
        }
        metadata.validate_layout(
            &self
                .sectors
                .iter()
                .map(|s| s.parameters.clone())
                .collect::<Vec<_>>(),
            &self.coefficient_orders,
            &self.runtime_parameters,
        )?;
        self.threshold = Some(Arc::new(metadata));
        Ok(())
    }
}

#[cfg(feature = "threshold-decomposition")]
mod build {
    use super::*;
    use crate::{
        generation,
        threshold::{
            gcad::DomainOrigin,
            records::{StagedVector, VectorKind},
            regularization::BoundContinuation,
        },
    };
    use symbolica::atom::AtomCore;
    use symbolica::atom::{AtomView, FunctionBuilder};
    fn digest(domain: &str, value: &impl serde::Serialize) -> Result<m::Digest, KernelError> {
        let mut h = blake3::Hasher::new();
        h.update(domain.as_bytes());
        serde_json::to_writer(&mut h, value)?;
        Ok(m::Digest(h.finalize().to_hex().to_string()))
    }
    fn canonical(expression: &Atom, renames: &BTreeMap<Symbol, Symbol>) -> String {
        expression
            .replace_map_bottom_up(|node, _, out| match node {
                AtomView::Var(v) => {
                    if let Some(s) = renames.get(&v.get_symbol()) {
                        **out = Atom::var(*s)
                    }
                }
                AtomView::Fun(f) => {
                    if let Some(s) = renames.get(&f.get_symbol()) {
                        let mut b = FunctionBuilder::new(*s);
                        for a in f.iter() {
                            b = b.add_arg(a)
                        }
                        **out = b.finish();
                    }
                }
                _ => {}
            })
            .to_canonical_string()
    }
    impl ThresholdMetadata {
        pub(crate) fn from_bound(
            bound: &BoundContinuation<'_>,
            staged: &[StagedVector],
            orders: Vec<i32>,
        ) -> Result<Self, KernelError> {
            let certificate = bound.certificate();
            let request = certificate.decomposition().request();
            if request.domain().coordinates().len() != 1
                || staged.len() != certificate.charts().len()
            {
                return Err(invalid("unsupported or incomplete bound chart family"));
            }
            match request.domain().origin() {
                DomainOrigin::NativeUnitCube if request.input().parameters().len() == 1 => {}
                DomainOrigin::AffineProjective { .. }
                    if request.input().parameters().len() == 2 => {}
                _ => {
                    return Err(invalid(
                        "original atlas authority unavailable for this prepared domain",
                    ));
                }
            }
            let mut symbols = request.input().parameters().to_vec();
            symbols.push(request.input().regulator());
            let original = (0..request.input().parameters().len())
                .map(m::NativeSymbolId)
                .collect::<Vec<_>>();
            let regulator = m::NativeSymbolId(symbols.len() - 1);
            let mut physical = request.kinematics().runtime_parameters.clone();
            for s in request.kinematics().exact_values.keys() {
                if !physical.contains(s) {
                    physical.push(*s)
                }
            }
            let physical_ids = physical
                .iter()
                .map(|s| {
                    let i = symbols.len();
                    symbols.push(*s);
                    m::NativeSymbolId(i)
                })
                .collect::<Vec<_>>();
            let unit = m::NativeSymbolId(symbols.len());
            symbols.push(bound.coordinates()[0]);
            let mut atoms = Vec::new();
            let mut push = |a: Atom| {
                let i = atoms.len();
                atoms.push(a);
                m::NativeAtomId(i)
            };
            let one = push(Atom::one());
            let mut bindings = Vec::new();
            for (id, s) in physical_ids.iter().zip(&physical) {
                let value = request
                    .kinematics()
                    .exact_values
                    .get(s)
                    .or_else(|| certificate.parameters().get(s))
                    .ok_or_else(|| invalid("incomplete physical fiber"))?;
                bindings.push(m::RationalBinding {
                    symbol: *id,
                    value: push(Atom::num(value.clone())),
                });
            }
            let patch_images = if let Some(p) = request.projective_preparation() {
                p.images().to_vec()
            } else {
                request
                    .domain()
                    .coordinates()
                    .iter()
                    .map(|s| Atom::var(*s))
                    .collect()
            };
            let patch_ids = patch_images
                .iter()
                .cloned()
                .map(&mut push)
                .collect::<Vec<_>>();
            let mut chart_ids = Vec::new();
            for chart in certificate.charts() {
                chart_ids.push((
                    push(chart.image().clone()),
                    push(chart.positive_measure().clone()),
                ));
            }
            let source = m::Digest(
                generation::source_identity(
                    request.input(),
                    &request.kinematics().runtime_parameters,
                    &[],
                )
                .map_err(invalid)?,
            );
            let mut renames = BTreeMap::new();
            renames.insert(
                bound.coordinates()[0],
                symbolica::symbol!("fastsecdec::threshold_semantic::unit_0"),
            );
            // The certified bridge has one native numerator head. Tagged body
            // slots follow source-term/coefficient order; derivatives retain the
            // native DERIVATIVE head. Neither role is inferred from Symbol IDs.
            let mut ordinary = bound
                .definitions()
                .iter()
                .filter(|d| d.derivative_order().is_none());
            let numerator = ordinary
                .next()
                .ok_or_else(|| invalid("missing numerator role"))?
                .head();
            if ordinary.any(|d| d.head() != numerator) {
                return Err(invalid("multiple unassociated numerator roles"));
            }
            renames.insert(
                numerator,
                symbolica::symbol!("fastsecdec::threshold_semantic::numerator"),
            );
            let mut definitions = BTreeMap::new();
            for d in bound.definitions() {
                let role = match d.derivative_order() {
                    None if d.head() == numerator && d.tags().len() == 1 => {
                        (usize::try_from(d.tags()[0].as_view()).map_err(invalid)?, 0)
                    }
                    Some(order)
                        if d.head() == Symbol::DERIVATIVE
                            && d.tags().len() == 4
                            && d.tags()[0].is_zero()
                            && d.tags()[1] == Atom::num(order)
                            && d.tags()[2] == Atom::var(numerator) =>
                    {
                        (
                            usize::try_from(d.tags()[3].as_view()).map_err(invalid)?,
                            order,
                        )
                    }
                    _ => return Err(invalid("unassociated numerator/derivative definition")),
                };
                if definitions.insert(role, d).is_some() {
                    return Err(invalid("duplicate native definition role"));
                }
            }
            let definitions = definitions.into_values().collect::<Vec<_>>();
            let canonical_values = atoms
                .iter()
                .map(|a| canonical(a, &renames))
                .collect::<Vec<_>>();
            let expressions = digest("threshold-native-expression-values-v1", &canonical_values)?;
            let raw = digest(
                "threshold-native-solve-evidence-v1",
                certificate.decomposition().native_result(),
            )?;
            let problem = digest("threshold-native-problem-v1", request.problem())?;
            let continuation = digest(
                "threshold-epsilon-strip-v1",
                &(
                    certificate
                        .convergence_strip()
                        .lower()
                        .map(ToString::to_string),
                    certificate
                        .convergence_strip()
                        .upper()
                        .map(ToString::to_string),
                    &orders,
                ),
            )?;
            let coverage = digest(
                "threshold-interval-partition-v1",
                &certificate
                    .charts()
                    .iter()
                    .map(|c| {
                        (
                            c.interval().0.to_string(),
                            c.interval().1.to_string(),
                            c.orientation(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )?;
            let admission = digest(
                "threshold-fixed-fiber-v1",
                &physical
                    .iter()
                    .zip(&bindings)
                    .map(|(s, b)| {
                        (
                            Atom::var(*s).to_canonical_string(),
                            canonical_values[b.value.0].clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )?;
            let patch_coordinates = request
                .domain()
                .coordinates()
                .iter()
                .map(|s| m::NativeSymbolId(symbols.iter().position(|t| t == s).unwrap()))
                .collect::<Vec<_>>();
            let patch = m::Patch {
                id: m::PatchId(0),
                source: m::SourceOrigin::OriginalInput,
                map: m::MapDescriptor::RationalV1 {
                    geometry: m::MapGeometry {
                        expressions: expressions.clone(),
                        coordinates: patch_coordinates.clone(),
                        images: patch_ids,
                        positive_measure: one,
                        orientation: m::Orientation::Forward,
                        certificate: coverage.clone(),
                    },
                },
                coverage: coverage.clone(),
            };
            let options = bound.generation_options();
            let continuation_policy = serde_json::json!({"mode":options.mode,"subtraction":options.subtraction,"max_order":options.max_order,"max_subtractions_per_axis":options.max_subtractions_per_axis,"max_subtraction_terms":options.max_subtraction_terms,"coefficient_expansion":options.coefficient_expansion});
            let numerator_closure = definitions
                .iter()
                .map(|d| {
                    digest(
                        "threshold-canonical-function-v1",
                        &(
                            d.derivative_order(),
                            d.tags()
                                .iter()
                                .map(|a| canonical(a, &renames))
                                .collect::<Vec<_>>(),
                            canonical(d.body(), &renames),
                        ),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut cells = BTreeMap::new();
            let mut endpoint_charts = Vec::new();
            let mut contributions = Vec::new();
            for (index, (chart, staged)) in certificate.charts().iter().zip(staged).enumerate() {
                if staged.chart != index {
                    return Err(invalid("staged chart order"));
                }
                let raw_index = chart.admission().map().source().cell_index();
                cells.entry(raw_index).or_insert(m::Cell {
                    id: m::CellId(raw_index),
                    patch: m::PatchId(0),
                    native: m::NativeCellLocator {
                        request_record: problem.clone(),
                        raw_evidence: raw.clone(),
                        raw_cell_index: raw_index,
                    },
                    coordinate_order: patch_coordinates.clone(),
                    coverage: coverage.clone(),
                });
                endpoint_charts.push(m::EndpointChart{id:m::EndpointChartId(index),origin:m::EndpointOrigin::Cell(m::CellId(raw_index)),path:vec![index],map:m::MapDescriptor::RationalV1{geometry:m::MapGeometry {
                    expressions:expressions.clone(),coordinates:vec![unit],images:vec![chart_ids[index].0],positive_measure:chart_ids[index].1,orientation:if chart.orientation()>0{m::Orientation::Forward}else{m::Orientation::Reversed},certificate:coverage.clone(),
                }},endpoint_certificate:digest("threshold-native-closed-endpoint-v1",&serde_json::json!({"policy":continuation_policy,"strip":continuation,"numerator_closure":numerator_closure,"regular_epsilon":certificate.prefactor_witness().rational_witness().epsilon().to_string(),"gamma_witness":certificate.prefactor_witness().gamma_witnesses().iter().map(|g|(g.term(),g.factor(),g.argument_value().to_string(),g.pole_distance().map(ToString::to_string))).collect::<Vec<_>>(),"terms":(0..request.prepared_terms().len()).map(|term| {
                    let (prefactor,regular,power)=chart.normalized_term(term).expect("certified complete term");
                    serde_json::json!({"prefactor":canonical(prefactor,&renames),"regular":canonical(regular,&renames),"power":canonical(power,&renames),"units":chart.units(term).expect("certified units").iter().map(|u|serde_json::json!({"factor":u.factor_index(),"multiplicity":u.multiplicity(),"residual":canonical(u.residual(),&renames),"endpoints":[u.endpoint_values().0.to_string(),u.endpoint_values().1.to_string()]})).collect::<Vec<_>>()})
                }).collect::<Vec<_>>()}))?,causal_phase_certificate:digest("threshold-bound-phase-density-v1",&canonical(&bound.chart_expressions()[index],&renames))?});
                let kind = match staged.kind {
                    VectorKind::Stochastic {} => m::ContributionKind::Stochastic {
                        coordinates: vec![unit],
                    },
                    VectorKind::Exact {} => m::ContributionKind::Exact,
                    VectorKind::ZeroInLayout {} => m::ContributionKind::CertifiedZero {
                        certificate: digest(
                            "threshold-zero-in-requested-layout-v1",
                            &(&source, index, &orders),
                        )?,
                    },
                };
                contributions.push(m::Contribution {
                    id: m::ContributionId(index),
                    chart: m::EndpointChartId(index),
                    group: m::GroupId(0),
                    kind,
                });
            }
            let semantic = serde_json::to_string(
                &serde_json::json!({"source":source,"continuation_policy":continuation_policy,"strategy":"gcad_first","origin":format!("{:?}",request.domain().origin()),"domain":request.domain().strict_positive().iter().map(|a|canonical(a,&renames)).collect::<Vec<_>>(),"parameter_chamber":request.kinematics().strict_positive.iter().map(|a|canonical(a,&renames)).collect::<Vec<_>>(),"fiber":physical.iter().zip(&bindings).map(|(s,b)|(Atom::var(*s).to_canonical_string(),canonical_values[b.value.0].clone())).collect::<Vec<_>>(),"maps":canonical_values,"continued_charts":bound.chart_expressions().iter().map(|a|digest("threshold-canonical-continued-chart-v1",&canonical(a,&renames))).collect::<Result<Vec<_>,_>>()?,"functions":definitions.iter().map(|d|digest("threshold-canonical-function-v1",&(d.derivative_order(),d.tags().iter().map(|a|canonical(a,&renames)).collect::<Vec<_>>(),canonical(d.body(),&renames)))).collect::<Result<Vec<_>,_>>()?,"strip":[certificate.convergence_strip().lower().map(ToString::to_string),certificate.convergence_strip().upper().map(ToString::to_string)]}),
            )?;
            let manifest = m::LineageManifestV1 {
                version: 1,
                preparation: m::Preparation {
                    source_identity: source,
                    extent: m::SourceExtent::Full {
                        ordinary_geometry: None,
                    },
                    strategy: m::ResolvedStrategy::GcadFirst,
                    actual_options: digest(
                        "threshold-continuation-policy-v1",
                        &continuation_policy,
                    )?,
                    expressions,
                    partition_certificate: coverage,
                    original_coordinates: original,
                    regulator,
                    fiber: m::FixedRationalFiber {
                        physical_parameters: physical_ids,
                        bindings,
                        admission,
                    },
                },
                patches: vec![patch],
                cells: cells.into_values().collect(),
                endpoint_charts,
                continuation_groups: vec![m::ContinuationGroup {
                    id: m::GroupId(0),
                    charts: (0..staged.len()).map(m::EndpointChartId).collect(),
                    continuation: m::Continuation::EpsilonStripV1 {
                        certificate: continuation,
                    },
                }],
                contributions,
            };
            let descriptor = manifest.descriptor_digest().map_err(invalid)?;
            let records = manifest
                .contributions
                .iter()
                .filter_map(|c| {
                    let kind = match &c.kind {
                        m::ContributionKind::Stochastic { coordinates } => {
                            m::RecordKind::Stochastic {
                                coordinates: coordinates.clone(),
                            }
                        }
                        m::ContributionKind::Exact => m::RecordKind::Exact,
                        m::ContributionKind::CertifiedZero { .. } => return None,
                    };
                    Some(m::RecordLineageV1 {
                        manifest: descriptor.clone(),
                        contributions: vec![c.id],
                        kind,
                    })
                })
                .collect();
            Ok(Self {
                manifest,
                atoms,
                symbols,
                records,
                resident: m::ResidentLineageV1 {
                    manifest: descriptor,
                    selection: m::ResidentSelection::Complete,
                },
                semantic,
                orders,
            })
        }
    }
}

/// Inspection-only scope declaration retained by generic result manifests.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdResultScope {
    source_extent: m::SourceExtent,
    selection: m::ResidentSelection,
}
impl ThresholdResultScope {
    pub fn is_full_original(&self) -> bool {
        matches!(self.source_extent, m::SourceExtent::Full { .. })
            && matches!(self.selection, m::ResidentSelection::Complete)
    }
    pub fn source_extent(&self) -> &m::SourceExtent {
        &self.source_extent
    }
    pub fn selection(&self) -> &m::ResidentSelection {
        &self.selection
    }
    pub(crate) fn validate(&self) -> Result<(), KernelError> {
        if let m::SourceExtent::Selected { selection, .. } = &self.source_extent {
            selection.validate().map_err(invalid)?;
        }
        Ok(())
    }
}
