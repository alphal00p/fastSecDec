# Hard four-loop omitted-order boundary

The frozen native artifact produced by release `561657b` has order vector
`[-2,-1,0]`. Its version-two kernel payload contains three coefficient slots per
sector, three zero exact offsets, and chart/domain metadata. It does not retain
native Laurent remainder bounds, trailing exponents or an explicit certificate
that every omitted lower coefficient vanishes. The completed generation and
integration records retain this same three-component layout.

At that revision, `generation/laurent.rs::expand_template` asks Symbolica for
an absolute series and collects its nonzero terms. `generation/mod.rs` traverses
all representatives, updates the global minimum from their sparse coefficient
keys and emits the contiguous minimum-through-requested-maximum vector. That
source contract explains the observed minimum, but it is not an independently
retained lower-order proof in the portable artifact.

The independently generated external package has an explicit order -3 slot in
addition to -2 through zero. Any eventual physical tuple must retain all four.
The existing native `reference::compare` already forms their union: -3 receives
`MissingEstimate`, and the overall comparison remains ineligible even when
common-order diagnostic pulls are available. A provider-reported `0 +/- 0`
does not create an exact native observation, and the old three-component
estimate/covariance must remain unchanged.

A narrow separate gate can use the existing native generation entry with the
same exact input/domain and requested maximum -3. The current Laurent adapter
requires the native absolute remainder to exceed the requested order. Require
successful complete chart coverage, native exact-zero coefficients and offsets,
and no numerical kernels; retain source/input hashes and the complete generation
record. A successful zero result through -3 would constitute independent
native evidence without Monte Carlo, a new Laurent algorithm or numerical zero
inference.

The existing double-box exact-leading-pole regression instead starts from an
explicit stored coefficient and verifies its native rational primitive and
endpoint values. That mechanism cannot directly certify an absent hard-case
coefficient from this artifact.

## Separate executed certificate

The independently reviewed `output/probes/hard_lower_order_zero.rs` and shell
runner used the existing normal library, excluding all `cfg(test)` skip/capture
hooks. A first standalone link rejected inconsistent Symbolica feature builds;
its failed log remains. Fresh build two binds the same reviewed source to the
correct CLI-normal FastSecDec/Symbolica dependency fingerprints and copied
immutable rlibs. No scientific execution occurred with the failed link.

`output/diagnostics/hard-lower-order-zero-attempt-1` completed successfully under
180 seconds plus five seconds grace, 30 GiB virtual-address cap and CPU8. The
process exited zero in 67.388249 seconds with 574,508 KiB peak resident memory.
The native Complete event occurred at 64.698023 seconds; the result's
65.187202-second span also includes subsequent proof validation and hashing.
These overlapping diagnostics are not a generation-performance benchmark.

All 699 representatives completed subtraction and native Laurent extraction.
All 2,760 ordered charts, geometry maps, source coordinates, domain assessments
and representative associations equal the retained artifact's metadata after
removing only the numerical-kernel association field. The returned layout is
`[-3]`, its exact offset is native literal zero, and there are no numerical
kernels. The complete result and all source hashes pass postflight checks.
This independently certifies zero through order -3 under the current native
generation contract. It remains separate from the historical numerical
estimate and does not change its `MissingEstimate` comparison row or covariance.
