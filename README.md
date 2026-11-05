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
- `contracts/pirc_214_oracle_quorum_v3` — experimental feed-scoped multi-reporter quorum and median aggregation; Draft PR #1104, not yet compiled or approved for settlement.
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

## Economic settlement quality gates
- [Shared economic golden vectors](fixtures/economic-engine-vectors.json)
- [Economic vector validator](scripts/test_economic_vectors.py)
- [Soroban economic parity gate](docs/ECONOMIC_SETTLEMENT_PARITY_GATE.md)
- [Economic implementation plan](docs/ECONOMIC_ENGINE_IMPLEMENTATION_PLAN.md)
- [Economic settlement contract implementation](docs/ECONOMIC_SETTLEMENT_CONTRACT.md)
- Soroban settlement crate: `contracts/economic_settlement`
- [Contract security test matrix](docs/SECURITY_TEST_MATRIX.md)
- CI: `.github/workflows/economic-contract-validation.yml`

## Economic execution and supply invariants
- [Economic Mint, Burn, and Settlement Invariants](docs/ECONOMIC_MINT_BURN_AND_SETTLEMENT_INVARIANTS.md)
- [Economic Execution Flow](docs/ECONOMIC_EXECUTION_FLOW.md)

These documents specify implementation gates for contract-issued assets and revenue-funded settlement. They do not grant control over native Pi or authorize deployment, minting, burning, redemption or treasury operations.

## Factory, OUSD, Oracle and Cryptographic Gates
- [RAW Factory, OUSD, Oracle and Cryptographic Execution Gates](docs/RAW_FACTORY_OUSD_ORACLE_AND_CRYPTO_GATES.md)
- [Contract Capability and Evidence Matrix](docs/CONTRACT_CAPABILITY_AND_EVIDENCE_MATRIX.md)

OUSD remains a test-only token reference without a peg, reserve or redemption. Oracle and AI records require independent validation; a proof hash is not a proof verifier. RAW factory and physical evidence scaffolds are not production deployers or possession verification. Production use requires network-specific tests, security review and read-back evidence.

## Cross-system execution contract

The SmartContracts repository is the execution layer of a three-repository architecture:
- PiRC owns normative specifications, schemas, economic policy and governance.
- SmartContracts owns Soroban/Rust state transitions, authorization, storage, events and deterministic contract tests.
- Unity Sphere owns developer workflows, APIs, network adapters, PiDEX UI, RAW evidence orchestration and monitoring.

### Contract lifecycle

`SPECIFIED → TEMPLATE_APPROVED → BUILT → UNIT_TESTED → SECURITY_REVIEWED → TESTNET_DEPLOYED → READBACK_VERIFIED → REGISTERED`

Every state transition requires a commit SHA, artifact/code hash, network identity, transaction/read-back evidence and an authorized reviewer. A contract ID in a document or source file is not proof of deployment.

### RAW factory requirements

A production factory must accept a governance-approved immutable template ID and code hash, asset ID, issuer, metadata commitment, network ID, policy/schema version and idempotency key. It must authorize the deployer, upload/install the approved WASM, use deterministic salt, handle retries safely, register the child contract, and read back the deployed code hash and initialized state. Event-only factories and placeholder transaction IDs remain scaffolds.

Physical evidence must bind asset ID, device identity, certificate/key epoch, nonce, expiry, timestamp, reading and signature. Verify signatures, revocation and replay; preserve inspector/custodian identity and dispute history. A document hash, QR/NFC tag or unsigned sensor reading is not proof of title or possession.

### PiDEX fixed-point settlement

For reserves `x,y`, input `dx`, fee denominator `D`, fee `F`:
`dx_net = floor(dx * (D-F) / D)`
`dy = floor(dx_net * y / (x + dx_net))`

Use integer atomic units and checked arithmetic. Validate zero reserves, overflow, token authorization, minimum output, slippage, fee accounting, pause state, reserve conservation, deterministic rounding and dust. CEX order-book VWAP and DEX pool spot price are distinct observations; neither alone proves executable liquidity.

### OUSD reserve and liability boundary

OUSD may be an internal unit of account, a basket-backed utility credit or a legally redeemable USD liability. These are different products. For a redeemable instrument:
`ReserveCoverage = eligible_reserves_after_haircuts / outstanding_redeemable_liabilities`

Minting requires issuer authority, segregated eligible reserves, custody evidence, reconciliation, independent attestations, redemption terms, liquidity and emergency controls. The current test token is not a peg, reserve proof or redemption implementation. Keep production mint/burn disabled until all gates pass.

### Oracle and cryptographic constraints

A signed observation proves key control, not truth. Require typed units, source identity, freshness, monotonic sequence, replay protection, independent-source quorum, deviation policy and deterministic aggregation. Chainlink feeds are chain-specific; an EVM AggregatorV3 contract is not automatically callable from Soroban. Any relayer/adapter requires explicit trust, signature and read-back design.

Post-quantum readiness requires actual algorithm implementation and target-runtime measurements. Evaluate FIPS 203 ML-KEM, FIPS 204 ML-DSA and FIPS 205 SLH-DSA; a hash or branch label is not PQ protection. Agents can draft and test code but cannot self-approve, sign arbitrary production transactions or bypass protected review.

### Conformance invariants

- Atomic-unit conservation across mint, burn, transfer, swap and settlement.
- No implicit conversion between internal reference, market quote, collateral mark and redeemable liability.
- No stale or below-quorum oracle result can authorize settlement.
- Idempotent requests cannot double-deploy, double-mint or double-settle.
- All privileged actions are role-checked and auditable.
- Failures, pauses and rejected transitions emit typed reasons without mutating protected state.
- Every deployment is verified by transaction, network, code hash and state read-back.
