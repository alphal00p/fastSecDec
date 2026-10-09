# Dynamic contour runtime boundary

2026-10-09. Proposed native runtime contract; dynamic production integration is
not enabled by this note. It complements the mathematical and artifact reviews.

## Reuse evidence

Symbolica supplies the prepared scalar root API in the isolated owner patch
reviewed in `contour-root-solver.md`. FastSecDec should supply the contour equation
and its native evaluator, not another root algorithm or polynomial evaluator.

The native univariate polynomial API was checked in the public API, source and
an executable tracked-domain probe. `UnivariatePolynomial<FloatField<T>>` needs
`Hash`, `Eq` and `InternalOrdering`, which `ErrorPropagatingFloat<T>` does not
implement. Its coefficient container also omits coefficients whose numerical
centres are zero. That API therefore does not preserve the required uncertainty
of arbitrary tracked coefficient inputs in this use.

There is an existing alternative. A Symbolica `ExpressionEvaluator` for H and
its native derivative, with all envelope coefficients as runtime inputs, handles
ordinary and tracked domains without those ring bounds. The executable probe
`target/contour-prepared-polynomial-probe.rs` builds that evaluator once. With
`a2=4`, `a4=a6=0`, and nonzero absolute uncertainties on the last two coefficients,
it preserves their contribution to H at u=1/2. The resulting absolute error is
7.8125e-8, rather than zero. An optimized local microprobe of 100,000 changing
coefficient evaluations measured about 37.5 ns per degree-six H/Hu call. This is
not the cost of the full root callback, contour map or integrand.

No new borrowed-coefficient Horner helper is needed on this evidence.

## Callback and differentiation contract

One versioned native callback returns the physical strength, with numeric inputs
`[a2,a4,...,a_(2n),S,L]`. It solves `H(u)=sum_j a_(2j)*u^(2j)=1` and returns
`lambda=S*L*u`. The full-sector dense coefficient layout stays fixed on faces,
including explicit zero coefficients. The function's leading symbolic tags
identify its arity and persisted helper-program digest; tags are not numerical
arguments and have no derivatives.

The native derivative hook supplies

- `d lambda / d S = lambda/S`;
- `d lambda / d L = lambda/L` at fixed coefficients;
- `d lambda / d a_(2j) = -lambda*u^(2j) / sum_k 2k*a_(2k)*u^(2k)`.

Symbolica derives higher jets and physical-coordinate chain rules from these
identities. The coefficients already depend on L and R, so those dependencies
remain visible to its differentiation. Reusing the same callback atom permits
native CPE to share the scalar root across derivatives.

Even without optional causal checks, malformed arguments, nonfinite data,
negative envelope coefficients, solver errors, and nonpositive or underflowed
physical lambda must produce an explicit failed evaluation. They must not be
clamped, replaced by a zero contribution, or silently interpreted as no
contour. Low-degree closed forms require independent native symbolic identity
checks and must preserve uncertainty in coefficients with zero centres.

## Resident preparation and restoration

Persist the optimized native H/Hu helper program with the recipe descriptor.
Restore it before resolving the main evaluator's external functions. Native
`EvaluationInfo::register_tagged` creates a specialized callback outside the
sampling loop. A routing registry holds only weak references to immutable
prepared helper owners and prunes dead entries on preparation; it cannot retain
sector sources or resident numerical workspaces. Keys include the helper digest,
so unrelated same-arity native programs cannot bind to one another.

Each returned callback captures its own workspace value. Its explicit `Clone`
clones the native evaluator and allocates a new scratch mutex, rather than
cloning an `Arc<Mutex<_>>`. Symbolica's `ExternalFunctionContainer::clone` invokes
the boxed callback's native `DynClone`, so copied evaluators can own independent
scratch. The callback itself retains a strong reference to its immutable helper
owner, ensuring that weak routing is never the only lifetime owner.

The mutex is required by Symbolica's `Fn + Send + Sync` external-function API,
while its evaluator has mutable scratch. H/Hu contains only arithmetic and no
root callback, so evaluating it cannot recursively acquire the callback mutex.
Measure the lock and full-root cost before accepting this representation.

`register_tagged` currently receives tags but no requested variable precision.
The proposed FastSecDec mapping boundary supplies a scoped preparation precision
while resolving a `Float` callback, mapping its saved helper program before
sampling. Fixed-precision f64 and double-double callbacks do not need that
context. This narrow lifecycle context is not another numeric implementation;
it avoids first-sample evaluator construction and an unbounded precision cache.
The proposal still needs an executable scope/clone/restore test before adoption.

## Required runtime-specific tests

- Centre-zero tracked coefficients, all native real and complex number domains.
- Native implicit higher jets against independent native explicit low-degree
  expressions, including coefficient-zero and coordinate-face limits.
- Cloned evaluators running concurrently with different coefficient arrays,
  checking independent workspace identity and actual coordinate results.
- Fresh-process restoration from saved helper and main programs, with no
  symbolic construction or Horner/CPE after decode.
- Bounded helper routing and workspace lifetime after resident owners are dropped.
- Essential invalid/underflow/solver failures with optional validation off.
- Eager, SymJIT scalar, and SymJIT batch agreement, including callback SIMD lanes.
- Actual full callback cost, root iterations and optional certification cost.

## Executed preparation probe

`target/contour-tagged-runtime-probe.rs` also passed against the independently
reviewed prepared-solver source at owner revision
`2e47574f2505a003d138aac20d45693b74186b9f`. The probe registers a native tagged
function, runs cloned evaluators concurrently with distinct coefficient arrays,
and verifies the actual `DynClone` path calls the workspace's deep clone. It
compares eager and SymJIT scalar roots over 10,000 changing sextic equations.
Preparation counts do not increase during sampling. The combined eager plus
SymJIT time was approximately 1,205 ns per pair of complete strict solves on
this host; this remains a local microprobe rather than an integrand benchmark.
Nested 192-/256-bit mapping contexts and restoration after an intentional
panic also pass. The ordinary numerical bracket is not a certified enclosure.

The callback foundation now uses native complex H/Hu arithmetic even when its
physical input centres are real. At an accepted scalar root it retains the
scalar report's real component and the imaginary part of the native Newton
correction. This avoids discarding uncertainty in zero-centred imaginary
coefficients, without changing the scalar root centre or introducing a second
AD implementation. Dedicated tracked-f64 and tracked-multiprecision controls, plus ordinary
double-double and multiprecision controls, are part of the runtime gate. Error propagation is heuristic and is
not advertised as a global certificate.


## Native foundation gate

The combined native contour filter passes **51 tests**, including all ten new
runtime callback controls and the existing fixed-mode regression suite. The
new controls include fresh-process saved-helper/main-program restoration,
actual complex SymJIT batches with implicit derivatives, native cubic jets,
tracked zero-centred real and imaginary coefficients, callback clone/lifetime
behavior, optional-check-independent bad input/underflow failure, and scoped
precision restoration. The command uses the reviewed combined Symbolica owner
revision `2e47574f2505a003d138aac20d45693b74186b9f` and the local SymJIT complex
SIMD adapter fix. A valid Symbolica license must be inherited by subprocess
controls; without it, a restricted parent's one-thread allowance prevents the
fresh-process child from starting.

The owner currently has no `EvaluationDomain` implementation for
`ErrorPropagatingFloat<DoubleFloat>` or its complex counterpart. Those domains
are explicitly absent from this prepared callback factory. Existing FastSecDec
kernels use ordinary double-double and tracked f64/Float, all of which are
covered. No parallel numeric wrapper was added to disguise the owner gap.

These are foundation tests, not a claim that dynamic artifacts, production
validation, interfaces or physical integration are delivered. The fixed v9
payload and fixed sampling path remain unchanged by this registration.
