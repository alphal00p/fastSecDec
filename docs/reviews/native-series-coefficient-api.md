# Native series coefficient and sharing APIs

Read-only API/source/test audit; no executable experiment or dependency change
is introduced here. The actual Cargo override is
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica`, revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` with the four recorded local fixes.
The separate reference checkout is newer; the APIs below were checked against
the pinned worktree too. Its `domains/atom.rs` and `atom/alias.rs` agree with
the inspected reference files.

| Native capability | Source evidence | Consequence for this problem |
| --- | --- | --- |
| `AtomField::custom_normalization` | `src/domains/atom.rs:42`, normalization helpers at 80/90 and `RingOps` from 127; division/inversion from 328 | A cloneable thread-safe callback can replace coefficients after native arithmetic operations. No custom series multiplication is needed. The operation has already constructed its Atom before the callback. |
| Caller-supplied series coefficient field | `Series::new` in `src/poly/series.rs:261`; `Mul<&Atom>` from `src/derivative.rs:655`, `Add<&Atom>` from 726 | A one-series multiplied by the original Atom reaches native `series_impl` with `self.get_field()` and native relative-depth retry. A zero-series plus the Atom instead enforces an absolute bound. These public routes need neither private internals nor a patch to `AtomCore::series`. |
| Ordinary `.series` entry | `src/atom/core.rs:728`, `src/derivative.rs:377` | Hardcodes a fresh `AtomField` with statistical zero testing initially disabled and no normalization hook. It has no field parameter. |
| Post-expansion coefficient mapping | `Series::map_coeff`, `src/poly/series.rs:717` | Maps stored coefficients, preserves metadata and invokes native truncation. It cannot prevent memory spent constructing the original coefficients. |
| Native alias container | `AliasedAtom`, `src/atom/alias.rs:29` | Owns root plus alias definitions, nested application, duplicate fusion, pruning, conflict handling and evaluator construction. These operations must be reused rather than replaced by a second generic shared-expression container. |
| Evaluator sharing | `AliasedAtom::evaluator_multiple` at alias.rs:173; evaluator-tree common-subexpression elimination | Can carry native aliases to evaluation without expanding all definitions into each output. This occurs after coefficient construction, so it is not by itself a series-generation solution. |

The public one-series multiplication and zero-series addition routes are
source-supported candidates, not executed proofs. A one-series of native width
one, followed if necessary by the width derived from its actual bound, aligns
with the successful relative-depth oracle protocol. Native tests
`series_sub_atom` and `series_div_atom`
(`derivative.rs:1132/1147` in the patched worktree) exercise arithmetic between a
series and a new Atom with preserved bounds. The `map_coeff` test at
`poly/series.rs:1606` demonstrates a coefficient cancellation changing the actual
leading order. The AtomField documentation tests native division cancellation.
No direct custom-normalization-plus-series test was found in the checked native
tests/examples. That small capability test would be required before an actual
coefficient-normalization experiment.

`Series::constant` stores its input coefficient directly, and some series-add
branches copy coefficients rather than performing coefficient arithmetic.
Consequently the normalization hook is not a guaranteed callback for every
coefficient entering storage. A proposed identity-hook test should record actual
callback coverage; it must not infer universal interception from the field API.

The native alias tests cover creation, nested application, conflict detection,
renaming and arithmetic with alias-map preservation. There is no inspected
`Ring` implementation whose elements are `AliasedAtom`, and `Series<AtomField>`
stores ordinary Atom coefficients in a vector. Atom bodies use encoded byte
trees, not a shared coefficient DAG. Introducing a new aliased coefficient ring
would therefore be new algebra and is not recommended by this audit.

Two correctness boundaries matter before using aliases during expansion:

1. `AliasedAtom` implements `AtomCore` by exposing its **root**; it does not
   override `series` to expand through alias definitions. Calling `.series`
   on an aliased root is not an alias-aware series engine. Any hidden body must
   be known independent of the regulator, and its map must remain attached to
   later evaluation.
2. Opaque aliases can hide relationships that make a coefficient exactly zero.
   Native pole detection and inversion rely on those relationships. Even the
   native `map_coeff` test explicitly changes a leading order after coefficient
   normalization. A compact root alone is not proof of equivalent Laurent
   bounds; cancellation controls and the full original-expression oracles are
   still necessary. Reconstructing leading terms or series arithmetic manually
   would defeat the required native ownership.

Applying compaction only after coordinate subtraction avoids introducing new
unknown coordinate derivatives, which caused the early opaque-function growth.
It does not remove the zero-detection concern during epsilon expansion. The
least invasive next capability probe would count identity normalization calls
through the public native-field route and compare its complete native output
against ordinary series on pole/cancellation controls. Only then should a
separately approved native alias normalization policy be considered. No claim
about memory reduction, actual representative cost or production suitability is
made from this source audit.
