# Bounded on-shell triple-box and issue_1 diagnostics

These 2026-10-04 trials use the preserved release executable from
`561657bd972baf8f22a8a9d1a34a97384a654f35`, SHA-256
`845ad38432c4e6303c666c581736af9ca382297c55dbe3ba4911c1d092d7dad9`.
Its location and dependency provenance are recorded in
[the off-shell campaign](triple-box-offshell-diagnostics.md). The native backend
is SymJIT 2.26.4 at portable O2. Later source changes do not affect this executable.
This is bounded scientific workload evidence, not a matched performance claim.

The ignored `output/probes/bounded_cli` wrapper records exact argument vectors,
stdout, status JSONL and process results. Generation receives 300 seconds and
integration 180 seconds; SIGINT is followed by five seconds of grace before the
wrapper kills its own child. RSS is sampled at 50 ms. The old executable emits
whole-sector status arrays and checkpoints frequently, so reported elapsed time
includes their IO and serialization costs.

## On-shell triple_box

The unchanged `examples/runs/triple_box.toml` card has massless external legs,
`s=t=-1`, dimension `4-2*eps`, unit explicit measure multiplier, and requests
through order zero. Its graph retains repeated propagators at degree-two
vertices; an ordinary undotted ladder master is not an oracle for this fixture.

Generation produced 2112 charts and 1026 verified complete-density symmetry
representatives, then exhausted the 300-second watchdog in Laurent expansion.
The child was killed after the grace period at 305.183 seconds; sampled peak RSS
was 2,396,976 KiB. No artifact, compiled kernels, numerical estimate, or usable
partial integral was produced.

The last status at 288.504 seconds reports accumulated geometry 1.309 s,
mapping 1.905 s, symmetry 1.622 s, subtraction 1.479 s and Laurent work 282.024 s.
The display counts the representative **being entered**: displayed ordinal 39
means zero-based representative index 38, and the final displayed 46 means
index 45, with 45 earlier representatives completed. These are local traversal
indices, not portable or cross-program sector identifiers. Ordinal 39 spent
roughly 145 seconds in one expansion; a subsequent call was still running at
the timeout. Geometry and subtraction are not the measured bottleneck.

Evidence: `output/diagnostics/triple-box-onshell/generate.{argv.txt,json,status.jsonl,process.json}`.
The next diagnostic captures the exact native Laurent template at index 38
(and index 45 if needed), without treating skipped earlier expansions as a
usable generated result. Production series behavior is unchanged.

## issue_1

The unchanged direct card defines the seven-dimensional positive orthant density
`F^(eps-2)`, with unit prefactor and no graph measure or projective delta function.
`F` comes from `examples/parametric/issue_1_f.sym`. Generation through order two
completed in 3.463 seconds with 96,528 KiB sampled peak RSS, producing 328 kernels
and the complete generated order vector `[0,1,2]`. Its artifact identity is
`243c30249da2fd79e2c3aac0a60efa3f9eec6c2c7fcabe8534faed5b4d5e9fd9`.

All 328 seven-dimensional kernels received 1024 points in each of eight complete
shifts, seed 20261004, two workers, Korobov3 and Kuo38005. The actual native rule
has modulus 1024 and generator `[1,309,235,573,145,523,153]`. The checkpoint retains
all shift plans and accepted package partials. Exactly 2,686,976 planned
kernel-points were accepted, with complete full-vector covariance.

| Order | Native estimate | Native standard error | Historical decimal target |
| --- | ---: | ---: | ---: |
| 0 | 10.500981947349961 | 0.18828428346678433 | 10.36927755 |
| 1 | 100.74245158818789 | 2.020328587505415 | 99.74055923 |
| 2 | 773.6557155915172 | 14.835556765564267 | 761.8752944 |

The covariance of the mean, in row-major order, is
`[0.0354509714006004,0.26746844900214933,1.518053744456505,
0.26746844900214933,4.081727601491626,29.061390594132998,
1.518053744456505,29.061390594132998,220.09374454427967]`.
The run stopped at its work limit and did **not** satisfy the requested tolerance.
It remains unverified. The historical manual target supplies no uncertainties;
the library correctly reports unavailable pulls and ineligible comparison.
Its asserted negative-order and imaginary zero rows remain reference-only rows,
not fabricated native estimates.

Whole integration-process time was 4.065 s (131,940 KiB sampled peak), including
2.962 s artifact loading and 1.043 s integration/reporting. There were 398
conditioning checks and rescues, 398 additional weighted replays, maximum
precision 256 bits and no evaluation failures. These small-count diagnostics
do not establish convergence or performance parity.

Evidence: `output/diagnostics/issue-1/{generate,integrate}.{argv.txt,json,status.jsonl,process.json}`,
`integral.fsd.json`, `integral.checkpoint.json`, and the native numerical-only
`integral.result.json`. The saved result retains full scope, complete
contributions, actual allocation, covariance, timing and the unverified stored
reference comparison.
