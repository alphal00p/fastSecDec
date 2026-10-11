# Certified regular algebraic sections and composed maps

`threshold::maps::regular` admits generic-degree implicit boundaries from an
actual verified GCAD cell. It constructs native composed map programs without
expanding the whole transformed density. It does not yet certify that density's
endpoints or produce a complete integration artifact.

## Exact admission

The caller proposes a rational root bracket and a positive derivative margin.
The native cell supplies the full polynomial, selector, lifting roles and
original domain. Each prefix bound is checked as a positive multiple of an
original one-variable affine constraint; later coordinates and target signs
cannot silently shrink that domain.

Existing symGCAD domain certificates establish derivative and endpoint signs.
Continuity extends the derivative margin to the closed cylinder, and weak
endpoint signs give a unique root there. Native exact root isolation at the
saved anchor associates it with the cell's actual selector. All these receipts
remain attached to the immutable source. Interior positivity or a sample's
isolating interval alone does not establish this admission. Unsupported
domains, collisions and insufficient margins remain explicit refusals.

## Native execution

Symbolica's `EvaluatorComposer` wires coefficient programs, root values, interval
widths and coordinate images together. Subsequent stages consume earlier slots;
they do not recursively substitute large nonlinear expressions. The full root
polynomial is evaluated, with a prepared polynomial/derivative program and
native `nsolve_bracketed`; no all-root search occurs during map evaluation.
Certified linear sections lower directly to native `-c0/c1` arithmetic, avoiding
both a root solve and a callback lock for constant or linear bounds.

For coefficient `c_i`, the callback's symbolic derivative is `-r^i/P_r`.
Symbolica differentiates the root and minor again for higher derivatives.
No numerical endpoint AD is introduced. Immutable helpers and scoped factories
own callback metadata, with clone-local mutable scratch and explicit error
boundaries. The numerical root result is not a certified interval.

Every width is checked separately. Interior widths must be positive; a closed
face may collapse, but that zero is a map value, not an endpoint-unit proof.
The triangular Jacobian is multiplied once. The existing projective delta gauge
contributes its certified unit measure.

## Review and evidence

Root and the independent resolver agent reviewed the equations, domain/selector
association, callback ownership, full chain rule, measure and failure paths.
Seven isolated native groups pass in 0.17 seconds, covering actual cubic/quintic
cells, all four cells of a nested example, first/second derivatives, f64,
DoubleFloat and 192-bit Float, complex callbacks, clone lifetime, native codecs,
invalid proposals, resource limits and cancellation. Strict isolated Clippy
passes in 26.425 seconds.

After import, the joined native threshold suite passes 188 tests with no failures
and one pre-existing ignored test in 29.10 seconds. Strict library/test Clippy
passes in 32.24 seconds; workspace formatting and diff checks pass. These gates
include the direct linear-bound lowering described above.

The supervised test process took 1.348 seconds with 30,109,696 bytes observed
peak RSS at 100 ms sampling. This is a small correctness control, not a sampling
performance result or a hard memory bound. Native certificate operations still
need caller-owned hard process limits.

API/source/probe checks reuse native polynomial substitution and division,
symGCAD domain/root facilities, Symbolica evaluator composition, callback
derivatives, root refinement and serialization. No replacement CAS, AD, root
solver, graph or numerical estimator was added. Ordinary linear cell APIs are
unchanged; HEPKit generation continues to own its existing native objects.

## Remaining boundaries

Map programs currently reject runtime-parameter axes pending their typed chamber
admission. Proof owners retain the full GCAD decomposition; detached sector
records must remove that residency before claiming bounded-memory integration.
The codec tests restore main and helper IR in the same process with the same
live proof owner. Fresh-process callback registration, saved proof admission,
whole-kernel tracked-error precision rescue and indexed integration artifacts
are separate gates.

Closed-face normalization, singular-root transport, common analytic regulators,
multidimensional symbolic subtraction and the general integration atlas remain
unfinished. The [secant-unit construction](no-deformation-secant-units.md) is the
reviewed next bridge for regular sections; its stronger interval conditions must
be checked independently rather than inferred from the root bracket.
