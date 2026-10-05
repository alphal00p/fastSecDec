# Independent Issue 1 sector-sum reference transport review

The independently authored writer `output/probes/record_issue_one_together.rs`
and `examples/references/issue_1_together.json` preserve the completed provider
observation audited in [the coordinator outcome review](issue-one-together-outcome.md).
The writer constructs the public native `ReferenceResult`, uses native comparison
and encoding/reading, and introduces no estimator or uncertainty reconstruction.

The three real rows at orders 0, 1 and 2 exactly match the audited physical tuple:

| Order | Value | Retained standard error |
|---:|---:|---:|
| 0 | 10.34539579586942 | 0.01622526223753092 |
| 1 | 99.81872959411426 | 0.15313872419854435 |
| 2 | 760.9167768416243 | 0.9781050881960318 |

All uncertainties remain positive native standard errors. Imaginary values and
errors and cross-coefficient covariance remain explicitly unavailable. The
ordinary constituent's `together=True` path samples each complete 616-sector
sum, retaining cross-sector covariance through native sampling. This supports
the Checked source/normalization/uncertainty-path/transport status; it does not
certify general calibration, exactness or tolerance convergence. The highest
requested order is +2 and its reported relative error exceeds 0.001.

Read-only verification confirmed that the checked-in fixture equals the writer's
output, all build-manifest and nine coordinator-record SHA-256 checks pass, and
the embedded audit and all three decimal rows equal their original records.
The saved native estimate is unchanged, including every entry of its complete
3-by-3 covariance matrix. Native comparison before and after the explicit
validation assignment retains identical rows; its diagnostic pulls are about
0.823285, 0.455906 and 0.856816. The writer also round-trips the versioned native
reference format. The transport regression independently checks the complete
layout, values, errors, provenance and separate earlier observation.

The new fixture SHA-256 is
`638e0e7103054d4c35a73341899d3a52f5c93c9688ec7a1baca57e600233cc6d`.
The earlier disteval fixture remains byte-identical at
`ab74294add79facfb73ef0242c05759827993b1a3f761bed83f1b1f22ce625c3`,
separately Unverified because of the documented cross-kernel covariance omission.
No observations were pooled or replaced. The reported 797856 evaluations are
three separate scalar summed-coefficient calls at N8311/R32, not full-vector
native evaluations or disteval scalar-kernel work. The review introduces no
cost-ratio claim.

Detailed retained checks are in
`output/reference-fixtures/issue-one-together-native-reference`, including
`build.sha256`, `independent-records.sha256`, the unchanged estimate and both
comparison reports. This is a transport review independent of the writer; the
coordinator independently reviewed the external source, normalization and
executed numerical outcome. No additional scientific process was run for this
review.
