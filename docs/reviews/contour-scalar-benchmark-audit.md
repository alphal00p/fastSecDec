# Independent scalar campaign audit

Status: maintained input and driver source reviewed, 2026-10-10. Five native
example controls, the unchanged coordinate-hasher control, strict workspace
all-target Clippy and formatting pass. No scalar generation or sampling result
is accepted by this review. The optimized executable, final campaign pins and
raw epoch acceptance remain separate gates.

## Native inputs and references

The [four-case design](contour-scalar-benchmark-design.md) fixes the physical
points, measures and reference conventions. The maintained
[fixture module](../../crates/fastsecdec/examples/contour_scalar_benchmark/fixtures.rs)
uses HEPKit `GraphIntegral`, native graph routing and kinematics, and
`ScalarParametricIntegral`. It checks numerator one and retains native U/F,
propagators, exponents and measure in its admission report. No DOT parser,
graph type or routing implementation is added.

The triangle and box use the existing physical OneLOop expression provider,
with its complete finite/pole/double-pole vector and explicit `1/r_Gamma`
normalization. The scalar sunrise replaces the old graph's numerator through
the native API, retains the nonzero external scale, and checks its U/F,
`-1/4` ultraviolet pole and finite imaginary part `-pi/2`. It is a branch and
endpoint control, not a second interior-threshold example.

The connected five-propagator kite checks native denominators and exact U/F
against the reference graph. Its explicit minus-one measure gives the
reference's `Gamma(1+2*eps)` prefactor. Symbolica Gamma/Atom arithmetic evaluates
the published series at exact `s=3/1000` through order eight, with an absolute
series-tail bound of `3.60836e-28`; that bound excludes floating-point roundoff.
The native causal limit is `4.403658192582334 + 1.5704037847171362*i`. The
upstream printed number uses a finite imaginary regulator in the logarithm;
the native test reproduces it at the original tolerance and bounds the
regulator displacement separately. No Python, Mathematica or SymPy evaluation
is used.

## Generation and execution boundaries

The [maintained example](../../crates/fastsecdec/examples/contour_scalar_benchmark.rs)
has separate prepare, generate, admit, select and sample actions. All six
arms use native Symbolic endpoint IBP and coefficient expansion. The Jacobian
choice is independent: fixed, polynomial and sign-aware recipes each have
Symbolic and contour-only Dual variants. Actual generated mode, residual
layout and build-time contour-partial statistics are recorded. A zero active
partial count in a degenerate case is reported rather than interpreted as
distinct numerical work.

Compilation reuses the existing native SymJIT backend, one caller core,
Horner count zero, CPE upper bound 1000 and initial relative series width two.
Fixture/reference preparation, native generation, compilation and serialization
must be timed separately while sharing the generation action's total deadline.
Saved records use the existing native codec and primary-cache path. Restoration
checks mathematical identity, recipe, Jacobian policy, source/coordinate/layout
schema and absence of unbound physical parameters. It records cache outcomes.

Admission reuses the actual CLI pilot helper and native full-integral readiness.
The descending cap ladder is fixed in advance. Selection requires all six
owners and chooses the largest common admitted cap without sampling or reading
reference errors. Only explicit native causal refusal permits moving lower;
structural, precision and unsupported-operation errors remain failures.
Every production action restores and pilots a fresh owner.

## Statistical review

Each arm uses two independent run seeds. The six arms deliberately reuse the
same two seeds within a case: this supports paired comparisons and does not
create twelve independent observations. Pilot seeds are separate. The native
pilot helper's test verifies that validation does not advance production RNG
state. Final seed freshness is checked against the prior physical campaign
before the one-shot scalar plan is frozen.

Each `1024/2048/4096 × 8` epoch constructs a fresh native `QmcSession` and fresh
evaluation contexts. Different lattice sizes are never pooled. The three
epochs sharing one run seed are not three independent repetitions. The driver
uses native Kuo 33002, Korobov3, worker packages and weighted batch evaluation;
it adds no sampler, covariance accumulator or estimator. Native full-vector
estimates include exact offsets once and retain all poles and cross-component
covariance. Reference subtraction and covariance traces are reporting only.

The shared coordinate hasher records actual transformed coordinates, weights,
sector, shift and point range. A paired Jacobian comparison requires equal
native layouts/settings and equal complete coordinate-range hashes; equal
seeds alone are insufficient. A partially completed epoch has no accepted
estimate. Earlier complete epochs can survive a later time limit, with process
closure and censoring recorded explicitly. Native deadline cancellation is
reported separately from a numerical failure, preserving any processed prefix
and failing-point context without accepting incomplete statistical work.

## Budgets, evidence and remaining acceptance

The approved task allowance is 5220 seconds (87 minutes), inside one 90-minute
campaign wall clock, at most two native processes and 100 GB combined sampled
RSS. Separate actions have fixed generation, common-cap admission, fresh setup
and production budgets. The campaign must not reset the global deadline,
launch work after it expires, retry a censored case automatically, or silently
replace an input. Its process monitor includes its own RSS, kills only owned
groups and retains closure evidence.

Initial script review requested stronger pre-launch deadline checks, agreement
between linked-build and freeze hashes, and collector checks for native
case/recipe/Jacobian identities, pinned setup proofs, process closure and a
complete inventory of missing epochs. The corrected scripts pass independent
source review. The collector retains all 144 planned epoch records, including
missing or truncated ones, and 72 explicit paired-Jacobian checks. It verifies
the native BLAKE3 input pins with the standard hash package; Python performs
only process orchestration, identity checks and reporting. The final optimized
plan and actual output acceptance remain pending. Raw output remains under
`target/contour-scalar-runtime/` and is not tracked.

The maintained-source gate is recorded in
`target/contour-scalar-final-gates.log`: five example tests, one existing hasher
test and strict all-target Clippy. These are input/protocol controls, not
benchmark convergence evidence. The existing core/dependency graph is reused;
no broader native, portable or installed-wheel acceptance is inferred from
this example-only milestone.
