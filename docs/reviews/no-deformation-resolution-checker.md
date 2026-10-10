# Supplied local marked-ideal certificate checker

The `threshold::resolution` module checks supplied local polynomial certificates.
It does **not** implement a general Bierstone–Milman producer, choose a globally
maximal center, prove termination, construct a real integration atlas, or finish
the no-deformation strategy. Its current scope is native Q-polynomial ambient
charts, mutually inverse polynomial relative frames, principal opens, adapted
coordinate centers and a supplied standard blowup chart. Rational/algebraic
inverse frames and parameter-stratum relation ideals remain outside this first
registered slice.

The general obligations remain those in
[the resolver specification](../NO_DEFORMATION_RESOLVER.md) and
[its proof addendum](../NO_DEFORMATION_RESOLVER_PROOFS.md). An accepted local
certificate cannot be substituted for their producer, all-strata, real-coverage
or analytic-continuation gates.

## Native reuse and checked objects

The small modules under
[`threshold/resolution`](../../crates/fastsecdec/src/threshold/resolution/mod.rs)
separate native algebra and resource accounting, local charts/frames, marked
ideals, and controlled transforms. They use Symbolica Q-polynomials, exact
derivatives and substitution, native quotient/remainder, Groebner reduction,
native integer gcd and native matrix determinants. There is no alternative
ideal, polynomial, rational, matrix or differentiation engine.

The public inputs retain native source/target ideals and declared coordinate
roles. Checked wrappers have private state. Parameters stay inert. Principal
opens retain explicit inverse-unit relations and a rational real witness; the
witness establishes nonemptiness at that point, not smoothness or coverage of
an entire parameter chamber.

The checks include:

- Both polynomial frame compositions equal the identity, and the native
  relative Jacobian is a unit on the declared open. Retained boundary equations
  become a checked unit times distinct adapted coordinates, with IDs, birth
  stages and source multiplicities. The unit determinant alone is insufficient
  to claim boundary adaptation.
- The center lies in the marked-ideal cosupport: all normal multi-derivatives
  of order below the mark vanish modulo the center ideal in the verified
  relative frame/localization. A witness must lie on that center. Nonlinear
  frames use native pullback before differentiation; original-coordinate
  partials cannot replace adapted derivatives.
- Common marked sums use a checked LCM and retain every mixed generator in
  each ideal power. Ordinary and adapted logarithmic derivative towers remain
  distinct. The supplied coefficient operation checks a local ordinary upper
  order bound, the contact-coordinate derivative-ideal membership and boundary
  transversality, and retains the restricted principal-open guards.
- Companion arithmetic checks native extraction and recombination of the
  supplied boundary monomial for every generator, excludes remaining common
  adapted boundary factors, and checks the supplied local residual order. A
  unit residual returns the separate monomial case.
- A controlled transform matches the standard coordinate blowup formula. The
  native determinant, exceptional division, source recombination and equality
  of the claimed target ideal are checked. Inherited divisor equations, units,
  IDs and multiplicities remain in the record.

`DivisorTransform::exceptional_multiplicity` is the vanishing order of the
**reduced defining equation** (zero or one in these adapted charts).
`source_multiplicity` is separate and must weight that order when a later
producer forms the full divisor. Neither quantity is the marked ideal's
controlled weight. The checker does not assert real absolute-Jacobian
orientation or coverage by the other blowup charts.

`RecordedInvariant` is explicitly supplied bookkeeping. It is not a certified
global maximum or a termination invariant. The coefficient/companion arithmetic
does not select the next published BM algorithm stage on the caller's behalf.

## Errors and caller-owned resources

Malformed or contradicted certificates return explicit errors. Checked marks,
LCMs, generator counts, degree/term bounds, product sizes and a cumulative
ideal-vector-slot budget precede large checker-owned constructions. Native F4
internals have no exposed hard allocation/time guarantee here. Their work must
remain under caller-owned hard resource bounds; a cutoff is inconclusive.

Exact zero generators are normalized away at ideal admission. A focused native
probe found the owner currently attempts to make zero generators monic and
panics; the two failing controls and backtrace are preserved, and an isolated
owner fix is handled separately. No sampled zero test or alternate Groebner
implementation was introduced. An empty mathematical ideal is distinct from a
vacuous principal open: the latter is rejected.

No library-owned worker pool, integration loop, evaluator policy, artifact
schema or physics reference changes are part of this slice.

## Executable evidence and independent review

Before registration, all six source files were frozen and independently read
by the root and runtime reviewers. The registered files are byte-identical to
that reviewed snapshot. Native consumer revision:
`c540d3f68c90fe7bff1e507458e57a20fb95b11c`.

The private direct Rust test build ran **6/6 controls**, zero ignored, in
0.02 seconds reported native test time; strict direct Clippy passed. Tests cover
the nonlinear relative frame `(x,y-p*z²,z)`, center/coefficient/transform
recombination; wrong inverse frames, centers, boundaries and contact witnesses;
missing exceptional factors, wrong target ideals and mutated divisor records;
localized membership and vacuous opens; mixed ideal powers and preflight
limits; and companion extraction with unchanged divisor history. These tests
assert exact native identities and meaningful failure cases.

Separate native operation probes passed **5/5 controls**, covering localized
nonvacuity, native inverse-frame chain rule, mixed marked products,
coefficient/companion operations, logarithmic derivative factors and finite
derivative-open algebra. They establish API capability for this implementation;
their count is separate from the six maintained checker tests.

Ignored receipts, exact compiler/link inputs and logs are retained at
`target/no-deformation-resolution-draft/{handoff,result}.json` and
`target/no-deformation-resolution-probe/result.json`. The frozen handoff SHA256
is `73dc6cb0a2a75c883b7dea5c86894d27bfe992b0d4ce985ff1ba780ed96c81ee`.
The registered module passes **6/6** focused shared-target Cargo tests and
strict native Cargo Clippy (`--lib --tests`, threshold feature, warnings denied).
The logs are `target/no-deformation-resolution-integrated-tests.log` and
`target/no-deformation-resolution-integrated-clippy.log`. Earlier surrounding
threshold gate counts do not include these six tests. No endpoint-normalization
producer or numerical integration claim follows from this local checker gate.

## Separate next-stage feasibility evidence

The subsequent ignored étale probe passed **5/5 controls** and strict direct
Clippy against the same owner. It uses native `Matrix::solve` and determinant
clearing to construct relative derivations, differentiates inverse-unit
relations explicitly, checks relation-ideal preservation with inert parameters,
and verifies commuting lifts in a two-free-coordinate algebraic tower.

The finite-open example is `g=x²+y²-p`, `I=(g²)` on `p != 0`. Native ideal
arithmetic proves that the opens `x != 0` and `y != 0` cover `g=0`; one open
alone fails, and the assertion fails at `p=0`. Collision, wrong-minor and
wrong-derivative controls are retained. Inverting `2x` on `x²=0` is explicitly
rejected as an empty localization.

That probe and its proposed data contract are at
`target/no-deformation-etale-probe/{probe.rs,result.json,README.md}`. They are
not registered in the supplied polynomial checker. Algebraic open coverage
does not select real sheets or certify every parameter fiber. The circle has
no real points for `p<0` despite its nonempty complex algebraic fibers.

The next implementation needs separately reviewed affine quotient charts,
relative derivation frames and finite-open certificates, followed by a producer
of admissible centers and the published invariant/history mechanism. Effective
real chart coverage, all closed-face normal forms and derivative limits,
ramified branch handling, measure transport and auxiliary-regulator removal
remain substantial open parts of the requested general algorithm.
