# Native regular-numerator monomial extraction

The unchanged ggHH CLI reached its first mapped sector after successful input,
domain and geometry work, then exceeded the 180-second generation bound. A
single 60-second attribution probe used the actual mapper with test-only flushed
stage records. Completed full-support extraction consumed 48.919 seconds; the
other completed mapping operations consumed about 0.51 seconds. Its final entry
was source-support extraction for regular polynomial factor 152, term 50, chart
0. Both bounded attempts and their unchanged-input checks are retained in
`gghh-native-cli-generation-6` and `gghh-mapping-attribution-1` under
`output/diagnostics/`.

Existing Symbolica APIs supply a smaller operation. The fixed-variable
`to_polynomial_in_vars_with_field` collector accepts a single mapped coordinate,
leaving other coordinates in native Atom coefficients. Native
`MultivariatePolynomial::degree_bounds(0).0` supplies the lowest retained power;
`ldegree` would instead return the leading monomial's power and is not used.
`AtomField::statistical_zero_test = false` prevents probabilistic removal of
coefficients. Any coefficient that still contains the selected variable rejects
the candidate. No polynomial walker or new algebra implementation is needed.

The private `generation/mapping/regular.rs` helper applies only to already
admitted regular polynomial factors under nonnegative coordinate maps. A
structurally nonzero coordinate face keeps candidate power zero. Otherwise the
native one-variable collector proposes a power. Literal zero, unsupported
conversion, negative powers or failed residual recognition return to the
existing full-support path. Singular factors and signed orthant maps retain
their existing path and domain checks.

The proposed powers are a common-monomial candidate, not a maximal-valuation or
nonzero certificate. A disguised zero coefficient may lower a proposed power.
The existing native factor collection and polynomial-recognition guard must
admit the complete quotient before the candidate is used; its product with the
extracted monomial is the original mapped expression. Keeping extra coordinate
zeros in that regular quotient preserves the expression. Unrecognized
cancellation falls back to the prior sparse-support path.
Such a lower power can conservatively overestimate later subtraction and pole
work; existing complete-vector and subtraction controls therefore remain part
of the acceptance checks. Collection uses the original unsigned support
exponent type, followed by checked conversion of both degree bounds to the
existing signed residual range.
These checks apply to returned native polynomials; they do not catch a native
collector overflow before return. A compact degree-2^31 diagnostic demonstrated
that the native unsigned exponent's checked addition itself uses a signed
limit. Both the prior full-support conversion and the candidate have this
existing limitation. No production panic interception or dependency change is
introduced; a near-limit sparse control stays within the native range.

Independent source review accepted the native ownership and conservative guard.
All six focused mapper controls passed: independently collected full-support
equivalence, hidden coefficient cancellation, unsupported inputs, the unchanged
singular orthant control, a compact mapped power of degree 10,000 and a sparse
near-limit exponent. Fifteen existing subtraction controls and sixteen public
generation controls also passed, including complete Laurent vectors and
cancellations. All four processes, including the separate native-limit
diagnostic, exited 0 and were reaped with source/executable postchecks passing.
The first five-pass/one-failure attempt is preserved: its sole failure was an
incorrect diagnostic expectation that the native degree-2^31 conversion would
return normally. Correcting that test did not change production code.

Evidence is `output/diagnostics/regular-monomial-candidate-{1,2}`. Ordinary ggHH
attempt 7 completed 17 mapping intervals (sector indices 0 through 16 of 30)
under the same 180-second deadline, five-second grace and 30 GiB address-space
bound. The first mapped sector took 3.4704 seconds. Accumulated mapping time was
42.2875 seconds and the symmetry phase 131.7836 seconds. That phase includes
density assembly, incidence encoding, native Graphica canonization and exact
permutation verification; the outer timing does not identify which operation
dominates. The deadline
caused a cooperative `generation cancelled` exit, reaped at 180.7711 seconds,
with peak RSS 190,340 KiB and all bound hashes unchanged. No artifact or
numerical result was published. The former full-support memory growth is
removed on the observed prefix; complete generation remains an open gate.

Build 8 retains the exact mathematical library identities of build 7; no input,
assertion, estimator or precision change is part of this mapper adjustment.
These bounded development-binary observations are capacity evidence, not an
isolated benchmark or a complete-generation speed claim.

A single follow-up attribution used the same native input and generation
options for 60 seconds, with flushed test-only symmetry records and per-factor
mapping tracing disabled. Five charts completed registration; the sixth was in
native Graphica canonization at the deadline. Completed density assembly took
0.01954 seconds, incidence encoding 0.73252 seconds and canonization 36.20872
seconds. There were no candidate matches or permutation-verification calls.
The process was reaped at 60.00979 seconds with peak RSS 113,944 KiB, no core
dump and unchanged source/input/binary checks. These timings include trace
overhead and describe a prefix, not completed generation.

The Cargo records show that both Graphica and its generic FastSecDec caller
used optimization level zero, while Symbolica/Numerica used level two. The
smallest justified next step is therefore the unchanged optimized ordinary CLI,
before an algorithm change. Graphica's existing public `canonize` remains the
owner; FeynKit's U/F-only parametric map cannot replace the required complete
numerator-density proof. Evidence is
`output/diagnostics/gghh-symmetry-attribution-1`. Its first compile-only attempt
selected Rust 1.97.1 through plain `nix-shell` and was intentionally stopped and
preserved. Only the corrected, explicitly selected Rust 1.98.1 test binary ran
the diagnostic.

A separate source review checked the attribution boundary: the profiling driver,
profile module and trace calls are all test-only. Moving the existing
`GraphProfile` storage into its own module preserves the earlier profiling API;
the ordinary registry still performs the same native canonization and exact
permutation verification. Explicit Rust 1.98.1 `rustfmt --check` passed for all
five affected attribution files. No diagnostic file I/O is linked into the
ordinary CLI.

The optimized ordinary CLI follow-up completed all 30 mappings and coefficient
expansions. It reached kernel construction with orders `[-1, 0]` and then
reported the unsupported native color invariant `cas(2,coad(8))`. The process
exited 1 and was reaped at 58.9922 seconds, below its unchanged 180-second bound,
with peak RSS 274,076 KiB. Mapping took 38.8190 seconds, symmetry 9.9097 seconds
and coefficient expansion 9.1159 seconds. All source/input/executable postchecks
passed. This closes the observed mapping-capacity concern for this fixture;
compiled generation and integration remain incomplete. No artifact was emitted
and the conditional integration was not started. Evidence is
`output/diagnostics/gghh-native-cli-generation-8`.

Before that run, the CLI build's FeynKit provenance path was corrected to the
actual Cargo dependency owner. The first release executable remains archived
and was not used for science. The second executable's embedded revision and
source state match the dependency manifests in Cargo's compiler output; the
three existing dependency-provenance controls pass. Both libraries and CLI use
the ordinary optimized release profile, while numerical kernels still default
to portable SymJIT O2.

After native color closure in the example exporter, the unchanged optimized
CLI completed and saved all 30 kernels in 61.2853 seconds. A bounded ordinary
integration then completed all 245,760 samples across eight shifts per sector
in 8.7809 seconds on eight workers, retaining both orders and their full joint
covariance. The finite coefficient's relative error is 1.0273%, so this proves
native feasibility without a one-per-mil convergence claim. See the
[native feasibility review](gghh-native-feasibility.md) for timing boundaries,
precision rescues and preserved evidence.
