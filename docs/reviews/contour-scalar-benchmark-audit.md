# Independent scalar campaign audit

Status: maintained input and driver source reviewed, 2026-10-10. Five native
example controls, the unchanged coordinate-hasher control, strict workspace
all-target Clippy and formatting pass. The optimized executable, concrete plan
and all 144 native sampling epochs pass independent review, including all 72
paired-coordinate checks. This is a bounded comparison, not an asymptotic
convergence or uncertainty-calibration result.

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
only process orchestration, identity checks and reporting. Raw output remains
under `target/contour-scalar-runtime/` and is not tracked.

The concrete pre-authorization plan SHA-256 is
`6000ec490b1dd979a65ad6ec0c1f7dc813a9c8649d7f352272931ed09e0d5a8d`;
the campaign-start receipt records the subsequent authorization update. Its
23 file pins, seven compiled example source files, nine dependency binaries
and 345 current core Rust source files match the build evidence. The optimized
executable SHA-256 is
`2c6ebc2f269a76b4d21682e5fcb0f720a9b8047603fbaf4222460055be170299`.
Only the noncompiled asset README changed after linking; its old/new hashes
and documentation-only exception are explicit. Native OneLOop is the locked
`27c3723434b7d99cf70ce612b0b8041d3f5c0e78` revision, using the same
Symbolica `74225696`/SymJIT `d74993f` dependency graph as the optimized core.
The protected user release executable remains untouched.

The plan uses distinct physical cores 200 and 201. Its eight case/run seeds
and four pilot seeds are distinct, with only the intended within-case arm
sharing; the prior physical-campaign seed inventory has no matches. The output
directory was absent at admission. No automatic retries or numerical work
occurred during the build/plan review. Benchmark acceptance uses the recorded
native epochs, process closure and paired-coordinate checks below.

The maintained-source gate is recorded in
`target/contour-scalar-final-gates.log`: five example tests, one existing hasher
test and strict all-target Clippy. These are input/protocol controls, not
benchmark convergence evidence. The existing core/dependency graph is reused;
no broader native, portable or installed-wheel acceptance is inferred from
this example-only milestone.

## Completed bounded campaign

The campaign closed in `90.234559` seconds with sampled combined peak RSS
`961445888` bytes, including its Python coordinator. All 24 generation actions,
24 common-cap admissions and 48 fresh-owner production actions completed; no
time/RSS limit, numerical failure, unstable-point substitution or cutoff zero
occurred. Every case selected the predeclared common cap `0.1`. The native
dynamic settings retain `S=0.8`, `R=1`; a common cap does not imply a constant
dynamic strength equal to the fixed strength.

The acceptance receipt SHA-256 is
`05abcb5e0295336e8049edcbf643d54f4d1928d73712aac71f21e3e1b57118e3`.
An independent check verified its 390 raw file hashes, all 144 complete native
snapshot estimates and full covariance layouts, planned/completed coverage,
actual coordinate counts and zero optional production checks. All 72
Symbolic/Dual pairs have identical coordinate/weight range hashes and matching
settings. Their complete-vector means differ by at most `1.47e-16` after
scaling each component by `max(1, abs(value))`. Estimates and covariance are
copied from native sessions; neither lattice sizes nor the two seeds are pooled.

Two observed degeneracies have a native source explanation:

- Triangle and box retain affine projective substitutions, quadratic F and
  linear U. `DynamicEnvelope` adds causal corrections only from odd F ray
  orders at least three, and positive-factor corrections only from even U
  orders at least two. Both lists are empty here, so polynomial and sign-aware
  constructions reduce to the same base radius equation. Their measured
  vectors and operation counts agree; these are not two different radius
  functions at these points.
- Sunrise F is a pure monomial under its retained sector maps. Monomial
  extraction leaves a constant causal factor. All three Dual builds record
  zero active image-partial slots; the actual native pilot has no required
  charts and zero checks. All six arms report the same measured vectors and
  operation counts (7 additions, 25 multiplications, 1 inversion and 4 function
  calls), but their representations are not byte-identical. Fixed Symbolic/
  Dual exact-program sizes are `846`/`812` bytes; either dynamic recipe uses
  `862`/`828` bytes. Symbolic/Dual JIT-IR sizes are `975`/`1065` bytes. Fixed
  records have three inputs and dynamic records five; construction and input
  metadata differ even though this contour is inactive. These results confirm
  the planned branch/endpoint control without claiming identical saved IR.

The independent finite reference errors are retained separately from native
standard errors. The box and kite remain visibly less precise at the largest
fixed work count than the triangle and sunrise. Two seeds and three small
lattices do not establish asymptotic rates, calibrated statistical coverage or
a variance-optimal deformation. No additional sampling is justified merely by
that bounded uncertainty.

Generation RSS describes the entire native process, including fixture and
reference preparation, compilation and serialization. Its timing comparison
uses only native generation plus compilation. The six original sunrise
processes finished between RSS polls: their recorded zeros mean **unobserved**,
not zero memory. A separately authorized generation-only follow-up retained
that distinction. Its first direct `wait4` measurements included an inherited
Python launcher floor of about 69.31 MB and were rejected as native peak
measurements. Six corrected GNU time child measurements give native high-water
RSS from `59777024` to `59817984` bytes. The GNU time parents independently
showed current/high-water RSS of only 1.675–1.798 MB. All six repeated native
owners preserve the original mathematical identities, schemas, references and
settings; all groups closed without limits. The initial diagnostic and corrected
follow-up together took 3.686 seconds and performed no sampling.

The corrected memory receipt SHA-256 is
`533dba5285dd76e6b561216c2eca9eefb576fdbac15fc3ebe0cd836b222b7897`
under `target/contour-scalar-runtime/sunrise-memory-gnu/`. These are separate
kernel high-water measurements of the whole native generation process. They
do not replace original timing observations, improve the original RSS sampling
resolution retrospectively, or add independent production runs.
