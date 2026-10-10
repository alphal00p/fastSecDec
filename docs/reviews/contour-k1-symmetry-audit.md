# K1 generation after compact contour mapping

2026-10-10. Independent read-only audit of the native generation path accepted
in `3303005`. The generation agent owns the bounded K1 campaigns and their
monitor. This audit reads source, JSON status and durable receipts only; it
does not start another generation run, decode the large binary chart records,
or change production source.

## Observed stage transition

The first 300-second campaign mapped all 186 charts, then entered exact
symmetry admission. Its status samples place mapping between approximately
0.7 and 249.1 seconds and symmetry between 249.7 and 299.6 seconds. The second
300-second resume remained in symmetry after restoring completed discovery
receipts. No evaluator had completed at its cutoff.

The third, separately authorized resume completed symmetry and scheduled all
186 representative sector jobs. Its first two durable sector receipts report
native compilation times of 28.269616924 and 28.320904466 seconds. Their native
exact programs are 141492 and 135137 bytes; saved SymJIT IR is 248073 and
240371 bytes. The stochastic records are 16690979 and 16631237 bytes. This is
successful compilation progress, not a repeated mapping failure.

The CLI's `compilation_seconds` starts immediately before
`compile_with_settings_parameters_and_dispatch`. That operation includes
`PreparedDynamic::build`, native sector lowering and backend construction,
kernel assembly and initial artifact serialization. The later indexed record
write is outside that timer. Consequently these observations do not identify
Horner/CPE, checker preparation, SymJIT or serialization individually as the
dominant part of the measured 28 seconds.

## Repeated symmetry data loading

`generation/streaming/prepare.rs::compare_symmetry` currently:

1. Restores the small source context and the complete active `ChartData`.
2. Calls `symmetry::prepare_program`, which selects the retained
   `SourceWitness` for these dynamic charts.
3. Restores each earlier candidate's complete `ChartData` in turn, prepares
   its witness, and applies `PreparedDensity::equivalent_to`.

The complete chart payload contains the mapped density, contour images and
Jacobian, native helper descriptor, independent checker sources and compact
function definitions. These are required for later generation but are not
inputs to this dynamic source-witness comparison. `read_with_program` verifies
the file in one streaming pass, reads it again, restores the descriptor and
native StateMap, and decodes the entire native payload. Candidate residency is
bounded to one candidate at a time; this is repeated work rather than a
coordinator retaining all charts.

Discovery has already computed each native canonical form to obtain its
lookup bucket, then dropped it. Symmetry reconstructs these forms for the
active chart and candidates. The bucket is deliberately coarse: its sorted
node labels/valences and edge labels omit connectivity. A shared bucket is
never an equality proof. The final comparison checks the actual native
canonical graphs and then proves the complete branch-aware density, designated
F, ordered U declarations and recipe under simultaneous native coordinate
substitution. Those checks must remain mandatory.

The saved records give a concrete lower bound on completed work:

| Receipt-derived quantity | Value |
|---|---:|
| Discovered chart records | 186 |
| Minimum / median / maximum chart bytes | 31250120 / 32470256.5 / 35333902 |
| Total distinct chart bytes | 6191141780 |
| Completed symmetry receipts at second cutoff | 132 |
| Candidate records read by those completed comparisons | 264 |
| Active plus candidate chart decodes | 396 |
| Sum of those decoded chart lengths | 13129094501 bytes |
| Verification plus read logical traffic | 26258189002 bytes |
| Accepted equivalences among those comparisons | 0 |

These counts use only entries with a completed `response_hash` in
`target/generation-agent-ltd-compact-k1/journal-after-second.json`. The file
also contains one issued but incomplete entry; counting every entry would
overstate completed work. Candidate reads stop at the accepted representative,
if any. In this snapshot every completed response retained its own source as
representative. Cancelled work, source/preparation reads, receipt validation
and serialization allocations are excluded. Logical traffic is not measured
physical disk traffic: the operating-system page cache may satisfy reads.

The progress label does not separately time decoding, native graph encoding,
Graphica canonicalization and exact substitution. Their elapsed-time shares
remain unmeasured. In particular, this audit does not claim Graphica is the
symmetry bottleneck merely because the stage is named symmetry.

## Native reuse and possible next measurement

No missing or broken upstream operation has been reproduced. Graphica already
owns canonicalization and exposes the canonical graph and vertex map;
Symbolica owns the exact simultaneous substitutions. `SourceWitness` already
has the native `Encode`/`Decode<StateMap>` implementation, and existing staged
records export native symbols before decoding Atoms. No new algebra, graph
canonicalizer or custom Atom serialization is needed to separate the witness
from heavy mapped metadata.

If a later bounded timing probe confirms this cost matters, the narrow first
candidate is a compact native source-witness record emitted during discovery.
It must bind source identity, recipe, chart index and the corresponding mapped
record identity. Loading it must retain the existing branch-aware witness and
exact comparison; hashes remain lookup and integrity checks. Other recipes
must preserve their existing mapped-density/helper ownership requirements.
Reusing an in-memory canonical form could be considered separately, but no
new graph persistence format or process-independent canonical hash is justified
by the present evidence. In particular, native Atom ordering and graph vertex
numbering must not be assumed to survive independent worker symbol insertion.

After the active measured campaign ends, an isolated saved-record probe can
separate full record verification/decode, witness preparation and exact
comparison for one actual chart/candidate pair. It should compare the same
native witness and permutation before and after any proposed transport change.
Such a probe has not been run for this audit. Compilation is already making
progress, so symmetry transport optimization must not be presented as the
remaining prerequisite for the first compiled artifact.

Evidence: `target/generation-agent-ltd-compact-k1/{preparation-summary.json,
journal-after-second.json}`, the three campaign status logs under `runs/`, and
the first two `dynamic-polynomial-v1-sector-*.json` durable receipts. Raw
campaign files remain ignored.
