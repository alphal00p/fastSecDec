# Runtime kinematic parameters — implementation evidence (2026-10-06–07)

The later [on-shell generation correction](gghh-on-shell-generation.md) fixes
the two incoming-gluon self-products to exact zero before sector finding.
Current ggHH cards therefore have thirteen runtime Gram entries; the fifteen
entries below describe the earlier generic-kinematics milestone.

The ggHH run card now names fifteen external Gram entries with `symbol`.
`point.toml` supplies their previous physical values at integration time through
`--parameters`; repeated `--parameter NAME=VALUE` arguments override the file.
At that milestone, model masses and couplings remained generation inputs. The
subsequent [runtime model revision](runtime-model-parameters-review.md)
supersedes that restriction: contributing independent model inputs are now
evaluator parameters, with analytic dependent couplings retained. Other examples
and the committed tests remain intentionally unchanged.

## Existing owners reused

- HEPKit `Kinematics::with_scalar_product` accepts native symbolic Atoms and its
  `GraphIntegral` family retains them through Gaussian parameterization. No new
  graph, scalar-product algebra, or Gram inversion is introduced.
- Symbolica `Atom::evaluator_multiple` already takes an ordered list of arbitrary
  scalar input Atoms. Upstream `tests/evaluation.rs` exercises several ordered
  parameters and repeated evaluation at different values. Its evaluator builder
  and native evaluator implementations were inspected before the plumbing change.
- The focused Rust probe evaluates `x*k+k^2` at `(x,k)=(1/4,4)` and `(1/4,-3)`,
  obtaining exactly `17` and `8.25`. This establishes that runtime values do not
  require substitution, recompilation, or a second numeric evaluator.
- Existing exact native programs, SymJIT O2 / Symbolica interpreter mappings,
  complete-vector precision rescue, and weighted replay remain their owners.
  Native scalar binding resolution and Atom evaluation also parse point-file
  values; no expression parser or algebra helper was added.

## Native and eager library routes

The ordinary CLI keeps SymJIT O2. Consumers selecting
`fastsecdec = { path = "path/to/fastsecdec/crates/fastsecdec", default-features = false, features = ["portable"] }`
use the existing eager Symbolica interpreter and portable number backends.
Both backends expose the same caller-driven sequence:
`generation::generate`,
`GeneratedIntegral::compile_with_parameters_and_progress(&symbols, progress)`,
`KernelSet::bind_parameters(&values)`, and `KernelSet::evaluation_context` for
caller-owned QMC or Havana work. Here `compile` prepares the evaluator; the
portable implementation does not invoke SymJIT or generate machine code.
The standalone portable consumer in `tests/portable-kernel/Cargo.toml` exercises
this complete generation, preparation, binding, artifact-load and integration
boundary without native reference-only dev dependencies. The later Pyodide/
marimo binding can reuse these public native owners; that notebook wiring is
not part of this round. Native and portable caches retain their distinct backend
policies, so cross-backend cache loading is not claimed.

## Boundary and identity

FastSecDec records a common ordered real-parameter schema and appends it after
sector integration coordinates in each evaluator. Only coordinates must lie in
`[0,1]`; scalar inputs must be finite. Cancellation profiles are restricted to
the coordinate prefix, including precision replay. Worker clones retain the
bound values and their complete Laurent-vector evaluator.

Binding requires exactly the declared names and rejects missing, unknown,
duplicate canonical names and nonfinite values. Point expressions are evaluated
once to binary64; increasing evaluator precision preserves that supplied binary64
point and does not claim additional physical input precision.

Analytically folded offsets use the same native parameterized evaluator path.
Unbound offsets are explicitly unavailable (`NaN` internally), so the existing
native integration-manifest admission rejects them even for an exact-only
integral. Sector evaluation and worker-context construction also reject unbound
parameters. Binding changes the numerical identity using the complete ordered
input bit patterns; checkpoints additionally store canonical values in caller
settings. Compiled template identity and saved template bytes stay independent
of the point. A saved template must be rebound after loading.

## Validation boundary

`cargo check -p fastsecdec-cli --locked` passes for the implementation. The
installed Rust 1.99 toolchain was used because this host has no `nix-shell`.
The simple native ordered-input probe above passes. The expanded native manual probe passes cold binary loading, caller-owned
worker cloning, weighted MPFR precision replay, changed-point replay rejection,
and exact-only offsets including binary reload/rebinding. The independent
portable-feature run also passes the same checks using the standalone portable
consumer; this verifies the interpreter/portable arithmetic on the host, not
an actual browser or Wasm execution. Probe sources and
outputs remain untracked.

A manual CLI check generated one symbolic massless bubble once and integrated
it at `s=-1` (from a native expression in a TOML point file) and `s=-2`
(command-line value), each with 8,192 QMC evaluations. The pole agrees between
points, and the finite difference agrees with `ln(2)` within `1.9e-13`; both
runs retain complete covariance and report zero evaluation failures. Saved
numerical identities differ. Missing parameters, changed-point checkpoint
resume, and a reference bound to the other point are rejected. The original
bubble example is unchanged; the manual inputs/results live under ignored
`output/generation-revision/`.

The independent integration/reuse audit is assigned to the parallel-generation
agent. Old ggHH numerical timings and estimates remain explicitly historical:
they are not validation of the new symbolic-kinematics generation or evidence of
new performance. No new notebook, browser, or amplitude-reference result is
claimed. Existing one-loop master/reduction APIs remain available as numerical
references; this parameter transport change does not duplicate them.
