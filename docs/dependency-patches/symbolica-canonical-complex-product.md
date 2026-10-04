# Canonical complex product parentheses

Symbolica's canonical printer treated only `AtomView::Add` as requiring
parentheses inside a product. A complex `AtomView::Num` can itself print as a
sum. Consequently `(2+3i)*x` was printed as `2+3i*x`, and importing that canonical
string silently changed the expression. The same wrong string could identify
both expressions, defeating a content hash based on canonical output.

The adjacent minimal patch extends the existing canonical-printer precedence
predicate to parenthesize a nonreal numeric factor. It adds no FastSecDec
serializer or algebra. Native regression cases cover complex product factors,
negative and fractional parts, pure imaginary factors, bases and exponents of
powers, nested products, and function arguments. FastSecDec's independent
complex artifact test additionally checks the numerical value after save/load:
`(2+3i)*x` at `x=1/4` must yield `(1/2,3/4)`.

The local patch is intended for upstream review; no push has been performed.
Validation passed: the focused native
`cargo test -p symbolica --lib canonical_complex_product_roundtrips -- --test-threads=1`
regression, plus FastSecDec's four public complex-kernel tests. The latter
independently exercise all eight precedence contexts, complete portable
save/load, real/imaginary covariance, and Gamma/logarithm precision rescue on a
worker. The full short mathematical validation additionally passed 12 library,
16 generation, and five independent generation tests.
