# Prepared moving-root runtime reuse

The adopted Symbolica revision `c540d3f` already provides
`solve::nsolve_bracketed`, accepting a caller-owned prepared value/derivative
callback. Its implementation is in `src/solve/bracketed.rs`; owner controls in
`tests/prepared_bracketed_root.rs` cover native precision domains, tracked
uncertainty, bracket convergence, nonfinite values and explicit solver limits.
No new numerical root solver is justified.

A focused executable probe prepares one native Horner-10 vector for
`P(r,a)=r^5+r-a`, its native symbolic derivative, and the first two implicit
derivatives. For this control only, `a>0` gives a current bracket `(0,a)` and
`P_r=1+5r^4>0`. Five distinct parameters pass root and independent finite
difference checks. Reordering requests yields identical root bits. Reusing a
sample-fiber interval at another parameter is explicitly rejected. Three
controls pass against the adopted native library; they neither construct an
evaluator per sample nor search for all complex roots or consume sampling RNG.

This is API/source/probe evidence, not a general moving-root certificate. A
future threshold callback must derive a valid current bracket and root selector
from its algebraic branch certificate, handle boundary collisions through the
resolved branch, and preserve the existing precision-rescue and native symbolic
derivative machinery. The native bracketed solver assumes its supplied bracket;
its floating-point success alone is not a global branch proof.

Symbolica's general `root`/`isolate_real_roots` APIs use shared root-set caches;
their generic all-root path should not be used unnecessarily for every moving
coefficient fiber. Native exact rational interval isolation and symGCAD's
`FiberRoot`/`refine_once` remain useful certified preparation operations. A
sample fiber's enclosure is not a reusable runtime bracket.

The ignored probe and logs are in `target/no-deformation-root-runtime/`.
Probe SHA256: `664e6905e49f7680f594d69036a4872dbe3fa5a34e7c00408a8369310bf2372e`.
Native Symbolica rlib SHA256:
`e792cd8f93c24e741d3502f3827bd80c17c7e3bfe136c465447e6c4af3396a25`.
