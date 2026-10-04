# Bounded off-shell triple-box campaign

These are complete-allocation diagnostics, not independently certified values or
matched performance results. The initial executable is the optimized release CLI
from clean FastSecDec commit `561657bd972baf8f22a8a9d1a34a97384a654f35`, copied to
`output/diagnostics/triple-box-offshell/fastsecdec-561657b` before subsequent source
work. Its SHA-256 is
`845ad38432c4e6303c666c581736af9ca382297c55dbe3ba4911c1d092d7dad9`.
The recorded build uses Symbolica 3.0.1 and SymJIT 2.26.4; artifacts additionally
retain the exact native dependency revisions and local-patch identities.

## The supplied topology has repeated propagators

The historical name must not be interpreted as the ordinary planar ladder with
ten distinct propagators. Native `triple_box.dot` has degree-two internal
vertices `v4` and `v6`, so the adjacent edge pairs carry identical momenta. In the
already checked numerator graph's native loop basis, the denominators are

```
(k1+k3+p2)^2, (k1+k3)^2, k1^2, k1^2, k2^2, k2^2,
(k2-p3)^2, (k1+k3+p1+p2)^2, k3^2, (k1-k2)^2.
```

Every original edge power remains one. The two repeated pairs therefore behave
as squared massless propagators; off-shell external momenta alone do not imply
finiteness. The fixture and its historical-equivalence integral are unchanged.
An ordinary undotted-ladder formula would be an inappropriate reference.

The standalone native audit in `output/probes/triple_box_domain.rs` reconstructs
both diagrams through HEPKit, binds the mass to zero before parameterization and
checks their ordered U/F polynomials for exact equality. The actual Gram matrix
for independent outgoing momenta is `[[-1,0,1],[0,-1,0],[1,0,-1]]`.
For the eliminated `p4=-p1-p2-p3`, native scalar-product evaluation gives
`p4^2=-1`; all four virtualities are therefore `-1`, with `s=t=-2` and the
remaining channel zero. Scalar measure multiplier, numerator prefactor, overall
factor and projector all equal one. The normalized loop measure is the existing
`prod d^Dk/(i*pi^(D/2))` convention.

Native parameterization gives
`Gamma(4+3*eps) * U^(2+4*eps) * F^(-4-3*eps)` on the projective simplex. U has 56
terms, all with coefficient one. F has 106 terms: 80 with coefficient one and 26
with coefficient two. Thus the no-threshold sign certificate is exact, while
boundary singularities still require subtraction.

Recomputing the native exact fan gives 2,496 charts. The numbers of charts with
zero, one, two and three nonintegrable coordinate powers at epsilon zero are
respectively 1,268, 670, 354 and 204. For example, chart 529 has six zero powers
followed by `[-2-3*eps, -1-2*eps, -1-eps]`. These are the Jacobian powers plus the
exact U/F valuations, before subtraction. Their endpoint Taylor terms explain
why generation must retain possible poles down to epsilon minus three. This
does not establish which Laurent residues cancel after integration. Neither the
numerical leading coefficient nor any other output is replaced by zero.

The full native audit log is
`output/diagnostics/triple-box-offshell/domain-audit.log`.

The existing HEPKit `IntegralFamily::partial_fraction(&[1;10], 32)` also passed
its bounded executable reuse check. It returns exactly one term with coefficient
one and original-order signed powers `[1,1,0,2,0,2,1,1,1,1]`, leaving eight active
propagators. Reconstructing its denominator product gives the original native
Atom by canonical equality, without expansion, rational combination or a custom
deduplication algorithm. Calling `IntegralFamily::sector` on that returned power
vector gives eight denominators with active powers `[1,1,2,2,1,1,1,1]`. A second
canonical reconstruction check passes, while the three loop variables, three
independent external variables and native kinematics are preserved. This supplies
the executable check for native positive-power projection in a future family
adapter; negative powers must remain numerator factors if later present.
Neither this finding nor the probe
changes the historical graph or the production parameterization used below.

## Scalar generation and complete first allocation

The ignored std-only process wrapper `output/probes/bounded_cli.rs` saves the
expanded arguments, stdout, status JSON lines, sampled peak resident memory and
exit outcome. It requests normal CLI cancellation at the stage deadline, then
allows five seconds before terminating its own child if cancellation cannot
return. This wrapper contains no algebra or numerical estimator.

Generation of the shipped `triple_box_offshell.toml` card completed within the
300-second budget. Its 2,496 charts reduce by verified complete-density symmetry
to 1,182 nine-dimensional kernels, with output orders `[-3,-2,-1,0]`. The full
process took 51.705 seconds and sampled peak RSS was 1,284,408 KiB. Native timers
report geometry 3.498 s, mapping 4.339 s, symmetry 5.964 s, subtraction 0.375 s and
Laurent extraction 0.796 s. The final per-kernel compilation callback was at
18.135 s of compilation; serialization and publication add further work to the
whole-process figure. The portable artifact occupies about 346 MiB.

Fresh-process integration loaded that artifact, selected native published
Kuo38005 explicitly, used Korobov3, 1,024 points per shift, eight complete shared
shifts, seed 20261004 and two caller-owned workers. All 1,182 kernels received
their full allocation: 9,682,944 accepted kernel-point evaluations. The native
saved-result document retains full covariance, allocation metadata, exact
contributions, every sector's contribution and precision diagnostics.
The checkpoint confirms actual modulus 1024 and the reduced nine-dimensional
generator `[1,309,235,573,145,523,153,653,577]` for every sector. Complete native
shift arrays and accepted package records remain in the checkpoint; the nominal
catalogue name is not the only retained lattice evidence.

| Epsilon order | Mean | Estimated standard error |
| --- | ---: | ---: |
| -3 | -0.00569812737712 | 0.00598614284989 |
| -2 | -0.273511892568 | 0.0343550061038 |
| -1 | 1.14585006888 | 0.131240507255 |
| 0 | 2.96944325400 | 0.603821166512 |

The allocation is complete, but the requested accuracy is not reached:
`converged=false`, stopping reason `work limit`. No external integrated reference
has been assigned to these values. The leading result is compatible with zero
at this resolution; that is not an exact pole-cancellation proof.

The 180-second watchdog did not fire. The whole process took 146.205 s, including
37.704 s of artifact loading and 108.146 s in the CLI integration phase. Sampled
peak RSS was 1,864,280 KiB. There were 815,397 conditioning checks, 814,825 rescues,
2,069 weighted checks and 2,041 additional replays, with a maximum precision of
448 bits and zero evaluation failures. The whole-sector streamed snapshots and
repeated checkpoint writes contribute substantial I/O, so these wall times must
not be used as an isolated kernel-throughput comparison.

Evidence under `output/diagnostics/triple-box-offshell/` includes
`scalar.generate.{argv.txt,json,status.jsonl,process.json}`,
`scalar.integrate.{argv.txt,json,status.jsonl,process.json}`, the portable
`scalar.fsd.json`, resumable `scalar.checkpoint.json` and native versioned
`scalar.result.json`. Partial stages and limits are never substituted for a
complete estimate.

## Original rank-two numerator baseline

After the exact topology/domain audit and the short unrelated CLI regression
handoff, the same fixed executable generated the shipped
`triple_box_offshell_rank2_numerator.toml` card without applying partial fractions.
Its numerator is `k1.k3+2*(k2.p1)*(k2.p2)` in the explicitly tested loop basis.
The existing coupled-sunset gate validates the shared Gaussian construction;
it is not an independent integrated reference for this three-loop value.

Generation again retained 2,496 charts and 1,182 nine-dimensional kernels with
all four orders `[-3,-2,-1,0]`. It completed in 55.406 s within the 300-second
bound, with sampled peak RSS 659,132 KiB. Native timings were parameterization
0.219 s, geometry 3.504 s, mapping 21.782 s, symmetry 8.476 s, subtraction 0.278 s
and Laurent extraction 0.672 s. The complete artifact is about 144 MiB. These
stage figures describe this representation and run, not a controlled scalar
versus numerator performance experiment.

The fresh-process numerical trial used the same explicit Kuo38005 rule, actual
modulus/vector, Korobov3, seed, eight shifts, 1,024 points and two workers as the
scalar. All 9,682,944 planned kernel-point evaluations completed.

| Epsilon order | Mean | Estimated standard error |
| --- | ---: | ---: |
| -3 | -0.0451800378456 | 0.00348472897019 |
| -2 | 0.374106973232 | 0.0242249355232 |
| -1 | -0.469202970805 | 0.0390706571262 |
| 0 | 0.329021590135 | 0.174370309903 |

The native result again records full coverage but `converged=false` and stopping
reason `WorkLimit`, with validation status `unverified`. The numerator and scalar
are different integrals; no order or sector is aligned to infer equality, and no
pole is removed based on expectations from an undotted ladder.

The complete process took 56.313 s, with 20.778 s of artifact loading and
35.236 s in the numerical/report/checkpoint phase. Sampled peak RSS was 853,104
KiB. There were 278,296 conditioning checks, 273,276 rescues and 2,011 additional
weighted replays, with a maximum precision of 320 bits and zero failures. The
180-second numerical watchdog did not fire. Evidence follows the same naming
scheme as the scalar, prefixed `rank2`, including `rank2.result.json` and
`rank2.checkpoint.json`. Both checkpoints' actual rules were independently
extracted to `scalar.actual-rules.json` and `rank2.actual-rules.json` and agree.

Further work requires an independent repeated-propagator reference and a
prespecified convergence allocation. The native eight-active-factor projection
provides a smaller, exactly equivalent input for that investigation while these
unchanged original-graph baselines remain available for comparison.
