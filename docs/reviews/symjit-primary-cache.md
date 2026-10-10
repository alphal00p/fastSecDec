# Native primary JIT persistence and loader ownership audit

This review concerns current Symbolica `74225696` / SymJIT `d74993f`, separately
from the historical 2.26.4 measurements in [the loader profile](symjit-loader-profile.md).
The implementation uses existing owner codecs. This document separates the
native API proof, measured loader costs, and final acceptance gates.

## Existing owner capability

`JITCompiledEvaluator<T>` already implements native bincode `Encode` and
`Decode<Context>` for both `f64` and Numerica `Complex<f64>`. Its native payload
contains external-function descriptors, the saved SymJIT application, and JIT
compilation settings. Decode resolves the function factories and restores the
application through the owner's `load` method. FastSecDec must use that API,
not a duplicate tuple decoder, raw machine-code format, or new callback engine.
The exact Symbolica program remains authoritative and available for precision
rescue, conditioning, inspection, and admitted cache fallback.

The isolated probe under `target/symjit-loader-profile/current-native` links the
verified candidate7 release graph and includes the immutable native contour
helper/requested-function modules. Separate writer and reader processes passed:

- real and complex JIT versus exact eager evaluation at four points;
- restored implicit-root helper before callback resolution, then dropping the
  helper and registration scope before evaluation;
- restoring the same bytes under plain and checked/observed factories;
- changing runtime S/L/R, four observed requests, and cloning into another thread;
- explicit failure without a required observer and for invalid strength input.

The tiny reader completed in 2.69 ms; this is an API test, not physical loading
performance. Child high-water RSS was 12.59 MB. The periodic `/proc` sampler
missed the short-lived process and reported zero, so its sample is not used as
peak memory. Both process groups closed cleanly. Raw execution, source, build
command, and outputs are retained under `target/symjit-loader-profile`.

A separate compile probe found a genuine serde API defect: its Deserialize
implementation unnecessarily requires `symjit::Element + Copy`, which excludes
Symbolica's own `Complex<f64>`. Native bincode has the appropriate `Clone`
bound already and passes the fresh-process test. The isolated upstream fix and
real/complex serde roundtrip tests are published below; the production cache
does not require a dependency update to use the working native bincode API.

## Artifact and runtime contract

The v13 transport wraps an unchanged authoritative native record and
optional per-sector JIT payloads. Cache compatibility must bind the exact-program
digest, scalar domain and layout, dimensions, native JIT codec marker, and owner
compatibility. Mathematical identities must remain those of the base record;
record byte hashes and sizes necessarily change when a cache is added.

Decode the selected record once and select the primary evaluator before its
constructor runs. Do not construct an exact-to-JIT evaluator only to discard it
for the saved one. Restore within the same `MappingRequirements::prepare` scope
that resolves helpers, factories and observation context, then apply the same
outer dynamic/error fences. Keep precision workspaces independent. Do not save
physical bindings, pilot readiness, counters, or runtime diagnostics as cache
state. Full `KernelSet` tests demonstrate certificate validation, fresh Pilot
ownership, rebinding, and corrupt-cache errors.

Missing or incompatible caches may rebuild through the existing exact path only
after its existing owner and numerical-policy admission. Corrupt claimed caches
must return explicit errors. A portable decoder must not accidentally admit a
native SymJIT/GMP record that the existing portable policy rejects. Portable
production remains exact-only; no new native-to-WASM interchange is claimed.

Preserve the public contract that loading and then `to_bytes` returns the loaded
artifact. Newly generated records can contain caches automatically. Existing
indexed archives should refresh one record at a time through native readers and
writers, retaining recipe/catalogue mathematical IDs and recomputing transport
lengths/digests. Report restored/missing/incompatible/not-applicable outcomes
independently of persisted generation statistics.

## Current loader ownership findings

The source review identified the following work, measured before the change:

- The indexed reader owned record bytes, but the base loader copied them again
  into `portable_artifact`; assembly then discarded that retained copy.
- Whole-owner cloning copied the retained artifact. Sector cloning copied the
  immutable exact IR while actual numerical workspaces also require their own
  state. An immutable Arc must not accidentally share mutable evaluator state.
- Primary construction and precision/conditioning construction independently
  derived the same mapping requirements, including an exported-IR scan.
- Switching dynamic checked/plain factories rebuilt exact-to-JIT code. Native
  JIT decode under the new factory scope may avoid this translation; ordinary
  parameter binding should not be described as recompilation.

These findings do not establish their relative cost. The bounded current-owner
single-record load/clone probe below does not regenerate physics or perform
sampling. No broad loader redesign is justified by the old v8 profile.

## Bounded current-record measurements

The core-only probe links the verified candidate7 release, using current
Symbolica `74225696` / SymJIT `d74993f`. It reads actual fixed and polynomial D05
source-zero records from the preserved candidate3 archives. These archives used
historical NumericalDual endpoint construction; this is a loader ownership
measurement, not renewed scientific acceptance of that endpoint policy. There
is no binding, evaluation, pilot, regeneration, or integration in this probe.

| Source-zero record | Fixed | Polynomial |
| --- | ---: | ---: |
| Retained record bytes | 2,637,175 | 5,736,815 |
| Public indexed load, validation disabled | 0.21653 s | 0.39416 s |
| Warm record read alone | 0.00042 s | 0.00082 s |
| Separate equivalent `to_vec` copy | 0.00139 s | 0.00268 s |
| Sixteen sector clones | 0.02489 s | 0.04930 s |
| RSS after load / after sixteen clones | 25.21 / 58.93 MB | 46.62 / 119.80 MB |

The standalone copy times are not internal subphase measurements of the load;
they reproduce the same byte-copy operation separately. They are below 1% of
the aggregate load times on these two records. Native whole-owner cloning also
confirms a distinct retained byte buffer of the full record size. The clone
measurements justify considering immutable sharing; they do not separate exact
IR, callback state, stack, compressed JIT bytes, or allocator retention. Actual
mutable workspaces must remain independent.

A separate polynomial run with optional validation enabled took 1.11635 s for
public `load_sector`, compared with 0.39416 s above. These are single concurrent
runs, not a repeated isolated benchmark; no exact differential attribution or
whole-archive extrapolation is claimed. Native record digests and mathematical
IDs matched. All three bounded process groups closed with no limits; state RSS
comes from `/proc/self/smaps_rollup`. Short-lived process sampling can miss the
peak, and memory after dropping clones can retain allocator pages.

Raw logs are in `target/symjit-loader-profile/runs/current-fixed-record`,
`current-polynomial-record`, and `current-polynomial-record-validated`; source
and exact link commands are in `current-loader`.

Full cache checksum and exact-payload digest recomputation must follow the
existing optional integrity setting (`FIRST_PHASE_PLAN.md`, saved-restoration
requirements). Default trusted loading still checks format, domain, dimensions,
backend/owner compatibility and native decoder errors; it must not claim full
integrity verification. Corrupt-cache tests should distinguish mandatory decode
errors from checksum failures requested with validation enabled.

## Upstream serde correction

[Symbolica PR64](https://github.com/symbolica-dev/symbolica/pull/64),
`96490cb`, fixes the independent complex-serde bound defect atop current
upstream `community` `f4e7870`. The patch only matches Deserialize's bounds to
its existing native load/Decode requirements; it does not change the payload.
Two new serde roundtrips and fifteen existing evaluation tests pass, with one
pre-existing ignore. The previous bound demonstrably fails compilation on the
new complex test. Formatting and independent source review pass. Tests use a
coherent direct-rustc native dependency graph, recorded in
`target/symjit-loader-profile/owner-serde/result.json`.

This PR is published as ValentinHirschi. It is not a FastSecDec dependency
update or a requirement for the already working native bincode cache route.

The PR is attached to this task (root confirmed the successful attachment).
GitHub denied the formal reviewer request for insufficient repository
permissions; [the review request to benruijl](https://github.com/symbolica-dev/symbolica/pull/64#issuecomment-6096966372)
was posted as a comment instead. No formal reviewer assignment is claimed.

## Implementation review

The initial v13 source review confirms that the unchanged base record's ID is
checked before primary construction, cache count/layout are checked, and native
restoration runs inside the existing mapping scope with the same outer fences.
The indexed ownership path transfers the caller's record Vec when retained and
skips retention for resident assembly. The explicit refresh loops hold one
record's native owners and replacement buffer at a time, change transport
length/digest only, and require unchanged mathematical catalogue identity.
Maintained real/complex and dynamic fresh-process controls pass. The complete
workspace and final lint acceptance are recorded below.

A revision compatibility marker for the JIT owner is required independently of
the stable exact-IR codec and SymJIT version: the owner documents its saved JIT
payload as revision-dependent. A cheap native owner-version comparison can
select the admitted exact fallback without a full payload hash. Native cache
compatibility must not depend solely on a CLI-level provenance check.

The final compatibility marker uses the existing native
`LicenseManager::get_version()` result. A different owner revision selects the
already admitted exact fallback before attempting to decode its JIT payload.
Native JIT decoding errors and trailing bytes remain explicit failures.

The dimension check has a specific trusted-producer boundary: the sidecar's
scalar/layout/input/output fields are compared with the authoritative exact
program. Current Symbolica exposes no dimensions getter on
`JITCompiledEvaluator` or its `BatchEvaluator` trait. Therefore this does not
independently verify the shape of valid but deliberately substituted JIT bytes
behind a forged sidecar. Loading a second `Application` just to inspect its
private shape would repeat backend work and callback reconstruction. No custom
native-tuple decoder, unsafe introspection, or duplicate load is introduced.
Optional checksums detect changed saved bytes when requested; neither they nor
trusted loading establish equivalence for arbitrarily fabricated native IR.

The final source also shares immutable exact templates and retained artifact
buffers through `Arc`, while evaluator stacks, callback environments, precision
caches and pilot ownership remain independently constructed. Policy remapping
serializes the current native primary and decodes it inside the new native
factory scope. This preserves requested-root/observer ownership without
retranslating the exact IR. Its setup and clone costs will be measured separately
from initial cache restoration.

Both CLI generation routes construct compiled native kernels before saving:
ordinary generation calls `compile_with_settings_parameters_and_dispatch`, and
serial generation does so before native `indexed::write_unit`. Their saved
records therefore retain the primary cache. The library's explicitly exact-only
`GeneratedIntegral::to_kernel_bytes` continues to avoid constructing a JIT and
retains its previous format. The public regression now compares mathematical
identity and complete vectors across those two intentional transports, while
checking each transport's own unchanged-byte roundtrip after numerical work.

## Matched optimized cache and remapping measurements

The immutable current core was built with optimization level 3 against the
verified candidate7 dependency graph, with thin LTO at probe linking. All externs
were selected by the actual core dependency fingerprints. The source archive is
`d56a44a844d205efb05d11947d477319219a15045a3310dc7d2baa94c0f12a54`.
It matches the accepted production source; the later public-test correction
described above changes tests only and is outside this profile snapshot.
Each measurement used a fresh process pinned to CPU250, avoiding the independent
physical cost probe on CPU0. Other work continued on the host. The two existing
source-zero subsets above were refreshed through native archive APIs without
regeneration; originals were unchanged. Catalogue, recipe, native and bound
content IDs matched between cache misses and hits.

| Measurement | Fixed source zero | Polynomial source zero |
| --- | ---: | ---: |
| Legacy exact-only record load, current core | 180.38 ms | 386.54 ms |
| Cached record load, same core | 143.73 ms | 302.45 ms |
| Observed load reduction | 20.3% | 21.8% |
| Record bytes before / after | 2,637,175 / 2,989,003 | 5,736,815 / 6,499,910 |
| RSS immediately after miss / hit | 26.63 / 25.66 MB | 46.11 / 46.56 MB |
| Sixteen sector clones, miss / hit | 0.74 / 0.78 ms | 2.14 / 1.32 ms |
| RSS added by those clones, miss / hit | 0.94 / 1.26 MB | 3.58 / 1.67 MB |
| Whole-owner clone, miss / hit | 0.35 / 0.31 ms | 0.60 / 0.52 ms |
| First Pilot binding, miss / hit | 0.39 / 0.19 ms | 3.25 / 3.23 ms |
| Repeated same-policy binding, miss / hit | 0.14 / 0.13 ms | 3.08 / 3.02 ms |
| Switch to Always, miss / hit | 0.14 / 0.15 ms | 82.64 / 86.65 ms |
| Switch back to Pilot, miss / hit | 0.13 / 0.13 ms | 81.78 / 86.43 ms |
| Enable aggregate diagnostics, miss / hit | 0.011 / 0.0005 ms | 79.66 / 78.68 ms |
| Disable diagnostics, miss / hit | 0.0002 / 0.0002 ms | 78.58 / 79.39 ms |

These single runs establish a modest loading benefit for these records, with
approximately 13.3% larger record bytes. They are not a whole-archive forecast or
a repeated benchmark. Fixed policy changes do not require the dynamic callback
reconstruction exercised by the polynomial row. Polynomial remapping remains
real setup work even though it avoids exact-to-JIT translation; persisted cache
hits do not eliminate it. Binding may evaluate native exact offsets, but this
probe invokes no pilot point, stochastic integrand evaluation or sampling.

Whole-owner clones assert shared retained-buffer addresses. Maintained tests
also assert shared immutable exact-template addresses and independent numerical
behavior after dropping original owners and moving clones to another thread.
The earlier clone timings/RSS above used another executable and were not pinned
comparisons; they are supporting ownership evidence, not a precise speedup
ratio. RSS is sampled at stated process stages and includes allocator retention;
it is not summed across records or interpreted as live allocations alone.

All six refresh/load process groups closed without limits. The private core
build closed in 37.38 seconds at 2.27 GB sampled peak; the wrapper thin-LTO link
closed in 116.04 seconds at 2.96 GB. Both preserved the protected user release
binary and shared build outputs. Two early build-resolver attempts stopped before
compilation and are retained separately. Evidence and hashes are in
`target/symjit-loader-profile/current-cache/result.json`; raw per-phase records
are under `target/symjit-loader-profile/runs/current-cache-*`.

## Final acceptance

The complete current native workspace passes **935 tests, with 34 explicit
ignores**, across 91 result groups. The added ignored child test is executed in
a fresh process by its parent cache test; it is not an untested claim. Earlier
core/public/focused subsets overlap this count and are not added again. The
first broad attempt stopped at the obsolete cross-format byte-equality
assertion; after the test-only correction, the complete rerun passes. Native
arithmetic and production source were unchanged by that correction.

Strict workspace/all-target Clippy passes in 27.67 seconds, and formatting and
diff checks pass. The isolated binding all-target check with `python_stubgen`
passes in 16.62 seconds; its strict lint passes in 4.35 seconds. The standalone
portable host suite passes all 83 tests with no ignores, and the corrected
export test also passes on that graph. The additional targeted run overlaps the
83-test count. These checks do not claim a new installed wheel or actual WASM
execution.

Coverage includes native exact/JIT codec restoration, both scalar domains,
polynomial and sign-aware dynamic callbacks in a fresh process, independent
clone evaluation, DD/arbitrary-precision rescue, actual Pilot readiness and
policy changes, incompatible-owner fallback before native payload decode,
mandatory format/header checks, optional integrity hashes, selected-record
refresh and unchanged family identities. CLI ordinary and serial publication,
loading, cancellation and complete-vector/covariance process tests pass in the
workspace. Existing HEPKit graph, numerator/reduction and one-loop reference
tests remain part of the same gate; no alternate physical or algebra owner was
introduced.

`target/contour-primary-cache-gates.json` records source and log hashes. The only
source change during the broad gate is the reviewed public test correction.
The protected `target/release/fastsecdec` retains its original hash, inode and
size. All owned build, profile and test processes are closed. This implementation
review has no remaining blocker; physical campaign acceptance remains separate.
