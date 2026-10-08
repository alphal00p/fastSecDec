# Matched one-worker ggHH generation memory comparison

On 2026-10-08, the shipped `examples/gghh_double_box/run.toml` completed both
normal and serial generation with one worker. Serial generation reduced sampled
peak aggregate RSS from **931.3 MiB to 220.0 MiB (76.4%)**, with wall time increasing
from **226.5 seconds to 310.6 seconds (37.2%)**. Both runs generated all 30 sectors.

| Generation | Peak parent RSS | Peak parent + descendants RSS | Monitored wall time | Reported generation time | Maximum child processes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Normal, `--workers 1` | 931.320 MiB | 931.320 MiB | 226.478 s | 226.369 s | 0 |
| Serial, `--workers 1 --serial` | 11.668 MiB | 220.008 MiB | 310.616 s | 310.548 s | 1 |

Normal generation's worker is a thread, so its memory is included in the parent
RSS. Serial generation uses a recyclable child process; the aggregate column
includes its resident memory. Parent and aggregate peaks are independent maxima
and must not be added. The exact aggregate peaks were 953,672 KiB and 225,288 KiB.

## Matched configuration and completion

The run card was used unchanged: symbolic generation, Taylor subtraction,
`coefficient_series`, minimal numerator contraction, and expansion through
epsilon order zero. Both artifacts contain 30 six-dimensional sectors with
orders `[-1, -1, 0, 0]` and components `[Real, Imag, Real, Imag]`. Every sector in
both artifacts reports the portable `symjit_o2` backend. The complete evaluator
settings match, including one compiler core, 10 Horner iterations, 1000 CPE
rounds, and direct translation.

Both processes exited successfully, published their artifacts, and reported
completion. Source fingerprints, dependency identities, runtime parameter
names, source-chart modes, evaluator settings, dimensions, and Laurent layouts
match. Their common scientific content ID is:

```text
c0df7cb1f4c47a8a0a885bc237da43c77e02aa3d02e3bf54b64ca5110dac0198
```

Native archive byte identity is not required: local sector layouts and native
symbol-state encodings can differ. An independent agent reviewed the monitor,
completed manifests, shared executable and comparison results and corroborated
the matched measurement.

One compact-summary discrepancy remains: normal generation includes the `MT`
nonzero-mass requirement in `kernel.runtime_mass_constraints`; the serial compact
summary omits that field. The native serial artifact retains and enforces it:
a follow-up binding check with `model::MT=0` and an explicitly empty sector
selection immediately rejected the massless specialization before sampling.
This check was outside the measured runs. No production code was changed.

## Measurement method and reproducibility

The normal run finished before the serial run began. Each invocation had a
600-second limit and a 15 GiB aggregate-RSS limit; neither limit was reached.
The ignored Perl monitor recursively walks `/proc/PID/task/*/children` for the
owned process tree and reads `/proc/PID/statm` every 20 ms. It never includes
unrelated host processes. It recorded 11,273 normal samples and 15,467 serial
samples; maximum observed intervals were 22.085 ms and 20.311 ms, respectively.

These are sampled RSS peaks, including shared pages in each process's RSS,
rather than proportional memory or a guarantee of capturing arbitrarily short
spikes. This was one sequential pair without global cache flushing or isolation
from other host work, so the timings are descriptive measurements, not a broad
performance benchmark. No numerical integration was part of this comparison.

Both commands used the same hardlink to `target/release/fastsecdec`, built from
source commit `fbf862b4ab468613a0ade344d118549c78540eaf`. Its SHA-256 is:

```text
b475d55769b5d2cbe8d70f80413bf5ea837297c741051067079e6282518091d4
```

The supplied Symbolica license and `SYMBOLICA_HIDE_BANNER=1` were present in
both environments. The exact generation commands were:

```sh
output/serial-one-worker-comparison/fastsecdec --plain --json --status-json \
  --status-interval-ms 500 generate examples/gghh_double_box/run.toml \
  --workers 1 --output output/serial-one-worker-comparison/normal.fsd

output/serial-one-worker-comparison/fastsecdec --plain --json --status-json \
  --status-interval-ms 500 generate examples/gghh_double_box/run.toml \
  --workers 1 --serial --output output/serial-one-worker-comparison/serial.fsd
```

The unchanged input SHA-256 values, verified again after both runs, are:

| Input in `examples/gghh_double_box/` | SHA-256 |
| --- | --- |
| `run.toml` | `0c5c8e9fdcb24b9d17747e4f49beb90628e5416973715f24d63640a8ff35dc10` |
| `graph.dot` | `c3abfa63bfd3c0c93da847a857514129a5fbcf5fab8e05145494b0761e5227e4` |
| `model.json` | `ba2a306a0e149da39ee3cdf92a10882e0e2a084154279cc3f8cd62a7e4130048` |
| `parameters.json` | `4f4859a88b40144c42c7ebaf0da79b40bd43d317c9b17796007c1b74dca75da1` |

Ignored artifacts, monitor code, 20 ms traces, completion reports, manifests,
binding-check output and `comparison.json` are retained under
`output/serial-one-worker-comparison/`. No raw output or executable is tracked.
