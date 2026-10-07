# ggHH numerical-dual formula preparation measurements

This follow-up measures the dynamic in-memory formula preparation and native
per-key evaluator cache. The matched input remains the parametric gg → HH double
box with runtime kinematics and contributing model inputs. No three-loop run is
part of this measurement.

## Protocol

The prepared cards live at the same relative directory depth as the preceding
benchmark under ignored `output/numerical-dual-cache-study/bench/`. Taylor and
IBP each use eight caller-owned workers; a separate one-worker Taylor run checks
native artifact byte identity. Settings remain SymJIT O2, ten Horner iterations,
at most 1000 CPE rounds, one native evaluator optimizer core, and epsilon order
zero. The final saved settings are checked against these defaults after each run.

The release CLI is built before measurement, as is the existing optimized
ignored comparison helper with the workspace feature union. Timed runs execute
sequentially with no concurrent build or benchmark. `/usr/bin/time -l` records
process wall/user/system time and peak RSS. Saved human metadata supplies
separate map/valuation discovery, unique-formula preparation, sector assembly,
evaluator compilation and total times, plus unique/completed/eligible/shared
formula counts. Serial and parallel native `.dat` files must be byte-identical;
observational `.json` timings and worker counts are expected to differ.

The existing scientific/performance helper compares new dual artifacts to the
preserved symbolic Taylor/IBP artifacts using identical seeded chart points,
representative permutations and symmetry multiplicities. It retains the whole
Laurent vector with explicit real/imaginary component matching and counts every
native f64 call. Its bounded interior timing uses 256-point batches, 64 measured
batches per sector, seed 4917, and the explicit benchmark-only stability settings
already documented for that helper. Generation artifact policies are unchanged.

Raw artifacts, logs, configurations and summaries remain ignored. The prepared
`run-generation.sh` refuses to overwrite any existing output artifact, and
`summarize.py` checks one/eight-worker byte identity before writing its report.

## Measured generation results

The final release CLI and optimized workspace-union CLI helper built successfully
on 2026-10-07, before timing. The three sequential runs passed with the recorded
settings and 21 runtime inputs. Every source chart used the numerical-dual lane;
there was no symbolic fallback in this input.

| Wall-time component | Taylor, 8 workers | IBP, 8 workers | Taylor, 1 worker |
| --- | ---: | ---: | ---: |
| Input | 0.129 s | 0.092 s | 0.090 s |
| Parametrization | 1.716 s | 1.708 s | 1.818 s |
| Domain metadata | 0.0016 s | 0.0006 s | 0.0009 s |
| Geometry | 0.0020 s | 0.0019 s | 0.0063 s |
| Map/valuation discovery | 9.543 s | 9.437 s | 10.292 s |
| Unique-formula preparation | 0.176 s | 0.178 s | 0.625 s |
| Sector assembly | 0.0148 s | 0.0151 s | 0.0779 s |
| Evaluator construction and compilation | 11.694 s | 9.747 s | 25.064 s |
| Generation total | **23.302 s** | **21.199 s** | **37.996 s** |
| Whole process, including artifact writing | 23.50 s | 21.37 s | 38.15 s |
| Peak process RSS | 2.880 GiB | 2.890 GiB | 2.358 GiB |

Each run discovered **four unique formulas for 30 eligible sector uses**, built
all four, and reused them for the remaining **26 uses**. This is actual complete
recipe sharing, not an estimate from similar derivative orders. Four formula
jobs can therefore occupy at most four workers for this input. Formula discovery
and compilation use the caller's existing executor; the cache adds no executor.

These phase wall times are additive rather than sums of worker durations. The
generation totals include a further 0.020–0.025 seconds of coordinator/artifact
preparation overhead; final file writes are outside the saved total. The assembly
row is the existing `coefficient_expansion_seconds` field in this lane. The
compilation row includes native source/jet evaluator construction, composition,
Horner/CPE optimization, real/complex lowering and SymJIT O2 compilation. Those
substeps are not separately measured, so none is assigned an invented duration.

Taylor's complete one- and eight-worker `.dat` files are byte-identical. IBP also
produces that same `.dat` for this particular ggHH input. Their SHA-256 is
`21951024ba4c69ad4ef80d3f181334c974535b9cb7be43d41cf1b806b08af74f`.
All 30 per-sector evaluator statistics, including exact-program bytes, operation
counts and SymJIT IR bytes, equal the preserved pre-cache numerical-dual
statistics for each strategy. The runtime schema is likewise unchanged. This
supports unchanged effective evaluator requirements; it does not replace a
numerical comparison at bound points.

The preserved earlier single runs took 23.342 seconds for dual Taylor and
32.277 seconds for dual IBP. Taylor therefore shows no material whole-generation
speedup in this measurement. The old 10.468/11.636-second coefficient buckets
combined discovery, formula work and assembly. Their comparable new sums are
9.734/9.630 seconds; the new split identifies discovery as most of this cost.
The 0.176/0.178-second formula phase is only 0.76%/0.84% of the new total, while
discovery is 40.95%/44.52% and evaluator construction is 50.18%/45.98%.
IBP's lower observed total is largely associated with its smaller measured
compilation time in this run. These are single passes, not a repeated controlled
attribution benchmark. In particular, no individual valuation or optimizer
operation is identified as the bottleneck without a profile.

## Numerical and runtime evidence, with the current limitation

The optimized four-artifact helper was invoked but stopped before evaluating
any points. The preserved symbolic references record Symbolica revision
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4` with local `source_state = "clean"`;
the final build records the same revision with the public Git source URI. The
artifact dependency guard correctly rejects that provenance mismatch. Neither
the guard nor the saved artifacts was modified. At the user's request to commit,
push and stop, no additional symbolic regeneration or benchmark was started.
**A fresh four-artifact ggHH parity/runtime rerun is therefore not claimed.**

The preceding ggHH scientific comparison remains recorded in the earlier
numerical-dual review: full Laurent vectors agreed with maximum relative
difference about 3.126e-15. Current milestone controls separately passed 544
native workspace tests, the additional CLI progress regression, 72 portable
controls, and strict workspace/bindings Clippy. New native cache tests cover
exact keys, structural-zero separation, minimal shifted shapes, concurrent
sharing and deterministic native IR; current full-size worker-count determinism
is checked above.

The preserved warmed evaluator measurements also answer where runtime work was
spent. For numerical-dual Taylor, native primary-f64 evaluation occupied
11.365674217 of 11.409839830 worker-loop seconds with 256-point batches
(99.613%); IBP used 11.444377276 of 11.487710333 seconds (99.623%). True scalar
calls gave 99.660% and 99.662%, respectively. Their measured means were about
23.12/23.28 microseconds per batch row and 33.43/32.51 microseconds per scalar
call. Thus native evaluation dominated those bounded warm loops. These fractions
exclude cold loading, setup and complete integration orchestration, and use the
explicit primary-f64 benchmark policy; they do not assert that all real
integration workloads have those fractions. The current unchanged evaluator
statistics give no evidence of a runtime speedup from formula-cache reuse.

Reproducible evidence remains under ignored
`output/numerical-dual-cache-study/bench/`: final build logs, both cards, three
status/time logs and artifacts, `generation-summary.json`, and the failed
provenance-check `comparison.log`. Historical successful comparisons and timings
remain in `output/numerical-dual-study/bench/report.json` and `scalar-report.json`.
