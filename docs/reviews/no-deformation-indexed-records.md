# Detached threshold compilation and indexed restoration

The native threshold factory now separates preparation from compilation jobs.
Preparation writes coefficient vectors and record-local expression tables using
the existing context-aware Symbolica codec. A detached job retains compact
lineage and immutable file references; it does not retain the GCAD proof,
continuation owner or every sector's expressions. The caller still owns
preparation scheduling, worker lifetimes and hard memory limits.

An explicit setup/exact-offset carrier stores the shared parent directory once.
Stochastic records refer to it and carry only their own native tables and
program. The new indexed v3 directory and standalone v16 envelope coexist with
the previous formats. Reading the directory does not decode evaluators; loading
one sector reads the carrier and that record. Saved optimized programs and
compatible primary caches use the existing restoration paths, without symbolic
generation or Horner/CPE optimization.

Compilation work binds the preparation record and job index. A live native
plan checks returned work identity, contribution lineage and expected native
program identity before copying the hashed bytes to the archive. Duplicate or
missing contributions cannot produce a complete directory. Serialized work is
transport, not a deserialized GCAD proof. Local-to-parent lineage is separate
from legacy source-sector numbering. A selected exact-only owner retains its
selected scope; only the complete catalogue supplies the full accumulation
scope and identity.

The shared staging reader hashes and decodes the same bounded byte buffer,
closing the former verification/reopen gap. Publication continues to use the
existing immutable data file and atomic manifest transaction. Root and the
independent runtime agent reviewed native reuse, scientific scope, worker
receipts and record residency. Fresh-process controls exercise detached work,
shuffled completion order, corrupt receipts, selective loading, legacy
compatibility and a physical bubble against the existing OneLOop reference.

The native preparation peak remains separate from sector residency. Transport
byte caps do not bound decoded compiler memory; process exit remains necessary
to release native arenas. Initial carrier loading currently decodes the compact
carrier twice when it is itself requested. Global proof replay, the complete
CLI recovery adapter and general algebraic endpoint charts remain separate
delivery gates. This format work does not complete threshold decomposition.
