# Six-line QMC convergence investigation

This is a diagnostic investigation, not a timing acceptance result or evidence
that the randomized estimator is biased. No integration defaults have changed.

## Native evidence and complete coverage

The saved `three_point_2loop_6line` artifact contains six five-dimensional
representative sectors, representing 22 geometric charts, one real finite
coefficient, and a zero exact contribution. Both checkpoints restore through
the public native `QmcSession::restore` validator. All sectors have the complete
common set of requested shifts. Native `contributions()` and
`complete_shift_estimates()` expose those data without graph parsing or JIT.

| Rule size × shifts | Seed | Estimate | Standard error | Evaluations |
| --- | --- | --- | --- | --- |
| 1024 × 8 | 20261004 | 0.14093670463185146 | 0.0009008172620999176 | 49152 |
| 8192 × 32 | 20261005 | 0.1427027551492941 | 0.0005388525607783324 | 1572864 |

The external pySecDec result is 0.1432374698185829 ±
0.0000027254512470557134. The tighter native result is compatible with it.
The change of both seed and shift count means these two runs alone do not
establish a convergence rate. They do expose a practical efficiency concern:
the per-shift dispersion has not fallen with the eightfold lattice growth.

The tighter run's sector means and marginal standard errors are:

| Sector | Mean | Standard error |
| --- | --- | --- |
| 0 | 0.025558080359492977 | 0.00013304296166877946 |
| 1 | 0.00688732607224001 | 0.000031373731466415764 |
| 2 | 0.007957736347451045 | 0.00005939147446576494 |
| 3 | 0.004498187973132354 | 0.000029984598945895755 |
| 4 | 0.0315960507763609 | 0.0001637688374333957 |
| 5 | 0.06620537362061679 | 0.0003249025507757005 |

The sectors share shifts and are correlated. In particular, the native
covariance of the means of sectors 4 and 5 is 5.3099339563455134e-8; their
positive covariance materially contributes to the total uncertainty. Summing
marginal variances would give the wrong uncertainty. The 32 total shift means
range from approximately 0.13814 to 0.14771, without a single extreme failure.
The tighter run reports 18 precision replays and no evaluation failures.

The ignored, Rust-only reader `output/probes/qmc_checkpoint_report.rs` reuses
`QmcEstimate::from_shift_means` for its joint total-plus-sector diagnostic
covariance; it implements no alternative statistics. Full output is retained
in `output/diagnostics/massive-multiloop/three_point_2loop_6line.shift-analysis.jsonl`.

## Exact lattice relation

The bundled published Kuo33002 generator starts with
`[1, 182667, 213731, 255351, 96013]`. At modulus 8192 this becomes
`[1, 2443, 739, 1399, 5901]`. For the small integer frequency vector
`h = [1, 1, 1, 1, 2]`, exact integer arithmetic gives

```
h . z_full    = 843776 = 103 * 8192
h . z_reduced =  16384 =   2 * 8192
```

Thus this five-coordinate Fourier mode lies in the dual lattice at every
power-of-two modulus through 8192. At 16384 its residue is 8192, so this
particular alias disappears. Kuo33002 uses order-three construction weights;
this is a higher-order interaction. Korobov3 introduces a product of five
nonconstant Jacobians even when the physical integrand is the constant one.
That observation motivates the independent constant-integrand control below.
It does not indicate invalid point generation or an invalid random-shift error
estimate.

The catalog provenance and imported data hashes are already recorded in
Numerica's `src/numerical_integration/qmc/data/README.md`. The author gives the
linear-index rule formula and explicitly permits it for complete power-of-two
point sets. [Kuo's published catalog](https://web.maths.unsw.edu.au/~fkuo/lattice/).

## Reference implementation is not the same default lattice

The frozen Pathfinder CLI defaults to the `pysecdec-default` prime CBC/PT
catalog; QMCPy/Kuo is optional. With the per-iteration cap removed, a request
for 8192 chooses 8311 points and first five components
`[1, 3068, 1811, 1128, 1964]`. The frequency above gives residue 1625, not zero.
The default per-iteration cap is 4096, so an unmodified requested-8192 run can
actually use 4261 points. Matched comparisons must record actual counts.
These facts were independently traced by the Pathfinder reviewer in
`FSD.py` and `src/qmc_lattice.py`.

The separate external `IntegralLibrary` oracle also has its own defaults:
Korobov3, a minimum lattice request of 10000, and dimension-dependent merged
vector tables. Its default fit function is `None`, not `PolySingular`.
A nominal external maximum-evaluations request below that minimum is not a
matched sample budget. Neither its timing nor its error bar should be used as
a sample-for-sample performance endorsement of either implementation.

## Published alternatives and reuse boundary

The next diagnostic uses public `RuleSource::Supplied` for alternative vectors;
it needs neither a pySecDec dependency nor a generating-vector search. A
particularly relevant published candidate is the ten-dimensional HKKN
equal-weight alpha-three Korobov vector, available through 2^20 points.
Its full vector is
`[1,364981,245389,97823,488939,62609,400749,385317,21281,223487]`.
[Nuyens' author catalog](https://people.cs.kuleuven.be/~dirk.nuyens/qmc-generators/)
attributes it to Hickernell, Kritzer, Kuo and Nuyens (2011).

For dimensions beyond ten, Kuo38005 (equal product weights) and Kuo39101
(decaying product weights) are additional published candidates. Their presence
does not establish suitability for every sector dimension or justify choosing
a new default from one fixture.

The QMCSoftware-maintained LDData distribution supplies all three as attributed
numerical data, with an explicit Apache2 notice. The immutable distribution
revision used for inspection is `5c55b76bf6b6aba3415a0dece9cc1269ff1be883`:
[license notice](https://huggingface.co/datasets/Sou-Cheng/LDData/blob/5c55b76bf6b6aba3415a0dece9cc1269ff1be883/LICENSE.txt),
[HKKN numerical file](https://huggingface.co/datasets/Sou-Cheng/LDData/blob/5c55b76bf6b6aba3415a0dece9cc1269ff1be883/mps.exew_base2_m20_a3_HKKN.txt).
This gives a practical attributed-data import route if the comparison supports
it. No integration code from those distributions is required or copied.
All 10 HKKN, 5000 Kuo38005 and 3600 Kuo39101 components were compared against
their respective author's original text files; every component agrees. The
downloaded distribution-file SHA256 hashes are:

```
HKKN:  c423a2d92f8d891fff91aebd5ceb224b0a6c154f39da1964b55245296689364c
38005: d6558c9dac142f871d92f83c0d32f83da500021c2580d1c954e4570362515e8b
39101: 2a17256bba283d44ea3c8488c5ccb5a964a79150bc5a288ba54c8330ac0e7d2d
```

By contrast, no separate permissive license for the dn1 prime data was found
in the inspected GPL qmc source or the author's old construction-script page.
The supplied five-component diagnostic does not establish a production import
policy for that table. The alternative Apache2 numerical distribution avoids
making the C++ implementation a dependency or changing project licensing.

## Constant-integrand control

The ignored Rust probe `output/probes/qmc_rule_quality.rs` uses existing native
`QmcSession`, `QmcWorker`, periodization and complete-shift statistics. It
evaluates a physical constant one in five dimensions, with 32 shifts and four
independent seeds, comparing current Kuo counts through 32768, the supplied
8311 prime vector, HKKN8192/16384 and two product-weight vectors. Each also has
a no-periodization control. Exact vector/modulus relations and all shift means
are retained in machine-readable output.

The probe completed 80 runs and 28,604,160 point evaluations. All 40
unperiodized controls return exactly one with zero shift variance. With
Korobov3 the four independent seeds give these standard-error ranges:

| Vector | Points per shift | Minimum SE | Maximum SE |
| --- | --- | --- | --- |
| Kuo33002 | 1024 | 9.70e-4 | 1.30e-3 |
| Kuo33002 | 4096 | 9.62e-4 | 1.30e-3 |
| Kuo33002 | 8192 | 9.63e-4 | 1.30e-3 |
| Kuo33002 | 16384 | 4.93e-4 | 5.35e-4 |
| Kuo33002 | 32768 | 5.29e-8 | 6.59e-8 |
| supplied dn1 prime | 8311 | 3.72e-5 | 4.46e-5 |
| HKKN alpha3 | 8192 | 4.70e-7 | 6.56e-7 |
| HKKN alpha3 | 16384 | 4.12e-7 | 4.45e-7 |
| Kuo38005 | 8192 | 4.26e-7 | 5.17e-7 |
| Kuo39101 | 8192 | 5.31e-6 | 6.97e-6 |

Full results, settings, generating vectors and complete shift vectors are in
`output/diagnostics/massive-multiloop/constant-five-dimensional-rule-quality.jsonl`.
The near-identical current-rule means and uncertainties across 1024, 4096 and
8192 isolate a serious interaction between the vector construction and the
periodized five-dimensional integrand, independently of Feynman algebra,
kernel compilation, replay, or sector summation. The tested alternative vectors
avoid this particular plateau. This remains a quality comparison on one
analytic diagnostic, not a claim of a universally superior catalog. The
next check is a paired comparison on the unchanged six-line artifact.

## Paired unchanged-artifact result

`output/probes/six_line_rule_comparison.rs` loads the unchanged O2 artifact once
and uses the existing weighted replay contexts, native worker periodization,
and native complete-shift estimator. All runs use seed 20261005, 32 shifts and
1024-point packages. The caller uses one worker; the earlier two-worker result
is reproduced bit for bit, so these diagnostic timings are not a parallel
performance acceptance result.

| Vector | Points per shift | Mean | Standard error |
| --- | --- | --- | --- |
| Kuo33002 | 8192 | 0.1427027551492941 | 5.388525607783324e-4 |
| HKKN alpha3 | 8192 | 0.14323890269184456 | 2.0313147896879997e-6 |
| Kuo38005 | 8192 | 0.14323929622047865 | 4.664590980007524e-7 |
| supplied dn1 prime | 8311 | 0.14326380712887105 | 7.910632799973312e-6 |
| Kuo33002 | 32768 | 0.14323966219018616 | 1.951074377176864e-7 |

Every run completes full support, with 13–19 precision replays and no failures.
HKKN, Kuo38005 and the larger current rule agree with the independent oracle.
The single supplied-prime run lies about 3.15 combined reported errors above
that oracle; this individual fluctuation is retained rather than discarded.
The product-weight vector improves the reported error by approximately 1155
at the same work count in this fixture. This is now direct evidence that the
current catalog is poorly suited here, not just a suspected analytic mechanism.

The artifact and kernel identities, supplied generators, settings, full shift
vectors, per-sector covariance reports and precision counters are preserved in
`output/diagnostics/massive-multiloop/three_point_2loop_6line.rule-comparison.jsonl`.
The original stdout, including the native Symbolica startup banner, is saved
alongside it with suffix `.stdout`.

## Broader dimension checks before selecting a default

The separate numeric-only `output/probes/qmc_catalog_dimensions.rs` uses native
Numerica packages, periodization and accumulators for both a constant one and
`product((1+x_i)/1.5)`, whose exact integrals are one. It compares three seeds
at dimensions 3, 7 and 9, with 1024 and 8192 points and 32 shifts. Complete
results are in `constant-affine-catalog-dimensions.jsonl` in the same evidence
directory. These 72 runs show why one fixture must not choose a global default:

- At dimension 3 and 8192 points, all four catalogs give errors below 2.4e-9.
- At dimension 7 and 8192 points, HKKN and the two product-weight catalogs
  improve the constant error relative to Kuo33002; Kuo39101 is strongest here.
- At dimension 9 and 8192 points, Kuo38005's constant error is 0.0070–0.0084,
  worse than current Kuo33002's 0.0041–0.0045. HKKN gives 0.0019–0.0024 and
  Kuo39101 gives 0.00158–0.00173. The affine control has the same ordering.
- At dimension 7 and 1024 points, Kuo38005 is also worse than the current rule.

Accordingly, an unconditional global switch to Kuo38005 is not supported.
The ten-dimensional HKKN rule is a plausible phase-one candidate, while the
existing catalog and explicit supplied-vector API should remain available.
Representative physical cases at other dimensions and documented count limits
remain necessary before accepting a new default or fallback policy.
