# Complete 1000 GeV D05 contour generation

> Scope correction, 2026-10-10: the physical measurements below used
> `GenerationMode::NumericalDual` for endpoint reduction. They are historical
> diagnostics and do **not** measure the subsequently requested comparison of
> symbolic endpoint reduction with symbolic versus contour-only dual Jacobians.
> High-order endpoint-jet costs do not establish a limitation of first-order
> Jacobian dualization. The corrected construction and its measurements require
> separate acceptance; see [the clarification](../../CONTOUR_DEFORMATION_PLAN.md)
> and [the symbolic-endpoint audit](contour-symbolic-endpoints.md).

2026-10-10. Fresh fixed and polynomial-dynamic artifacts both contain all 30
six-dimensional sectors of the native incoming-++ D05 double box. Each contains
60 exact/stochastic records with source indices 0–29 and the complete
`[pole real, pole imaginary, finite real, finite imaginary]` output layout.
This accepts generation and publication; causal admission and the four final
integration measurements are separate runtime gates. No independent analytic
1000 GeV integral reference is claimed here.

## Input, build and native settings

The maintained [1000 GeV fixture](../../examples/contour/gghh_double_box_1000/README.md)
uses the existing HEPKit graph, model, colour-projected numerator and native
kinematic exporter. It retains cos(theta)=4/5, Higgs mass 125 GeV, top mass and
Yukawa mass 172.5 GeV, both incoming positive helicities, all 19 runtime physics
bindings and the two structural on-shell zero products. The dimension is
`4-2*eps`; the measure multiplier is one, with the existing
`prod_l d^D k_l/(i*pi^(D/2))` normalization, unnormalized colour delta and no
spin/colour averaging. The native physical source identity is
`d13a5cdc7aa156d2630b78ffc2738c732a17f7b26ea2b47f7e8b72bca934d414`.

Both fresh campaigns use the same verified optimized candidate3 executable:

- Source archive SHA256:
  `3a430ad45c0df36a1676b1b29769974064416e7afa54fef23a143ddb89248b53`.
- CLI SHA256:
  `5f6bab8d1f64af883665cc12816d6087d90e85b47e9075ab290539ad257747e2`.
- Symbolica/Numerica: `74225696cd445247fa81c499c5110decd19257ed`;
  SymJIT: `d74993ffd76a6fc322a7bcf3963fa786783a38a8`.
- NumericalDual generation, integrate-by-parts subtraction, Laurent order zero,
  coefficient-series expansion and **Symbolic** contour Jacobian construction.
  Native Auto selects SymJIT O2; direct translation, Horner iterations zero,
  CPE maximum 1000 and one optimizer core are unchanged.

The native CLI uses `generate --serial`, explicitly selecting `fixed-v1` with
eight workers or `dynamic-polynomial-v1` with four workers. Both use fresh output
and staging directories. The strict CLI dependency fence and journal build
identity are preserved: no old-owner journal is relabelled or resumed. The
native source and geometry stay parameterized; runtime strength/cap selection
does not alter these generation settings.

## Completed campaigns

The monitor polls the coordinator and its worker process group with a 50 ms
sleep; actual sample spacing also includes the `/proc` scan time.
RSS below is the sampled aggregate of those processes, in bytes, excluding
unrelated jobs and filesystem cache. Both campaigns retain the user's explicit
100,000,000,000-byte limit and close with exit zero, no live workers and an empty
process group.

| Recipe | Workers | Monitored elapsed seconds | Peak aggregate RSS bytes | Published data bytes |
|---|---:|---:|---:|---:|
| Fixed | 8 | 84.192330 | 14,172,545,024 | 79,036,643 |
| Polynomial dynamic | 4 | 1,104.377935 | 59,636,342,784 | 175,079,674 |

These are complete observed generation campaigns, not summed worker CPU time.
The polynomial campaign actually overlapped the final fixed integrations and
other checks, as permitted by the user. Its original raw monitor's planned
“no final timed integration overlap” text is retained as historical text and
does not describe the executed concurrency.

| Recipe | Manifest SHA256 | Data SHA256 |
|---|---|---|
| Fixed | `0e37a414dc9cde5c7beb79e1f1c604f430f53c13a3b7700ea809b5efe5227e8d` | `927f9191a68583738c2ad34e61400cffb209d49245ce3001a310e5f23f3f1452` |
| Polynomial | `8468fffb4848e73ada38d93489f3b28b544c9cd2f752cd3dcca58e25d0b96ab0` | `96105494dcc46f7a6e018ad8fd02b60a054fd057df68a4278f9069d68de36d47` |

The new fixed data file is byte-identical to the preceding owner's completed
fixed file; its new manifest records the new dependency identity. The isolated
polynomial source-zero native owner is also byte-identical across those owner
revisions. These are byte comparisons and fresh native checks, not cross-owner
CLI restores that bypass its dependency gate.

The fixed campaign completed within its original 900-second allocation. The
polynomial campaign received an explicit seamless extension to 1800 seconds
from its original start, without restarting any scientific worker. At 852.974
seconds a new watchdog verified the coordinator/old-monitor PID start identities,
began RSS enforcement and paused only the old monitor. Both bridge observations
contribute to the peak. After the scientific workers closed, the new watchdog
resumed the old parent for reaping and verified the empty group. Failure paths
kill the owned group and resume the pinned original guard. The authoritative
`extension-execution.json` reports 1104.377935 seconds and explicitly disclaims
an original-900-second gate pass; the superseded raw monitor remains intact.

## Serial ownership and remaining memory costs

The actual polynomial argv contains `--serial --workers 4`. A live mapping
sample at 09:19:43 UTC recorded a 10,543,104-byte coordinator and four workers
using 2,765,070,336, 1,715,335,168, 497,106,944 and 517,906,432 bytes. A later disk
inventory counted 29 completed mapped records occupying 38,896,576,348 bytes,
with individual records between 1,158,610,567 and 1,556,669,366 bytes. The 30
prepared source records occupied 96,639,107 bytes. These are disk spools, not
coordinator-resident symbolic expressions.

At the campaign's maximum aggregate sample the coordinator used 11,423,744
bytes. The four active workers used 20,502,052,864, 14,547,705,856,
11,456,180,224 and 13,118,980,096 bytes. These simultaneous per-worker samples
explain the aggregate; they are not claimed as each worker's lifetime maximum.

The source audit confirms that `PreparedChartSource`, `DiscoveredSector` and
formula/compiled receipts contain native record references and compact keys.
All mappings precede compilation because the existing phase boundary collects
formula keys and prepares their shared native subtraction formulas. NumericalDual
does not perform the Symbolic mode's global exact symmetry search. This phase
barrier does not require holding all mapped Atoms in memory.

Each worker restores one chart, constructs its complete unit, compiles it,
drops the generated owner, persists the exact/stochastic records and returns
only receipts. Its process is reaped before the slot is reused. Native source
and evaluator caches therefore have that active unit's process lifetime. The
coordinator copies persisted records to the final indexed archive; it does not
accumulate completed kernels. Successful publication removes the temporary
run directory after the final archive is durable.

Remaining local amplification is concrete: `streaming/codec.rs` clones Atoms
into its native table, builds an encoded payload, places it in an envelope and
encodes/copies the final envelope buffer. Compilation also temporarily overlaps
the generated unit with its compiled kernel before the explicit drop. Native
IR construction/CSE has its own temporary allocations. Streaming codec buffers
and reducing these local overlaps are possible follow-ups; neither an
unbounded coordinator cache nor retention of all completed maps was found.
No source change is inferred solely from a large per-sector peak.

## Accepted optimizations and optional Dual tradeoff

The [fixed shared-J review](contour-fixed-shared-jacobian.md) records the native
structured six-dimensional determinant and shared fixed Jacobian body, including
the full image-derivative determinant oracle and checked saved-vector parity.
The [Dual review](contour-dual-jacobian.md) records native image Dualizer →
determinant Composer → complete density before outer subtraction jets, plus
the caller-owned shared Jacobian-prefix cache. Neither introduces a new
Jacobian callback, custom algebra or incomplete first-order-only subtraction;
the existing native dynamic-strength callback remains.

The [native CSE review](contour-native-cse.md) records the separately tested
owner correction: remap existing operands before forming existing lookup keys,
preserving branch scope. This removes repeated depth-wise scans. CPE remains
the native bounded optimization, and its limit has not been tuned to hide the
earlier bottleneck. Different campaign contexts prevent attributing the whole
elapsed-time difference solely to that owner patch.

Optional Dual construction was measured on source zero after the owner fix,
using the same verified release graph and native SymJIT O2. Its full monitored
fixed/poly attempts took 33.193/224.186 seconds and peaked at
3,244,945,408/8,628,269,056 bytes. These include preparation, IBP construction,
compilation, saving and checked values. The respective evaluator-compilation
intervals were 6.801/190.550 seconds. The actual Symbolic polynomial source-zero
worker reported 91.296 seconds for compilation. Thus Dual has not demonstrated
a generation advantage; whole-campaign elapsed time is not a source-zero
comparison, and old debug-versus-new-release ratios are not used.

Dual saves 1,143,353 bytes versus 2,642,927 for fixed source zero, and 2,162,020
versus 5,743,519 for polynomial source zero. Separate fresh-process restores
reproduce each complete checked vector exactly. Across representations, the
maximum scaled differences are 4.075e-15 and 3.708e-15. Fixed lambda 1e-6 passes;
both retain the same causal refusal at 1e-5. Polynomial S=0.8, R=1 passes both
declared caps 1e-6 and 1e-5 at the two prescribed interior points. Matched native
sampling cost is owned by the runtime campaign; smaller saved programs alone
do not establish faster evaluation or an amortization threshold.

## Evidence and independent review

Raw benchmark output remains ignored. Full commands, source/build/input hashes,
record identities and closures are in `target/contour-gghh-double-box-1000-candidate3-{fixed,polynomial}/handoff.json`.
The polynomial directory also retains `serial-memory-live-audit.json` under
`runs/generation-900s/`, both monitor streams and the extension handshake.
Source-zero raw owners, independent restore records and comparisons are under
`target/contour-gghh-double-box-1000-candidate3-dual-probe/`.

Independent foundation review verified both complete publications' bytes,
metadata, source coverage and process closure, the monitor extension, and all
six source-zero generation/restore process groups with independently recomputed
complete-vector parity. Its ignored reports are
`target/foundation-d05-compiler-audit/candidate3-{fixed,polynomial}-publication-audit.json`.
Native semantic admission of every complete-artifact record and the final
integration estimates remain the runtime owner's separate evidence.
