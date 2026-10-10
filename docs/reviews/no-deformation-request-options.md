# Native threshold request settings and exact density bindings

`threshold::ThresholdDecompositionOptions` configures geometry preparation;
it does not enable an ordinary generation request or claim a complete threshold
integrator. The public `ThresholdStrategy` retains `auto`, `gcad_first` and
`sector_first`. Automatic routing starts from compact projective/native input
unless original `source_sectors` require the existing unsubtracted source charts.
An explicit contradictory strategy is rejected. Source and threshold-cell IDs
remain distinct; raw threshold-cell selection is scoped to one geometry problem.
No list is silently applied to every source in a multi-problem decomposition.

Request admission rejects contour recipes and numerical-dual endpoint reduction.
Full-input geometry delegates to the existing native unit-cube or affine
projective constructors; source-first preparation requires its own admitted
source-chart owner. Positive-orthant compactification remains explicit unfinished
work. Existing symGCAD solver/limit types and the resolution checker limits are
reused. No second configuration copy of every solver setting is introduced.

The request still owns the complete original density and exact kinematics.
`GcadKinematics::specialize_exact` uses Symbolica simultaneous replacement for
both geometric polynomials and arbitrary density Atoms. Causal phase admission
now specializes the full exponent before inspecting its affine regulator form,
zero value, or causal semantics. It retains the unspecialized original exponent
in the source identity. This fixes a rejected valid input such as a declared
power `p + c*epsilon` with exact bindings `p=-1, c=2`; its lower-lip phase keeps
the complete `2*epsilon` contribution. Runtime-parameter-dependent phase powers
remain unsupported until tied to a separately admitted parameter fiber.

The native source/API reuse was already established by the projective and phase
probes. The added executable options probe passes three groups: source routing
and incompatible recipes; exact fixed/parametric native geometry with projective
normalization; and owner-limit/unsupported-preparation refusal. A first probe
fixture used a nonhomogeneous projective factor and was correctly rejected by
the native input constructor; the corrected fixture has balanced homogeneous
F/U powers. These are native API controls, not CLI or integration acceptance.

The runtime reviewer independently accepted the options and exact-binding source
diffs. The maintained phase regression also checks an exactly zero bound generic
exponent, complex numerator binding and unchanged source identity. Integrated
Cargo verification passes all **44** threshold tests, including the three
options controls and five phase controls; strict native Clippy passes. See
`target/no-deformation-native-request-contact-gates.log`. Formatting passes
after removing an initial blank line in the options test file; its separate
log is `target/no-deformation-native-request-contact-fmt.log`.

CLI/TOML activation, proof-bearing complete generation, uniform parameter
admission, saved threshold recipes and HEPKit threshold entry points remain
later integration work. Default native and portable generation are unchanged.
