# Native Marimo browser acceptance (2026-10-07)

This review distinguishes actual browser interaction from Python constructor
checks. The installed native community host is revision
`9960ba17207bc49f3a045e5a1ffea37eb4b8187a` with the isolated FastSecDec bindings,
Python 3.12.6 and Marimo 0.24.2. The local build used Rust 1.99 and Apple Clang.
`nix-shell` is unavailable on this host. Native portable-feature tests do not
establish execution of this new notebook in Pyodide.

## Scientific browser checks

The browser built 152 native diagrams (5 one-loop, 147 two-loop), selecting D035
by default. D035 generated no numerical sectors and returned the native exact
zero through both QMC and Havana. No numerical failure was converted into zero.
Changing selection to D150 did not start generation or sampling.

Explicit D150 generation produced four eager evaluators, Laurent orders -1 and
0, and 13 runtime inputs. The simplified numerator displays symbolic couplings,
masses and Gram products. Its native scoped pager navigated from terms 1–25 to
26–36 and into a nested sum. Explicit sector inspection selected sector 0 at
order 0, paged its actual integrand, then selected sector 1 at order -1. The native
complexity bound reduced a large first page to two terms and the next page
contained terms 3–9. An invalid sector produced a clear inspection error.

The two-million-point box QMC browser run paused at 262,144 assessed points with
complete-lattice observations and resumed to 2,097,152 points / 128 complete
sector shifts. Full Laurent covariance remained available. While QMC ran,
physical controls changed from sqrt(s)=300, mt=172.5, cos(theta)=0.8 to
sqrt(s)=320, mt=180, cos(theta)=0.3; its original bound dot_0_1 remained 45,000.
A subsequent explicit Havana allocation reused generated evaluators and bound
51,200 for that product, completing 32,768 production points / eight batches.
Choosing 64 points for Kuo-33002 was rejected without discarding the completed
result. Triangle Generate / Inspect / QMC / Havana also completed through the
scalar frontend (8,192 QMC sector points, 4,096 MC production points).

## Browser-discovered corrections

- Marimo's frontend rejects a 50 ms refresh even though its Python constructor
  accepts it. The final timer is 125 ms; native work retains a 50 ms budget and
  observations retain a separate one-second cadence.
- The first native Gram preparation numerically froze polarization normalization.
  All nonzero Gram entries now remain runtime symbols; only structural zero
  products are specialized. The large binary-rational numerator constants
  disappeared. Independent coupling-scaling and template-byte controls accompany
  this browser observation.
- Native static import discovery incorrectly visited browser-only micropip.
  Platform-local dynamic imports now keep those packages out of native discovery.
- A native Pager first displayed in the recurrent dispatch cell lost its Marimo
  comm after later integration actions. Viewer creation/display must belong to a
  stable prepared-input or inspection-only cell, independent of monitor updates.
- Native owners are thread-affine. Optional export must serialize in an explicit
  caller action and provide byte-backed download links, rather than closing over
  owners inside Marimo's background download callback.

A further collapse/reopen check exposed Marimo's accordion unmounting its
children and disposing of the last Pager view. Raw/simplified numerator
containers now use native HTML disclosures whose descendants remain mounted.
This is separate from the owning-cell correction; no private Marimo API or
native numerical implementation was changed.

The final 4,096-point caller packages preserve 256-point numerical evaluator
batches. A fresh browser run completed the same 2,097,152-point box allocation in
5.88 seconds active wall time versus the earlier 115.92 seconds. An independent
timer-free native comparison confirmed bit-identical complete means and full
covariance for 128-, 1,024- and 4,096-point packages. A larger browser allocation
paused at 110,592 / 4,194,304 points with four complete sector shifts and correctly
unavailable full uncertainty, then resumed to 4,194,304 points / 256 shifts.

The scalar frontend's explicit Prepare downloads action produced byte-backed
links. The downloaded 3,141-byte artifact loaded in a fresh process as two eager
triangle sectors with one runtime parameter. No serialization callback crosses a
thread-affine native owner into a background executor. The final ggHH export also
loaded in a fresh process: 28,242 bytes, eager backend, all 13 runtime inputs.
Its complete JSON report parsed successfully and contained no host paths.

## Final browser acceptance

The final disclosure implementation passed the complete sequence: open simplified
numerator, navigate to terms 26–36, collapse it, open/close raw numerator, reopen
simplified numerator, explicitly Inspect, complete QMC, complete Havana, then
repeat the collapse/raw/reopen sequence. The simplified pager retained terms
26–36 and the sector integrand retained its own first page. Returning to page one
worked normally. No new missing-model or application error appeared after the
final fresh connection; older errors in the browser log belong to the explicitly
replaced implementations/server connections above.

The final screenshot, retained locally under ignored
`output/notebook-workflow/verified-numerator.jpg`, shows the native formatted
parametric numerator, working controls, and the four-sector / thirteen-input
summary. Both notebook tabs remain available for direct inspection. The final
source passes 88 notebook tests and both Marimo checks. The installed binding
suite passes 73 tests. Native owned-source workspace gates pass 486 tests (25
intentionally ignored), strict Clippy and formatting; portable host gates pass
59 tests. See the independent reuse review for source/identity boundaries.

This closes local native browser acceptance. It does not claim a fresh actual
Pyodide/Wasm execution or remove the upstream special-function performance limit
below.

## Limits

The reviewed local path uses one caller and native eager evaluators throughout.
The scalar massless-box default QMC case has a separate known cold 1000-digit
`polygamma(1,2)` preparation cost; its bounded 120-second probe did not complete.
Triangle, rank-two box, coupled sunset and exact D035 passed both native methods;
scalar massless-box Havana passed. See `eager-bindings.md` for source-level reuse
and cache evidence. No precision policy or numerical result was substituted to
conceal this limitation.
