# Incoming ++ amplitude at 400 GeV

This is the above-top-threshold control of the existing complete top-loop
example. The native exporter supplies the common point, with `mH=125 GeV`,
`mt=172.5 GeV`, and `cos(theta)=4/5`. Model scope, couplings, eight labelled
diagrams, colour projection, loop measure and external-state conventions are
the same as the parent reproduction.

| Method | Real A++ | Imaginary A++ |
| --- | ---: | ---: |
| FastSecDec, fixed lambda `1e-6` | -0.0194290118804 | -0.0117485831701 |
| HEPKit / native OneLOop | -0.0194295209824 | -0.0117460318328 |
| MadLoop | -0.0194295209824 | -0.0117460318328 |

FastSecDec's joint real/imaginary standard error is `6.72874e-6`, or `0.0296%`
of the complex amplitude. Its full covariance is retained in
[the result](fastsecdec/result.json). The [comparison](comparison.json) passes
at 0.39 standard errors from the references; both Ward substitutions vanish,
and the Laurent poles cancel within their stated uncertainties. Pilot checks
were enabled and production validation disabled.

From the repository root, reproduce all three methods in a fresh directory:

```sh
example/gg_hh_one_loop_ME/threshold.sh output/gghh-400-fixed
target/release/examples/gghh_one_loop_compare output/gghh-400-fixed
```

The script uses the same source drivers as the 300 GeV example, without
overwriting its inputs or results. MadLoop runs in a private copy with its
ten-minute/15-GiB limit. The checked-in records are compact scientific
evidence; generated evaluators, raw status streams and build products remain
outside version control.

The [provenance](provenance.json) identifies the tested debug executables,
SymJIT O2 kernels and local dependency fixes. The first allocation's global
uncertainty was `0.1045%`; four box diagrams were then rerun with a higher
budget. The final sum uses one accepted result per diagram, never both the
initial and replacement estimates. All eight individual final targets were
met. This is a scientific acceptance check, not an optimized timing or a
dynamic-versus-fixed variance comparison.
