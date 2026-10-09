# Contour Python interface metadata audit

2026-10-09. The existing FeynKit diagram/family methods forward their native
owner and keyword objects to FastSecDec at call time. No graph conversion,
generation or numerical logic belongs in these forwarders.

Their advertised signatures and stub templates had fallen behind the actual
FastSecDec binding: they omitted runtime parameters, model-parameter policy,
generation mode, subtraction strategy and contour capability, and advertised
`coefficient_expansion="physical"` instead of the backend's actual
`"full_expression"` default. The missing-backend error also named the removed
experimental feature.

[FeynKit PR 129](https://github.com/alphal00p/gammaloop/pull/129) corrects those
interface descriptions without changing the forwarding implementation. It is
stacked on citation compatibility PR 128, at head
`917b20751ea5b05c38b5b027e54f236e93d44699`, and remains draft pending the owner's
full CI readiness process. The commit and publishing account are
ValentinHirschi; GitHub confirms the formal `benruijl` reviewer request.

The actual native owner unit test passes, checking both diagram and family
methods, runtime signature defaults, forwarded object identity, backend
exceptions and the missing-module cause. Both stub keyword/default schemas also
match the FastSecDec backend's parsed Python declaration. Independent reviews by
the coordinating and runtime agents found no blocker. Formatting and whitespace
checks pass.

The first standalone run exposed a test-environment assumption: only the leaf
mock backend was installed in `sys.modules`, while the parent Symbolica
packages did not exist in that interpreter. The fixture now installs and
restores only missing parent namespaces. All production-forwarding assertions
remain in place. The corrected test passed in 0.16 seconds after its native
library build; it does not require a preinstalled Community wheel.

The current Community `9a65fbb7` host additionally needs six `Citation.url`
initializers when moving to the current Symbolica record API. An isolated JJ
change adds only the existing software/arXiv/Zenodo links, including its newer
LiteRed2 citation. All six exact edited constructor literals compile against
public Symbolica `7ec1be45`; an independent review approves the URLs. This is a
focused API gate, separate from a complete wheel build or deployment. The staged
`3aa2608` validation host contains the corresponding five-field subset.

Community's generated contour stubs remain a separate packaging gate. Their
generation must consume the frozen FastSecDec milestone and these owner
interfaces; substantive implementation stays in the native FastSecDec binding
crate. The live notebook and Community installation remain untouched.
