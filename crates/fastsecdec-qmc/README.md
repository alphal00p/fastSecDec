# FastSecDec lattice QMC

Randomized rank-one lattice rules, worker-local point generation, guarded
periodization and complete-shift vector statistics. The caller owns evaluation,
parallel scheduling and the integration loop. Numerica continues to supply its
public random-number generator and integer gcd primitive; its ordinary Havana
Monte Carlo implementation is unchanged.

This crate relocates the tested QMC implementation from the Numerica feature
branch at commit `f6ecdac8237a30adfcd1be5944a95c5160e474ce`. The algorithms,
random sequences, serialized field layouts, version checks and catalogue bytes
are preserved. Module imports change, and seed expansion now accesses the same
four seed words through Numerica's public RNG state export. The original Numerica
branch and PR history remain available; FastSecDec owns this implementation for
now rather than requiring that fork as a dependency.

`serde` enables the existing bit-preserving plan and accumulator serialization.
`native` and `portable` select Numerica's existing numerical backends and can be
chosen without default features. The QMC lane itself uses `f64` coordinates and
vectors, retaining covariance between output components.

See [the example](examples/qmc.rs) for serial and caller-owned threaded loops.
The original MIT notice is included in `LICENSE-MIT`. Bundled lattice datasets
retain their original source files, copyright/license notices and provenance in
[src/data/README.md](src/data/README.md); the Apache-2.0 data license is separate
from the MIT implementation license.
