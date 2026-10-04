# Scientific and performance baseline protocol

## Frozen inputs

Reference source: FastSecDecPathFinder
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`.
Record the actual installed Symbolica, SymJIT, pySecDec, QMCPy, Python and Rust
versions together with both source revisions and complete command lines.
Reference setup and baseline runs are pending; no measured parity is claimed.

Use representative triangle/box, double-box, numerator-heavy multiloop, and hard
four-loop positive-orthant examples. Record exact measure multipliers, parameter
assignments, requested Laurent orders, domains and target coefficients.
Reference sector partitions and numbering are not compatibility requirements.

## Matched configuration

Both sides use SymJIT O2, the same worker count, precision policy, coefficient
range, transforms, and statistical target. For QMC comparisons use complete
sector support and complete Laurent-vector outputs. The reference's QMCPy linear
rank-one backend can use the same Kuo vector, lattice size, and shift count.
Where needed provide identical shifts explicitly for pointwise comparisons.
Record any unavoidable dependency-version differences instead of hiding them.

Do not compare a scalar coefficient kernel with an entire Laurent-vector kernel,
or compare per-sector medians when their decompositions differ. Measure complete
observable work and time to verified accuracy. Record additional historical
backend/rule comparisons separately from the binding matched-O2 gate.

## Measurements

Separate cold symbolic generation/compilation, warm artifact load, warmed batched
kernel evaluation, fixed-work integration, and time to scientific accuracy.
Record peak memory, real evaluation counts, worker-local point generation,
reduction, rescue counts, errors and error-certification state. Cold runs use
empty task-specific caches; warm runs explicitly identify the reused artifacts.

Alternate reference and Rust order. Take seven paired ordinary runs and at least
three expensive generation runs. Save raw timing samples and report medians and
dispersion. A per-case timing ratio above 1.05 fails the agreed gate and requires
investigation/optimization; averaging away a regression is not acceptance.

Use several independent randomization seeds for time-to-accuracy. Compare actual
error against analytic or independently verified coefficients as well as reported
uncertainty. Underestimated uncertainty, discarded failed samples, or a result
without meaningful coverage cannot pass through speed alone.

Linux x86-64 is the currently available execution platform. Record other targets
as unverified until their builds/tests are actually executed.
