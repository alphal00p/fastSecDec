# Historical gg → HH input identity

These are input files from the native FastSecDec example named in `origin.json`.
The manifest records the exact source commit, input SHA-256 hashes and Rust
generator source hashes. The original native selector uses Linnet connectivity
and cycles to select the six-top hexagon with a central gluon and one g/H pair
on each four-cycle.

These files are historical evidence only. The current notebooks generate a
complete native one- and two-loop catalogue after **Build diagrams**, and neither
select nor validate their current input against this archived diagram. The
standalone notebook contains no dependency on these assets.

The model and parameter card set mt = ymt = 172.5 GeV, mH = 125 GeV and zero
widths in that historical input. Current generation instead fixes only exact
structural zeros, including both incoming gluon mass shells, while all nonzero
Gram products and contributing model leaves remain runtime inputs. See the
[current workflow](../../README.md) for native `to_dots` preparation and
symbolic generation. No current notebook loads scalar products, kernels or
numerical results from this archive.
