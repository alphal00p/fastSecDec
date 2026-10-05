# Independent review of the eight-core accuracy baseline

This review covers the protocol in `eight-core-accuracy-protocol.md`. Runner
and smoke-result review remain pending; this document is not timing evidence.

The protocol uses the user's corrected observable: the largest signed requested
epsilon order, zero for the proposed triangle and box cases. The finite-part
relative error is an observed native sampling error, not a certified true-error
bound. Full Laurent vectors, covariance where exported, precision diagnostics
and complete allocation coverage remain necessary even though the timing target
uses one coefficient.

The prescribed completed-allocation ladder avoids comparing incompatible
stopping rules: native FastSecDec's tolerance checks every component, while the
reference CLI uses an aggregate measure. Independent final rows at fixed N and
16 shifts are inspected without pooling different lattice sizes, interpolating
a crossing or treating a partial snapshot as complete. The first successful
row gives a grid-observed bound; if the first 1024-point row succeeds, an earlier
crossing remains unmeasured. All earlier row costs remain in cumulative timing.

The explicit common Kuo33002/linear/full-support/Korobov3 choice, disabled
reference point cap and unchanged precision safeguards are appropriate. Equal
integer seeds do not imply equal random shifts across implementations. The
reference's validated complex O2 path and the documented backend-version
difference must remain visible in any comparison. No reference covariance or
aggregate work should be fabricated where its public output lacks them.

Eight physical cores must be verified from the actual allowed cpuset and
package/core identities. The proposed singleton native-library thread settings
avoid nested parallelism while each CLI retains eight workers. No overlapping
project timing or build is allowed in measured rows; host exclusivity is not
claimed. Generation, load/setup, numerical loop and full process costs are kept
separate. The maximum sector mean is correctly distinguished from an observed
individual-sample maximum; the latter belongs to a separate instrumented
diagnostic.

No protocol blocker was found. Before execution, the runner review must verify
the exact commands and source/build bindings, actual complete N/shift coverage,
finite nonzero target selection, preservation of lower orders and failures,
alternating fixed seed campaigns, and the cumulative cap. The reviewer asked
the author to make the 600-second cap's metric explicit and bound each next
watchdog by its remaining budget. Timeout grace/shutdown costs must remain
reported rather than appearing as a successful crossing. The labelled smoke
pair is excluded from timing summaries and must pass these checks before the
measured campaign begins.
