# Symbolic endpoint reduction and contour-only duals

2026-10-10. This independent audit applies the user's clarification in
[the contour plan](../../CONTOUR_DEFORMATION_PLAN.md#symbolic-endpoint-reduction-and-contour-only-duals).
The required comparison uses native symbolic endpoint reduction in both arms:
symbolic deformation Jacobian versus a Jacobian evaluated using native duals.
It does not authorize numerical-dual endpoint reduction.

## Confirmed implementation mismatch

The previously measured `ContourJacobian::Dual` path requires
`GenerationMode::NumericalDual`. It composes a first-order image Dualizer and
determinant into the smooth density, then applies the NumericalDual subtraction
pipeline's outer jets to that entire density. Those controls established the
mathematical correctness of that construction, but not compliance with the
clarified endpoint algorithm. Its generation and sampling measurements remain
historical evidence. In particular, their cost cannot reject the requested
first-order, contour-only dual approach.

The current symbolic path already applies native `Atom::derivative` to the
complete mapped regular density. The IBP boundary term evaluates that density
at one; each bulk step differentiates it before proceeding. Taylor subtraction
then differentiates before restricting its coordinate to zero. Thus the
Jacobian, local strength, image substitutions and factorwise analytic branches
all participate. There is no justification here for reordering IBP before
deformation or treating the Jacobian as constant.

## Mathematically admissible lowering under investigation

Write the image matrix entries as `m_ij(x) = partial_j z_i(x)` and retain the
native compact determinant call `D(m_11(x), ..., m_nn(x))` inside the full
deformed density. Symbolica's symbolic chain rule differentiates this call
during endpoint reduction. It produces determinant partial functions and the
required symbolic higher derivatives of the images, including derivatives of
the position-dependent strength. Existing native function-map registration
resolves determinant partial bodies with `Atom::derivative`.

At the final evaluator boundary, the candidate lowering supplies surviving
first-image-partial expressions from a **first-order image-only** Dualizer.
The already-symbolically-reduced density is compiled normally; no outer
Dualizer computes endpoint derivatives. Native evaluator composition and the
saved exact instruction codec are intended to remain the execution boundary.
This is a representation proposal, not an accepted implementation yet.

Faces require derivatives of the original image evaluated on the face.
Restricting an image first and differentiating the restricted expression loses
normal derivatives. Coordinate seeds must therefore remain one on a face;
non-coordinate runtime parameters have zero coordinate seeds. Repeated equal
matrix entries must share one native input, and literal entries must not be
reinterpreted as free parameters. Exact offsets must still resolve their full
semantic expressions after restriction and before exact aggregation.

## Native evidence and current limits

Source inspection covers `generation/subtraction.rs`,
`contour/definitions.rs`, `kernel/program.rs`, and the existing
`generation/numerical_dual/native/cache/jacobian.rs` and outer-jet pipeline.
These establish the current coupling and identify reusable native symbolic
derivative/function-map and evaluator-composition operations. No alternate CAS,
AD rule implementation, or dependency defect is proposed.

A focused ignored probe is being prepared under
`target/foundation-symbolic-endpoints/`. It will test arbitrary native-Atom
evaluator input binding through function-map calls, symbolic mixed derivatives,
face evaluation with original-image seeds, equal/literal matrix entries and
native saved-program round trips. The probe and the new production path are
pending. No new physical generation, final timing comparison or scientific
acceptance follows from this source audit alone.
