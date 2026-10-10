# Local residual extraction and companion construction

This milestone adds checked local operations. It does not complete the BM
driver, componentwise divisor extraction, global center gluing, a real
integration atlas, or an endpoint certificate.

## Native reuse and scope

`divide_cartier` requires an actual `VerifiedRelativeSnc` owner. Native
polynomial division is tried first. Where quotient relations are necessary,
Symbolica's lexicographic Gröbner basis of `K+(h*T-f,V*h-1)` supplies a candidate
quotient. Both auxiliary variables must disappear, and the original localized
ring independently verifies `h*q=f`. The temporary inverse does not become an
output guard. Unit divisors and nonmembership have distinct outcomes.

The API/source/probe audit found native `quot_rem`, F4 reduction, variable-map
rearrangement and inverse-variable saturation in Symbolica's solving code.
There was no directly usable original-generator quotient-witness API. Native
F4 supplies the missing discovery step; no division or elimination engine is
implemented here. The owner revision remains
`c540d3f68c90fe7bff1e507458e57a20fb95b11c`.

Whole-equation extraction preserves associations with original generators,
checks complete recombination, and retains final nondivisibility witnesses.
For `h=x*(x-1), I=(x^2)`, the disconnected divisor has different local powers.
The result therefore does not claim componentwise BM maximality.

Residual order is measured on the marked ideal's cosupport by inclusive
relative derivative ideals and exact unit/proper-ideal checks. Upper-order
opens give a checked neighborhood cover. Order zero means a unit near that
cosupport, not on the entire chart. Parameter directions remain inert.

The old-boundary constructor applies the local arithmetic of
[Bierstone–Milman (2008), Eq. 5.2](https://ems.press/content/serial-article-files/41043)
to a checked contact quotient and immutable history owner, preserving every
ideal product and the coefficient mark. It does not lift centers, glue them,
or authorize resetting history.

One shared native relative-derivative helper now serves residual order,
ordinary contact production and full-inclusive coefficient construction.
Normal-only diagnostic jets keep their distinct meaning. No numerical
endpoint jets or replacement differentiation machinery is added.

## Evidence and limits

Four production-constructor groups and eight separate API/math groups pass in
the direct harness, with strict Clippy. The run took 0.066 seconds. Its ignored
handoff is `target/no-deformation-residual-companion/handoff.json`, SHA-256
`6e40fa128280b1d8181b0af8beb5e2c74f3238a938ee59b2dd9d816a1f028a62`.
Root and the independent runtime agent reviewed the local mathematics, owner
association, original-ring checks, parameter behavior and shared native reuse.
Only the four production groups are registered; component discovery probes
and private harnesses remain untracked.

All 48 registered local-resolution tests pass in 1.14 seconds, including the
four new constructor groups and the preceding contact, boundary, blowup and
monomial-iteration regressions.
Strict registered native library/test Clippy and workspace formatting pass.

Controls include quotient discovery modulo `y=x^2`, guarded units, residual
maximum one on cosupport despite global maximum three, order zero only near
cosupport, and the exact old-boundary coefficient `(y^2,2)` on the selected
contact chart of `x^2+y^3`. Resource and foreign-owner refusals remain explicit.

Incomplete receipts retain completed derivative layers and current quotients.
They are in-memory evidence, not a durable checkpoint or recovery of partial
F4 internals. Native algebra still requires caller-owned hard process limits;
there is no aggregate RSS scaling claim. Component splitting, general history
updates, AJ normalization and all-face integration certificates remain pending.
