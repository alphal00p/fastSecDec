# Unchanged-science hard four-loop numerical continuation

The first attempt generated and compiled successfully, but its numerical
watchdog interrupted the fourth coefficient at 750 seconds. Provider setup
consumed roughly 636 seconds before native QMC started; all four calls had
begun by 731.038 seconds. The complete physical tuple is absent, and the first
attempt remains an unaccepted timeout. Its package is nevertheless complete and
unchanged, so neither symbolic regeneration nor C++ recompilation is needed.

Root proposed a separate fresh numerical attempt with a 1,200-second numerical
deadline and 1,250-second outer deadline, CPU0 and the same 30 GiB process-tree
limit. These are new predeclared bounds justified by the retained stage progress,
not an extension or rewrite of the first attempt. Success is not guaranteed.
The concrete ignored wrapper is `output/probes/retry_hard_four_loop_numeric.py`,
SHA256 `e675a610d931d79f511d42bf69fede88c6dad39bee687af45723bd8466d8fce1`.
It parsed successfully and passed root/HEP source review. The coordinator
authorized one fresh attempt after HEP accepted the immutable failed parent.

The wrapper admits only the successful-generation/successful-compilation parent
followed by the recorded numerical timeout, no physical tuple, and absent
process groups. It verifies the frozen original caller and phase wrapper,
recorded full density/domain/measure, all four native orders -3 through zero,
2,676-sector ordinary library and exact numerical settings. Its library SHA256
is `de2a9eadb602f7eaf904316bae0c49b1fcdba47016b391f9ae50e521a3f81222`.
The 43,518-file compiled/final manifest matches, and the new package copy must
match every byte and nanosecond modification time before proceeding. Parent
records, copied files and all scientific/guard/tool sources are bound in the
new preparation and checked again afterward.

Both fail-closed controls repeat. The only scientific invocation is the same
guarded frozen caller's `--integrate OUTPUT 180`: `together=True`, seed 20261219,
Korobov3, no fitted transform, published dn1 rule N=8311, 32 shifts, native
maxeval265952, one numerical thread, complete positive orthant and unit
prefactor. The provider's ordinary `wall_clock_limit=180` remains unused, as
before; the external 1,200/1,250-second deadlines are authoritative. The caller
persists the complete original string tuple before using the existing converter.
No earlier partial call is reused or pooled, no coefficient is dropped, and
the extra external order -3 remains explicit even if it reports zero.

Proposed invocation, after source review and coordinator launch handoff:

```sh
DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/retry_hard_four_loop_numeric.py \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/hard-four-loop-together-attempt-1 \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/hard-four-loop-together-attempt-2
```

The later stages import no Symbolica and may overlap separately coordinated
native correctness work on another CPU, with no performance claim. There is no
automatic additional retry. The existing ordinary `together=False` option is
left as a separate possible source investigation; this continuation deliberately
retains the already reviewed full-amplitude uncertainty path unchanged.

## Attempt 2 launch

The new directory is
`output/diagnostics/remaining-pysecdec/hard-four-loop-together-attempt-2`.
Preparation admitted the failed parent and verified/copied its 43,518 files
(692,292,862 bytes) with identical nanosecond mtimes. It binds the finalized
independent parent audit SHA256
`5b36eca998d915e90ff6a7d70f1470f9a25e023ddb4b6a7681f318b4ef2b5e43`.
Both fail-closed controls completed successfully. Numerical PID/process group
3863369 is running on CPU0 under the predeclared 1,200-second bound; the outer
scope retains 1,250 seconds. No generation, compilation or new numerical
settings were introduced.

## Retained terminal outcome

Attempt 2 completed successfully. The numerical child exited zero in
1,101.467362 seconds within its 1,200-second deadline, and the outer stage
exited zero in 1,105.096355 seconds within 1,250 seconds. All owned process
groups were reaped and are absent. Final checks report unchanged parent and
copied packages, parent records and sources. Both the original string tuple
and the converted provider result exist.

Native sum construction reached the first QMC call at provider timestamp
731.278 seconds. Four calls each report 265,952 evaluations; the last returned
at 1,100.308 seconds. These are scalar summed-coefficient evaluations, not native
complete-vector sample counts. The complete physical tuple reports:

| Order | Provider mean | Provider standard error |
| --- | ---: | ---: |
| -3 | 0 | 0 |
| -2 | -3.59880765915927636 | 0.00973153194133354570 |
| -1 | -16.6655009796509148 | 0.0890322313939589521 |
| 0 | -150.889530987369028 | 0.858594582402794293 |

The provider's zero error remains a reported statistical value, not an exact
reference classification. The unit prefactor leaves physical tuple member two
identical to member zero; all members remain retained. Independent outcome
review and versioned native transport are separate acceptance steps. The old
three-order native estimate and its covariance remain unchanged, with an
explicit missing-estimate comparison row at -3. The separate current-native
zero-through-minus-three proof does not pad that historical numerical vector.

The finite-part relative error is about 0.57%, so this fixed-work observation
does not meet one-per-mille precision or establish uncertainty calibration.

## Native transport

HEP independently accepted the complete physical tuple and source/normalization/
uncertainty path. Its frozen audit SHA256 is
`915c1e4604f63351ef0e9507614185f85ad60b1a836540839091ddd683feee95`.
The native writer uses the existing `ReferenceResult`, encoder and comparison
APIs; all eight provider mean/error bit patterns are asserted. The real-valued
provider supplies no separate imaginary observation or cross-order covariance,
so those fields remain null.

`examples/references/four_loop_hard.json`, SHA256
`328d55c9efc9760a24fb5ee03b9b2bccee5bf2aa99ecc2984b188f6e727ecd73`,
retains all four reported statistical errors. The historical estimate and all
nine covariance entries are unchanged. The comparison remains ineligible with
`MissingEstimate` at -3; common-order diagnostic pulls are -1.628491, -1.401696
and -1.256181. The independently accepted exact-zero certificate is attached
separately and supplies no synthetic numerical row.

The existing CLI reference transport target passed seven tests with two ignored
recording/replay helpers (`output/hard-four-loop-reference-tests.log`). The new
regression checks exact provider bits, source hashes, statistical zero-error
classification, missing-order semantics, unchanged covariance and roundtrip.
Focused Clippy and formatting checks passed. Final fixture transport review is
recorded separately in the independent reference note.
