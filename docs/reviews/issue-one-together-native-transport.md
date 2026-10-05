# Issue 1 sector-sum reference transport

The separately retained observation is
[`examples/references/issue_1_together.json`](../../examples/references/issue_1_together.json).
It records the completed ordinary constituent `IntegralLibrary(together=True)`
attempt reviewed in [the outcome audit](issue-one-together-outcome.md). All three
physical coefficients `[0,1,2]` and provider standard errors are copied from the
coordinator's structured independent record, which checked the original decimal
strings against all six native hexadecimal f64 tags. The native reference writer
performs no sampling, prefactor convolution or statistical reconstruction.

The existing native reference encoder and reader roundtrip the result. The
writer also reuses `read_result` and `reference::compare` on the complete original
Rust result, preserving its full covariance. The three diagnostic pulls are
`[0.823285164519348, 0.4559059197689274, 0.85681599131804]`. Changing the new
reference's validation from Unverified to Checked leaves every comparison row
unchanged; it only records the independently established input, normalization,
complete sector-sum uncertainty path and transport evidence.

Checked does not certify general uncertainty calibration or the requested
tolerance. The highest requested order is `+2`; its reported relative standard
error is about 0.001285, above 0.001. Every uncertainty remains `StandardError`.
The ordinary real interface does not provide imaginary observations or joint
cross-order covariance, so those attributes are explicitly null. Sampling the
whole sector sum incorporates cross-sector covariance within each coefficient.

The original [`issue_1.json`](../../examples/references/issue_1.json) remains
Unverified and unchanged (SHA-256
`ab74294add79facfb73ef0242c05759827993b1a3f761bed83f1b1f22ce625c3`). The new
fixture's SHA-256 is
`638e0e7103054d4c35a73341899d3a52f5c93c9688ec7a1baca57e600233cc6d`.
There is no combination of the two external observations, and no replacement of
the previous provider errors. Both refer to the full seven-dimensional positive
orthant of the unchanged supplied `F^(eps-2)`, with unit prefactor and sum weight.

The pure Rust writer is `output/probes/record_issue_one_together.rs`; it linked
the same unchanged release reference/result APIs as the previous Issue 1 writer
and did not initialize Symbolica. Its source, executable, linked library and
fixture hashes are retained in
`output/reference-fixtures/issue-one-together-native-reference/build.sha256`.
Original estimates, before/after comparisons and source/audit hashes are beside
that manifest. The focused native transport target passes four tests with two
explicitly ignored historical recording/campaign tests
(`output/issue-one-together-reference-transport-tests.log`). Its new regression
preserves all three means/errors, absent imaginary/cross-order evidence, unit
normalization, complete scope, actual work unit, unmet highest-order target and
the unchanged earlier Unverified observation.

The independent HEPKit reviewer verified exact fixture/writer identity, every
record/build hash, all provider values/errors, the unchanged native estimate
including all nine covariance entries, and the original fixture hash. No
source or transport blocker remains. This is separate from the coordinator's
independent external execution/normalization/uncertainty-path audit.
