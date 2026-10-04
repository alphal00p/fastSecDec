# FastSecDec

Native Rust sector decomposition and numerical integration for Feynman integrals,
built around HEPKit, Linnet, Symbolica, and Numerica/Havana.

The first phase is under active implementation. The complete design, scientific
scope, milestone gates, and original requirements are in
[FIRST_PHASE_PLAN.md](FIRST_PHASE_PLAN.md). The standalone CLI is being developed
alongside the library; the future Python bridge belongs to HEPKit.

- [Development environment and dependency setup](docs/DEVELOPMENT.md)
- [Reference regression traceability](docs/REGRESSION_MATRIX.md)
- [Correctness and performance comparison protocol](docs/BENCHMARK_PROTOCOL.md)

The workspace separates the public physics library (`fastsecdec`), exact sector
geometry (`fastsecdec-sectors`), and command-line orchestration (`fastsecdec-cli`).
The QMC library extension lives on a separate branch of Numerica. Reference
checkouts and generated artifacts are deliberately excluded from this repository.
