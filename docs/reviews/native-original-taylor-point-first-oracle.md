# Independent original Taylor point-first oracle

The ignored Rust source `output/probes/point_first_original_taylor.rs` is frozen
at SHA256 `b408c3ddf190d9accb36ee67df66321c5e2ca05b48a5f04ce2778213c529223d`.
It is source-reviewed and compiled, but has not been executed. It adapts the previously
reviewed native point-first oracle to the original captured Taylor expression,
without consuming any named-coefficient output or an unfinished IBP expression.

Inputs are `production-alias-capture-81/capture` and the completed Taylor identity
record `native-ibp-prepare-1/prepared/progress.json`. That record proved exact
identity of 872 Taylor pieces before the later IBP stage timed out. The oracle
requires that completed identity, binds every original expression/mapped/input
file to its archived digest, and does not reinterpret the record's incomplete
IBP status. It checks representative 80, source chart 119, multiplicity four,
nine coordinates and the original geometry/density identity. Multiplicity is
recorded but not applied.

Each separate process binds one of the three prescribed exact rational points
into the original epsilon-dependent Atom before requesting native series.
Relative width starts at one and increases only by the checked deficit from the
native remainder, with at most four calls. The final native absolute order must
exceed zero. The nonvacuous lowest integer order is inferred from native output;
every order through zero is retained, and only gaps inside the proven native
bound become exact zeros. Every coefficient is exported as a native Atom and
checked at 512 and 1024 bits with the existing scaled `1e-70` agreement test.
There is no interpolation, Laurent arithmetic implementation or binary64
rounding of the prescribed input points.

The shell launcher `output/probes/run_point_first_original_taylor.sh`, SHA256
`905cf9b9fbc2ab0a9d88f13231929d2273963e46df9def8e60c292b547d232b2`,
runs one fresh directory per point, at 180 seconds
plus five seconds of termination grace, CPU8 and a 30 GiB virtual-address cap.
The cap is not an RSS promise. The Rust process-group timer owns the deadline;
each failure and partial progress record remains retained, and there is no
automatic extension. A frozen executable, exact linked native dependency and
compiler/source manifests are required before execution. Large input digests
are streamed to avoid duplicating the 137-MB expression solely for hashing.

Root, PF and HEP reviewed the Rust source without a blocker; root and HEP also
accepted the shell launcher. Build 1 failed before linking because native
`parse!` requires the compile-time `CARGO_CRATE_NAME` normally provided by Cargo.
Build 2 sets that variable explicitly, preserves the same Rust source, and
compiled successfully using Rust 1.98.1 and the exact normal native rlib used by
PF's actual named probe. Both attempts and their logs remain retained in
`output/diagnostics/original-taylor-oracle-build-{1,2}`. Build 2 freezes the
compiler, source, native source archives, three direct rlibs, linker-search rlib
inventory, binary and their pre/post hashes; no native instance was initialized.
HEP independently verified build 2's frozen manifest and source/native/binary
binding in `independent-preflight.txt`. The executable SHA256 is
`afeb1b18ad1e84046924952f6811d5e2282515cab608316607ba71de6dce1a5d`;
the actual fixed Symbolica rlib SHA256 is
`01601b6df1747703693fdbf78818f9bf2bb74f4e4b1a888a5971b678cc60b61a`.
The initial runtime handoff was conditional. Root
required the bounded named-generation stage first, then these independent
oracles if compact composition succeeds. Symbolica processes remain serialized;
the current guarded external reference may overlap only after its unguarded
generation child has exited and been reaped. All such overlap is diagnostic,
with no matched performance claim.

The initial named-generation attempt reached its unchanged 180-second bound
without a complete coefficient vector. Its process was reaped, and the oracle
points remained unexecuted at that stage.

## First prescribed point after the interleaved program gate

A later independently reviewed interleaved candidate completed generation and
native program construction for all seven orders -6 through zero. After its
outcome audit, the coordinator authorized the unchanged frozen oracle sources
and build, starting with point zero. This oracle still reads only the original
captured Taylor expression and its original identity proof, never the candidate
coefficients or program.

`output/diagnostics/original-taylor-point-0-attempt-1` reached its fixed
180-second deadline and exited on SIGINT after 180.167230 seconds. Peak resident
memory was 1,748,504 KiB, separately from the 30-GiB virtual-address cap. Its
process was reaped and every frozen input hash still passes.

Literal exact-coordinate binding took 9.716036 seconds and left a
22,686,159-byte native Atom. The first native relative-width-one request
completed in 18.492427 seconds, returning leading order -6 and absolute
remainder -5. The second native request, width seven to cover through zero,
had not returned when the deadline expired. Only the bound input Atom and
partial progress exist; there is no complete coefficient vector, final
remainder or 512/1024-bit oracle result.

Points one and two and the dependent comparison reader were not launched.
This failed oracle remains retained without a silent deadline extension or
coefficient-correctness claim for the candidate program. Any next attempt needs
a separately reviewed plan and fresh evidence directory.

## Separately bounded unchanged point-zero retry

The coordinator subsequently authorized a fresh point-zero attempt with a
600-second limit and the same five-second grace, CPU8 and 30-GiB address-space
cap. `output/probes/run_point_first_original_taylor_600.sh`, SHA256
`2434d7655e2bfb14bb77d34299aa85d37ed98d2102bd1b0348e452612f10deaf`,
differs from the old shell only in the declared timeout and timer argument.
The executable, original expression, exact point, native series requests,
maximum four calls and all precision/bound checks are unchanged. HEP verified
the two-line diff and all 22 frozen build hashes before launch.

The fresh directory is
`output/diagnostics/original-taylor-point-0-attempt-2`. Its result is pending;
the earlier 180-second failure remains immutable. The remaining prescribed
points may run only after a complete independently reviewed point-zero result,
each with its own unchanged 600-second bound. Any failure stops the dependent
chain, with no further automatic increase or algorithm change.
