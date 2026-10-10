# Fixed-contour Jacobian sharing on the physical 1000 GeV D05 chart

The native fixed map now stores one root-free Jacobian function definition for
the regular densities of a chart. Symbolica supplies its derivatives and
inlines the retained body before the existing native Dualizer. The public
metadata still contains the original Jacobian, images and endpoint ratios.
This uses the existing `ContourDefinitions`, native State export, staging and
v12 artifact representation; it introduces no callback, determinant algorithm
or algebra implementation.

The proved real-gradient determinant route also applies at dimension six.
Dimensions one through five retain their previous route. The separate actual
six-dimensional map probe below validates this extension with the full native
Jacobian matrix, including faces. Dynamic roots remain explicit in the existing
dynamic representation; this change does not hide a strength call in a function
body.

## Scientific and ownership checks

The chart-local `Arc<ContourDefinitions>` reaches both contour metadata and
`ProgramData`, hence the existing sector cache, native function map and saved
records. Exact contributions continue to materialize native definitions before
aggregate cancellation. Fixed charts with retained definitions use the existing
branch-aware source witness: complete undeformed density, designated F, ordered
U declarations, recipe and dimensions receive the same exact coordinate
permutation proof as dynamic charts. Bare function names never establish
symmetry. Fixed charts without definitions retain the previous path.

Focused controls pass: mapping 3, symmetry 11, complete generation program 15,
artifact 48 (two explicit helper ignores), and streaming 16. The controls include
third native derivatives, an inert coordinate, endpoint restrictions, negative
F/U witness comparisons, a merged fixed chart after native save/restore, and the
complete analytic pole/finite vector for repeated endpoint subtraction in both
Symbolic/NumericalDual and Taylor/IBP modes. Existing eager and SymJIT dynamic
requested-vector checks also pass. Strict core library/test Clippy passes after
a return-type alias and test-only Copy cleanup. These are focused gates, not a
new full-workspace claim.

## Actual physical source and bounded mapping measurements

The source is chart zero from the preserved failed eight-worker generation of
the native 1000 GeV incoming-++ D05 fixture. It has six coordinates, 236 terms,
and 10,218,756 bytes of source atoms. The native prepared record is 10,373,055
bytes, BLAKE3
`a41a47544b116e377009b01c8b24099d306678adf344ca617265957553d7eee6`.
The map BLAKE3 is
`cb0c46bccb1aa6a1b3bab67f74ddd3cf6792dfe42e4d1baebc3c38741c07185f`.
All probes verify the original receipt and native record, then operate in
separate scratch directories. No old generation receipt was replaced.

The isolated determinant probe uses the actual F and unchanged images. At two
interior points and the x0=0/1 faces, for each of lambda=1e-6 and 1e-5, the
192-bit native Matrix determinant of all image partials agrees with both
representations. Maximum scaled discrepancies are 1.99e-57 (old template) and
1.13e-57 (structured). Retained determinant size changes from 1,363,406 to
836,891 bytes; native IR changes from 700 to 471 instructions. This probe alone
does not exercise subtraction or physical integration.

Whole native discovery measurements, including staging publication:

| Representation | Intermediate chart bytes | Sampled process peak bytes | Monitored seconds |
| --- | ---: | ---: | ---: |
| Original six-dimensional template | 683,813,705 | 2,178,412,544 | 10.931174 |
| Structured determinant only | 559,027,998 | 1,742,786,560 | 9.365726 |
| Structured determinant and shared fixed J | 365,568,299 | 1,287,626,752 | 7.948094 |

All three processes closed without a resource limit. The last representation
reduces retained chart bytes by 46.5% and sampled peak by 40.9% relative to the
original. These are non-idle feasibility measurements, not controlled timing
benchmarks. The raw records retain the initial candidate-link mistake: that
attempt selected the cached original core and is classified as a repeated
baseline, never as candidate evidence.

The shared-J run uses the coherent `fastsecdec-a9ae9e63b2ca6d28` native debug
graph. `fixed-shared-link-identity.json` records its exact rlib fingerprint,
SHA256, source hashes and probe command. The original profiling limits remain
recorded as executed; the latest discovery and compile attempts use the user's
new explicit 100,000,000,000-byte ceiling and separate 240-second wall bounds.

## Physical compilation and admission

The shared chart completed actual native formula construction and IBP
generation, producing one sector with Laurent orders [-1,0]. Native eager
compilation took 58.162 seconds; formula, generation and compilation reached
74.365 seconds. The saved native binary is 2,605,381 bytes. The process peaked at
2,233,032,704 bytes and closed at 76.315 seconds after a real contour refusal.

The refusal is retained: fixed lambda=1e-5 fails at
`[0.41,0.32,0.53,0.24,0.65,0.76]`, where the homotopy midpoint has a certified
positive imaginary F. A separate fresh process restores the saved binary,
binds the exact same 19 physical inputs, and performs the actual pilot and
readiness gate. Lambda=1e-6 passes both declared interior points and returns
the complete [pole real, pole imaginary, finite real, finite imaginary]
vectors. The larger cap remains refused. No full integral estimate or general
fixed-strength admission is claimed.

The original-chart compilation also completed: native compilation took 169.488
seconds, the whole monitored process 195.971 seconds, and sampled peak was
5,237,313,536 bytes. Its saved program is 2,443,077 bytes. Thus the new saved
binary is 6.6% larger even though the intermediate representation and observed
compilation work are smaller. These sequential runs still carry no idle-host
or general performance guarantee.

The complete four-component vectors at both admitted lambda=1e-6 points agree:
maximum absolute discrepancy is 1.4210854715202004e-14 and maximum component
discrepancy divided by one plus the original component magnitude is
8.089763278189307e-16. Orders, point values and vector shapes agree explicitly.
Both representations produce the exact same larger-cap causal-refusal message,
including the point and homotopy fraction. This establishes the scoped
compiled/restored parity; lambda=1e-5 remains unaccepted. Content IDs are allowed
to differ across the two representations and are not used as a numerical
comparator. `fixed-parity-result.json` records the raw evidence and saved-file
hashes. Both compilation groups and the fresh-restore group are closed.

Raw evidence is ignored under
`target/contour-gghh-double-box-1000-map-probe/`: `runs/`, the original and
reviewed driver sources, native link identities, monitor records, source
receipts and full check logs. The initial driver restored before dropping its
original compiled owner; the separate fresh-process restore explicitly closes
that ownership gap. The corrected driver drops the original owner before
restore and asserts the native integration-readiness gate.
