# Contour controls

These small, dimensionless direct-integral controls test the causal branch and
the interaction between deformation and endpoint subtraction. They precede the
native HEPKit graph and physical multiloop acceptance tests in
[the Phase B plan](../../CONTOUR_DEFORMATION_PLAN.md).

`threshold_bubble.toml` contains the Feynman-parameter density of an equal-mass
one-loop bubble, with its prefactor chosen as `1/eps` to isolate the logarithmic
branch. Its pole is 1; its finite coefficient is
`2-beta*log((1+beta)/(1-beta)) + i*pi*beta`, with `beta=sqrt(1/5)`.

`subtracted_pole.toml` combines an endpoint pole with an interior physical pole.
Its Laurent expansion starts at `-4/eps - 4*log(3) + 4*i*pi`. Its deformation
must be applied before endpoint subtraction.

The cards request contour-capable generation and fixed strength `lambda=0.2`.
The following commands exercise the common indexed artifact with either
integration scheduling mode:

```sh
fastsecdec generate examples/contour/threshold_bubble.toml -o output/contour/bubble --workers 1
fastsecdec integrate output/contour/bubble --workers 1
fastsecdec integrate output/contour/bubble --serial 1 --workers 1
```

`generate --serial` selects bounded-memory generation without changing the
artifact format. `--contour fixed --lambda 0.1` overrides the runtime strength.
The optional causal-validation policy defaults to `always`; selecting `pilot`
or `off` is separate from the mathematical prescription. Implementation and
acceptance progress is recorded in the Phase B review documents; these fixtures
are not a claim that the full physical test suite has passed.
