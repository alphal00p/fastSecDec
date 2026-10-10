# Verified open-cell maps: finite linear bounds

This slice retains exact GCAD cell descriptions and lowers their finite linear
bounds to native composed evaluators. It is an executable subset of the general
map interface, not completion of the algebraic resolver or permission to publish
a threshold integral. Nonlinear sections and unbounded integration cells return
an explicit unsupported result; their original selectors remain inspectable.

## Ownership, coordinates and guards

`CellMap` retains one `Arc<VerifiedDecomposition>` and its raw cell index. Native
root polynomial IDs, distinct-root ordinals, selector domains, sample enclosures,
exceptional polynomials and proof provenance remain with that owner. The map
never uses a sample isolating enclosure as a runtime bound. Native lifting order,
original prepared coordinate order and original projective coordinate order are
separate associations.

Parameters precede integration axes in the verified lifting order and never
become unit-cube dimensions. Fresh unit symbols are checked against the complete
original and prepared density: prefactors, all monomial powers, numerator and
singularity factors, complete exponents, physical regulator and kinematic/domain
inventories. Auxiliary regulators and numerator-only symbols are therefore
protected even when absent from the geometric GCAD aliases.

The first admission operation accepts exact rational parameter values only. It
checks the actual parameter cell intervals, parameter-only strict constraints
and parameter-only exceptional polynomials. The result remains tied to its
immutable map and request. This is not general symbolic or algebraic-fiber
admission and does not cover excluded exceptional parameter strata.

## Native algebra and evaluator composition

The native symGCAD polynomial parser and Symbolica
`to_univariate_polynomial_list` provide the two coefficients of a degree-one
section. Its exact root is `-b/a`, retained as a native rational expression in
preceding original coordinates. Ordinal zero is required; a `Positive` selector
also requires a strictly positive root. The exact point-control path rejects a
vanishing leading coefficient and nonpositive interval width, using native
rational polynomial evaluation rather than a replacement root-isolation engine.

Each integration step retains `x = lower + width*t`. Symbolica's native
`EvaluatorComposer` connects earlier image slots to later bounds, computes the
positive width product once and restores original output order. A reversed
lifting permutation changes a signed determinant but not the positive measure.
The optional affine projective preparation is composed once after the prepared
images; its proved delta measure is one, not induced surface area. The original
factorized density and factor associations remain available through the shared
request. No full-density substitution or expansion is required by this map
construction.

Compiled programs retain the declared runtime parameter inputs followed by the
unit coordinates. An admitted rebind creates a new guard wrapper sharing the
same immutable native program; it neither solves GCAD nor recompiles the map.
The low-level native evaluator is deliberately documented as unchecked: callers
must supply the admitted parameter prefix and an open-unit point. Raw native
execution is not a chamber check or an endpoint certificate.

Horner optimization occurs on native expression trees before linearization.
The inspected `EvaluatorComposer::finish` performs instruction pruning, CSE and
CPE, and cannot reconstruct a missing Horner search. Consequently every actual
expression-builder stage preserves the caller's Horner setting (default ten).
Only intermediate CPE is suppressed, with the caller's CPE policy applied to
the final composed graph. An early ignored draft's Horner-zero override was
corrected before registration; it was never used in a production campaign.

## Validation and remaining scope

The initial c540 native API probe passed two controls: exact linear coefficient
and rational fiber agreement, and 18 points with five composed outputs at
192-bit precision through a native instruction-codec round trip. The latter
agrees with a small explicit-substitution reference within `1e-50`. Its enum
iteration does not constitute native alternate-selector resolution evidence.

The complete ignored implementation then passed seven native controls in
0.04 seconds and strict direct Clippy. They cover finite split triangles and
owner lifetime; admitted parameter changes sharing the same program and
boundary refusal; nonlinear/unbounded refusal; projective original images and
unit delta measure; reversed lifting order with an independently differentiated
native Matrix determinant; actual zero-root `Positive` refusal and degree-drop
guards; and auxiliary/numerator-only symbol capture rejection. Program vectors
are compared with exact rational map values through native saved-instruction
round trips. These controls preserve the default Horner ten. The frozen source,
linked native dependency identities and raw logs are pinned in
`target/no-deformation-cell-map/handoff.json`.

Independent foundation review accepted the native reuse, exact owner/selector
association, chamber scope, original-order outputs, rebind sharing and Horner
policy. The registered module passes **7/7** native Cargo controls, strict
Cargo Clippy (`--lib --tests`, warnings denied) and workspace formatting.
The combined log is `target/no-deformation-cell-maps-integrated-gates.log`.
The module introduces no new numeric callback or AD layer. General moving-root
selection, algebraic tower precision recovery, unbounded compactification,
closed-face endpoint regularity, auxiliary-regulator continuation, artifact
lineage and complete-integral acceptance remain separate work. No generation
or sampling performance claim follows from these small controls.
