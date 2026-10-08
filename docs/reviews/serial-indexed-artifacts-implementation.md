# Indexed artifact implementation evidence

Implementation notes, 2026-10-08. This is an owner record; the independent
scientific, memory and HEPKit integration audits remain milestone gates.

The native `kernel::indexed` interface writes independently loadable records
using the existing Symbolica context/Atom and exact-evaluator codec. It does not
define another IR, parser, algebra system, thread pool or integration loop.
`IndexedWriter` copies worker records in bounded blocks and checks their receipt
digests before admission. A failed append poisons the writer. The compact
catalogue is committed at the end and maps each local vector into the complete
Laurent/component layout. Physical completion order does not enter its identity.

`write_unit` separates exact offsets and constraints from the numerical program.
`IndexedReader::load_exact` reads only those exact records; setup can bind a
physical point in a short-lived caller-owned process without loading numerical
sector programs or compiling them. `load_sector` restores only its selected
native record, including associated rich charts. Metadata-only inspection needs
only the outer JSON catalogue. Mandatory layout/range checks remain distinct
from the existing optional integrity and symbolic-proof validation policy.

The CLI publishes immutable generation-specific data files followed by an atomic
manifest replacement. A failed replacement leaves the previous data reachable;
an interrupted generation is not a complete manifest. Staged assembly and save
do not collect all native payloads in a coordinator byte vector. Ordinary
resident generation uses the same indexed writer and format. Supported old
monolithic native artifacts retain their ordinary reader.

Version-four CLI `content_id` identifies normalized scientific input, including
source fingerprints and native dependency identities. It excludes integration
steering, source locations, storage filenames and native representation.
`kernel_content_id` and the catalogue ID separately bind the exact saved
representation. This distinction is necessary: ordinary globally padded kernels
and serial local-layout kernels can represent the same input with different
programs and exact-offset partitions. Integration checkpoints must bind both
identities and the physical point; input equivalence alone cannot admit a
checkpoint with a different sector layout. Version-three identity is unchanged.

For heterogeneous ordinary loading, FastSecDec scatters numerical outputs into
the catalogue vector; original program bytes remain unchanged. Existing
Symbolica `EvaluatorComposer::{append,finish}` was inspected in the locked
`1deccb8` source: `finish` performs native instruction optimization, and `append`
rejects non-inlined function bodies. Using it for load-time padding would repeat
work and impose new restrictions. A narrow output-layout wrapper covers scalar,
weighted batch and native precision-rescue paths instead. No symbolic expression
is reconstructed or expanded, and no Horner/CPE optimization occurs on reload.

Focused native controls cover heterogeneous real/complex Laurent vectors,
weighted matrix output mapping, unchanged program bytes, completion-order
identity, selective exact-only reads, corruption, malformed ranges, truncated
footers and interrupted writes. CLI controls cover immutable replacement,
staged-file failure, native legacy loading and scheduling-independent scientific
identity. These controls are not a measured aggregate-RSS claim or completion of
the process scheduler, integration statistics or seed-partitioning acceptance.
