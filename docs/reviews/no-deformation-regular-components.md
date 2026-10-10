# Reuse and proof audit: regular component producer

Native owner: Symbolica `c540d3f68c90fe7bff1e507458e57a20fb95b11c`.
Registered FastSecDec resolution base at first standalone run: `caf71a4`.
No new CAS, root solver, graph, derivative, serialization, or scheduling engine.

## API / source / executable checks

1. Public Symbolica polynomial API has `GroebnerBasis::new`, native reduction and
   `rearrange_with_growth`; these implement exact elimination and ring changes.
   Source: `src/poly/groebner.rs` F4 construction/reduction and tests, and
   `src/poly/polynomial.rs` native variable growth/shrink tests. Explicit Lex order
   puts auxiliary symbols before every original variable. Original parameter
   roles shift by exactly the auxiliary count. No Q(p) coefficient-field change.
2. API/source searches for quotient-annihilator/ideal-colon/primary-decomposition
   constructors did not reveal a public operation producing original-quotient
   component witnesses. The missing piece is orchestration of existing native
   elimination, not an unavailable algebra capability. `solve.rs` already uses
   inverse-variable saturation; full native Groebner tools are reused here.
3. Focused prior probe in frozen companion `draft/components.rs` established the
   original regular-quotient construction. The new five constructor test groups
   cover checked owners, exact complete annihilator receipts, independent native
   recomputation for external candidates, and original ambient cover identities.
   Fake zero annihilator/idempotent is rejected for h=x(x-1), I=x^2.
4. Existing `QuotientNormalizer` checks two-way original-quotient ideal equality.
   `clear_units` performs native rational cancellation and denominator-unit proof.
   `divide_cartier` discovers original-localization quotients through native F4
   and rechecks exact recombination. `VerifiedOpenCover` checks the entire original
   localization, not merely the boundary or test points. No parallel helpers.
5. Fresh F4 source vectors pass through `Ideal::new` (drop literal zeros/dedup).
   Annihilator source has the nonzero equation 1-sum(u_i*f_i); zero source gives1.
   Idempotent source has explicit E equations. The adopted zero/one edge cases
   pass; no zero-only vector is sent directly to F4.

## Mathematical scope and primary references

A checked smooth finite-type Q-algebra is regular. Regular schemes are normal
([Stacks 28.9.4](https://stacks.math.columbia.edu/tag/02IR)); a Noetherian normal
scheme is a finite disjoint union of integral components
([Stacks 28.7.5–6](https://stacks.math.columbia.edu/tag/033H)). Such clopen components
have unique idempotents ([Stacks 10.21.3](https://stacks.math.columbia.edu/tag/00EE)).
These pages were read again during this review.

The annihilator algorithm is our derived consequence, not an attributed published
algorithm. On one integral component B_j, if all f_i vanish, 1-sum(u_i*f_i)=1,
so its elimination ideal is B_j. If some f_k !=0, map u_k to1/f_k in Frac(B_j)
and all other u_i to0; this proves that B_j injects into a target annihilating the
relation, hence the elimination ideal intersects B_j in0. Product decomposition
therefore identifies the exact eliminated ideal with Ann_B(J). Double annihilator
is its complementary idempotent ideal. This uses REGULARITY, not only reducedness;
components of a merely reduced singular algebra need not be disjoint.

The public regular owner is constructible only from the actual etale frame or
checked SNC singleton boundary. Source-zero ambient components are geometry
receipts, not zero integration contributions. Component patterns refer to
identically zero on algebraic components, not zeros at points or on special
parameter fibers. Neither native basis evidence nor open coverage supplies an
oriented real integration atlas, complete general BM recursion, AJ normalization,
or endpoint-integrability authority.

Resource limits bound checker-owned inputs, outputs and diagnostic counters.
Native F4 itself remains noninterruptible through the adopted API; caller-owned
process time/RSS caps remain necessary. Incomplete stage receipts are in-memory
progress, not durable checkpoints or resumable native F4 state.

Independent runtime and root reviews accepted this local construction and its
owner boundaries. The five new constructor groups cover disconnected divisors,
forged annihilators, true source-zero components, proper point zeros, parameter
roles, native inverse clearing and resource stops. Subsequent localization work
found a separate native F4 regression for three small polynomials. Its
[owner correction and shared native-basis admission](no-deformation-native-basis.md)
retain the failing basis as a regression. No checker condition is bypassed to
accept it, and these component controls do not claim the full resolver works.
