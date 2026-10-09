# Recipe-addressable artifact foundation

2026-10-09. Implementation review of the storage boundary proposed in
[the dynamic artifact design](contour-dynamic-artifacts.md).

`ProgramArchiveWriter` declares a complete recipe set and appends existing
native worker records without decoding them. Its flat indexed-v2 footer has
independent coefficient layouts, physical/runtime schemas, source-chart maps,
exact records and numerical sector IDs for each recipe. A missing requested
recipe or failed append prevents finalization. Publication and process
scheduling remain caller-owned.

`ProgramArchiveReader` reads only the directory. `select` returns a borrowed
recipe view before any native payload is decoded or JIT restored. Global byte
ranges, overflow, overlap and full physical coverage are checked independently
of payload-digest policy; source coverage and output projections are checked
within each recipe. Original native argument order is preserved separately
from its physical/deformation classification.

The existing v1 and new v2 paths share record decoding, output assembly,
layout checks and bounded copying. Selected rich chart IDs, contour-check IDs
and subtraction faces remain recipe-local. Independent review identified two
inherited admission gaps, now fixed in those helpers: missing original chart
IDs fail during directory admission, and invalid local chart/check indices
return `KernelError` instead of indexing panics.

A resident selected owner retains a portable single-recipe archive containing
only its original selected records. This preserves source and recipe identity
through `from_bytes(to_bytes())`, with no regeneration or optimization. Native
`KernelSet::from_bytes` rejects an ambiguous multi-recipe archive and directs
the caller to explicit selection. Sector-at-a-time loading retains only its
usual local native payload. Legacy indexed-v1/v9 files remain usable and keep
their existing identities; no source identity is invented for legacy data.

Eight new tests cover heterogeneous recipe layouts, runtime schemas and sector
counts; exact-only offsets and per-record pilots; completion-order identities;
single-recipe roundtrips; legacy adaptation; corrupt ranges, schemas and
payloads; failed/incomplete publication; and invalid local metadata indices.
A reader that records byte ranges proves that even deliberately corrupted
unselected records are never read, so they cannot be decoded or JIT restored.

Validation with the tested Symbolica ball-domain and SymJIT callback fixes:

- Indexed artifact filter: **11 passed** (8 new, 3 existing).
- Complete native artifact filter: **33 passed**.
- Contour filter with the new envelope foundation: **35 passed**, run by the
  generation agent.
- Strict core Clippy over all targets: passed, run by the generation agent.

This remains an opt-in storage foundation. Ordinary CLI generation still writes
the existing common v1 format. Only implemented `undeformed-v1` and `fixed-v1`
recipes are admitted. Their native-v9 schemas use the existing compatibility
classification; dynamic admission must use an explicit native-v10 descriptor,
not recognition of new parameter names. Native program-set generation, dynamic
records, CLI/Python recipe selection and the full execution matrix remain
separate delivery gates.
