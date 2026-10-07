# Historical reference input bytes

These are the exact run-card bytes used by the frozen independent references,
preserved from the pre-migration repository. They are evidence fixtures, not
current runnable examples. Their hashes remain those recorded in
`examples/references/*.json`; the reference observations and provenance were not
rewritten to imply a new external calculation.

`input::tests::runtime_example_points_preserve_frozen_reference_densities` loads
each historical graph card under its original fixed-model policy, then compares
its native parametric density with the current example at the explicitly saved
runtime point. Coordinate order, domain and regulator must also agree. Files
referenced by these cards retain their normal paths relative to `examples/runs`;
the test supplies those dependencies in a temporary directory.
