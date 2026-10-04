# Independent remaining-reference orchestration audit

This is a source/API review of the ignored launchers
`output/probes/run_remaining_reference.py`,
`capture_pysecdec_disteval.py`, `capture_pysecdec_series.py`, and
`projected_triple_reference.py`, against the frozen Pathfinder revision and
installed pySecDec 1.6.6. The follow-up issue-1, hard-orthant and projected
triple-box launchers had not been executed when reviewed. This review starts no
reference process and establishes no numerical reference result.

No blocking scientific input or normalization mismatch was found:

- The projected triple-box caller uses the existing reference DOT and kinematics
  readers, preserving the original loop/external basis and numerator. The two
  exactly repeated propagator pairs at positions 2/3 and 4/5 admit the recorded
  coefficient-one power replacement. Native
  `LoopIntegralFromPropagators(..., powerlist=...)` removes zero-power lines and
  their parameters itself (`loop_integral/from_propagators.py:166`). The caller
  checks the resulting active powers `[1,1,2,2,1,1,1,1]`.
- `kinematics.py:49` supplies fixed FORM-compatible numeric replacement strings,
  including the all-external-virtualities-minus-one and s12=s23=-2 point. The
  direct constructor therefore needs no additional numeric real parameters.
  Scalar mode changes only the numerator to one; rank-two mode preserves the
  supplied contracted expression.
- Native `LoopPackage` inserts `Gamma_factor` exactly once together with the
  raised-power measure (`loop_package.py:80–96`). No projected Jacobian, factorial
  or extra Gamma convolution is introduced. The direct caller retains the entire
  `IntegralLibrary(..., format="json")` tuple and identifies tuple member 2 as
  the full physical result, matching `integral_interface.py:1353–1368`.
- The U/F caller uses the existing `run_pysecdec_uf_all_sectors`, which rejects a
  sector filter and delegates decomposition to native make_package. Both cards
  retain `geometric_infinity_no_primary`, so the domain is the full positive
  orthant. The supplied global prefactors are one; the known generic U/F bridge
  prefactor omission is therefore inactive for these two inputs only.
- The native disteval lane receives initial points/shifts and a numerical
  deadline from `_run_pysecdec_disteval`, rather than an IntegralLibrary maxeval
  budget. The wrapper captures its native result before delegating to the
  unchanged parser. It neither adds absent orders nor sets missing uncertainty
  to zero. The original tuple capture similarly precedes parser invocation, so a
  parser exception cannot destroy already written evidence.

The orchestrator requires a new absolute output directory, records exact argv,
source revision/status/hashes and compiler/package versions, fixes one allowed
CPU and native generation threads, and uses the existing 600-second/30-GiB
process-tree watchdog. The direct propagator lane additionally passes
`processes=1` explicitly. The documented command uses the frozen reference
environment's interpreter, which matters because preparation package versions
are collected in the launcher process.

These are bounded correctness attempts, not matched performance measurements.
Native observed work, actual generated package prefactors, complete order
coverage, finite means/errors and the native full-result tuple must still be
checked for every successful attempt. Initial lattice requests and maxeval are
not observed evaluation counts. Raw imaginary values/errors must be retained
before any justified projection to real-only FastSecDec outputs. A timeout,
package-generation failure or successful process with no complete physical
result leaves the independent reference gate unresolved.

The already launched original double-box attempt was reported reaped after its
600-second bound with no numerical tuple. That is retained failed-attempt
evidence, not a reason to accept a historical placeholder target or alter any
native FastSecDec normalization.
