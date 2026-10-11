# Detached algebraic callback preparation

This review covers executable preparation for integrating the checked regular
algebraic sections into the existing kernel. It does not admit additional
geometry or complete general algebraic integration. The registered high-degree
map and endpoint owners remain described in the
[regular-section](no-deformation-regular-sections.md) and
[secant-family](no-deformation-secant-family.md) reviews.

## Native reuse and ownership

API, source and executable checks reuse Symbolica's tagged evaluation domains,
symbolic derivative hooks, native bracketed root solver, evaluator instructions
and State codec. The root program evaluates the complete polynomial and its
derivative. Its implicit coefficient derivative is `-r^i/P_r`; higher
derivatives and composition remain Symbolica operations. No root solver, CAS,
AD, uncertainty estimator or sampling method is added.

The first probe establishes that the current registered root callback lacks
tracked-error domains. A private adapter registers native real and complex
`ErrorPropagatingFloat<f64>` and `ErrorPropagatingFloat<Float>`. Native root
refinement carries real coefficient uncertainty. The existing contour callback's
complex Newton-correction pattern preserves imaginary uncertainty even when its
centre is zero. The real propagation can conservatively overcount dependencies
repeated during iteration; these estimates are not certified enclosures.

The detached arithmetic owner retains native instructions, a rational bracket
and a degree, without the source `CellMap` or verified GCAD tree. Callback tags
remain associated with the full polynomial, coefficient order, source variable
order, actual selector/domain, bracket and preparation recipe. Two roots of the
same polynomial retain different identities. Mutable evaluator scratch is
cloned independently, and callback failure survives subsequent arithmetic that
masks the returned NaN.

## Fresh-process evidence

A parent prepares two actual degree-six branches, saves native symbol state
and exact main/helper programs, and drops the proof owners. A separate child
links only Symbolica and bincode; GCAD and helper preparation are absent from
its compilation configuration. After registering the callback, the child
imports State and restores native programs without symbolic optimization.

The controls cover the two roots and their first two derivatives, independent
clones, fixed root-valued coefficients, failed constant mapping, masked
failures, and tracked complex Float at 96 and 192 bits. Ten refusal modes cover
missing registration or owner, swapped branches and mismatched selector,
polynomial, domain, recipe, coefficient order, degree or bracket. Association
and shape validation do not prove arbitrary saved instructions implement their
advertised polynomial. Production restoration must retain the existing trusted
producer and optional integrity-validation policy.

The numerical-domain probe also checks verified quintic and septic sections,
including a vanishing leading coefficient at a closed face. Root and the
independent resolver agent reviewed the native reuse, equations, branch
association, lifetimes and failure handling. Final probe execution took
0.1302 seconds; the fresh child took 0.0272 seconds. Strict standalone Clippy
passed for both parent configurations and the child. These are small diagnostic
times, not throughput measurements; short-process RSS sampling was inconclusive.

The ignored reproduction is pinned by
`target/no-deformation-algebraic-kernels/probe-handoff.json`, SHA-256
`49537ef9ff2a8635d11d55994e7254c7b8b0a074b0f06a3905cdf6218a5e921f`.
Root verified all 57 source/evidence hashes. Reported Float comparisons do not
certify full bit accuracy, and these probes add no dedicated DoubleFloat gate.

## Remaining production work

The next slice must connect the checked continuation owner to existing complete
Laurent-vector compilation, generic callback ownership, primary/conditioning/
rescue failure boundaries and selectively loaded native records. It must test
actual Eager and SymJIT execution, f64/DD/Float rescue and native QMC/MC covariance.
Coordinate-free algebraic coefficients must not be fabricated as rational exact
offsets or counted twice. Source-independent arithmetic restoration is now
demonstrated; full-kernel restoration and bounded sector residency are not.

General singular-root resolution, parameter chambers, the global integration
atlas, physical validation and portable threshold execution are outside these
probe results. The last remains deferred as authorized.
