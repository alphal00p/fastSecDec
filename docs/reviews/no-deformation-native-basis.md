# Native basis admission and the F4 cache correction

An exact localization probe exposed a native F4 error for the Lex-ordered
variables `(x,u,z,v)` and generators `x²-x-z`, `u-x*u-1`, `2*x*v-v-1`.
The returned basis omitted `(z+1/4)*v²-1/4`; two original generators had nonzero
remainders, and the native Buchberger checker rejected the result.

Independent instrumentation traced this to cached generator multiples being
replaced by arbitrary reduced matrix rows with the same leading monomial. Two
distinct critical-pair inputs could then become the same expression. The narrow
[Symbolica correction](https://github.com/symbolica-dev/symbolica/pull/68)
keeps exact-multiple memoization and removes those arbitrary replacements. It
retains the native F4 algorithm and row reduction. Three new owner groups and
23 existing native Gröbner controls pass. The unchanged public `community`
baseline fails all three new groups. Independent source and executable reviews
agree on the failure mechanism. Publication is by ValentinHirschi; GitHub declined
formal reviewer assignment, so the authorized review invitation tags `benruijl`
in the PR discussion.

FastSecDec now shares one private admission helper at its four native F4 sites.
It checks variable maps, output shape, native Buchberger closure and reduction of
every original generator. Reverse ideal inclusion relies on native F4's
constructive combinations; this is not a verifier for arbitrary supplied bases.
Literal zero generators are filtered before the owner call, preserving the
existing handling of the optional zero-generator robustness fix.

Checked pair counts and conservative term/degree bounds precede explicit
S-polynomial verification. These bounds do not interrupt allocations inside
native F4 or polynomial reduction. Caller-owned hard limits remain necessary.
Maintained regressions retain the actual incomplete basis, empty/zero inputs,
foreign variable maps and exhausted verification budgets. Independent runtime
and root reviews accept these boundaries; no alternative ideal or CAS engine
has been introduced.
