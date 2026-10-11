# Native represented graph inputs and dependency adoption

This milestone preserves the supplied numerical value before native graph
parameterization. It also adopts the small owner-library API needed to map
native kinematic assumptions without reconstructing HEPKit objects. It does
not complete represented Python inputs or the general threshold resolver.

## Native input and evidence boundaries

`GraphPoint` retains the native `FeynmanDiagram`, `Kinematics`, `EdgeId` powers,
scalar bindings, auxiliary momenta, measure, coordinates and dimension.
`ExactRepresentedGraphInput::prepare` converts supported Symbolica Float
literals in kinematic values, scalar bindings and the measure before family
arithmetic. It preserves their represented binary values, precision and source
roles. Uncertainty-bearing values are refused, not turned into exact data.

For example, a supplied binary64 mass `0.1` becomes
`3602879701896397/36028797018963968` before it is squared. Neither `1/10` nor
an already-rounded squared mass is substituted. Graph numerator, projector,
edge/vertex fragments and other unsupported Float payloads are diagnosed
explicitly. Native model mass defaults remain symbolic; explicit bindings
follow the conversion above.

The upstream scalar-assumption iterator and fallible mapper retain native
aliases and auxiliary momenta. No DOT parser, graph type, kinematics table or
independent expression walker is added. Existing Symbolica traversal and
native graph serialization provide conversion and source evidence.

GCAD requests retain exclusive exact, represented-parametric or pre-parametric
graph provenance. Raw geometry replay requires the original graph point,
repeats conversion and native parameterization, compares the complete witness,
and verifies the saved solve output. A digest locates evidence; it does not
replace equality or verification. Artifact-only numerical loading needs
neither the original graph point nor GCAD.

The root and a separate runtime reviewer checked source identity, authority,
replay and ecosystem reuse. Converter operation limits do not bound native
parameterization or GCAD peak memory. Runtime parameter chambers and the
full ggHH numerical projector/numerator path remain later gates.

## Public dependency identity

The selected public FeynKit owner is
`c81fa32710316164a738cb14d274a39b53c4cd4a` in
[`ValentinHirschi/gammaloop`](https://github.com/ValentinHirschi/gammaloop/tree/c81fa32710316164a738cb14d274a39b53c4cd4a).
It combines the tested mapper at
`aee8df6c6c063908380b20cc3d5a8c7acc8ffac7`
([PR #131](https://github.com/alphal00p/gammaloop/pull/131)) with the unchanged
16-line citation correction from
`8e3a643f388b45939d6573a648ef3a509086835e`
([PR #128](https://github.com/alphal00p/gammaloop/pull/128)). Both PRs retain
their original upstream review boundary. The combined dependency branch was
published using the verified ValentinHirschi account; no duplicate PR was
created.

API/source/probe review found no earlier public mapper preserving all native
scalar assumptions. The owner change supplies this structural operation;
FastSecDec owns the numerical-meaning policy and provenance. The owner tests
and strict Clippy pass.

Only changing direct dependency revisions produces duplicate native owners
through transitive ecosystem edges. Each consuming root therefore patches the
upstream Git source to the same public fork revision. Cargo metadata verifies
one FeynKit owner and one shared Symbolica/Numerica owner with no unused patch
warnings. Symbolica/Numerica `1ac765f`, symGCAD `a1132d4` and SymJIT `d74993f`
are unchanged.

The core and portable lock changes select the new source without registry
package/version changes. The binding lock adds 92 package/version pairs,
removes none, and upgrades no existing registry package. This is the newer
upstream `feynkit-py` → `linnet-py` → full Typst renderer dependency closure,
including PDF, fonts and image support. It is confined to the binding consumer.
The native numerical library and default CLI remain Python-free.

## Validation and reproducibility

The native graph slice passed 649 library tests in its isolated baseline, plus
14 staging controls and seven graph controls. Two graph test entries are
fresh-process helpers invoked explicitly by their parent tests: one verifies
saved raw geometry without solving again; the other reads only the numerical
artifact. Strict library/integration-test Clippy and formatting pass.

The combined dependency source passed the embedded native HEPKit host test,
including the analytic bubble and prepared-owner lifetime, in 6.30 seconds.
Two old-artifact controls pass in 29.04 seconds, covering all three contour
recipes and ordinary/serial threshold archives through restoration and
resaving. The portable consumer passes 88 tests with no failures or ignores;
the portable binding check and strict native binding Clippy pass. These are
native/portable host checks, not installed-wheel or browser measurements.

The root imported six manifest/lock files from
`target/no-deformation-owner-adoption/handoff.json`, SHA-256
`6c4410b8fcc695a7938fa2c912c78595241ef4d39d597517528683e3d7854434`,
and nine native files from
`target/no-deformation-preparametric-owner/handoff.json`, SHA-256
`b74a70bd0115ca93ae72ded2fba12b3b5ba45008b78e4e7bcefee0d6249bb5e0`.
Source preimages and 36 dependency/17 graph evidence records were checked.
The joined repository library gate passes 671 tests with no failures and 21
unchanged ignored entries in 184.47 seconds. All seven graph tests pass in
0.31 seconds; their two child helpers run through their parent controls.
Strict workspace/all-target Clippy passes in 58.46 seconds, along with
formatting and diff checks.

No community checkout, installed wheel or running notebook server was changed.
A private community dependency-resolution probe retains Vakint
`6203c6cbba6ae5e90329ba5081fad55319e678db` and one native owner; it is metadata
evidence only. Updating the host's FastSecDec pin and exercising the complete
community build remain separate work.
