//! Independent saved arithmetic programs for dynamic contour certificates.
//! Production callbacks supply the candidate; these programs never solve for
//! it, and their numerical mapping belongs to the resident checked owner.
use super::check_source::{CoefficientInput, DynamicCheckOutput, FactorIdentity};
use super::{DynamicChartRecipe, DynamicCheckSource, KernelError, ProgramRecipe, invalid};
use crate::{
    contour::ContourMetadata,
    kernel::{CompilationSettings, program},
};
use serde::{Deserialize, Serialize};
use symbolica::atom::{Atom, AtomCore, Symbol};

/// A namespace is mathematical and immutable; chart_index is a record-local
/// reporting projection. Both are needed when selected records are merged.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DynamicCheckProgram {
    /// Local metadata projection, absent for a context retained solely by an
    /// exact contribution originating in another source chart.
    pub chart_index: Option<usize>,
    pub recipe: ProgramRecipe,
    /// Self-contained full-sector proof structure. Its chart_index is always
    /// zero; the optional reporting projection above is deliberately separate.
    pub structure: DynamicChartRecipe,
    pub namespace: String,
    pub factors: FactorIdentity,
    pub coordinates: Vec<String>,
    pub runtime_parameters: Vec<String>,
    pub faces: Vec<Vec<(usize, u8)>>,
    pub schema: Vec<DynamicCheckOutput>,
    pub raw_program: Vec<u8>,
    pub coefficient_schema: Vec<CoefficientInput>,
    pub coefficient_program: Vec<u8>,
    pub unsupported: Option<String>,
}

impl DynamicCheckProgram {
    pub(crate) fn build(
        source: &DynamicCheckSource,
        chart: &DynamicChartRecipe,
        contour: &ContourMetadata,
        runtime: &[Symbol],
        settings: CompilationSettings,
    ) -> Result<Self, KernelError> {
        source.validate_for(chart)?;
        let factors =
            FactorIdentity::new(contour.causal_polynomial(), contour.positive_polynomials())
                .map_err(invalid)?;
        if factors != source.factors {
            return Err(invalid(
                "dynamic check compiler received a different causal/positive map",
            ));
        }
        let mut raw_inputs = source
            .parameters
            .iter()
            .chain(runtime)
            .copied()
            .collect::<Vec<_>>();
        let lambda = crate::contour::lambda_symbol();
        if raw_inputs.contains(&lambda) {
            return Err(invalid(
                "independent certificate strength overlaps a production input",
            ));
        }
        raw_inputs.push(lambda);
        if raw_inputs
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != raw_inputs.len()
        {
            return Err(invalid(
                "dynamic check input coordinates and runtime schema overlap",
            ));
        }
        let build = |outputs: &[Atom], inputs: &[Symbol]| {
            Atom::evaluator_multiple(
                outputs,
                &inputs.iter().copied().map(Atom::var).collect::<Vec<_>>(),
            )
            .optimization_settings(settings.native())
            .build()
            .map_err(|error| KernelError::Compilation(error.to_string()))
        };
        let raw = build(&source.outputs, &raw_inputs)?;
        let coefficients = build(
            &source.coefficients.outputs,
            &source.coefficients.parameters,
        )?;
        let unsupported = [&raw, &coefficients].into_iter().find_map(|program| {
            crate::kernel::contour::admit_certified_arithmetic(&program.export_instructions())
                .err()
                .map(|error| error.to_string())
        });
        let mut structure = chart.clone();
        structure.chart_index = 0;
        let value = Self {
            chart_index: Some(source.chart_index),
            recipe: source.recipe,
            structure,
            namespace: source.namespace.clone(),
            factors,
            coordinates: source
                .parameters
                .iter()
                .map(|symbol| Atom::var(*symbol).to_canonical_string())
                .collect(),
            runtime_parameters: runtime
                .iter()
                .map(|symbol| symbol.get_name().to_owned())
                .collect(),
            faces: contour.validation_faces().to_vec(),
            schema: source.schema.clone(),
            raw_program: program::encode(&raw)?,
            coefficient_schema: source.coefficients.schema.clone(),
            coefficient_program: program::encode(&coefficients)?,
            unsupported,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validate compact source associations without decoding programs. Checked
    /// numerical preparation additionally admits the native arithmetic IR and
    /// verifies its exact input/output lengths before mapping it to balls.
    pub(crate) fn validate(&self) -> Result<(), KernelError> {
        let chart = &self.structure;
        if !self.recipe.is_dynamic()
            || chart.chart_index != 0
            || self.coordinates.len() != chart.dimension
            || self.raw_program.is_empty()
            || self.coefficient_program.is_empty()
            || !super::check_source::identity::valid_digest(&self.namespace)
            || self.faces.is_empty()
            || self
                .coordinates
                .iter()
                .chain(&self.runtime_parameters)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.coordinates.len() + self.runtime_parameters.len()
        {
            return Err(invalid("invalid saved dynamic check source association"));
        }
        self.recipe
            .validate_runtime_schema(&self.runtime_parameters)?;
        self.factors
            .validate(chart.positive_orders.len())
            .map_err(invalid)?;
        let coordinates = self
            .coordinates
            .iter()
            .map(|name| Symbol::parse(name, "fastsecdec::artifact").map_err(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let runtime = self
            .runtime_parameters
            .iter()
            .map(|name| Symbol::parse(name, "fastsecdec::artifact").map_err(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        if coordinates
            .iter()
            .chain(&runtime)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != coordinates.len() + runtime.len()
        {
            return Err(invalid(
                "saved certificate coordinates and runtime symbols overlap",
            ));
        }
        if super::check_source::identity::namespace_for_chart(
            self.recipe,
            &coordinates,
            &self.factors,
            chart,
        )
        .map_err(invalid)?
            != self.namespace
        {
            return Err(invalid(
                "saved certificate namespace differs from its mathematical context",
            ));
        }
        let mut unique = std::collections::BTreeSet::new();
        for face in &self.faces {
            if !unique.insert(face)
                || face
                    .iter()
                    .any(|(axis, value)| *axis >= chart.dimension || *value > 1)
                || face.windows(2).any(|pair| pair[0].0 >= pair[1].0)
            {
                return Err(invalid("invalid saved full-sector face request"));
            }
        }
        if self.schema != super::check_source::output_schema(chart)
            || self.coefficient_schema
                != super::check_source::coefficient_schema(chart, self.recipe)?
        {
            return Err(invalid(
                "saved dynamic primitive schema differs from descriptor",
            ));
        }
        Ok(())
    }

    /// A repeated mathematical namespace may have distinct native programs
    /// and local projections. Every retained instance must still describe the
    /// same canonical full-sector proof; runtime checks each instance.
    pub(crate) fn same_mathematical_context(&self, other: &Self) -> bool {
        let mut left = self.structure.clone();
        let mut right = other.structure.clone();
        left.helper_digest.clear();
        right.helper_digest.clear();
        self.recipe == other.recipe
            && self.namespace == other.namespace
            && self.factors == other.factors
            && self.coordinates == other.coordinates
            && self.runtime_parameters == other.runtime_parameters
            && self.schema == other.schema
            && self.coefficient_schema == other.coefficient_schema
            && left == right
    }

    pub(crate) fn validate_metadata(
        &self,
        metadata: &crate::generation::GenerationMetadata,
    ) -> Result<(), KernelError> {
        self.validate()?;
        let Some(index) = self.chart_index else {
            return Ok(());
        };
        let chart = metadata
            .charts()
            .iter()
            .find(|chart| chart.source_index() == index)
            .ok_or_else(|| invalid("saved certificate projects to an unknown retained chart"))?;
        let contour = chart
            .contour()
            .ok_or_else(|| invalid("saved certificate projects to an undeformed chart"))?;
        let coordinates = chart
            .coordinates()
            .target_parameters()
            .iter()
            .map(|symbol| Atom::var(*symbol).to_canonical_string())
            .collect::<Vec<_>>();
        if self.coordinates != coordinates
            || self.factors
                != FactorIdentity::new(contour.causal_polynomial(), contour.positive_polynomials())
                    .map_err(invalid)?
            || self.faces != contour.validation_faces()
        {
            return Err(invalid(
                "saved certificate differs from its retained causal factors or face requests",
            ));
        }
        Ok(())
    }

    pub(crate) fn restore_arithmetic(
        &self,
    ) -> Result<(program::ExactProgram, program::ExactProgram), KernelError> {
        if let Some(reason) = &self.unsupported {
            return Err(KernelError::Contour(format!(
                "certified dynamic validation unavailable: {reason}"
            )));
        }
        let raw = program::decode(&self.raw_program)?;
        let coefficients = program::decode(&self.coefficient_program)?;
        for program in [&raw, &coefficients] {
            crate::kernel::contour::admit_certified_arithmetic(&program.export_instructions())?;
        }
        if raw.get_input_len() != self.coordinates.len() + self.runtime_parameters.len() + 1
            || raw.get_output_len() != self.schema.len()
            || coefficients.get_input_len() != self.coefficient_schema.len()
            || coefficients.get_output_len() != self.structure.coefficient_count
        {
            return Err(invalid(
                "saved dynamic check program input/output layout differs",
            ));
        }
        Ok((raw, coefficients))
    }
}
