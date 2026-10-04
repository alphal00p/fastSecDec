# Independent multiloop references

These six versioned `fastsecdec-reference` files retain the finite real
coefficient and positive reported standard error from independently generated
pySecDec C++ packages. They correspond to the equally named native run cards:
mass one, external virtualities minus one, `D=4-2*eps`, unit propagator powers and
normalized per-loop measure `d^Dk/(i*pi^(D/2))`.

`Checked` records the independent source, graph, normalization and transport
review, followed by a bounded comparison of the complete native vector. It does
not certify convergence or matched performance. The original external imaginary
value and error are explicitly retained in provenance; both were zero for these
Euclidean cases. Unknown actual lattice sizes and evaluation counts remain null.
Requested `maxeval` is not presented as measured work.

Each file preserves the command, source revisions, versions, raw-report digest,
native/external input digests and sampling limitations. See the
[numerical record](../../docs/reviews/massive-multiloop-diagnostics.md) and
[independent audit](../../docs/reviews/massive-multiloop-reference-independent.md).
Raw reports, generated packages and build output remain under ignored `output/`.

Native callers use `fastsecdec::reference::read_reference` and `compare` directly.
The CLI accepts a file through `[reference] path` in a run card or `--reference`;
normalization, kinematics and independent-sampling evidence remain explicit
comparison inputs. Selecting a new file through an override clears evidence
attached to a different target. Comparison never changes integration or stopping.

The ordinary `multiloop_references` test checks these transports, uncertainties,
projection and current native source identities without external software. Its
explicitly ignored recorder uses the existing raw reports and native reference
API to write candidate files under `output/reference-fixtures`; it does not run
pySecDec or overwrite these frozen files. The ignored `massive_multiloop` CLI
harness generates and integrates all six cards and reports the library's full
typed comparisons. Its five-standard-error threshold is an initial correctness
check, separate from convergence acceptance.
