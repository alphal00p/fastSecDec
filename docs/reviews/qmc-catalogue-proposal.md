# Explicit published QMC catalogues

## Decision and scope

Add attributed published generating-vector data and thin native constructors.
Keep Kuo33002 as the existing default during comparative validation. Preserve
the meaning of every serialized `Kuo` and `Kuo33002` setting. No CBC search,
candidate search, internal threads, new estimator, or automatic per-integrand
selection is introduced. The existing supplied-vector constructor remains
available for external data and larger point ranges.

The motivating exact relation, constant controls and unchanged-artifact
comparison are in [the six-line investigation](six-line-qmc-convergence.md).
They reject an unconditional switch to the five-dimensional winner: the same
vector loses against other candidates in the nine-dimensional controls.

## Native API

Numerica owns published data, validated point counts, dimension bounds and
provenance. Proposed additive API:

```rust
pub enum PublishedLattice {
    Kuo33002,
    Kuo38005,
    Kuo39101,
    HkknAlpha3,
}

impl PublishedLattice {
    pub fn min_points(self) -> u64;
    pub fn max_points(self) -> u64;
    pub fn max_dimension(self) -> usize;
}

impl Rank1Rule {
    pub fn published(
        catalogue: PublishedLattice,
        points: u64,
        dimension: usize,
    ) -> Result<Self, QmcError>;
    // Existing `kuo(points, dimension)` remains exactly Kuo33002.
}
```

The existing rule source enum gains the three concrete published provenance
variants. Deserialization verifies both the range and the complete generating
vector against the claimed catalogue, as Kuo33002 already does. A mismatch
rejects the document. Arbitrary supplied moduli still use the existing exact
modular arithmetic and coprimality validation.

FastSecDec's `integration::RuleSource` gains `Published(PublishedLattice)` as
an additive native choice. Existing `Kuo` remains a backwards-compatible
Kuo33002 spelling and remains the default in this slice. Validation and plan
construction call Numerica; FastSecDec must not copy tables or implement rule
selection arithmetic. Each sector receives the appropriate prefix of the
same chosen vector, preserving the democratic shared-shift contract.

Future CLI steering can expose a small `lattice = "kuo33002" | "hkkn-alpha3" |
"kuo38005" | "kuo39101"` setting. This setting belongs in checkpoint identity.
A missing old setting must not silently become a different rule on resume.
The CLI owner should either normalize an explicitly identified legacy setting
to Kuo33002 or reject its old envelope clearly. No checkpoint migration or CLI
implementation is part of the Numerica data change itself.

`PublishedLattice` is a native serde type, so a later CLI option should parse to
that type rather than maintain a duplicate catalogue enum. Existing
`RuleSource::Kuo` serialization stays the string `"Kuo"`; new settings use the
additive externally tagged `Published` variant. The integration checkpoint
already retains full native rule provenance and generators. It must agree with
the top-level selected setting on resume, including at a later refinement.
The CLI's integration configuration identity records the selected catalogue;
kernel expression identity remains independent of this numerical steering.

Settings validation calls the selected catalogue constructor even for an
all-exact problem, so an invalid point count cannot hide behind absence of
work. Each stochastic sector's full dimension is validated before scheduling;
unsupported dimensions produce an error, never a silent fallback or dimension
truncation. An all-exact problem has no stochastic dimension to invent.
Adaptive `freeze_production` must validate each chosen point count using the
same selected catalogue, while retaining its separate pilot/production streams
and transactional construction. Caller-provided allocations cannot bypass the
catalogue's count limits.

The CLI's present `qmc_design` hardcodes Kuo's count range. Its next adapter
must instead obtain the chosen catalogue's min/max counts from Numerica, grow
within those limits and then grow independent shifts. All three new catalogues
happen to share the same maximum; that coincidence should not become another
hardcoded rule. The current default remains unchanged until the numerical
evidence is reviewed and a separate default decision is made.

## Data and bounds

| Catalogue | Dimensions | Published count range | Construction |
| --- | --- | --- | --- |
| Kuo33002 | 9125 | powers of two, 1024–1048576 | order-three weights |
| Kuo38005 | 5000 | powers of two, 1024–1048576 | equal product weights |
| Kuo39101 | 3600 | powers of two, 1024–1048576 | decaying product weights |
| HKKN alpha3 | 10 | powers of two, 2–1048576 | equal-weight Korobov alpha3 |

The HKKN author describes all-N sequence use up to 2^20; the rank-one API
retains its existing minimum modulus of two. For these complete power-of-two
sets, native linear ordering and published radical-inverse ordering permute
the same points. All-N non-power-of-two sequences are outside this API.
These bounds are capability bounds, not promises of useful error at tiny N.

The three new numerical files are imported from the QMCSoftware-maintained
[LDData distribution](https://huggingface.co/datasets/Sou-Cheng/LDData/tree/5c55b76bf6b6aba3415a0dece9cc1269ff1be883)
at revision `5c55b76bf6b6aba3415a0dece9cc1269ff1be883`, whose
[Apache2 notice](https://huggingface.co/datasets/Sou-Cheng/LDData/blob/5c55b76bf6b6aba3415a0dece9cc1269ff1be883/LICENSE.txt)
is explicit. Every component has been compared against its original author's
file. Preserve the original text/header, source hashes, author attribution,
distribution notice and existing full Apache2 license in Numerica. A compact
little-endian representation may be derived once, with documented source and
payload hashes; builds and execution download nothing. These are numerical
data, not copied integration code.

## Validation and later default decision

Meaningful production tests cover catalog bounds/provenance, published first
components at multiple moduli, serialized-provenance tampering, and existing
partition/worker reproducibility with a newly published rule. A scientific
constant-integrand regression at five dimensions should demonstrate that a
selected higher-order-aware alternative avoids the documented Kuo33002
plateau using the same native periodization and shift covariance.

Before proposing a default, compare the unchanged four-, six- and
seven-dimensional physical artifacts using three independent seeds, matched
actual counts, native weighted precision checks, and complete shift vectors.
Use the existing native mean/covariance estimator also for the paired vectors
across catalogues; do not introduce a second statistical calculation. Record
failures and all trials, rather than selecting the smallest observed error.
Nine-dimensional constant/affine controls remain part of the evidence.

Any later default change must explicitly state its dimension policy and the
catalogue limits. Refinement may increase powers of two within that catalogue,
then add independent shifts at its published cap, following the existing
caller-driven policy. Switching catalogues invalidates reuse of old production
samples; it requires a new session. No dynamic change is justified by this
proposal. The user's application continues to own work scheduling, loops,
parallelization and stopping.

## Implemented slice and focused evidence

The native constructors, attributed data and FastSecDec settings adapter are
implemented with the default unchanged. `QmcSettings::validate` is now a
documented public preflight. `QmcSession::design` returns a typed `QmcDesign`
with selected settings and actual sector allocations; its concise display
avoids copying supplied generators into every progress event. `method()`
exposes the native statistical method for explicit resume validation. The CLI
adapter is independently owned and reviewed.

Focused validation:

- Numerica: 26 existing QMC tests and three new catalogue tests passed; all 29
  were rerun after the fixed-size `as_chunks` style adjustments.
- Numerica: nine existing Monte Carlo integration tests passed.
- FastSecDec: three catalogue/preflight/checkpoint/allocation tests, 15 existing
  QMC runtime tests and eight existing MC runtime tests passed.
- Standalone Numerica advisory Clippy completed with no diagnostics in
  `src/numerical_integration/qmc/` or `tests/qmc_catalogues.rs`. It reported 135
  pre-existing upstream diagnostics elsewhere (including duplicates), so a
  whole-Numerica `-D warnings` gate is not claimed. Those unrelated sources
  were left unchanged.

Logs: `output/numerica-catalogue-final-tests.log`,
`output/numerica-catalogue-mc-tests.log`, `output/runtime-catalogue-tests.log`,
and `output/numerica-catalogue-clippy-advisory.log`. Tests use Rust1.98.1;
Numerica declares MSRV1.89 and `slice::as_chunks` is stable since1.88, so this
change does not raise its declared compiler baseline.
