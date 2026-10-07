# Numerical-dual formula cache: independent review

Reviewed the generation-owned formula pipeline, chart instantiation, native
source/jet cache, opaque work admission and retained session boundary on
2026-10-07. This review is independent of the authors of those core changes.
The reviewer implemented the corresponding CLI/Python/status presentation and
public regression controls; those presentation changes received a separate
source review from the core author.

## Exact formula identity and native ownership

`formula::Key` contains the dimension, ordered complete-chart term list, exact
rational endpoint constant and epsilon slope for every axis, every complete
term prefactor, requested Laurent order, subtraction strategy and the applicable
subtraction/series/request limits. Keys are compared only inside one generation
context with a common regulator, ordered target coordinates and reserved-symbol
set. Native Atom and Rational equality and ordering own this comparison. There
is no string-polynomial identity, approximate comparison or independent CAS.

The context reserves symbols from every original prefactor, monomial power,
polynomial and exponent, even if the complete source density cancels some of
them. Native formal function/request symbols therefore cannot accidentally
capture such a name. The existing subtraction owner constructs one distinct
formal regular function for each term and expands the complete chart together.
Keeping the ordered full prefactors in the key preserves epsilon-dependent
prefactor cancellations and Laurent coverage; it does not cache separately
expanded terms or infer a zero from numerical values.

Regular source polynomials are deliberately absent from the formula key. They
are inputs to the opaque recipe, not part of its endpoint subtraction rule.
Each instantiated chart retains its own monomial map, original source terms,
coordinate association, conditioning profile and native request evaluator.
Only the immutable `Arc<Recipe>` is shared. The asymmetric numerator control
uses different regular values in two charts while sharing their endpoint rule,
so it can detect an accidental transfer of numerical source values.

The implementation calls the existing endpoint-power admission,
`numerical_dual::subtraction::expand`, Symbolica coefficient-series machinery
and native evaluator composition. Signed monomial maps, exact unregulated
endpoint admission, unsupported epsilon dependence and zero-dimensional exact
charts retain the previously admitted symbolic route. Failures remain errors;
this cache adds neither a threshold certification nor a numerical zero proof.

## Phase, scheduling and retention

Discovery completes before formula construction. A native ordered `BTreeMap`
assigns deterministic IDs to the distinct signatures, and source-chart order
is retained independently of those IDs. The existing caller-owned work API
submits formula jobs under `SymbolicStage::FormulaPreparation`. Its opaque
owner, stage, index, duplicate and full-coverage checks admit completion vectors
before any chart consumes a recipe. Different completion order cannot change
the formula/chart association. No library pool or background generation loop
was introduced.

The cooperative pipeline retains completed discoveries, formula owners and
instantiated charts. Its existing session observer finishes the current native
unit before honoring a pause. The same discovery/build/instantiate functions
serve synchronous, dispatched and retained execution. A terminal error never
returns a partially admitted integral. Cancellation remains bounded by native
unit boundaries, not by instruction-level preemption inside Symbolica.

Preparation counts distinguish completed unique builds, total unique formulas,
eligible chart uses and `uses - unique` shared uses. The last quantity is known
at discovery and is not presented as a completed cache-hit counter. Optional
observations distinguish historical/unobserved fields from a measured zero-work
phase. Worker rows identify formula jobs rather than pretending their IDs are
sector IDs. A final presentation review caught worker-local native polls using
a formula job index in the completed field: worker text now says only
“Building subtraction formula N”. The dispatcher alone supplies aggregate
completion counts. A focused regression covers a later-index formula running
before earlier formulas finish.

Mapping includes shared context setup, geometry-map/valuation discovery and
exact-key planning. Formula preparation has its own coordinator wall interval
for dispatched runs; retained sessions sum active native units and exclude
paused time. Sector assembly measures subsequent instantiation and any exact
fallback once. Nested fallback phase timers are suppressed, and coefficient
attempt observations from building a formula do not masquerade as a later
assembly stage. Compilation remains a separate phase. Human metadata preserves
these observations without changing the native v8 payload or its identity.

## Exact native evaluator reuse

The source-evaluator key is the original exact polynomial, ordered native input
symbols and complete compilation settings. The jet key additionally includes
the exact ordered ancestor-closed shape and structural-zero mask. A different
zero assumption cannot reuse a specialized native program. Each requested
multiindex, including a proven valuation shift at a boundary, retains its own
smallest native ancestor closure; no global or sector-wide maximum union was
added.

Registry mutexes publish per-key `OnceLock` cells and are released before
native evaluator construction or dualization. Identical keys initialize once
and share immutable exact programs; unrelated source or jet keys can initialize
on different caller workers. Errors are retained consistently for an exact key.
A native initializer panic propagates through the caller boundary and leaves
its standard `OnceLock` uninitialized, permitting retry. There is no process-wide
cache, new executor or external cache I/O.

The independent source audit found no remaining blocker. The separate native
cache review documents the public owner APIs and shape proof:
[numerical-dual-native-cache.md](numerical-dual-native-cache.md).

## Evidence and remaining validation

The core author ran 20 focused formula/chart/pipeline/native-cache controls and
one additional private formula-stage admission control successfully. These
cover exact key separation, whole-chart epsilon prefactors, Taylor/IBP recipe
parity, shared recipe ownership with different chart maps, retained formulas,
and foreign/wrong-stage/duplicate/missing completions. The cache author also
ran its eight focused cache/shape controls successfully (0.01 s test time),
including unrelated-key progress under a held initializer, same-key sharing,
error/panic behavior and normalized shifted jet values. Preserved logs are under
ignored `output/numerical-dual-cache-study/`; these are author-run executable
evidence coupled with this independent source review.

The public scientific target now exercises Taylor and IBP across synchronous,
one-unit retained and reversed caller-dispatch generation. It compares against
the symbolic full Laurent vector at interior points, verifies one formula for
two asymmetric source charts, and checks identical serialized eager artifacts.
It additionally cancels at the actual formula phase and rejects a missing set
of formula completions. A CLI regression checks one/four workers, phase order,
counts, exclusive timing, saved/cold-inspected metadata and unchanged binary
identity. Status and persistence controls distinguish omitted observations from
true zero work. The shared 13-control scientific target passes in both native
(1.00 s) and portable (0.63 s) builds, including these extended checks. The complete portable
consumer suite passes all 72 controls. Native status omission/zero-work and phase
timing controls also pass. The complete native workspace gate passes 544 tests,
with 26 ignored across 77 targets. It includes the CLI one/four-worker phase/persistence/identity
control and the private human-metadata historical-readback/identity-exclusion
control. Logs are `workspace-tests.log` and `portable-tests.log` under the same
ignored study directory.

The native evaluator reuse and formula cache do not by themselves establish a
speedup. Actual ggHH unique/use counts and phase wall times require the new
benchmark, and must not be borrowed from the earlier uncached implementation.
Isolated Python binding strict all-target Clippy with `python_stubgen` passes
(9.41 s). The installed notebook wheel has not been rebuilt for these new status
fields, so the newly extended Python runtime controls have not been executed.
External cache loading/saving is intentionally unimplemented.

### Final worker-label verification

The final display-only correction was validated by rerunning the complete CLI
binary unit suite: 51 passed and one ignored, including the new running-formula
index regression. Strict workspace all-target Clippy then passed in 1.39 s.
Together with the preceding complete workspace run (544 passed, 26 ignored),
this validates 545 distinct native tests; it is not a fresh 545-test whole
workspace run. The complete portable suite remains 72 passed. These reruns
change no scientific code or benchmark protocol.
