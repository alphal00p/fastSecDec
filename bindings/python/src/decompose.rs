//! Thin dispatch on existing HEPKit owners; scientific preparation stays native.

use std::collections::BTreeMap;

use fastsecdec::{Atom, input::prepare_family_input, parametric::ParametricIntegrand};
use feynkit_py::{PyFeynmanDiagram, PyIntegralFamily, PyKinematics};
use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};
use symbolica::{api::python::PythonExpression, symbol};

use super::{
    error,
    generation::{PyGeneratedIntegral, generate_native},
    input::{PyIntegral, scalar_bindings, symbol as expression_symbol},
};

/// Generate Laurent integrands from a native FeynmanDiagram or IntegralFamily.
///
/// A diagram requires explicit kinematics and uses its owned numerator,
/// projector and weights. Its optional powers map stable edge IDs to positive
/// integers. Supply a changed numerator on the native diagram itself, rather
/// than through this function's numerator argument.
///
/// A family requires one signed integer power per ordered denominator and an
/// explicitly weighted scalar numerator. Include graph/projector factors exactly
/// once; no graph weight is inferred. Positive powers select native denominators,
/// zero powers omit slots, and negative powers multiply the numerator. In
/// particular, completed auxiliary denominators do not acquire power one.
/// Kinematics defaults to the family's assumptions. An override may add
/// auxiliary-vector assumptions, but must retain the original external products
/// after scalar binding; it cannot undo substitutions already made in the family.
///
/// Scalar values specialize native kinematics, denominators and numerator
/// together. Auxiliary numerator vectors must be explicitly declared. The extra
/// measure_multiplier applies once beyond prod(d^D k/(i*pi^(D/2))). Integration
/// dimension defaults to 4-2*regulator; a symbolic tensor dimension is replaced
/// by that dimension during parameterization. Incompatible concrete dimensions
/// and scalar bindings that change formal momenta or tensor dimension fail.
///
/// None/True from observer continues; False cancels at a native event boundary.
/// Original observer exceptions propagate. The result retains native metadata;
/// compilation and numerical sessions remain separate explicit operations.
#[cfg_attr(
    feature = "python_stubgen",
    pyo3_stub_gen::derive::gen_stub_pyfunction(
        module = "symbolica.community.hepkit.sector_decomposition",
        python = r#"
import collections.abc
import typing
import symbolica
import symbolica.community.hepkit
import symbolica.community.hepkit.sector_decomposition

def sector_decompose(
    input: typing.Union[symbolica.community.hepkit.FeynmanDiagram, symbolica.community.hepkit.IntegralFamily],
    *, regulator: symbolica.Expression,
    kinematics: typing.Optional[symbolica.community.hepkit.Kinematics] = None,
    dimension: typing.Optional[symbolica.Expression] = None,
    powers: typing.Optional[typing.Union[collections.abc.Mapping[int, int], collections.abc.Sequence[int]]] = None,
    numerator: typing.Optional[symbolica.Expression] = None,
    scalar_values: typing.Optional[dict[symbolica.Expression, symbolica.Expression]] = None,
    auxiliary_momenta: typing.Optional[collections.abc.Sequence[symbolica.Expression]] = None,
    measure_multiplier: typing.Optional[symbolica.Expression] = None,
    max_order: int = 0, coefficient_expansion: str = "physical",
    observer: typing.Optional[collections.abc.Callable[[symbolica.community.hepkit.sector_decomposition.GenerationSnapshot], typing.Optional[bool]]] = None,
) -> symbolica.community.hepkit.sector_decomposition.GeneratedIntegral:
    """Generate from an existing native diagram or an explicitly weighted family.

    Diagrams require kinematics, use their owned numerator/weights, and take an
    optional positive-power mapping by stable edge ID. Families require a signed
    power sequence in native denominator order and a weighted scalar numerator;
    auxiliary slots never default to power one. Kinematics overrides must retain
    original external products after scalar binding, but may add auxiliary data.
    Dimension defaults to 4-2*regulator. Compilation and sessions are explicit.
    """
"#
    )
)]
#[pyfunction]
#[pyo3(signature = (input, *, regulator, kinematics=None, dimension=None, powers=None, numerator=None, scalar_values=None, auxiliary_momenta=None, measure_multiplier=None, max_order=0, coefficient_expansion="physical", observer=None))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn sector_decompose(
    py: Python<'_>,
    input: &Bound<'_, PyAny>,
    regulator: &PythonExpression,
    kinematics: Option<&PyKinematics>,
    dimension: Option<&PythonExpression>,
    powers: Option<&Bound<'_, PyAny>>,
    numerator: Option<&PythonExpression>,
    scalar_values: Option<&Bound<'_, PyDict>>,
    auxiliary_momenta: Option<Vec<PythonExpression>>,
    measure_multiplier: Option<&PythonExpression>,
    max_order: i32,
    coefficient_expansion: &str,
    observer: Option<Py<PyAny>>,
) -> PyResult<PyGeneratedIntegral> {
    if let Ok(diagram) = input.extract::<PyRef<'_, PyFeynmanDiagram>>() {
        let kinematics = kinematics.ok_or_else(|| {
            PyTypeError::new_err("a FeynmanDiagram requires explicit native kinematics")
        })?;
        if numerator.is_some() {
            return Err(PyTypeError::new_err(
                "a FeynmanDiagram uses its owned numerator and weights; replace its numerator on the native diagram before sector_decompose",
            ));
        }
        let powers = powers
            .map(|value| value.extract::<BTreeMap<usize, u32>>())
            .transpose()?;
        return PyIntegral::new(
            py,
            &diagram,
            kinematics,
            regulator,
            dimension,
            powers,
            scalar_values,
            auxiliary_momenta,
            measure_multiplier,
        )?
        .generate(py, max_order, coefficient_expansion, observer);
    }
    if let Ok(family) = input.extract::<PyRef<'_, PyIntegralFamily>>() {
        let powers = powers
            .ok_or_else(|| {
                PyTypeError::new_err("an IntegralFamily requires explicit signed powers")
            })?
            .extract::<Vec<i32>>()?;
        let numerator = numerator.ok_or_else(|| {
            PyTypeError::new_err(
                "an IntegralFamily requires an explicitly weighted scalar numerator",
            )
        })?;
        let regulator = expression_symbol(py, regulator, "regulator")?;
        let dimension = dimension.map_or_else(
            || Atom::num(4) - Atom::num(2) * Atom::var(regulator),
            |value| value.expr.clone(),
        );
        let bindings = scalar_bindings(py, scalar_values)?;
        let momenta: Vec<_> = auxiliary_momenta
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.expr)
            .collect();
        let weighted_numerator =
            &numerator.expr * measure_multiplier.map_or_else(Atom::one, |value| value.expr.clone());
        return generate_native(
            py,
            max_order,
            coefficient_expansion,
            observer.as_ref(),
            "Parametrizing the native integral family",
            || {
                let (family, powers, numerator) = prepare_family_input(
                    family.as_family(),
                    &powers,
                    weighted_numerator,
                    kinematics.map(PyKinematics::as_kinematics),
                    &bindings,
                    &momenta,
                )
                .map_err(|e| error::native(py, "input", e))?;
                let parameters = (0..powers.len())
                    .map(|i| symbol!(format!("fastsecdec::hepkit::x{i}")))
                    .collect();
                ParametricIntegrand::from_family(
                    &family, &powers, numerator, parameters, regulator, dimension,
                )
                .map_err(|e| error::native(py, "parametrization", e))
            },
        );
    }
    Err(PyTypeError::new_err(
        "input must be a native HEPKit FeynmanDiagram or IntegralFamily",
    ))
}
