# Native family projection: bounded off-shell triple-box diagnostics

Both the scalar and rank-two numerator completed generation and one complete
allocation after native FeynKit partial fractions reduced ten propagator entries
to eight active factors. The exact family identity is the equivalence evidence;
these first numerical allocations are diagnostics, not independently validated
Laurent coefficients or performance acceptance.

## Native operation and preserved input

The ignored shared loader `output/probes/triple_box_input.rs` reconstructs the
same native graphs and Gram matrix as the original
[off-shell audit](triple-box-offshell-diagnostics.md). It calls HEPKit's DOT/model,
kinematics and scalar-value APIs; it implements no routing or parser. The
independent momenta have Gram matrix `[[-1,0,1],[0,-1,0],[1,0,-1]]`, all four
virtualities are −1, the two selected channels are −2 and internal masses vanish.
The original graph and its ten unit edge powers remain unchanged on disk.

The diagnostic calls native `IntegralFamily::partial_fraction([1;10],32)` and
requires the independently observed single coefficient-one term with original
powers `[1,1,0,2,0,2,1,1,1,1]`. Native `sector` retains eight denominators with
positive powers `[1,1,2,2,1,1,1,1]`. Original and reconstructed rational denominator
products are exactly equal as native canonical Atoms; no expansion, custom
consolidation or hand-built reduction is used. Loop and independent external
momenta are explicitly checked unchanged.

The native contracted scalar numerator, native graph factors and extra measure
are multiplied once, then passed to the reviewed public `from_family` entry.
The scalar numerator is one; the rank-two graph retains
`k1.k3+2*(k2.p1)*(k2.p2)` in its original native basis. The existing Gaussian
implementation owns raised-power Gamma normalization, moments and measure.
Its separate exact duplicate-tadpole moment and weight 210 tests passed before
this campaign; this diagnostic does not substitute for those mathematical tests.

Both projected cases retain **1,182 seven-dimensional kernels** and all orders
`[-3,-2,-1,0]`. The original representation had the same representative count in
nine dimensions. No Laurent component is given a smaller integration support,
and no possible pole is discarded. This projection does not automatically run
for ordinary graph entry points.

## Build and bounded execution

The caller links the optimized release library from FastSecDec core revision
`fe3b72a311083ea3cafc1f15a73a43b484b4bd4c`, Symbolica 3.0.1 and SymJIT 2.26.4.
`output/diagnostics/triple-box-projected/build-provenance.json` binds the complete
caller/source hashes, compiler commands, Cargo build artifacts and native
repository revisions/patch identities. The fixed executable is
`projected-triple-fe3b72a` in that directory. The existing original-graph baseline
used optimized revision 561657b, so the two programs are not identical builds.
The preserved executable's SHA-256 is
`380396e5c328d344fc3dd9f50d4edb50cf03f3823c76a5c5368e5d0204dc41a0`.
Numerica is the clean `e4638da` feature revision; the provenance also records the
reviewed local Symbolica and FeynKit patches rather than treating their Git HEADs
alone as complete dependency identities.

Generation and integration are separate fresh processes. Generation and O2
compilation share a 300-second cooperative limit; the owned-child wrapper applies
a 310-second external deadline and five-second grace. Numerical work has a
180-second cooperative limit plus a 240-second external process limit to cover
artifact loading and serialization. The wrapper records exit, timeout, wall time
and sampled peak memory. All four processes exited successfully without timeout.

The native integration session uses explicit Kuo 38005, modulus 1024, eight common
independent shifts, seed 20261004, Korobov 3 and 1024-point packages. Two caller-owned
workers evaluate complete coefficient vectors through the unchanged weighted
precision/replay policy. The reduced seven-dimensional generator is
`[1,309,235,573,145,523,153]`. Both allocations accepted
**9,682,944 of 9,682,944 kernel-point evaluations**, with eight complete common
shift vectors and zero evaluation failures. Native centered totals and full
covariance remain authoritative; all marginal estimates, exact offsets, coverage,
actual shifts and accepted package records remain in reports/checkpoints.
Both retained kernel policies start at 128 bits, allow at most 4,096 bits and use
relative/absolute agreement tolerances `1e-12`/`1e-300`, with boundary threshold
`0.001`. The first-observation/growth replay policy uses factor 16 and a 128-bit
minimum. These are recorded evaluation checks, not a proof that every final
integral coefficient has that relative accuracy.

| Order | Projected scalar mean | Standard error | Projected rank-two mean | Standard error |
| --- | ---: | ---: | ---: | ---: |
| −3 | −0.00200605182584 | 0.00116441557572 | −0.0423076420947 | 0.000795957979871 |
| −2 | −0.256030139908 | 0.00626510026485 | 0.349609030586 | 0.00640399940488 |
| −1 | 0.887119546064 | 0.0537428654521 | −0.433038992633 | 0.0101974712121 |
| 0 | 2.60975756518 | 0.264181009754 | 0.552042295794 | 0.0417621185457 |

The corresponding original-graph scalar values were
`[-0.0056981,−0.273512,1.14585,2.96944]` with errors
`[0.0059861,0.0343550,0.131241,0.603821]`; original rank-two values were
`[-0.0451800,0.374107,−0.469203,0.329022]` with errors
`[0.0034847,0.0242249,0.0390707,0.174370]`. These separate first allocations show
no reason to alter the exact equivalence proof or remove any coefficient.
Their common integer seed and native shift prefixes can correlate the two
parameterizations. No independent-error combined pull, output bitwise-equality
claim or convergence certification is made. The smaller errors in this trial
are a useful observation, not a general QMC improvement established across seeds.

## Cost attribution, with limitations

| Observation | Scalar | Rank two |
| --- | ---: | ---: |
| Current projected generation process | 18.373 s | 16.411 s |
| Native generation before compilation | 3.767 s | 6.306 s |
| O2 compilation/constructing kernel set | 11.540 s | 8.522 s |
| Kernel transport size | 108,572,757 bytes | 53,827,580 bytes |
| Fresh artifact load | 14.891 s | 10.334 s |
| Numerical loop | 83.353 s | 24.065 s |
| Complete numerical process | 98.538 s | 34.705 s |
| Peak generation RSS | 409,068 KiB | 261,088 KiB |
| Peak numerical RSS | 467,464 KiB | 341,384 KiB |
| Conditioning checks | 815,968 | 280,075 |
| Precision rescues | 814,762 | 274,281 |
| Additional weighted replays | 2,238 | 2,104 |
| Maximum precision | 448 bits | 320 bits |

Summing native phase observations gives scalar geometry 0.228 s, mapping 1.004 s,
symmetry 0.511 s, subtraction 0.259 s and Laurent 1.199 s; rank-two gives 0.225 s,
3.876 s, 0.686 s, 0.219 s and 0.697 s respectively. Raw per-phase callbacks are retained.

Original CLI generation processes took 51.705 s / 55.406 s, with about 346 MiB / 144 MiB
artifacts; their complete integration processes took 146.205 s / 56.313 s. These are
not matched speedup ratios: the standalone caller writes one final native
checkpoint/report, whereas the original CLI also streamed full-sector statuses
and periodically rewrote checkpoints. Native source revisions differ, and a
sibling test-only release compilation was permitted during later projected
stages. Unrelated host work was not excluded. The observed smaller dimension,
transport and generation workload are concrete; end-to-end performance
acceptance requires a matched execution/persistence policy and isolated repeats.

Evidence is under `output/diagnostics/triple-box-projected/`, with scalar/rank2
`generate` and `integrate` stdout, status, arguments and process records, native
kernel transports and native integration checkpoints. No report is promoted to
an external reference. The loader also represents the separate on-shell card
explicitly, but that bounded projected run has not yet been executed.

The same native operation may help other real repeated-line topologies, including
the double-box degree-two vertex. That is a reuse opportunity to investigate,
not a reason to introduce custom denominator algebra or silently preprocess all
graphs. General automatic handling would need a reviewed policy for multiple
native terms, signed powers, exact coefficients and retained original topology.
