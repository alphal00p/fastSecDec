`numerica-f6ecdac.json` was generated using the original Numerica QMC owner at
`f6ecdac8237a30adfcd1be5944a95c5160e474ce`, before changing production dependency
resolution. It records four seed/stream combinations, including zero and the
maximum `u64`. Each uses the supplied seven-point generator `[1, 3]`, three
shifts, and five-point packages. Alternating packages are complete in the saved
accumulator; the integrand is `[x0*x1, -2*x0*x1]`.

The fixture contains exact old-owner JSON and bincode/serde bytes, every point's
floating-point bits, and the completed vector estimate/covariance. The maintained
test regenerates plans through the public Numerica RNG API, loads both old
encodings and completes the missing packages, checking byte and result equality.
No Numerica fork is needed to run this regression.
