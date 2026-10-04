# Published catalogue comparison on unchanged physical artifacts

This diagnostic follows the [five-dimensional plateau investigation](six-line-qmc-convergence.md)
and the [explicit catalogue proposal](qmc-catalogue-proposal.md). It does not
change the current default or claim matched performance acceptance.

The ignored Rust driver `output/probes/physical_catalogue_campaign.rs` loads
each existing portable artifact once. All 48 runs use existing O2 kernels,
weighted precision replay, Korobov3, native worker packages and native complete
shift covariance. The design is **four physical cases × four catalogues ×
three independent seeds**, with 8192 points per shift and 16 shifts in each run.
Seeds are 20261004, 20261005 and 20261006; packages have 1024 points. The caller
uses one worker. There are no evaluation failures in any trial.

For each artifact, the driver aligns full-integral complete-shift vectors by
`(seed, shift)` across catalogues and passes those 48 joint vectors to native
`QmcEstimate::from_shift_means`. This preserves cross-catalogue correlation
from common random shifts and uses no alternate estimator. Sectors are already
combined within each shift through the existing democratic session. The table
shows each catalogue's marginal component of that native joint estimate.

| Case / full dimension | Catalogue | Mean | Standard error |
| --- | --- | --- | --- |
| kite, 4D | Kuo33002 | -0.6808747965273705 | 8.909744772355992e-7 |
| | HKKN alpha3 | -0.6808762044966448 | 6.584203812889418e-8 |
| | Kuo38005 | -0.6808769481018928 | 9.004760008645729e-7 |
| | Kuo39101 | -0.6808764487619239 | 3.702722775758351e-7 |
| six-line two-loop, 5D | Kuo33002 | 0.14249559825118754 | 4.129529696693626e-4 |
| | HKKN alpha3 | 0.1432378815111518 | 1.657167965505611e-6 |
| | Kuo38005 | 0.14323947974682566 | 3.9969051064706027e-7 |
| | Kuo39101 | 0.14322861500747927 | 7.470185210104329e-5 |
| three-point three-loop, 6D | Kuo33002 | -1.5375817065602506 | 3.3687448621527124e-3 |
| | HKKN alpha3 | -1.538685979654414 | 5.7797489018882574e-5 |
| | Kuo38005 | -1.5398705612626948 | 1.7723296560860262e-3 |
| | Kuo39101 | -1.53898135888492 | 2.731119599706418e-4 |
| eight-line three-loop, 7D | Kuo33002 | 0.23325872861426217 | 4.8823684919913074e-4 |
| | HKKN alpha3 | 0.23372888805425446 | 3.06551593071649e-4 |
| | Kuo38005 | 0.23417558036642225 | 3.681984998762383e-4 |
| | Kuo39101 | 0.23399907415030238 | 2.165947804350684e-4 |

HKKN improves the reported uncertainty over the historical rule in every
physical dimension tested here. The improvement is particularly large at
five and six dimensions. The two other catalogues remain useful explicit
choices: Kuo38005 is strongest in the five-dimensional fixture, while
Kuo39101 is strongest in the seven-dimensional fixture. Neither wins globally.
The prior nine-dimensional constant and affine controls also favor HKKN or
Kuo39101 over Kuo38005. These observations support a separate review of HKKN
as a phase-one default within its ten-dimensional published limit, while
preserving explicit alternatives and refusing an unsupported dimension.

The present milestone deliberately retains the historical default. A later
decision must preserve old missing-field and checkpoint Kuo33002 semantics,
document any new default's dimension limit, and require an explicit caller
choice beyond that limit. It must not silently choose a different catalogue
or reuse samples after changing the rule.

Evidence is retained under `output/diagnostics/massive-multiloop/`:

- `physical-catalogue-campaign.jsonl`: all 48 runs, four native joint estimates,
  artifact/kernel identities, exact reduced generators, settings, complete
  shift vectors, sector contributions and precision diagnostics.
- `physical-catalogue-campaign.stdout`: original stdout, including the native
  Symbolica startup banner.
- `physical-catalogue-campaign.progress.log`: all completed cases/rules/seeds.

Published data/source/license hashes and author-file equality checks are in
Numerica's `qmc/data/README.md`. No integration code or generating-vector
construction algorithm is imported.
