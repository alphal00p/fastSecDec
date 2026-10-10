# Retained HEPKit IntegralFamily input

2026-10-10. Follow-up to the interface audit at milestone `a3d97cf`.

## Native ownership and public interface

`Integral.from_family(family, *, regulator, powers, numerator, kinematics=None,
dimension=None, scalar_values=None, auxiliary_momenta=None,
measure_multiplier=None, runtime_parameters=None)` retains the existing native
HEPKit `IntegralFamily` and optional `Kinematics`. Construction copies native
owners and validates scalar-symbol transport only. It does not specialize the
family, project denominators, contract the numerator, parametrize, decompose,
compile or sample.

The private input enum selects the existing `GraphIntegral` or native family
owner. Explicit generation invokes the existing `prepare_family_input`, then
`ParametricIntegrand::from_family`. These are exactly the native operations that
previously served the synchronous family branch in `decompose.rs`; that branch
now delegates through the retained owner. No algebra, graph, projection or
sampler implementation is added. The scalar numerator and explicit measure are
multiplied once before native preparation; graph/projector weights are never
inferred for a family.

Both owners feed the same existing synchronous generation, cooperative singleton
session and cooperative recipe-family session. Existing `sector_decompose`
return types are preserved. Its `contour=True`, and the corresponding singleton
option, still select fixed deformation. Dynamic alternatives are requested
explicitly through `generation_family_session`, with archive default and
optional resident selection independent.

`input_kind` is `graph` or `family`. `powers` retains the established list of
index/value tuples: stable edge IDs for graphs, zero-based denominator order for
families. The displayed value is signed i64, preserving all native u32 graph
powers and i32 family powers. Family zero and negative slots remain visible;
their native projection occurs only on generation. The settings card now shows
dynamic construction and S/L/R through existing getters.

## Verification and limits

The standalone binding all-target check and strict Clippy both passed with
`--locked --features python_stubgen`, using jobs=2 and the existing binding
target. Logs are `target/contour-family-input-bindings-check.log` (16.20 s) and
`target/contour-family-input-bindings-clippy.log` (4.98 s). These checks included
the concurrently reviewed native/Python operational-diagnostics wrappers.
No dependency or lockfile was changed.

The independent HEPKit integration reviewer accepted the source reuse,
ownership, error boundaries and test design. Synchronous byte-parity controls
intentionally exercise identical native compilation paths and options, as in
the existing family tests; they do not assert that differently assembled
archives have a canonical transport representation. Scientific archive checks
use the complete native output vector and independent references.

`test_contour_family_input.py` passed all eight cases on both refreshed coherent
hosts. It covers inert construction and pauses, signed/zero powers,
weighted numerator projection, graph/family parity, legacy return types,
singleton sessions, both dynamic constructions and generation modes, saved
archive restoration after input-owner drops, actual caller pilots and a complete
complex triangle reference. A retained symbolic-mass tadpole is rebound at two
physical values, checking native exact normalization, unchanged template bytes
and chart/layout ownership, and distinct bound physical identities. The test
also checks the complete dynamic settings card.

The installed candidate-1 source content SHA256 is
`9950910602b5c5fe040f30aa2a3d388ef8c28f63a4f6b7d11f02d0eec6d77c12`.
Its full native Python suite passed **249/249 in 78.45 s**; the actual Pyodide
suite passed **138/138 in 16.32 s**. Both include all eight new family-input
cases and the three companion status/diagnostics cases. These are execution
gates for this constructor and native owner path, beyond the earlier leaf
compilation gate. They do not establish browser responsiveness or a performance
advantage.
