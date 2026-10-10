# A446 native graph fixture

**Status: native exact tests and the full CLI interoperability test passed on
2026-10-10.** This fixture
binds a dressed graph's scalar denominator polynomials to the A446 corpus. It
does not claim a CAD completion or reproduce the finite integral's numerator,
dimension shifts, normalization or propagator-power prescription.

The graph is a theta with two three-edge massive paths and one massless rung:

```text
left --x1-- ga --x2-- za --x3-- right
left --x5-- gb --x4-- zb --x7-- right
left --------------x8--------- right
```

The six massive edges form a directed top-quark loop; the rung is a gluon.
External gluons enter `ga,gb`, while Z bosons leave `za,zb`. The native HEPKit
DOT parser/model/loop routing remain the owners; this file is not a second graph
representation consumed by the code.

| Native internal EdgeId | Corpus parameter | Particle | Directed endpoints |
|---|---|---|---|
| 4 | x1 | t | left → ga |
| 5 | x2 | t | ga → za |
| 6 | x3 | t | za → right |
| 7 | x5 | t | gb → left |
| 8 | x4 | t | zb → gb |
| 9 | x7 | t | right → zb |
| 10 | x8 | g | left → right |

External EdgeIds 0,1 are incoming `P0,P1`; 2,3 are outgoing `P2,P3`.
The exact kinematic convention is

```text
P0² = P1² = 0,  P2² = P3² = mz2,
s = (P0+P1)²,  t = (P0-P3)²,
u = (P0-P2)² = 2*mz2-s-t.
```

All ten symmetric Gram entries are supplied to native `Kinematics`, including
the momentum subsequently eliminated by native momentum conservation. In
particular `P0·P2=(s+t-mz2)/2` and `P1·P2=(mz2-t)/2`. Swapping the two Z labels
changes which of `t,u` enters the polynomial; it is not an innocuous omission
from the provenance. The native mass convention is `q²-MT²`; corpus `mt2` is
identified with the square of that model parameter. The internal width `WT` is
explicitly zero. External on-shell squared masses are supplied by the Gram
conditions, not taken from floating default model values.

`expected_uf.json` contains only the literal U/F sparse arrays from
`tests/ZZ_integrals/finiteA_7_446_11_0__0_5_1_1_1_1_0_1_1.py` in the GCAD
reference corpus at upstream revision
`fb6ebe3aa0e3fdd073f410d01050a4ed7e235f4e` (the revision pinned by symGCAD's
`PROVENANCE.toml`). The actual upstream checkout root was
`/common/dev/gcad/DO_NOT_PUSH_REFERENCES/gcad`; its source file was checked
byte-for-byte against the extraction copy in `.reference/gcad`, which is not
itself a Git checkout. No enclosing symGCAD revision is used as upstream
provenance.
Its source SHA-256 is
`764d59c5b77e4076992083183d230dba6b046197b614ad7fa3996aaf935eb345`.
There are 15 U terms and 57 F terms with symbolic coefficients in
`mt2,mz2,s,t`. Extraction used Python AST literal reading only; polynomial
construction and every equality check use Symbolica in the Rust test.

The expected first polynomial is

```text
U = (x1+x2+x3)*(x4+x5+x7)
    + x8*(x1+x2+x3+x4+x5+x7).
```

The tests call `GraphSymanzik::from_graph` and compare its signed F against
every original coefficient, first symbolically with `mt2=MT²`, then at
`mt2=1,mz2=5/18,s=10,t=-3`. They also reconstruct the graph through native stable
DOT, retain the exact edge map, check raised-power independence and reject a
deliberately wrong asymmetric parameter assignment. No canonicalization up to
an unknown sign or scale is accepted.

A separate small test enumerates the 15 spanning trees with native Linnet,
forms each complementary edge product in Symbolica and reconstructs the corpus
U independently of the determinant calculation. This is not a two-forest F
implementation; the full F comparison uses native `IntegralFamily::symanzik`
against the independently retained corpus coefficients.

Run from the repository root in the documented Nix environment:

```sh
cargo test -p fastsecdec --test a446_symanzik -- --test-threads=1
```

The serial setting follows existing native model symbol-registration tests.
No sector generation, numerator contraction, CAD solve or numerical integration
is needed for this provenance gate.

The coordinated validation command, run from this repository, was:

```sh
source /common/dev/gcad/.symgcad-env
ulimit -v 12582912
nix develop /common/dev/gcad -c taskset -c 1,2,10-13 \
  cargo test --locked -j2 -p fastsecdec \
  --test graph_symanzik --test native_input --test a446_symanzik \
  -- --test-threads=1
```

All 24 tests passed: three A446, four generic graph U/F, and 17 existing native
input tests. The A446 target reported 0.21 seconds (one development-profile
run, not a performance benchmark). The retained log is
`/common/dev/gcad/artifacts/fastsecdec-native-symanzik-tests-03.log`, SHA-256
`a0c6c7bd121003e873bb761d54eeb2fa6c172e98650c882f518589b989761776`.
Tested fixture SHA-256 values are:

| File | SHA-256 |
|---|---|
| `graph.dot` | `70041d3deefb94d4bf4f2a31bc5c3aec116dc7bee3eea2b243bc175f0b4a02e6` |
| `expected_uf.json` | `29c977ae02eaa75411e2867175fdbbcd11a955d849619ba183baaf748c2d5216` |
| `crates/fastsecdec/tests/a446_symanzik.rs` | `c3fa7ca3569ae62aed3299fbc6f91aed4d08dcede6e9453809b8162a2336cde7` |

These checks establish native graph-to-polynomial provenance. CLI serialization,
independent consumer import, and any later solver coverage are separate gates.

## Reproducible CLI input

The tracked `run.toml` uses this graph and the native Standard Model, while
`point.toml` fixes runtime `model::MT=1`, `mz2=5/18`, `s=10`, and `t=-3`.
The model JSON is generated by the native public serializer; it is not vendored:

```sh
cargo run --locked -p fastsecdec --example prepare_a446 -- output/a446-input
cargo run --locked -p fastsecdec-cli -- export-symanzik \
  output/a446-input/run.toml --point output/a446-input/point.toml \
  --output output/a446-export
```

Both directories must be fresh. The preparation command prints file BLAKE3
hashes, and the exporter binds the actual graph/model/card/point inputs in its
provenance. Omitting `--point` retains the top mass and Gram invariants as
symbolic variables; the zero internal width is an explicit card restriction.

An optional Linux process-level test is provided in
`crates/fastsecdec-cli/tests/a446_symgcad_interop.rs`. It reconstructs the
exported signed U/F against these corpus coefficients before import, verifies
every internal edge slot, and keeps the imported mathematical problem intact.
Only solver configuration and limits are then added; the selected homogeneity
and monotone reductions still require their ordinary checked proofs. In native denominator aliases,
the previously successful corpus order becomes
`[x4,x1,x6,x5,x3,x2,x0]`, with homogeneity anchor `x4`; the original names are
`[x4,x2,x8,x7,x5,x3,x1]`. A fresh worker must still complete and pass independent
verification. The process test requires an explicitly supplied `SYMGCAD_BIN`
and an external memory allocation; it is ignored in ordinary Cargo tests.

The process gate passed with eight independently verified cells and both F
signs. It retained all original fixed-point mathematical inputs; this is a
workflow reproduction of an already solved fixed corpus case, not a new
corpus completion or an integration result. Native symbolic export contained
15 U monomials and 98 F monomials (expanding the original coefficient
polynomials); exact specialization retained 15 and 57, respectively.

The one development-profile process test reported 0.85 seconds, including
export/import/solve/verify, without a performance claim. Evidence is retained
under `/common/dev/gcad/artifacts/fastsecdec-a446-interop-01`:

| Artifact | SHA-256 |
|---|---|
| Adjacent `fastsecdec-a446-interop-01.log` | `2264467a9cbf5fa2c9e0c3ede613c11976268dc13f3dbb2f73fe51ee33f67c6c` |
| `export/symanzik.json` | `0a9f511085e9ebdb81904f0ee5e8b00aeca1783596d22e0fb6a09c6bbeed9795` |
| `export/problem.toml` | `810929e1e8335fb9b5bdd80d08cf7e7b67af7e71ff9f9810e688ea2752977282` |
| `result.json` | `25c85937d3870d4c7390862504a0cecffb6a8be0337f35b553a94456df8b330f` |
| `verify.stdout.json` | `c25219106f3efa6cadf9071c2f7b0ebe11fb048b588090b602180bf927e16bdf` |

The actual ignored-test invocation used the already compiled test executable:

```sh
source /common/dev/gcad/.symgcad-env
ulimit -v 4194304
SYMGCAD_BIN=/common/dev/gcad/artifacts/fastsecdec-consumer-01/bin/symgcad \
FASTSECDEC_INTEROP_SECONDS=180 \
FASTSECDEC_INTEROP_OUTPUT=/common/dev/gcad/artifacts/fastsecdec-a446-interop-01 \
nix develop /common/dev/gcad -c taskset -c 3 \
  target/debug/deps/a446_symgcad_interop-6ff7169c241c21a0 \
  --ignored --nocapture --test-threads=1
```

The independent symGCAD executable SHA-256 was
`514fbc7565756703000de2f092fd43fc342e7071364bce5ee4ec927a07d4e975`.
For a new build, use `cargo test --locked -p fastsecdec-cli --test
a446_symgcad_interop -- --ignored --nocapture --test-threads=1` with the same
explicit environment variables and a fresh output path, rather than assuming
the test-executable hash suffix is stable. The `prepare_a446` unit regression
also passed: it roundtrips native model serialization and rejects an existing
output without changing it. Its build/test log
`/common/dev/gcad/artifacts/fastsecdec-cli-workflow-tests-02.log` has SHA-256
`5b8d8b23ff4ff1998175e9c241e9fbf47d8837ce1ccdf8ff98b19e89f72274f6`.
