# OneLOop test-provider compatibility with SymJIT 2.26.4

The user requested the latest Symbolica and SymJIT releases for evaluator
representation experiments. On 2026-10-04 the live crates.io index identified
Symbolica 3.0.1 and SymJIT 2.26.4 as the latest non-yanked releases. FastSecDec's
Symbolica worktree already includes that release; its SymJIT pin was 2.26.0.

Updating the root pin initially failed because the development-only native
OneLOop provider also pins exactly 2.26.0. Its compiled-evaluator cache header
contains that exact backend version. The independent HEPKit reviewer inspected
the provider's API use and cache checks before the local change: the provider
uses Symbolica's evaluator interfaces and has no direct `symjit::` calls.

The [two-line patch](oneloop-symjit-2.26.4.patch) updates the manifest pin and
cache header together. Updating only the manifest would mislabel cache
compatibility. This changes no master formula or reduction algorithm. Existing
comments about earlier backend defects remain historical evidence.

FastSecDec disables the provider's default `prebuilt` feature and obtains its
reference numbers through the native Expression backend. Old bundled caches
retain their original version and are intentionally rejected if that feature is
enabled against the new header; regenerating those optional assets is outside
this test-provider patch.

Validation passed: `native_master_jit_cache_round_trip_uses_current_backend_and_rejects_old_version`
rebuilds a native B0 evaluator, reloads its cache through the provider's public
API, compares complete vectors at two Euclidean points with its Expression
backend, and rejects a deliberately old 2.26.0 header. The full updated workspace
gate passed 202 tests, including all native scalar/numerator comparisons. Logs:
`output/symjit-update-native-cache-test.log` and
`output/symjit-2.26.4-workspace-tests.log`.

The provider remains a local development dependency at revision
`a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a` with this small patch. It is not a
FastSecDec production dependency, and no dependency repository was pushed.
