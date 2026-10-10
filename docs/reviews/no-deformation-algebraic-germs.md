# Generic local algebraic factor germs

The native algebraic-branch building block retains a monic defining polynomial
of arbitrary admitted degree and a certified exact real fiber. A coprime split
at that fiber defines local implicit factors through their coefficient-product
equations. There is no quadratic formula requirement, radical expansion, or
replacement of the bulk branch by a truncated series.

Symbolica supplies the algebraic coefficient field, selected roots, real
embedding, polynomial product, resultant, matrix determinant and differentiation.
Public-API and source inspection were followed by executable probes of product
minors, native constant fields and the existing checked étale derivative frame.
The actual product minor is compared with the native resultant; the complete
frame retains the original minor and guards and, where needed, the primitive
constant's minimal-polynomial derivative. This work exposed the narrow
[native resultant correction](https://github.com/symbolica-dev/symbolica/pull/69).

All constants are imported into one native algebraic field before retaining
their images, avoiding stale primitive-element representations. The selected
primitive element and supplied values must be real. Parameter derivatives and
algebraic constants remain inert in integration directions. Every returned jet
retains its branch owner, including its exact field and embedding.

Controls include a selected root of `z^5-z-1`, nested `sqrt(2+sqrt(2))`, mixed
derivatives, a nontrivial inherited implicit frame, guards, parameters, resource
refusal and owner lifetime. Independent root and resolver-agent reviews accept
the local mathematics and ecosystem reuse. The existing two-symbol ring
extension delegates to the shared variable-length extension; no second CAS or
AD engine is introduced.

This proves a germ near a selected regular fiber. It does not yet certify a
branch over an entire cell, establish normal crossings, resolve collisions,
construct an Abhyankar–Jung ramification, or issue a closed-face endpoint
certificate. The determinant preflight uses a conservative factorial bound;
some sparse high-degree cases can therefore return resource-incomplete until
that bound is sharpened or the caller increases its limit. Native internal
allocations remain subject to caller-owned process limits.
