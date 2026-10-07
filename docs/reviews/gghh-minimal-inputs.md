# Minimal ggHH inputs: independent native review

Reviewed on 2026-10-07. Accepted: the cleanup changes file organization and
point-card notation, with no change to the native graphs, physical model,
parameter card or generation cards. This review does not validate a three-loop
integral.

## Native topology and color

A fresh Rust probe imports each final `graph.dot` with `Model::from_json`,
`ParameterCard::from_json`, `Model::apply_parameter_card` and strict
`FeynmanDiagram::from_dot`. It checks unchanged model fingerprints,
`FeynmanDiagram::validate`, native loop-basis validation and equality of the
complete native JSON payload after DOT export/reimport. Topology checks use
Linnet's `is_connected`, `cyclotomatic_number`, `all_cycles_of` and native
edge/node iterators, without another graph type or DOT parser.

| Input | Native physical diagram ID | Internal topology |
| --- | --- | --- |
| Double box, D05 | `a4655aa3f84bd42b000eaa16b9c54104` | 6 vertices, 7 internal edges, 2 loops; one six-top cycle and one gluon |
| Original triple box, D020 | `d9ae720f395bceaadd28c78a55a7ecaf` | 8 vertices, 10 internal edges, 3 loops; two disjoint four-top cycles and two connecting gluons |
| Triple box `_bis`, D068 | `cced04ecc95ba7488574be8dc7bb689d` | 8 vertices, 10 internal edges, 3 loops; one eight-top outer cycle and two gluon rungs |

D020's top cycles are `{4,5,9,11}` and `{6,7,8,12}`. Each gluon, edges 10 and
13, connects these cycles. The incoming gluons attach to the first cycle and
the two outgoing Higgs bosons to the second. D068's top cycle is
`{4,5,6,7,8,9,10,13}` and its rungs are 11 and 12. Its native circuit lengths
are `[4,4,4,6,6,8]`. The three four-edge faces contain respectively the two
incoming gluons, no external leg, and the two Higgs legs. Each top edge belongs
to one such face and each rung to two: the adjacent-box ladder is uncrossed.

The original graph's **two-gluon color-singlet exchange** follows from more
than its topology. Its native model assigns color 1 to the Higgs, color 3 to
the top, `Identity(1,2)` to the `ttH` vertex and `T(3,2,1)` to `ttg`.
The top loop attached to the two Higgs bosons therefore supplies
`tr(T^a T^b) = T_F delta_ab`. A separate bounded native Idenso proof constructs
the two indexed generators with `CS.t_pattern` and reduces them using
`SymbolicTensor::simplify_algebra` with the native SU(3) invariant conversion.
It yields exactly `delta_ab/2`, with `T_F=1/2`.

This singlet tensor concerns the two **exchanged** gluons. It is distinct from
the original graph's stored, unnormalized incoming-gluon color-delta projector.
The `_bis` and double-box projectors contain only the two Lorentz polarizations,
as their incoming color projection was already reduced during export. No extra
color multiplier, average or single-gluon interpretation is introduced.

## Minimal files and unchanged inputs

Each run directory now contains exactly `README.md`, `graph.dot`, `model.json`,
`parameters.json`, `run.toml` and `point.toml`. Independent byte comparisons
against the pre-cleanup commit pass for all three graphs, models, parameter
cards and run cards. Thus both generation-time incoming-gluon mass shells,
all runtime symbols, and the existing mode/contraction choices are preserved:
double-box symbolic; original triple-box numerical dual/dots; `_bis`
numerical dual/minimal.

Each point card has 19 finite TOML floats: 13 Gram products and six model
inputs. The cleanup author's native conversion probe compares every old
expression evaluated by `Atom::evaluate::<f64>` with the new float loaded
through `Rational::try_from(f64)` and native evaluation. Its source and log
were reviewed; all **57 bit patterns match**. In particular,
`eps1eps2 = -0.9999999999999998` is retained, not idealized to -1. The removed
`p0p0`/`p1p1` runtime keys remain absent; their exact zeros belong in generation.

The exporter's required D05 source moves unchanged to
`crates/fastsecdec/tests/fixtures/gghh-double-box-source.dot`, SHA-256
`1bd26a5a2358a8f14822ee9c1f639e52a41fa36dde90c5e12d71c9b215f87ce7`.
The exporter continues to use native diagram membership, strict native payload
checks, parameter-card application and color reduction. Its point formatter
uses native real evaluation, rejects nonfinite/complex values, and writes
round-tripping Rust float literals. Ordinary export now emits only the five
machine input files; the checked-in README supplies the sixth file.

Live README/SVG references and the exporter's source include were updated.
Notebook code and tests do not read the removed CLI diagnostic files. The
notebook's separate `fixtures/gghh/origin.json` is historical hash provenance;
older review references remain records of earlier checks rather than current
runtime dependencies.

## Executed checks and limits

- Independent native topology, strict import/card/routing/DOT roundtrip and
  isolated color-trace probe: passed for all three current graphs.
- Existing cold CLI native on-shell card regression: **1 passed** in 0.07 s;
  it imports all three inputs without three-loop parameterization.
- Exporter owner checks, independently reviewed: **2 tests passed**, strict
  example Clippy passed, and a complete native two-loop export passed. It
  selected D05 among 192 generated diagrams, emitted exactly five files, and
  reproduced the checked-in graph/model/card bytes and all 19 point f64 values.
- Independent directory/byte/finite-float checks passed. Root reports workspace
  formatting and diff whitespace checks passed.

Ignored evidence is under `output/gghh-input-cleanup/`: `topology_review.rs`,
`topology-native.log`, `native-model-color.json`, `independent-files.json`,
`cli-native-import.log`, `floatify.rs`, `float-bits.log`, and
`exporter-{tests,clippy,run}.log`.

No three-loop tensor numerator was contracted, parameterized, sector-decomposed,
compiled or integrated for this cleanup. The small color trace validates the
singlet interpretation only. D020's original enumeration history remains
unknown; removing diagnostic copies does not establish one.
