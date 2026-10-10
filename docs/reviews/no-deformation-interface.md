# Native U/F interchange audit (2026-10-10)

Scope: a graph-driven, exact pre-sector export on the explicitly authorized
`no_deformation` branch. It prepares future no-deformation work without
implementing that algorithm or changing the numerical integration pipeline.

## Existing owners inspected

At locked FeynKit `259df8790f27b8d3ef32778cd7195942691b4ef0`,
`FeynmanDiagram::from_dot` / `to_dot`, `loop_momentum_basis`, `edges`, and
`propagator_family` own the dressed graph, stable edge IDs, physical/dummy
classification, routing and denominator order. `IntegralFamily::symanzik`
validates labels and computes U=det(M), F=Q adj(M) Q−U J from the admitted
quadratic family. It uses exact native Symbolica matrix algebra and does not
contract graph numerators. `GraphIntegral` already validates widths, masses,
custom denominators, positive powers and runtime model bindings.

The new `GraphSymanzik` is a small result/view API delegating to this owner.
CLI graph admission is extracted from the existing generation loader; both
paths use the same code. Ordinary generation then contracts/parameterizes its
numerator exactly as before; U/F export stops before that step. There is no
second DOT parser, graph, momentum router, spanning-tree algorithm, CAS,
integrator, or worker pool.

A small independent graph-combinatoric check uses Linnet, but
`all_spanning_forests_of` enumerates maximal spanning forests of connected
components, not all two-forests. Its selected-hedge component iterator also
omits isolated vertices. We therefore do not introduce an incomplete forest
validator: the tests enumerate A446's native spanning trees to reconstruct U,
and use independently derived bubble/sunrise polynomials, unequal masses,
native stable-DOT replay and exact edge slots for the remaining checks.
The empty two-forest of a bubble must be retained in any future enumeration.

## Trust and applicability

The JSON is a bounded sparse rational interchange, not a mathematical proof.
An independent consumer reconstructs raw signed polynomials, exact point
substitutions and the ordinary problem. Native topology/Gram metadata is
provenance only. Relative file paths and source hashes bind the admitted
inputs; build source/dependency identities identify the producer. No relative
source file is opened by the consumer merely because it appears in metadata.

The interface keeps projective origin and positive-orthant sign analysis
explicit. Homogeneous degrees are checked termwise, including after exact
point cancellations. Neither a positive-coefficient Euclidean F nor absence
of physical zero sheets is assumed. Zero U is invalid native admission and
zero F is an explicit unsupported CLI outcome, not a silently dropped split.
Graph-only positive powers do not acquire signed-sector or pinch semantics.

The point format rejects TOML floating values and nonliteral arithmetic.
Native run-card/model admission remains authoritative; its already-fixed
scalar bindings are exported separately from the explicit runtime point.
Integration's floating default point is deliberately not an export point.
Unresolved functional/nonrational coefficients cause a clear error.

The independent read-only review traced the extracted graph admission back to
the previous generation path. It preserves model/card scalar binding, real
mass and zero-width admission, native momentum conservation, the external Gram
map, positive edge powers and auxiliary external momenta. The ordinary path
continues into its existing numerator contraction; the export path does not.
Changing propagator orientation or powers does not silently change the exact
parameter-to-denominator association. Zero/negative graph powers remain
rejected; signed or pinched family sectors need a separate explicit interface.

Exporter point bindings are closed signed rational literals. Native symbol
names and aliases are resolved without ambiguous shadowing, duplicated
bindings or assignments to Feynman coordinates. Constraint parsing estimates
term count, degree and coefficient height on raw tokens before symbolic
expansion; a negative integer token is supported. Exact sparse reconstruction
retains rational scale and physical F sign. Source fingerprints describe bytes
actually admitted by the native loader; output files require a new directory
and use atomic no-clobber writes. Neither metadata nor a hash authorizes a
mathematical assumption in the consumer.

The native parser, model and determinant calculation still require an outer
wall/memory limit: the serialization caps are not an execution-time guarantee.
The independent symGCAD consumer has stricter coefficient/degree limits than
the exporter for some large version-1 inputs. Explicit rejection at that
boundary is safe, but a producer success alone is not an import-success claim.

## Validation

The focused native probe compiled against the locked native APIs and passed
24 tests: three A446, four new generic graph U/F and 17 existing native-input
regressions. It exercises massive bubble U=x+y, F=(x+y)²−sxy; unequal masses give
F=(x−y)(x−4y) at s=10. The two-loop equal-mass sunrise checks
U=xy+xz+yz and F=U(x+y+z)−10xyz. Additional checks cover loop numerators,
raised powers/measure independence, stable edge binding, native DOT replay,
invalid labels and explicit zero-F behavior. A446's complete 15-term U and
57-term signed F match the upstream GCAD coefficients symbolically, including
the exact asymmetric edge assignment, and at
`mt2=1,mz2=5/18,s=10,t=-3`. A separate native Linnet enumeration of its 15 trees
reconstructs U. Its external `mz2` is a declared Gram invariant; the test does
not substitute an implicit Standard Model MZ default or claim full-amplitude
equivalence. See the [fixture provenance](../../examples/no_deformation/a446/README.md).

The exact native command is recorded there. The retained log
`/common/dev/gcad/artifacts/fastsecdec-native-symanzik-tests-03.log` has SHA-256
`a0c6c7bd121003e873bb761d54eeb2fa6c172e98650c882f518589b989761776`.
No correctness blocker was found in the reviewed native graph/U/F ownership,
admission extraction, point/constraint checks or signed export contract.

All six focused CLI exporter tests also passed. They independently compare
exported fixed-point polynomials to these hand formulas, bind companion TOML
hashes, reject floating/coordinate/unknown substitutions, retain symbolic
kinematic constraints, and refuse overwriting output. The command used the
same Nix/resource wrapper with
`cargo test --locked -j2 -p fastsecdec-cli --bin fastsecdec symanzik_export::tests -- --test-threads=1`.
Its log, `/common/dev/gcad/artifacts/fastsecdec-symanzik-cli-tests-01.log`, has
SHA-256 `5cb8a0f846b140fe58852494fef419cc1d716fe2966cabccc939e94df4678703`.
The nine focused independent consumer tests passed separately, including
signed-scale, domain, point, map and sparse-resource tampering; their log
`/common/dev/gcad/artifacts/fastsecdec-consumer-focused-tests-01.log` has SHA-256
`650ee10c095a7f698111312b518dd6112a83ecf2f0b409d1bb72af5bc19e6bff`.
The actual process gates subsequently passed: massive bubble export, import,
solve and fresh verification produced three cells; the native A446 fixture
produced eight cells with both F signs. The A446 test first compared the actual
exported U/F to every retained corpus coefficient with the exact point and
native edge map. Its imported problem then changed only solver configuration
and resource limits, keeping all mathematical fields. Homogeneity and monotone
reductions were replayed by the ordinary independent verifier.
An additional symbolic bubble gate retained the explicit `s>4` constraint and
verified three cells, with its exact input/hash audit retained at
`/common/dev/gcad/artifacts/fastsecdec-symbolic-bubble-01/audit.json`.

The retained A446 process log
`/common/dev/gcad/artifacts/fastsecdec-a446-interop-01.log` has SHA-256
`2264467a9cbf5fa2c9e0c3ede613c11976268dc13f3dbb2f73fe51ee33f67c6c`;
its original-coordinate proof has SHA-256
`25c85937d3870d4c7390862504a0cecffb6a8be0337f35b553a94456df8b330f`.
The bubble process log
`/common/dev/gcad/artifacts/fastsecdec-bubble-interop-02.log` has SHA-256
`e9d93d71baedec5686a5f16c5dde6939935c6a9ab23a7f9340f32779513f1114`.
This establishes the tested graph-to-polynomial-to-CAD workflow. It does not
extend the historical corpus completion count or implement no-deformation
integration, amplitude numerator matching, or simplex output transport.
Root's supplemental exact rational sanity check also passed 16 fresh points
(seed `20261010`), 112 constraint checks and 16 F checks, with no failures or
unchecked evaluations. Its report SHA-256 is
`9ec7a4bb60bbdda781332ddbac6ccaf7c0b809650deabf53af6a4b07d4695c6a` at
`artifacts/fastsecdec-a446-interop-01/supplemental-sanity.json` in the symGCAD
workspace. This sampled diagnostic is supplementary to the exact verifier.

Independent reviewers: algorithm_research (native graph/forest/kinematic
ownership and physical A446 fixture), symbolica_audit (separate-process exact
consumer, sparse binding and homogeneity checks). Their reviews do not grant
coverage to any downstream solve; normal independent solver verification is
still required.
