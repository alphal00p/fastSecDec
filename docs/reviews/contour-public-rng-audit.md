# Independent public-dependency RNG and stream audit

2026-10-09. Read-only audit of registry Numerica 3.0.1 versus the Numerica owner
at public Symbolica commit `eccd0396599053f245e91b86fd73720a3b07e88c`, and the
new SymJIT dependency lane. No Rust implementation was changed and no competing
Cargo process was started while the coordinator ran the public-source gates.

**Conclusion:** the Numerica transition does not change production RNG state,
seed expansion, jumps, sampling draw counts, Havana grids or their serialized
checkpoint representation. It needs no additional RNG checkpoint-version guard.
Existing artifact/mathematical identity checks still apply; this does not grant
permission to combine statistics from different contour recipes or parameters.

## Exact source comparison

The registry source was compared with the fetched public checkout, not merely
with package version numbers. These files are byte-for-byte identical:

| File | SHA256 in both sources |
| --- | --- |
| `src/numerical_integration.rs` | `09e9d2751987ee1e16c7c45d75c39e6c3b04b107f5f043e48e5f6009932a35d6` |
| Registry `Cargo.toml.orig` / public `Cargo.toml` | `d926d83fa3cfa1cd4cf82b5e7f73800113e0fafc0809831b39b789b8b94492d8` |

A recursive comparison of Numerica's complete `src` trees found only two
different files: the tracked `hypot` addition in `domains/float/error.rs`, and
the negative large-integer bincode sign repair in `domains/integer.rs`. Neither
changes RNG or sampling code. The f64 sampling implementation, discrete and
continuous grids, accumulators and serde representations are unchanged. The
integer codec repair is an artifact-arithmetic issue, not a sampling-frontier
or Havana checkpoint-layout change; it must not be used to bypass existing
artifact dependency checks.

Numerica and FastSecDec still use `rand 0.9.5` / `rand_core 0.9.5` in the current
lockfile. The new `rand 0.10.3` / `rand_core 0.10.1` lane belongs to public
SymJIT `33100ae869057f35d9865c933a48bac6699acdd4`, not to MonteCarloRng.

## Native RNG and reservation contracts

`MonteCarloRng` retains the same four-u64 xoshiro256** state, SplitMix64 seed
expansion, little-endian 32-byte export/import, `next_u64`, `next_u32`,
`fill_bytes`, `jump` and `long_jump`. The jump polynomials are unchanged:
ordinary jump advances by `2^128` draws, long jump by `2^192`; the nonzero-state
generator has period `2^256-1`. Native owner tests compare the generated words
with `rand_xoshiro::Xoshiro256StarStar`, including zero-seed handling, both jump
operations and a saved-state continuation.

FastSecDec's coordinator-owned `integration/streams.rs` reserves one complete
shift/batch state and advances its persisted frontier by one native jump.
Checked `u64` stream counters allow at most `u64::MAX` reservations; each draw
budget is at most `u64::MAX`, strictly below the jump spacing. All reserved
segments fit below `2^193` steps, far below the period. Workers import their
reserved state and cannot reset or allocate the coordinator frontier.

The draw bounds still follow directly from the identical native implementation:

- Serial QMC consumes one `next_u64` per coordinate of its random shift. Lattice
  point enumeration then consumes no RNG. A reservation owns a complete lattice.
- Continuous Havana consumes one f64 uniform draw per axis, plus at most one
  uniform-floor selector. There is no rejection loop in these f64 samplers.
  `mc_draws` checks `points*(dimension+1)` for overflow.
- Ordinary discrete Havana adds one discrete-bin selection before its
  continuous child. Its existing `batch_draws` reserves
  `points*(maximum_dimension+2)`, also with checked arithmetic. The unrelated
  native `Grid::Uniform` integer-sampling path is not used by this proposal.

The serial reservation identity still binds integral, sector, epoch, phase,
replica, stream, run and lease. Reissues preserve coordinate work but acquire a
new fence after the caller confirms the old worker is dead. Duplicate, unissued
and stale returns are rejected before updating statistics. Persisted pending
work and the stream frontier remain intact through changed worker counts.

Ordinary QMC intentionally shares shifts across sectors in democratic mode;
that covariance contract is unchanged. Its serialized plan already stores
actual shift coordinates. Appending replicas preserves prior shifts and only
issues new point ranges. This intentional cross-sector sharing is distinct from
accidentally assigning the same reservation to two workers for the same sector.

## Contour evaluation and validation RNG separation

Read-through of fixed maps, causal-log callbacks, dynamic envelope/strength
callbacks, native prepared bracketed solving, and kernel contour checks found
no production RNG access. Radius iteration depends only on supplied numerical
arguments and solver options. Scoped precision and bounded failure diagnostics
are preparation/evaluation state, not randomness. Enabling a contour callback
does not consume a sampling stream.

The CLI pilot owns its separate `fastsecdec-contour-pilot-uniform-v1` stream,
derived from mathematical kernel identity, seed and chart identity. It neither
borrows nor reserves from a production session, and its samples never enter
production accumulators. Optional policy changes remain excluded from
mathematical checkpoint identity, while a changed contour prescription or
physical binding changes that identity.

SymJIT's separate rand lane is used for an internal compile-time nonce that
prevents external-call common-subexpression merging, plus explicitly named
`random` / `cplx_random` operations. Contour builders emit neither random
operation. The compiler nonce does not draw from Numerica's production state;
its presence is not a claim of bitwise identical JIT machine code across builds.

## Existing executable controls and acceptance boundary

The reviewed regression suite inspects actual coordinates and ranges, not merely
task IDs. In particular:

- `actual_coordinate_sequences_are_distinct_retries_are_identical_and_stale_leases_reject`;
- `actual_coordinate_matrix_survives_epochs_reverse_returns_reload_and_worker_resize`,
  covering QMC/MC, adaptive modes and both refinement policies;
- `native_rng_draw_bound_probe_covers_every_havana_floor_branch`;
- `identity_and_refinement_counter_exhaustion_are_transactional`;
- ordinary QMC/Havana/discrete-Havana append-and-restore coordinate regressions;
- `numerica_checkpoints_and_randomizations_survive_the_crate_move_bit_for_bit`,
  retaining historical serialized plans, accumulator state and exact point bits;
- `named_validation_sampling_replays_without_advancing_the_production_rng`.

The earlier fixed-contour workspace logs show the coordinate, refinement and
draw-bound controls passing. The coordinator is rerunning the complete suite
against the public owner identities for the combined milestone. This audit's
compatibility conclusion rests on exact source/schema equality; it does not
claim that an uncompleted public-source test run has passed. Any future change
to native RNG/state encoding or sampling draw contracts requires a renewed
audit and, if incompatible, an explicit checkpoint protocol change rather than
silent stream reseeding.
