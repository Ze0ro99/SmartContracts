# PiRC Smart Contracts

Soroban contract workspace for PiRC proposal implementations and reference modules.

## Repository role
This repository owns executable Soroban contract code, public interfaces, storage schemas, contract-level tests and reproducible build configuration. Normative requirements remain in PiRC; application/API integration remains in Unity Sphere.

## Current audit status
The default branch contains 106 Rust source files but only four distinct Rust file contents by Git blob SHA. Large groups of numbered contract crates reuse generic identity or asset templates. These are scaffolds, not proof that every numbered standard is independently implemented.

See [Contract Coverage and Security Audit](docs/CONTRACT_COVERAGE_AND_SECURITY_AUDIT.md).

## Integration branch reference modules
- `contracts/pirc_207_registry_v3` — admin-controlled asset metadata registry.
- `contracts/pirc_214_oracle_v2` — authenticated reporter observations with monotonic sequence and ledger timestamp.
- `contracts/pirc_225_por_v2` — auditor-controlled reserve/liability attestations.

These reference modules require compilation, test execution, security review and network-specific verification before use. They are not deployed.

## Standards coverage and new references

- [Cross-repository standards coverage matrix](docs/PIRC_STANDARDS_COVERAGE_MATRIX.md)
- [Contract coverage and security audit](docs/CONTRACT_COVERAGE_AND_SECURITY_AUDIT.md)
- PiRC-207 Registry V3 reference crate: `contracts/pirc_207_registry_v3`
- PiRC-214 Oracle V2 reference crate: `contracts/pirc_214_oracle_v2`
- PiRC-225 Proof-of-Reserves V2 reference crate: `contracts/pirc_225_por_v2`
- PiRC-237 AI Oracle reference crate: `contracts/pirc_237_ai_oracle`
- OUSD test-only token reference crate: `contracts/ousd_test_token`

These modules are independent reference crates. They require CI compilation, test review and security review; the AI proof hash is a commitment only, not ZK verification.

## Engineering requirements
Pin Soroban SDK and toolchain versions; enforce explicit authorization; prevent duplicate initialization; use checked arithmetic and documented atomic units; test success, failure, boundary and adversarial paths; emit lifecycle events; and require independent review before production deployment.

## Economic settlement parity gate

- Shared arithmetic vectors: [`fixtures/economic-engine-vectors.json`](fixtures/economic-engine-vectors.json)
- Contract parity and release gates: [`docs/ECONOMIC_SETTLEMENT_PARITY_GATE.md`](docs/ECONOMIC_SETTLEMENT_PARITY_GATE.md)

These artifacts define expected behavior only. Soroban parity is not established until contract tests consume the fixture and pass in CI.
