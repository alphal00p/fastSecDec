# Issue 1: completed native sector-sum reference

The separately bounded ordinary-constituent attempt completes all three physical
coefficients. It uses existing `IntegralLibrary(together=True)`, which sums all
616 sector integrands before applying native QMC to each Laurent coefficient.
This addresses the specific omitted cross-sector covariance in the earlier
disteval result without reconstructing an uncertainty externally.

| Epsilon order | Physical real mean | Provider standard error |
|---:|---:|---:|
| 0 | 10.3453957958694200 | 0.0162252622375309193 |
| 1 | 99.8187295941142594 | 0.153138724198544351 |
| 2 | 760.916776841624255 | 0.978105088196031813 |

These are the original native physical string-tuple values. The existing native
JSON converter's hexadecimal float records are also retained. The returned
prefactor is exactly one through order two, and the before-/after-prefactor
tuple members agree. No second prefactor, imaginary zero claim, coefficient
selection, uncertainty reconstruction or averaging of observations is applied.
The previous disteval fixture remains separately preserved and Unverified.
The separate native versioned fixture is now
[`issue_1_together.json`](../../examples/references/issue_1_together.json).
Independent transport review verifies its original values/errors, full native
estimate/covariance and use of the existing encoder/comparison API. Its Checked
label records the audited source, normalization and native sector-sum uncertainty
path; it explicitly does not certify calibration or exactness. The three native
comparison pulls are 0.8233, 0.4559 and 0.8568. The earlier fixture is unchanged.
The focused native transport target passes four tests with two explicit probes
ignored. The [writer record](issue-one-together-native-transport.md) and
[independent transport review](issue-one-together-transport-independent.md)
retain the exact fixture, source and original-estimate identities.

## Actual execution and independent source/outcome checks

The attempt follows
[the independently reviewed proposal](issue-one-together-continuation-proposal.md).
It copies the already completed parent, preserving and verifying its files,
then builds the ordinary constituent's native pylink library with eight make
jobs on eight distinct physical cores. All 2469 planned C++ compilations
complete; no new FORM generation is needed. Compile, numerical and enclosing
processes exit zero in 222.587, 103.307 and 327.090 seconds respectively, within
the prescribed 600/180/800-second bounds. These are diagnostic continuation
times, not matched generation or convergence measurements.

The source and generated-wrapper review establishes the ordinary integral
header and its sector-sum branch. Native runtime metadata confirms the correct
constituent, 616 sectors, one epsilon regulator, no physical parameters, and
orders zero through two. Runtime logs explicitly record summing the integrands
before integration, then three complete native QMC calls. Each reports N8311,
R32, one iteration and 265952 evaluations under the prescribed seed20261218,
Korobov3, no fit function and one numerical CPU worker.

Thus the observed numerical work is **797856 scalar summed-coefficient point
evaluations** across three separate coefficient calls. Each summed integrand
contains all 616 sector contributions. This is a different unit from both
disteval's per-coefficient-kernel count and FastSecDec's full-vector evaluations;
no direct cost ratio follows. Joint covariance between different epsilon
coefficients is not provided by this interface and remains unknown.

The coordinator independently checked all copied-parent and external-source
hashes after execution, the actual compiled library route, complete metadata,
original physical tuple, unit normalization and native N/R/count logs. Both
guard controls passed before execution, no Symbolica module was loaded, and the
owned process finished and was reaped. The independent hash verification logs
are `output/issue-one-together-{source,parent}-verification.log`.

The uncertainty is now computed from the sampled sector sum, addressing the
identified disteval omission. A single run does not establish general error
calibration, exactness or 1-per-mille convergence. The highest requested order
is epsilon two; its reported relative standard error is about 0.001285, above
the user's 0.001 target. No coefficient is substituted to claim convergence.

All raw evidence, compiled-source identities, immutable-copy manifest, guarded
commands and failed/successful earlier observations are retained under
`output/diagnostics/remaining-pysecdec/issue1-together-attempt-1` and its named
parent. Production FastSecDec remains independent of this external provider.
