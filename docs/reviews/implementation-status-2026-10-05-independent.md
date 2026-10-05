# Independent check of the 2026-10-05 status report

Read `implementation-status-2026-10-05.md` against the frozen fourteen native
eight-core rows and their descriptive summary, the prior paired timer audit,
the sample-latency evidence, preparation/smoke records and original/projected
large-case process records. This is a source/data review, with no new scientific
run or estimator. The status report itself remains coordinator-owned.

No numerical discrepancy was found. In particular:

- The target is the highest signed requested power, epsilon zero for the small
  cases and epsilon two for Issue 1. The report does not select the deepest pole
  or infer an earlier/minimal-work threshold crossing from the first tested
  complete allocation.
- Eight-core native medians agree with the retained summaries: triangle/box
  driver intervals 0.032968704/0.041598931 seconds, whole processes
  0.043067946/0.054015710 seconds, and separate loads
  0.004713895/0.006712451 seconds. Counts, sixteen common shifts and complete
  `[-2,-1,0]` vectors agree. Reference instance-limit failures remain unavailable
  performance observations, not inferred eight-core times.
- The older paired worker costs are the one-worker N8192/R16 campaign. They are
  not conflated with the eight-worker N1024/R16 runs. Native accepted-worker and
  Pathfinder evaluator/Python/global-integrator buckets retain their different
  boundaries. Sector means are not relabelled individual-sample maxima.
- Every individual latency mean and maximum agrees with the separate native
  single-seed instrumented diagnostic. Observed maxima remain finite samples
  including possible scheduling delay, not arithmetic-only worst-case bounds.
- Projected native generation times 18.373002647/16.411499577 seconds are actual
  whole-process observations. The large fixed-work campaigns use two workers;
  they do not establish eight-core time to the stated accuracy.
- Alias/cache commits are clearly separated from the older frozen performance
  builds. Representative-level oracle/weighted checks are not presented as full
  difficult-case performance acceptance or uncertainty calibration.

Two wording clarifications were sent to the coordinator. The old native
generation baseline compiled O2 before persisting canonical expressions, then
compiled O2 again during fresh loading; “saves expressions for later O2” should
not imply the first compilation was absent. Also, the larger table's worker
costs belong to the **original graph** campaigns, whereas its generation column
lists both original and projected families. Naming that association avoids an
accidental cross-representation comparison. Neither finding changes a number.
Both wording clarifications are now applied in the report.

For timer interpretation, the eight-core driver's reported elapsed interval
starts after eager loading and pool/problem setup. The separately retained
whole-process row is the broader operational boundary. General calibration and
the remaining full-vector/reference gates remain open as stated.
