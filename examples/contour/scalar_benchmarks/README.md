# Native scalar fixtures

The maintained `contour_scalar_benchmark` Rust example constructs these inputs
with HEPKit's native graph, kinematic and scalar-parametric APIs. Input/reference
admission, generation, causal admission and sampling are separate actions. The
[benchmark design](../../../docs/reviews/contour-scalar-benchmark-design.md)
defines the six constructions and their resource limits.

| Case | Point | Native reference |
| --- | --- | --- |
| `triangle` | `C0(0,0,5;1,1,1)`, `mu²=1` | OneLOop expression backend |
| `box` | `D0(0,0,0,0,5,-1;1,1,1,1)`, `mu²=1` | OneLOop expression backend |
| `sunrise` | Three massless lines, `p²=1` | Symbolica Laurent expansion of the gamma-function identity |
| `kite` | Three massive and two massless lines, `p²=3/1000`, `m²=1` | Native evaluation of the published convergent series through `n=8` |

The one-loop inputs reuse the maintained triangle/box graphs and include the
explicit `1/r_Gamma` measure multiplier used by the existing OneLOop comparison.
The sunrise replaces the existing sunset numerator with one through the native
diagram API. Its measure has no extra Euler-gamma factor; its pole is `-1/4`
and the finite imaginary part is `-pi/2`.

The new [kite graph](kite.dot) uses the scalar [model](model.json), with massive
`phi` and massless `chi`. Native graph routing must reproduce these ordered
denominators exactly:

```text
k0²-1, (k0+p)²-1, (k0-k1)²-1, k1², (k1+p)².
```

Admission checks the native U/F polynomials, every propagator power and the
complete scalar prefactor. The explicit measure multiplier `-1` matches the
[upstream zero-threshold reference](https://github.com/gudrunhe/secdec/blob/2b3287ecd59436147350ae630a6fdd19eaba9097/examples/bubble2L_largem_ebr/integrate_bubble2L_full.py).
It gives `Gamma(1+2*eps) U^(-1+3*eps) (F-i0)^(-1-2*eps)`.
The native reference uses the exact zero-regulator `log(s)-i*pi`. The upstream
printed value instead uses `s+i*1e-15`; a separate native control reproduces
that finite-regulator value. Its imaginary component differs from the limit
by about `1.67e-13`. The recorded series truncation bound excludes binary64
roundoff and this separate regulator displacement. No external CAS or pySecDec
execution is involved.

All benchmark arms retain symbolic endpoint IBP. Their generation uses the same
caller-driven native entry point with one compiler core. This compact example
does not use the CLI's disk-backed serial generation coordinator. Generation and
sampling observations must therefore identify their actual execution strategy.

## Reproduction

Run from the repository root in `nix-shell`, with the usual Symbolica license
environment. Build once before starting measured actions:

```bash
cargo build --release --locked -p fastsecdec --example contour_scalar_benchmark
scalar_exe=target/release/examples/contour_scalar_benchmark
```

Every action below also has the Cargo form
`cargo run --release --locked -p fastsecdec --example contour_scalar_benchmark -- ACTION ...`.
All output directories must be fresh; actions refuse to overwrite them.

| Action | Arguments after the executable |
| --- | --- |
| Native input/reference admission, all four cases | `prepare OUT` |
| Generate one native saved owner | `generate CASE fixed\|polynomial\|sign-aware symbolic\|dual OUT` |
| Fresh restore and native Pilot16 at each frozen cap | `admit SAVED_JSON OUT` |
| Select the largest cap admitted by all six arms | `select OUT ADMISSION_JSON...` (exactly six files) |
| Fresh restore, pilot and three separate native QMC epochs | `sample SAVED_JSON COMMON_CAP_JSON SEED OUT` |

The following executes one case sequentially, with the kite first as required
by the bounded design. Choose a new `scalar_out` for each attempt. Case indices
are `triangle=0`, `box=1`, `sunrise=2`, `kite=3`.

```bash
set -euo pipefail
scalar_case=kite
scalar_case_index=3
scalar_out=target/scalar-benchmark-kite
mkdir "$scalar_out"
timeout --kill-after=10s 300s "$scalar_exe" prepare "$scalar_out/references"

scalar_arms=(fixed-symbolic fixed-dual polynomial-symbolic polynomial-dual sign-aware-symbolic sign-aware-dual)
scalar_admissions=()
for arm in "${scalar_arms[@]}"; do
    recipe=${arm%-*}
    jacobian=${arm##*-}
    timeout --kill-after=10s 120s "$scalar_exe" generate \
        "$scalar_case" "$recipe" "$jacobian" "$scalar_out/$arm"
    timeout --kill-after=10s 15s "$scalar_exe" admit \
        "$scalar_out/$arm/saved.json" "$scalar_out/$arm-admission"
    scalar_admissions+=("$scalar_out/$arm-admission/admission.json")
done
"$scalar_exe" select "$scalar_out/common" "${scalar_admissions[@]}"

for replica in 1 2; do
    scalar_seed=$((202610103000 + replica + 100 * scalar_case_index))
    scalar_order=("${scalar_arms[@]}")
    if (( replica == 2 )); then
        scalar_order=("${scalar_arms[@]:3}" "${scalar_arms[@]:0:3}")
    fi
    for arm in "${scalar_order[@]}"; do
        timeout --kill-after=10s 35s "$scalar_exe" sample \
            "$scalar_out/$arm/saved.json" "$scalar_out/common/common-cap.json" \
            "$scalar_seed" "$scalar_out/$arm-seed-$scalar_seed"
    done
done
```

Repeat the per-case construction/admission/sampling steps for triangle, box
and sunrise with their indices. The all-four `prepare` action need only run
once. A failed or timed-out action remains a failed/censored result; do not
replace it with zero or silently change its settings. The shell stops on that
failure, preserving every completed output for review.

All six generation arms use Symbolic endpoint IBP through order zero, native
named coefficient expansion with initial relative width two, SymJIT O2,
Horner iterations zero, CPE cap 1000 and one compiler core. Cap admission uses
the frozen ladder `0.1, 0.03, 0.01, 0.003`; both dynamic recipes use `S=0.8`
and `R=1`. Selection requires native admission of all six owners at a common
cap and never consults production values or the reference.

Each sampling action uses Kuo33002, Korobov3, eight shifts and separate
`1024`, `2048`, `4096`-point epochs, with 1024-point packages. It saves native
full Laurent estimates/covariances, exact offsets, weighted-point/replay
digests, precision observations and reference errors. Epoch estimates are
not pooled. Generation permits 120 seconds including input/reference work;
admission has a cumulative 15-second budget; sampling permits 15 seconds for
restore/bind/pilot and 20 seconds for the three epochs.

The example has cooperative stage deadlines; the `timeout` commands above
also bound individual process wall time. For the measured batch, an external
supervisor must retain process/RSS observations, enforce the overall
90-minute allocation and 100,000,000,000-byte aggregate RSS limit, and run at
most two arms concurrently. These commands require no ignored Python wrapper,
but do not themselves measure or enforce an aggregate RSS limit.
