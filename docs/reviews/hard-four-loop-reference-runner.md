# Concrete hard-four-loop ordinary reference runner

The source-only proposal in [hard-four-loop-reference-proposal.md](hard-four-loop-reference-proposal.md)
has passed the independent HEPKit API review. The new ignored caller is
`output/probes/run_hard_four_loop_together.py`; the earlier unexecuted draft and
the frozen projected triple-box caller remain unchanged. Production FastSecDec
does not depend on this reference environment.

The runner delegates inherited-card parsing and validation to the frozen FSD
request API, then uses its existing U/F loader and decomposition-polynomial
adapter. It checks the prescribed hashes of both external cards and the native
card/U/F files, and requires the original external U/F text to equal the native
input after whitespace removal. It admits only the nine original ordered
variables, no physical parameters or sector filter, unit coordinate measure,
the documented four-loop/dimension metadata, and `U*F^(eps-3)` with prefactor
one. These checks precede native package generation.

Public `pySecDec.code_writer.make_package` receives F as the only decomposition
factor and U as `other_polynomials`. All actual keyword arguments and the
normalized native input are written before generation. The existing native
orthant maps and subtraction retain U, including its signed value and monomial
growth at infinity. There is no added projective constraint or Gamma factor.

The ordinary constituent is compiled with native `make -j1` and integrated with
`IntegralLibrary(together=True)`. The full sector sum therefore enters each
native coefficient uncertainty. Every returned order and complex component is
retained in the original string tuple before the existing native converter is
called; physical member two is recorded without additional convolution. No
cross-order covariance or actual-work count is inferred.

The prescribed initial request is Korobov3, `cbcpt_dn1_100`, N8311/R32,
seed20261219, no fitted transform, one numerical thread, and maxeval265952.
This requests 265952 scalar summed-coefficient evaluations per coefficient;
only actual logs can establish completed work or further allocations. It is
not an accuracy or timing acceptance claim.

## Supervision and launch boundary

Each invocation requires a fresh absolute output directory. The reviewed
process-tree watchdog supervises a shared 600-second attempt and 30 GiB RSS
limit, with its explicit one-second polling and five-second interruption/kill
graces. Numerical work receives at most 180 seconds, clipped to the remaining
shared deadline; the clipped limit also reaches the native numerical call.
Every completed stage, failure, and later unstarted stage is recorded. The
outer watchdog remains authoritative if a child does not return.

Recorded controls first prove the fail-closed guard rejects Symbolica imports
and that ordinary IntegralLibrary/conversion imports succeed under that guard.
Generation then runs in the exclusive Symbolica slot. FORM/export Python
helpers and numerical work run under the same guard. Native source/tool hashes,
compiler versions, package versions, generated metadata/libraries and final
source immutability are retained. AST syntax and referenced additional source
paths were checked without initializing Symbolica. The subsequently authorized
guard/import controls passed under
`output/diagnostics/reference-import-controls-20261005T020026Z`: the negative
guard rejects Symbolica before loading, and this frozen runner imports ordinary
IntegralLibrary/conversion under the guard. Both exits are zero with empty
stderr; timer and SHA256 postflight checks preserve every source/executable
identity. These controls perform no symbolic generation or integration.

After independent concrete-source review and explicit runtime handoff:

```sh
/common/dev/fastsecdec/DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  /common/dev/fastsecdec/output/probes/run_hard_four_loop_together.py \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/hard4loop-together-attempt-1
```

Current runner SHA256:
`f2d2a98b9f16d02c79c2dbe6400049145a8bebed22b73141c37a4be44293e83b`.
Concrete-source independent HEPKit review passed: original input and orthant
scope, native F-only transformation of U, complete ordinary sector sum and
tuple preservation, clipped supervision and source provenance have no open
source finding. See [the independent review](hard-four-loop-independent.md).
Generated package metadata, complete returned orders and provider uncertainties
remain outcome gates. There is no scientific
execution or reference result from this route yet.

The projected scalar/rank-two runner remains frozen at SHA256
`c14bb4faef75a026d6d09ba83a8d836dd1aea3920e6140043552c6beabb63beb`,
ready for its separately coordinated next slot.
