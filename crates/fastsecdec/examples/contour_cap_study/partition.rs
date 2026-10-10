//! Research-only attribution of the retained base chart's sufficient radius.
//! Native Symbolica owns the envelope, derivatives, solver and determinant.
//! This does not replace a production callback or identify a subtraction face.
use fastsecdec::{
    contour::{
        DynamicConstruction,
        dynamic::{
            DynamicEnvelope, displacement_cap_symbol, lambda_cap_symbol, radius_fraction_symbol,
        },
    },
    generation::ChartRecord,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::{Complex, Float, FloatField, RealLike},
    evaluate::ExpressionEvaluator,
    solve::{BracketedRootConvergence, BracketedRootOptions, nsolve_bracketed},
    tensors::matrix::Matrix,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(super) struct PreparedPartition {
    metadata: Value,
    dimension: usize,
    positive_count: usize,
    evaluator: ExpressionEvaluator<Complex<f64>>,
    factors: ExpressionEvaluator<Complex<f64>>,
    output_count: usize,
}

impl PreparedPartition {
    pub(super) fn new(
        chart: &ChartRecord,
        physics: &BTreeMap<Symbol, f64>,
        construction: DynamicConstruction,
    ) -> Result<Self> {
        let contour = chart.contour().ok_or("chart has no retained contour")?;
        let parameters = chart.coordinates().target_parameters();
        let dimension = parameters.len();
        if dimension == 0 || contour.images().len() != dimension {
            return Err("partition needs a nonempty complete native chart".into());
        }
        if parameters
            .iter()
            .any(|parameter| physics.contains_key(parameter))
        {
            return Err("physical inputs overlap chart coordinates".into());
        }
        let bind = |expression: &Atom| {
            physics
                .iter()
                .fold(expression.clone(), |value, (symbol, input)| {
                    value.replace(*symbol).with(Atom::num(*input))
                })
        };
        let f = bind(contour.causal_polynomial());
        let positive = contour
            .positive_polynomials()
            .iter()
            .map(bind)
            .collect::<Vec<_>>();
        let envelope = DynamicEnvelope::new(parameters, f.clone(), &positive)?;
        let u = Atom::var(radius_fraction_symbol());
        let l = Atom::var(lambda_cap_symbol());
        let r = Atom::var(displacement_cap_symbol());
        let norm = envelope.direction().iter().map(|v| v.pow(2)).sum::<Atom>();
        let cap_part = u.pow(2);
        let displacement_part = (&l * &u).pow(2) * &norm / r.pow(2);
        let (causal_part, positive_part, level) = match construction {
            DynamicConstruction::Polynomial => (
                envelope
                    .causal_terms()
                    .iter()
                    .map(|term| {
                        Atom::num(envelope.causal_terms().len())
                            * term.squared_bound()
                            * (&l * &u).pow(i64::from(2 * term.order() - 2))
                    })
                    .sum::<Atom>(),
                envelope
                    .positive_factors()
                    .iter()
                    .map(|factor| {
                        factor
                            .terms()
                            .iter()
                            .map(|term| {
                                Atom::num(factor.terms().len())
                                    * term.squared_bound()
                                    * (&l * &u).pow(i64::from(2 * term.order()))
                            })
                            .sum::<Atom>()
                    })
                    .sum::<Atom>(),
                envelope.polynomial_level().clone(),
            ),
            DynamicConstruction::SignAware => (
                envelope
                    .causal_terms()
                    .iter()
                    .map(|term| {
                        term.positive_bound().expression() * u.pow(i64::from(term.order() - 1))
                    })
                    .sum::<Atom>()
                    .pow(2),
                envelope
                    .positive_factors()
                    .iter()
                    .map(|factor| {
                        factor
                            .terms()
                            .iter()
                            .map(|term| {
                                term.positive_bound().expression() * u.pow(i64::from(term.order()))
                            })
                            .sum::<Atom>()
                            .pow(2)
                    })
                    .sum::<Atom>(),
                envelope.sign_aware_level().clone(),
            ),
        };
        // Differentiate the full chart expressions before any face restriction.
        let mut outputs = vec![
            &level - 1,
            level.derivative(radius_fraction_symbol()),
            cap_part,
            displacement_part,
            causal_part,
            positive_part,
            norm,
            envelope.leading_causal_magnitude().clone(),
        ];
        outputs.extend(parameters.iter().map(|p| level.derivative(*p)));
        outputs.extend(envelope.direction().iter().cloned());
        for v in envelope.direction() {
            outputs.extend(parameters.iter().map(|p| v.derivative(*p)));
        }
        let inputs = parameters
            .iter()
            .copied()
            .chain([
                lambda_cap_symbol(),
                displacement_cap_symbol(),
                radius_fraction_symbol(),
            ])
            .map(Atom::var)
            .collect::<Vec<_>>();
        let evaluator = Atom::evaluator_multiple(&outputs, &inputs)
            .horner_iterations(10)
            .cpe_iterations(Some(1000))
            .cores(1)
            .build()?
            .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
        let factors = Atom::evaluator_multiple(
            &[vec![f.clone()], positive.clone()].concat(),
            &parameters
                .iter()
                .copied()
                .map(Atom::var)
                .collect::<Vec<_>>(),
        )
        .horner_iterations(10)
        .cpe_iterations(Some(1000))
        .cores(1)
        .build()?
        .map_coeff(&|c| Complex::new(c.re.to_f64(), c.im.to_f64()));
        Ok(Self {
            metadata: json!({
                "scope":"retained base chart; not a residual callback or subtraction-face attribution",
                "source_index":chart.source_index(), "representative":chart.representative(),
                "kernel_sector":chart.kernel_sector(),
                "representative_permutation":chart.representative_permutation(),
                "coordinates":parameters.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "construction":construction, "regularity":envelope.regularity().to_string(),
                "causal_orders":envelope.causal_terms().iter().map(|t|t.order()).collect::<Vec<_>>(),
                "positive_orders":envelope.positive_factors().iter().map(|f|f.terms().iter().map(|t|t.order()).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "retained_validation_faces":contour.validation_faces(),
                "causal_polynomial":f.to_string(),
            }),
            dimension,
            positive_count: positive.len(),
            evaluator,
            factors,
            output_count: outputs.len(),
        })
    }

    pub(super) fn evaluate(
        &mut self,
        point: &[f64],
        safety: f64,
        cap: f64,
        displacement: f64,
    ) -> Result<Value> {
        if point.len() != self.dimension
            || point
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
            || !safety.is_finite()
            || safety <= 0.0
            || safety >= 1.0
            || !cap.is_finite()
            || cap <= 0.0
            || !displacement.is_finite()
            || displacement <= 0.0
        {
            return Err("invalid partition point or S/L/R".into());
        }
        let mut input = point
            .iter()
            .copied()
            .chain([cap, displacement, 0.])
            .map(|v| Complex::new(v, 0.))
            .collect::<Vec<_>>();
        let mut values = vec![Complex::new(0., 0.); self.output_count];
        let options = BracketedRootOptions {
            absolute_tolerance: 0.,
            relative_tolerance: 2e-14,
            max_iterations: 256,
            initial_guess: Some(0.5),
            convergence: BracketedRootConvergence::Bracket,
        };
        let answer = nsolve_bracketed(0., 1., &options, |u| {
            input[self.dimension + 2].re = *u;
            if self.evaluator.try_evaluate(&input, &mut values).is_err() {
                return (f64::NAN, f64::NAN);
            }
            (values[0].re, values[1].re)
        })?;
        let u = answer.root;
        input[self.dimension + 2].re = u;
        self.evaluator.try_evaluate(&input, &mut values)?;
        if values
            .iter()
            .any(|v| !v.re.is_finite() || !v.im.is_finite() || v.im != 0.)
        {
            return Err("nonfinite/nonreal partition output".into());
        }
        let values = values.iter().map(|v| v.re).collect::<Vec<_>>();
        let shares = &values[2..6];
        if values[0].abs() > 1e-9
            || (shares.iter().sum::<f64>() - 1.).abs() > 1e-9
            || shares.iter().any(|v| *v < -1e-14)
            || u * values[1] < 2. - 1e-8
        {
            return Err("native root/partition identity failed".into());
        }
        let lambda = safety * cap * u;
        let gradient = values[8..8 + self.dimension]
            .iter()
            .map(|h_x| -safety * cap * h_x / values[1])
            .collect::<Vec<_>>();
        if !lambda.is_finite() || lambda <= 0. || gradient.iter().any(|v| !v.is_finite()) {
            return Err("nonfinite local strength or implicit gradient".into());
        }
        let v = &values[8 + self.dimension..8 + 2 * self.dimension];
        let dv = &values[8 + 2 * self.dimension..];
        let images = point
            .iter()
            .zip(v)
            .map(|(x, v)| Complex::new(*x, -lambda * v))
            .collect::<Vec<_>>();
        let mut factors = vec![Complex::new(0., 0.); self.positive_count + 1];
        self.factors.try_evaluate(&images, &mut factors)?;
        if factors
            .iter()
            .any(|z| !z.re.is_finite() || !z.im.is_finite())
        {
            return Err("nonfinite deformed factor diagnostic".into());
        }
        let n = self.dimension;
        let matrix = |full: bool| {
            (0..n)
                .flat_map(|i| {
                    let gradient = &gradient;
                    (0..n).map(move |j| {
                        Complex::new(
                            f64::from(i == j),
                            -lambda * dv[i * n + j] - if full { v[i] * gradient[j] } else { 0. },
                        )
                    })
                })
                .collect::<Vec<_>>()
        };
        // Native Matrix requires its exact-key-capable Float field, not raw f64.
        // Converting the rounded entries adds no input precision or certificate.
        let dimension = u32::try_from(self.dimension)?;
        let determinant = |full| -> Result<Complex<Float>> {
            Ok(Matrix::from_linear(
                matrix(full)
                    .into_iter()
                    .map(|z| Complex::new(Float::with_val(128, z.re), Float::with_val(128, z.im)))
                    .collect(),
                dimension,
                dimension,
                FloatField::from_rep(Complex::new(
                    Float::with_val(128, 0),
                    Float::with_val(128, 0),
                )),
            )?
            .det()?)
        };
        let full = determinant(true)?;
        let frozen = determinant(false)?;
        if [&full.re, &full.im, &frozen.re, &frozen.im]
            .iter()
            .any(|v| !v.to_f64().is_finite())
        {
            return Err("nonfinite native determinant diagnostic".into());
        }
        Ok(json!({
            "metadata":self.metadata,"point":point,"S":safety,"L":cap,"R":displacement,
            "u":u,"radius":cap*u,"lambda":lambda,
            "shares":{"cap":shares[0],"displacement":shares[1],"causal":shares[2],"positive":shares[3]},
            "root_residual":values[0],"u_Hu":u*values[1],
            "direction":v,"direction_norm":values[6].sqrt(),"leading_causal_A":values[7],
            "lambda_gradient":gradient,
            "lambda_gradient_norm":gradient.iter().map(|v|v*v).sum::<f64>().sqrt(),
            "physical_displacement":lambda*values[6].sqrt(),
            "normalized_displacement":lambda*values[6].sqrt()/displacement,
            "jacobian_full":[full.re.to_f64(),full.im.to_f64()],
            "jacobian_frozen_strength":[frozen.re.to_f64(),frozen.im.to_f64()],
            "images":images.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "deformed_F":[factors[0].re,factors[0].im],
            "deformed_positive":factors[1..].iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "solver_iterations":answer.iterations,"solver_evaluations":answer.evaluations,
            "diagnostic_only":true,"causal_certificate":false,
        }))
    }
}
