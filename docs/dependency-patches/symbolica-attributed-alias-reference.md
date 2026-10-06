# Attributed references retain registered aliases

Status (2026-10-06): superseded by public Symbolica `community` commit
`473b4b8dbc2f9bff8658a047196ba0877238bf9e`. Its existing-symbol reuse also retains
callbacks and user data. The local patch and bootstrap application have been
removed. The account below records the original reproduction and validation.

## Historical rationale and validation

Fresh-process loading of a Gamma-series artifact failed on canonical
`symbolica::{}::γ`. Native state initializers already register that constant
with the alias `symbolica::euler_gamma`. The attributed-symbol parser supplied
an empty alias list to strict symbol registration, which rejected the otherwise
identical reference. Initializing special functions earlier does not repair
this metadata mismatch.

The adjacent parser patch retains existing aliases when interpreting an
attributed reference. The syntax specifies attributes and tags, not aliases;
those explicitly supplied attributes and tags are still validated by the
existing registration code. No alias is added to a previously unknown symbol.
The native regression covers Euler gamma, a positive aliased user symbol, and
rejection of conflicting explicit attributes.

FastSecDec also has a fresh-process artifact regression: a writer creates a
complex Gamma expansion through epsilon^4, exits, and a separate reader loads
the artifact and evaluates it on a numeric worker. This exercises Euler gamma,
higher polygamma constants, native callback registration, and complex
serialization. Validation passed: the focused native
`canonical_attributed_references_preserve_aliases` test, the two process-test
entries (writer and reader run in separate child processes), all four complex
kernel tests, and all 16 generation tests. No upstream push has been performed.
