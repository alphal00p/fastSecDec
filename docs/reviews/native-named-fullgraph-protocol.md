# Original on-shell graph: native prepared named-route protocol

The bounded trial uses the ordinary CLI with both native named coefficients
and the existing exact `SingleUnitTerm` family preparation. The CLI adapter is
accepted at `e42a017`: 38 focused tests, formatting and CLI all-target Clippy
pass, with an [independent review](cli-family-preparation-independent.md).
The preceding prepared trial completed coefficient generation and sector
compilation, then failed before publishing an artifact. The persistence rerun
at `c3ec42a` passes generation/publication and fresh-process inspection within
the same limits. Its 180-second integration stage times out with partial
checkpoint coverage and no full-integral estimate. All processes are reaped.
Build and attempt outcomes
are retained in [native-named-fullgraph-results.md](native-named-fullgraph-results.md).
This protocol adds no
algebra, input eligibility probe or production default change.

The earlier public representative gate remains accepted: all 21 signed
coefficient/point comparisons and 24 cold weighted vectors pass. Its aggregate
record is `output/diagnostics/native-named-public-actual-generation-2/public-path-independent-review.json`
(SHA-256 `9457fe8c8097609f953a0ad1c3e29126246e9e283af7cfac656ac47ade5fde15`).
The prior combined workspace gate at `22dc1d9` passed 370 tests, with 23 explicit
ignored probes, formatting and all-target Clippy. The new freeze binds these
accepted records and subsequent CLI and native persistence reviews, retaining the actual
source changes instead of attributing the new binary to the earlier source.

## Preserved earlier attempts

`native-named-fullgraph-1` built successfully, but its freezer rejected the
ordinary release feature set because it omitted the test workspace's
`tracing_max_level_info`. Independent source review confirmed that this flag
maps solely to `tracing/release_max_level_info`. The failed freeze is retained.
The fresh second freeze explicitly records this sole logging difference;
scientific and codec features and native dependency sources are unchanged.

`native-named-fullgraph-2` used the original ten-parameter representation and
the named coefficient route. The coordinator intentionally cancelled it after
**1,006.899916801 seconds**, at **372 of 1,026 representatives**, to adopt the
existing exact prepared-family route for the same graph. Peak RSS was
14,077,364 KiB. The process returned the typed `generation cancelled` error,
exited 1 and was reaped; this was **not** the 1,800-second deadline. All 58 frozen
hashes passed. No artifact, inspection, integration, checkpoint or result was
created. The separate cancellation record retains the reason, process identity,
progress, memory observation and signals. The accepted independent record is
`output/diagnostics/native-named-fullgraph-2/independent-review.json`
(SHA-256 `6fd458bd29ddb3eef6f04c7110c9d79ce100af607aaf18769671b2abb508f433`).
Its timer's group cleanup after the leader exited is not a deadline kill.

The original and equivalent eight-parameter external reference generations
separately reached their 30-GiB **RSS** bounds and supply no numerical reference.
Their outcomes do not change this native trial's address-space limit or permit
a zero result. Reference evidence and subsequent reference work have separate
owners and retained attempts.

## Unchanged physical input and two explicit options

The new ignored harness is `output/probes/native_named_prepared_fullgraph/`.
Its graph, model and massless parameter card are byte-identical to the original
`output/diagnostics/production-alias-fullgraph-1` inputs. The run card adds only:

```toml
[generation.coefficient_expansion]
method = "native_named"

[generation.family_preparation.SingleUnitTerm]
max_states = 32
```

Removing exactly these two parsed tables must recover the original parsed card.
The preparation bound is the existing native policy's 32 partial-fraction
states; it is not a deadline. Named attempt/width/request caps remain omitted.

| Original input | SHA-256 |
| --- | --- |
| Run card | `d3b9078b3042a895f1766f03e108c48f034ff9ec17af23b881d79a465f90b196` |
| Triple-box DOT | `f36bb36842ce37447ff2e5d8e0f8416463482c47c9454a23da7e4cf9e6ac9a89` |
| Scalar model | `b89bf9ca4162d3784896ced020da4efe06aa9b803334fb88761fd501b0cf89f2` |
| Massless card | `98f6f3644366c68da8abcdd06841b1008410957082442c5806aa8d02ad13f1a3` |

Dimension `4-2*eps`, on-shell `s12=s23=-1`, unit measure, requested maximum
epsilon order zero, native Taylor subtraction, sequential geometry and the
original threshold policy remain unchanged. There is no subset, altered
numerator, omitted pole or sampled-zero inference. The native preparation API
has already established the exact reduction to eight active parameters for
this original graph. The actual CLI report, active original indices/powers and
coordinate metadata remain the evidence for what this execution used. The
prepared projective domain has seven integration coordinates; it is not an
invertible relabeling of the redundant original Schwinger coordinates. The
full integral is the scientific scope. Old chart/representative counts do not
constrain the prepared geometry.

## Release freeze and bounded stages

Compile the committed persistence milestone with pinned Rust 1.98.1, two Cargo jobs on
CPUs 10 and 11, using `cargo build -p fastsecdec-cli --release --locked
--message-format=json`. Select the unique successful normal `fastsecdec` binary
from Cargo's artifact record. Retain its hash, exact compiler/linker identities,
source archive, native archives/patches, selected rlibs and feature fingerprints.
The persistence freezer overlays the independently reviewed native and CLI persistence
source maps, including the Cargo `serde_json/raw_value` feature, on the prior
accepted source map. Shared reviewed hashes must agree; all other production
and native sources still match. It binds the completed combined workspace,
formatting and Clippy result and the committed persistence milestone.
Documentation changes are recorded separately from compiled input. Existing
original-input proofs and public representative/oracle results are reused;
no new mathematical eligibility probe is introduced.

Native evaluation remains production SymJIT **O2** with direct translation,
distinct from the Rust release profile. The exact release Symbolica features
remain `bincode`, `float-mpfr`, `integer-gmp`, `native_code_generation`, `serde`;
the previously reviewed logging-only difference is explicit. The retained
official release check from 2026-10-05 08:52:48 UTC found Symbolica 3.0.1 and
SymJIT 2.26.4 unchanged; it is historical evidence, not a new package query or
the executable's build identity.

A fresh `output/diagnostics/native-named-prepared-fullgraph-*` campaign binds
the copied launcher, timer, inputs, prerequisite records, source/dependency
archives and selected release executable. Existing scientific output paths are
rejected. Check immutable hashes before and after. The unchanged sequence stops
on the first failure, with no automatic retry or allocation increase:

| Stage | Whole-process deadline | Grace | CPU affinity | Address-space cap |
| --- | ---: | ---: | --- | ---: |
| Public generate, compile and save | 1,800 s | 5 s | 8 | 30 GiB |
| Separate-process cold artifact inspection | 180 s | 5 s | 8 | 30 GiB |
| Complete fixed full-integral allocation | 180 s | 5 s | 8,9 | 30 GiB |

The persistence rerun retains these bounds and scientific inputs. Its only
process-limit change is `prlimit --core=0:0` on each stage, requesting no core
image after an abort. The host uses a piped core handler, for which Linux may
ignore this limit; suppression of the prior core-dump delay is unverified.
The existing watchdog remains in force. This does not change global host
settings, numerical behavior or address-space admission. The prior final
compilation callback occurred at 1,668.039795
seconds, leaving **131.960205 seconds** within the 1,800-second stage for
artifact construction and publication. This is headroom, not a guarantee that
the new writer fits. Actual elapsed time and memory remain the acceptance
evidence; no automatic deadline extension follows a failure.

Integration uses `--full-integral`, 1,024 points per shift, eight shifts, seed
20261004, two caller-owned workers and `kuo38005`. The saved native `QmcDesign`
is authoritative for the effective method, periodization and allocation.
Every successful stage must be reaped before the next begins. Native scientific
execution waits for any external Symbolica generation to exit and be reaped;
release compilation may overlap on its assigned CPUs, with overlap recorded.

## Retained output and acceptance scope

Retain the native preparation report and original source hashes in provenance,
every generated signed order through zero, exact offsets, complete chart/domain
metadata, conditioning profiles, covariance and full saved-result scope. Do not
trim extra leading coefficients. Cold inspection validates ordinary artifact
loading; it alone does not establish integral accuracy.

Status remains JSON at the existing 100-ms cadence. Preserve attempt, formal
piece/request/alias counts and completed phase timings; incomplete active work
is not a completed duration. Record wall/user/system time, peak RSS, readily
available virtual size, page faults, context switches and concurrent workloads.
Retain partial evidence on failure without presenting it as a complete vector.

This is a bounded full-input capability trial. Its fixed allocation makes no
one-per-mille convergence, eight-core throughput or matched performance claim.
Full-integral numerical agreement and performance acceptance require their own
completed evidence. No optional tuning campaign follows automatically.

After the accepted fourth artifact/inspection and numerical timeout, further
numerical work must reuse that saved artifact. Existing CLI `--resume` can
restore a copied checkpoint while retaining the original attempt unchanged;
native restore validates complete problem scope, settings, design and accepted
replay state. The checkpoint is older than the final observed progress. No
600-second extension or eight-worker run has been frozen or launched: the
heterogeneous accepted prefix and eager evaluator ownership require a bounded
source/cost assessment first. This does not authorize regeneration, reduced
coverage, changed precision rules or an automatic retry.
