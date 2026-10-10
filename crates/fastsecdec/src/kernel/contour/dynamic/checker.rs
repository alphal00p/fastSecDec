//! Resident independent arithmetic for a captured production strength. No
//! callback, scalar solver, symbolic transformation or sampling RNG is used.
use super::{
    primitives,
    radius::{self, RadiusFailure, RadiusProof},
};
use crate::kernel::{
    KernelError, PositiveFactorProof,
    program::ExactProgram,
    recipe::{CoefficientInput, DynamicCheckOutput, DynamicCheckProgram},
};
use std::sync::Arc;
use symbolica::{
    domains::{
        float::{ComplexBall, Float, FloatLike, RealBall, SingleFloat},
        rational::Rational,
    },
    evaluate::ExpressionEvaluator,
};

#[derive(Clone)]
struct Limits {
    safety: Rational,
    cap: Rational,
    displacement: Rational,
    root_relative_width: Rational,
}

#[derive(Clone)]
enum Primitive {
    Cap,
    Displacement,
    Norm(usize),
    CausalSquared(usize),
    PositiveSquared {
        coefficient: usize,
        original: usize,
        proof: Rational,
    },
    CausalPositive {
        mean: usize,
        gap: usize,
    },
    PositivePositive(usize),
}

#[derive(Clone)]
pub(super) struct Check {
    pub source: Arc<DynamicCheckProgram>,
    raw: ExactProgram,
    coefficients: ExactProgram,
    level: ExactProgram,
    primitives: Vec<Primitive>,
    runtime: Vec<f64>,
    limits: Limits,
    numeric: Vec<Numeric>,
}

#[derive(Clone)]
struct Numeric {
    bits: u32,
    raw: ExpressionEvaluator<ComplexBall>,
    coefficients: ExpressionEvaluator<ComplexBall>,
    level: ExpressionEvaluator<ComplexBall>,
    input: Vec<ComplexBall>,
    output: Vec<ComplexBall>,
    primitive_input: Vec<ComplexBall>,
    coefficients_output: Vec<ComplexBall>,
    level_input: Vec<ComplexBall>,
    level_output: [ComplexBall; 2],
}

impl Check {
    /// The saved helper describes H-1 and H'. It is reused as native arithmetic
    /// with independently enclosed coefficients, never called as a root solver.
    pub(super) fn prepare(
        source: Arc<DynamicCheckProgram>,
        level: ExactProgram,
        runtime: Vec<f64>,
        deformation: crate::contour::ContourMode,
        root_relative_width: Rational,
    ) -> Result<Self, KernelError> {
        source.validate()?;
        let crate::contour::ContourMode::Dynamical {
            safety_fraction,
            lambda_cap,
            displacement_cap,
            ..
        } = deformation
        else {
            return Err(KernelError::Contour(
                "dynamic certificate requires dynamical settings".into(),
            ));
        };
        crate::contour::ContourSettings {
            deformation,
            ..Default::default()
        }
        .validate()
        .map_err(KernelError::Parameters)?;
        if source.recipe != deformation.program_recipe()
            || runtime.len() != source.runtime_parameters.len()
            || runtime.iter().any(|value| !value.is_finite())
            || root_relative_width <= 0
        {
            return Err(KernelError::Contour(
                "dynamic certificate binding differs from its mathematical recipe".into(),
            ));
        }
        for (name, expected) in source.recipe.recipe_parameters().iter().zip([
            safety_fraction,
            lambda_cap,
            displacement_cap,
        ]) {
            let index = source
                .runtime_parameters
                .iter()
                .position(|value| value == *name)
                .ok_or_else(|| {
                    KernelError::Contour("dynamic certificate lacks a recipe parameter".into())
                })?;
            if runtime[index] != expected {
                return Err(KernelError::Contour(
                    "dynamic certificate settings differ from the bound runtime inputs".into(),
                ));
            }
        }
        let rational = |value| Float::with_val(53, value).to_rational();
        let limits = Limits {
            safety: rational(safety_fraction),
            cap: rational(lambda_cap),
            displacement: rational(displacement_cap),
            root_relative_width,
        };
        let (raw, coefficients) = source.restore_arithmetic()?;
        crate::kernel::contour::admit_certified_arithmetic(&level.export_instructions())?;
        if level.get_input_len() != source.structure.coefficient_count + 1
            || level.get_output_len() != 2
        {
            return Err(KernelError::Contour(
                "certificate root-level program has a different schema".into(),
            ));
        }
        let slot = |kind: DynamicCheckOutput| {
            source
                .schema
                .iter()
                .position(|value| *value == kind)
                .ok_or_else(|| {
                    KernelError::Contour(
                        "certificate primitive is absent from the saved schema".into(),
                    )
                })
        };
        let primitives =
            source
                .coefficient_schema
                .iter()
                .map(|kind| {
                    Ok(match *kind {
                        CoefficientInput::LambdaCap => Primitive::Cap,
                        CoefficientInput::DisplacementCap => Primitive::Displacement,
                        CoefficientInput::DirectionNormSquared => {
                            Primitive::Norm(slot(DynamicCheckOutput::DirectionNormSquared)?)
                        }
                        CoefficientInput::CausalSquared { order } => Primitive::CausalSquared(
                            slot(DynamicCheckOutput::CausalSquaredBound { order })?,
                        ),
                        CoefficientInput::PositiveSquared { factor, order } => {
                            let PositiveFactorProof::NonnegativeCoefficients { lower_bound } =
                                &source.structure.positive_proofs[factor];
                            Primitive::PositiveSquared {
                                coefficient: slot(DynamicCheckOutput::PositiveRayCoefficient {
                                    factor,
                                    order,
                                })?,
                                original: slot(DynamicCheckOutput::PositiveOriginal { factor })?,
                                proof: lower_bound.clone(),
                            }
                        }
                        CoefficientInput::CausalPositive { order } => Primitive::CausalPositive {
                            mean: slot(DynamicCheckOutput::SpectralMean { order })?,
                            gap: slot(DynamicCheckOutput::SpectralGapSquared { order })?,
                        },
                        CoefficientInput::PositivePositive { factor, order } => {
                            Primitive::PositivePositive(slot(
                                DynamicCheckOutput::PositiveHarmfulCoefficient { factor, order },
                            )?)
                        }
                    })
                })
                .collect::<Result<_, KernelError>>()?;
        Ok(Self {
            source,
            raw,
            coefficients,
            level,
            primitives,
            runtime,
            limits,
            numeric: Vec::new(),
        })
    }

    #[cfg(test)]
    pub(super) fn evaluate(
        &mut self,
        coordinates: &[f64],
        face: &[(usize, u8)],
        candidate: &Rational,
        bits: u32,
    ) -> Result<RadiusProof, RadiusFailure> {
        self.evaluate_projected(
            &coordinates.iter().copied().map(Some).collect::<Vec<_>>(),
            face,
            candidate,
            bits,
        )
    }

    /// A missing reduced-coordinate projection means the complete closed cube,
    /// not an invented sample. The saved checker must certify it uniformly.
    pub(super) fn evaluate_projected(
        &mut self,
        coordinates: &[Option<f64>],
        face: &[(usize, u8)],
        candidate: &Rational,
        bits: u32,
    ) -> Result<RadiusProof, RadiusFailure> {
        let limits = &self.limits;
        if bits < 53
            || coordinates.len() != self.source.coordinates.len()
            || coordinates.iter().any(|value| {
                value.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            })
            || face
                .iter()
                .any(|(axis, value)| *axis >= coordinates.len() || *value > 1)
            || face.windows(2).any(|pair| pair[0].0 >= pair[1].0)
            || candidate <= &0
            || limits.safety <= 0
            || limits.safety >= 1
            || limits.cap <= 0
            || limits.displacement <= 0
        {
            return Err(RadiusFailure::Invalid(
                "invalid physical candidate/check inputs",
            ));
        }
        let index = if let Some(index) = self.numeric.iter().position(|entry| entry.bits == bits) {
            index
        } else {
            // Bounded per-resident cache. No native IR traversal or mapping is
            // repeated for a precision still resident in this four-entry cache.
            if self.numeric.len() == 4 {
                self.numeric.remove(0);
            }
            self.numeric.push(Numeric::new(
                &self.raw,
                &self.coefficients,
                &self.level,
                &self.runtime,
                bits,
            ));
            self.numeric.len() - 1
        };
        let numeric = &mut self.numeric[index];
        for (target, value) in numeric.input.iter_mut().zip(coordinates) {
            *target = complex(
                match value {
                    Some(value) => RealBall::exact(Float::with_val(bits, *value)),
                    None => {
                        RealBall::from_rational_bounds(&Rational::from(0), &Rational::from(1), bits)
                    }
                },
                bits,
            );
        }
        for (axis, value) in face {
            numeric.input[*axis] = complex(exact(&Rational::from(*value), bits), bits);
        }
        *numeric.input.last_mut().unwrap() = complex(exact(candidate, bits), bits);
        numeric.raw.evaluate(&numeric.input, &mut numeric.output);
        for (target, primitive) in numeric.primitive_input.iter_mut().zip(&self.primitives) {
            let value = match primitive {
                Primitive::Cap => exact(&limits.cap, bits),
                Primitive::Displacement => exact(&limits.displacement, bits),
                Primitive::Norm(index) | Primitive::CausalSquared(index) => {
                    primitives::nonnegative(&real(&numeric.output[*index])?)?
                }
                Primitive::PositiveSquared {
                    coefficient,
                    original,
                    proof,
                } => primitives::normalized_square(
                    &real(&numeric.output[*coefficient])?,
                    &real(&numeric.output[*original])?,
                    proof,
                )?,
                Primitive::CausalPositive { mean, gap } => primitives::spectral_positive(
                    &real(&numeric.output[*mean])?,
                    &real(&numeric.output[*gap])?,
                    &self.source.structure.regularity,
                )?,
                Primitive::PositivePositive(index) => primitives::smooth_positive(
                    &real(&numeric.output[*index])?,
                    &self.source.structure.regularity,
                )?,
            };
            *target = complex(value, bits);
        }
        numeric
            .coefficients
            .evaluate(&numeric.primitive_input, &mut numeric.coefficients_output);
        for (index, (target, value)) in numeric.level_input[1..]
            .iter_mut()
            .zip(&numeric.coefficients_output)
            .enumerate()
        {
            *target = complex(
                primitives::with_proved_lower_bound(
                    &real(value)?,
                    &Rational::from(u8::from(index == 0)),
                )?,
                bits,
            );
        }
        numeric.level_input[0] = complex(exact(&(candidate / &limits.cap), bits), bits);
        numeric
            .level
            .evaluate(&numeric.level_input, &mut numeric.level_output);
        let actual_level = real(&numeric.level_output[0])? + exact(&Rational::from(1), bits);
        let intended_candidate = candidate / &(&limits.safety * &limits.cap);
        numeric.level_input[0] = complex(exact(&intended_candidate, bits), bits);
        numeric
            .level
            .evaluate(&numeric.level_input, &mut numeric.level_output);
        let intended_level = real(&numeric.level_output[0])? + exact(&Rational::from(1), bits);
        // Radius certification also controls every retained U branch. A bare
        // F=A=0 does not diagnose a surviving pole after exact density
        // cancellation; genuine numerical failures receive that diagnosis at
        // the caller's failure boundary, with this retained full context.
        radius::certify(
            &intended_candidate,
            &actual_level,
            &intended_level,
            &limits.root_relative_width,
            bits,
        )
    }
}

impl Numeric {
    fn new(
        raw: &ExactProgram,
        coefficients: &ExactProgram,
        level: &ExactProgram,
        runtime: &[f64],
        bits: u32,
    ) -> Self {
        let zero = complex(exact(&Rational::from(0), bits), bits);
        let map = |program: &ExactProgram| {
            program.clone().map_coeff_with_prec(
                &|value| ComplexBall::from_rational_ball(value, &Rational::from(0), bits),
                bits,
            )
        };
        let mut input = vec![zero.clone(); raw.get_input_len()];
        let dimension = input.len() - runtime.len() - 1;
        for (target, value) in input[dimension..].iter_mut().zip(runtime) {
            *target = complex(RealBall::exact(Float::with_val(bits, *value)), bits);
        }
        Self {
            bits,
            raw: map(raw),
            coefficients: map(coefficients),
            level: map(level),
            input,
            output: vec![zero.clone(); raw.get_output_len()],
            primitive_input: vec![zero.clone(); coefficients.get_input_len()],
            coefficients_output: vec![zero.clone(); coefficients.get_output_len()],
            level_input: vec![zero.clone(); level.get_input_len()],
            level_output: [zero.clone(), zero],
        }
    }
}
fn exact(value: &Rational, bits: u32) -> RealBall {
    RealBall::from_rational_bounds(value, value, bits)
}
fn complex(value: RealBall, bits: u32) -> ComplexBall {
    ComplexBall::new(value, exact(&Rational::from(0), bits))
}
fn real(value: &ComplexBall) -> Result<RealBall, RadiusFailure> {
    if !value.im.is_fully_zero() || !value.re.is_finite() {
        return Err(RadiusFailure::Unresolved(
            "primitive has unresolved nonreal/nonfinite enclosure",
        ));
    }
    Ok(value.re.clone())
}

#[cfg(test)]
mod tests;
