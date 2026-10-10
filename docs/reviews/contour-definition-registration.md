# Native contour definition registration

The Symbolic-endpoint D05 compilation profile found repeated work in
`ContourDefinitions::function_map`. A five-second, 49 Hz sample of the owned
fixed source8 worker attributed 46.53% inclusive time to the native visitor used
there. A separate 30.69% self-time Atom hashing entry belonged to native
`linearize_impl`; it was not evidence of repeated derivative-body construction.
The profiles remain under `target/contour-symbolic-endpoint-hotpaths/`.

Registration used the same private call decoder as materialization and
simplification. That decoder copied every actual argument into owned Atoms,
although registration reads only the native definition, derivative orders,
formal parameters and derivative body. The narrow change omits those copies
for registration. Materialization and simplification retain them. Native
`AtomCore::visitor` still descends through every argument, including nested
compact calls; signature, derivative-order and definition-scope checks are
unchanged. Native Atom differentiation and FunctionMap registration remain the
only algebra/evaluator owners. No new traversal, cache, codec or numeric
callback was added.

Before the change, public APIs and native source were checked at Symbolica
`74225696cd445247fa81c499c5110decd19257ed`. `get_all_symbols` also walks the
expression and does not provide derivative orders; replacing the visitor with
whole-function deduplication would add hashing of large actual arguments.
Neither is needed to remove the unused ownership.

A standalone old/new probe registered 96 roots containing nested native calls
and mixed derivatives, totaling 3,825,024 Atom bytes. The resulting native exact
evaluator bytes were identical. Registration took 12.546 ms before and 12.201 ms
after in this small fixture; no material physical compilation speedup is
claimed. The complete probe closed in 0.102 s with a 12.7 MB sampled RSS peak
under a 60-second/2 GB bound. Its sources, exact compiler inputs and result are
retained in `target/contour-definition-traversal-probe/`.

The registered regression compares nested calls and native mixed derivatives
through third order against materialized bodies. It also requires malformed
and missing nested definitions to remain errors. The focused native gates
passed: nine definition controls, five Symbolic-endpoint contour compiler
controls, one complete higher-jet saved-owner control and one streamed native
source/map control. Independent source and test review passed. Logs are
`target/contour-definition-traversal-{tests-r2,symbolic-tests,dual-tests,staged-tests}.log`.

The native FunctionMap `Never` inlining policy is a separate research question.
It is not enabled by this change. The current Dual composition paths require
inlined bodies, and ordinary Symbolic-J policy changes require their own native
derivative, callback, saved-owner and physical performance evidence. Existing
generation campaigns continue using their immutable pre-change binaries.
