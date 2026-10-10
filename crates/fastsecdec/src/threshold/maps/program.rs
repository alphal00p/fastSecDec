use super::{GcadError, ParameterAdmission, Result};
use std::{collections::BTreeMap, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::{EvaluatorComposer, ExpressionEvaluator, OptimizationSettings, Slot},
};

/// One reusable native exact map program and an admitted parameter fiber.
/// Runtime parameter inputs remain present in the compiled arithmetic. Rebind
/// checks the new rational chamber point and shares the same immutable program.
#[derive(Clone, Debug)]
pub struct CompiledCellMap {
    admission: ParameterAdmission,
    program: Arc<ExpressionEvaluator<Complex<Rational>>>,
    input_symbols: Vec<Symbol>,
    parameter_count: usize,
    prepared_dimension: usize,
    original_dimension: usize,
}
impl CompiledCellMap {
    pub fn admission(&self) -> &ParameterAdmission {
        &self.admission
    }
    /// Ordered runtime parameters followed by unit coordinates. The parameter
    /// prefix must equal this owner's admitted values when evaluating its map.
    pub fn inputs(&self) -> &[Symbol] {
        &self.input_symbols
    }
    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }
    pub fn admitted_parameter_values(&self) -> impl ExactSizeIterator<Item = &Rational> {
        self.input_symbols[..self.parameter_count]
            .iter()
            .map(|s| &self.admission.values[s])
    }
    /// Outputs are prepared images, original input images, then one triangular
    /// positive measure. The native delta gauge contributes its certified one.
    /// This low-level native evaluator is unchecked: use this admission
    /// parameter prefix and an open-unit point. A raw evaluation is neither a
    /// chamber check nor a closed-face endpoint regularity certificate.
    pub fn program(&self) -> &ExpressionEvaluator<Complex<Rational>> {
        &self.program
    }
    pub fn prepared_dimension(&self) -> usize {
        self.prepared_dimension
    }
    pub fn original_dimension(&self) -> usize {
        self.original_dimension
    }
    pub fn measure_output(&self) -> usize {
        self.prepared_dimension + self.original_dimension
    }
    pub fn rebind(&self, values: &BTreeMap<Symbol, Rational>) -> Result<Self> {
        let admission = self.admission.map.admit_parameters(values)?;
        Ok(Self {
            admission,
            program: self.program.clone(),
            input_symbols: self.input_symbols.clone(),
            parameter_count: self.parameter_count,
            prepared_dimension: self.prepared_dimension,
            original_dimension: self.original_dimension,
        })
    }
}
impl ParameterAdmission {
    pub fn compile(&self, settings: OptimizationSettings) -> Result<CompiledCellMap> {
        let source = &self.map.source;
        let request = source.owner.request();
        let parameters = &request.kinematics().runtime_parameters;
        let parameter_count = parameters.len();
        let input_symbols = parameters
            .iter()
            .chain(&source.unit_coordinates)
            .copied()
            .collect::<Vec<_>>();
        // Horner acts on native expression trees, before linearization; final
        // Composer CSE/CPE cannot recover it. Preserve caller Horner settings in
        // every stage and defer only common-pair extraction to the final graph.
        let intermediate = settings.clone().cpe_iterations(Some(0));
        let build = |outputs: &[Atom], inputs: &[Atom]| {
            Atom::evaluator_multiple(outputs, inputs)
                .optimization_settings(intermediate.clone())
                .build()
                .map_err(|e| GcadError::Unsupported(e.to_string()))
        };
        let mut composer = EvaluatorComposer::new(input_symbols.len());
        let mut coordinate_slots = BTreeMap::new();
        let mut widths = Vec::with_capacity(source.unit_coordinates.len());
        let mut prefix_symbols = parameters
            .iter()
            .copied()
            .map(Atom::var)
            .collect::<Vec<_>>();
        let mut prefix_slots = (0..parameter_count).map(Slot::Param).collect::<Vec<_>>();
        for (index, axis) in source.axes.iter().enumerate() {
            let Some(unit_index) = axis.unit_index else {
                continue;
            };
            let bounds = &self.map.axes[index];
            let lower = bounds
                .lower
                .as_ref()
                .expect("finite linear lowering")
                .expression
                .clone();
            let upper = bounds
                .upper
                .as_ref()
                .expect("finite linear lowering")
                .expression
                .clone();
            let width = upper - &lower;
            let t = Atom::var(source.unit_coordinates[unit_index]);
            let mut inputs = prefix_symbols.clone();
            inputs.push(t.clone());
            let body = build(&[lower + &width * t, width], &inputs)?;
            let mut slots = prefix_slots.clone();
            slots.push(Slot::Param(parameter_count + unit_index));
            let outputs = composer
                .append(&body, &slots)
                .map_err(|e| GcadError::Unsupported(e.to_string()))?;
            coordinate_slots.insert(axis.symbol, outputs[0]);
            prefix_symbols.push(Atom::var(axis.symbol));
            prefix_slots.push(outputs[0]);
            widths.push(outputs[1]);
        }
        let prepared = request
            .domain()
            .coordinates()
            .iter()
            .map(|s| coordinate_slots[s])
            .collect::<Vec<_>>();
        let original = if let Some(preparation) = request.projective_preparation() {
            let inputs = preparation
                .coordinates()
                .iter()
                .copied()
                .map(Atom::var)
                .collect::<Vec<_>>();
            composer
                .append(&build(preparation.images(), &inputs)?, &prepared)
                .map_err(|e| GcadError::Unsupported(e.to_string()))?
        } else {
            prepared.clone()
        };
        // Native stage parameters are scalar placeholders, wired to each
        // previously computed width; no recursive symbolic image substitution.
        let width_inputs = source
            .unit_coordinates
            .iter()
            .copied()
            .map(Atom::var)
            .collect::<Vec<_>>();
        let width_product = width_inputs.iter().cloned().product::<Atom>();
        let measure = composer
            .append(&build(&[width_product], &width_inputs)?, &widths)
            .map_err(|e| GcadError::Unsupported(e.to_string()))?[0];
        let mut outputs = prepared;
        outputs.extend(original);
        outputs.push(measure);
        let program = composer
            .finish(&outputs, settings)
            .map_err(|e| GcadError::Unsupported(e.to_string()))?;
        Ok(CompiledCellMap {
            admission: self.clone(),
            program: Arc::new(program),
            input_symbols,
            parameter_count,
            prepared_dimension: request.domain().coordinates().len(),
            original_dimension: request.input().parameters().len(),
        })
    }
}
