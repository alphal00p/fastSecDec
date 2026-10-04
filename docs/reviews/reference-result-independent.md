# Independent reference-result review

Reviewed the connected `fastsecdec::reference` implementation on 2026-10-04 after its author froze the API. No actionable correctness finding remains in the reviewed scope. The nine focused tests were run independently and passed; the log is `output/reference-independent-tests.log`.

The adapter consumes the existing native `VectorEstimate`. It performs key alignment and scalar comparison only; it introduces no estimator, covariance approximation, symbolic convention conversion or integration stopping rule. The reuse investigation in `reference-result-reuse-audit.md` checks the relevant existing public APIs and consumer paths. Its small application-specific adapter is justified by the absence of a reusable external-reference comparison API.

Scientific checks covered:

- Keys are explicit `(Laurent order, real/imaginary component)` pairs. Comparison covers their union, preserves absent rows, rejects duplicates, and never assumes a missing coefficient is zero.
- Unknown uncertainty, reported standard error and declared exactness remain distinct. Historical null errors stay unknown; historical numeric zero errors remain reported errors. Parsing never upgrades historical validation status.
- Kernel identity mismatch is an error. Normalization, kinematics and independence are caller-recorded evidence rather than inferred conversions. Unknown independence and known correlation suppress combined-error pulls; no cross-covariance is invented.
- Known independent scalar errors use `hypot`. Unknown errors have no pull; zero combined error reports explicit equality/inequality. Derived nonfinite numbers fail explicitly instead of leaking invalid JSON. Per-component pulls do not imply a joint vector test.
- Comparison eligibility records evidence and complete production coverage; it is distinct from numerical agreement and mathematical truth. Unverified references can have clearly labelled diagnostic pulls when their reported uncertainty and independence are known.
- The native version-one envelope validates its discriminator and version, rejects ambiguous historical/native documents and unversioned objects, and validates the reference on both reading and writing. Provenance, uncertainty and validation survive transport unchanged.

The focused tests exercise shuffled complex keys, missing rows, covariance shape, finite-value boundaries, independence and compatibility evidence, exact zero versus unknown errors, actual historical fixtures, and strict transport dispatch. Additional duplicate tests were unnecessary. The CLI's use of these assumptions and its presentation remain a separate review boundary; this review does not independently certify any historical target or its stated error.
