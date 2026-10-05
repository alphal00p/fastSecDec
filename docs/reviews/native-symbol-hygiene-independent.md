# Native formal-symbol admission — independent review

The disconnected control and public native API composition pass source and
outcome review on 2026-10-05. The [author's source record](native-symbol-hygiene.md)
identifies the original gap: a bare native symbol lookup can reuse a reserved
name carrying another caller's attributes or hooks. Explicit empty attributes
alone do not reject existing hooks, because an absent requested hook is treated
as unconstrained by native `custom_function_matches`.

The reviewed composite uses native `SymbolBuilder` to compare empty attributes,
tags, aliases and user data, then requires native `Symbol::is_exportable()`.
That predicate checks all five hook families. Existing input-owned symbols are
looked up and skipped before strict admission; an unused incompatible reserved
name produces a typed conflict. The native registry and local `AliasedAtom`
maps remain the only owners. No global reset, body registry, callback-semantic
comparison or dependency patch is introduced.

`output/diagnostics/native-symbol-hygiene-1` exits zero in 0.007197841 seconds,
without a timeout, at peak child RSS 6,152 KiB. Independent preflight verifies
22 build and 27 attempt entries; all 27 postflight entries remain unchanged.
The control rejects all nine attributes, three other metadata cases and all
five hook families. It confirms that the empty builder alone admits those
five hooks and that the full composite rejects them without executing a
foreign callback. Two input-owned collisions are skipped safely.

Sixty-four repeated local jobs reuse their handles without growing the bounded
reserved namespace. Two simultaneously live alias vectors use the same roots
and handles with distinct local bodies, yielding `[7,-1]` and `[18,6]` through
independent native evaluators. Native program values survive dropping a local
vector, and a late attempt to attach a callback to a plain symbol is rejected.
The test observes the relevant namespace, not unrelated global symbols or a
claim of constant process memory.

Reviewed main and admission source SHA-256 values are
`4ea1395dcc3a67f6c8d4cc221cab91c1c1fc581f07ef9e76f019f5fd3f9229c3`
and `8384e266353471368ccd4736bf6b7e904f03aef806c4d79a5496e6e2e1b217a4`.
The result SHA-256 is
`e52dc33c367e98595f926439690bf316eaeae42d5631aa62a09b1c578d010015`.
This accepts the narrow native admission and local alias ownership mechanism.
The production allocator still needs the same collision, repeated-generation
and simultaneous-vector controls after integration. Existing scientific,
precision and cold artifact gates remain independently required.
