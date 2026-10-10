# Threshold-decomposed bubble

`threshold_bubble.toml` uses the native HEPKit scalar bubble graph with
`s=16`, equal squared masses `m²=3`, and `D=4-2*eps`. Its causal polynomial
has the two interior zeros `x=1/4,3/4`.

```sh
fastsecdec generate examples/no_deformation/threshold_bubble.toml \
  --serial --workers 2 --output bubble.fsd
fastsecdec integrate bubble.fsd --method qmc --workers 2 \
  --points 4096 --shifts 8 --target-order 0
fastsecdec integrate bubble.fsd --method mc --serial 0.1 --workers 2 \
  --points 4096 --shifts 8 --target-order 0 --max-rounds 3
```

`generation.threshold_decomposition=true` in the card enables generation;
`--threshold-decomposition` is its CLI equivalent. Generation and integration
serial settings are independent and use the same artifact format.

With the measure multiplier specified in the card, the analytic result is

```text
eps^-1:  1
eps^0:   2 - 3*log(3)/2 + i*pi/2
         0.3520815669978355 + 1.5707963267948966*i
```

This example exercises the currently supported fixed, one-dimensional rational
cell path. General algebraic endpoints and runtime parameter chambers remain
under development; unsupported inputs fail explicitly. It is not a substitute
for the general resolver acceptance gates in `NO_DEFORMATION_PLAN.md`.
