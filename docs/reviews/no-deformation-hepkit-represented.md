# Represented graph inputs through the existing HEPKit interface

This milestone connects the existing native pre-parametric graph owner to
HEPKit's `Integral` and `sector_decompose` entrypoints. It does not complete
general threshold decomposition. The admitted preparation path still has one
compact integration coordinate and the existing rational-fiber certificates.

`Integral(diagram, kinematics, ...)` retains the original native graph point.
It no longer constructs the propagator family in its constructor. Explicit
generation or parametrization runs the same native graph/family operations as
before. Type, graph-view, tensor-dimension role and propagator-power checks
remain immediate; basis, width and full scalar-point checks consequently occur
later. Display, snapshots and construction perform no generation. Native
propagator enumeration is extracted from the existing graph owner, not
reimplemented in the binding.

`ThresholdSettings(numerical_meaning="represented_values")` explicitly selects
the native conversion policy. Supported Float kinematics, scalar bindings and
measure values are converted before propagator-family arithmetic, preserving
their represented values rather than guessing simpler rationals. Original
graph objects, literal provenance and the conversion policy remain part of
source identity. Ordinary generation retains its exact-input requirements.
Represented integral-family provenance and Float payloads inside graph
numerators, projectors and weights remain unsupported; these paths refuse
rather than silently changing their meaning.

The native `prepare_graph`, `resume_graph` and `resume_graph_evidence` APIs reuse
the existing request, verification, continuation, record planning and
publication implementation. Recovery requires the original native graph
point, compares its reconstructed provenance, and re-verifies stored raw GCAD
evidence without solving again. The binding obtains source authority only
from live native preparation, never from imported checkpoint JSON. Published
numerical artifacts remain independent of the original graph and preparation
owners.

An optional versioned configuration extension stores the represented-input
policy. Exact v1 configuration bytes remain unchanged. For historical
represented-parametric v1 records, recovery obtains the missing policy from
the reconstructed native request and preserves the existing identities. New
configuration/request mismatches are rejected. Graph recovery without its
original provenance is rejected as well.

Progress snapshots expose named native conversion and parametrization stages,
literal counts, total input-stage time and parametrization duration. Display
does not borrow a running native calculation. Input
conversion and transport limits are not claims of a hard process-RSS bound;
global preparation and original-input ownership remain separate from local
compilation residency.

The host test also exposed an existing selected-result export gap. A selected
native `KernelSet` can lack retained artifact bytes even though its native
serializer is available. Python now borrows retained bytes when present and
otherwise delegates to that serializer. The byte accessor's sole error is
absence; decoder or integrity errors are not swallowed. No Python codec or
new numerical artifact format is introduced.

Independent runtime and root reviews checked native ownership, unchanged
ordinary call order, recovery authority, configuration compatibility, status
boundaries and serializer reuse. The rebuilt embedded Symbolica/HEPKit host
passes lazy Float graph construction, callback cancellation and interruption,
raw recovery without a new solve, full complex above-threshold bubble
comparison, and selected-result export/reload after preparation-owner release.
Exact-input source identity agrees with the prior eager native call sequence.

Validation: the joined native library suite passes 678 tests, with the existing
21 ignored tests unchanged; strict workspace Clippy and formatting checks pass.
The private source-matched gates pass 14 native preparation/recovery tests,
five binding ownership tests and the embedded host test. Native and portable
stub rendering/AST checks, strict binding lint and the portable all-target
check pass. Portable metadata excludes symGCAD. These are interface and
scientific controls, not generation or sampling performance measurements.

This is a source-matched native host check, not evidence that an installed wheel
or a running notebook server has been updated. Threshold support in WASM,
represented-family inputs and the complete physical ggHH graph payload remain
separate work.
