# Smart Contract Architecture and Implementation Gates

## Scope
Executable contract code implements behavior specified by a pinned PiRC revision. Token names do not imply legal or economic guarantees.

## Domains
Asset registry and metadata verification; payment/settlement state machine; treasury accounting and role-controlled actions; OUSD test-token and reserve-verification interfaces only after issuer/redemption design; PiDEX pool/router interfaces; oracle receiver and validated data records; governance, multisig and timelock controls; RWA and commodity/service registry interfaces.

## Required properties
1. Explicit authorization and role separation.
2. Checked arithmetic and bounded inputs.
3. Replay protection and network/domain separation.
4. Safe state transitions and reentrancy protections where applicable.
5. Restricted pause/emergency authority.
6. Events for material state transitions.
7. Deterministic unit, integration and invariant tests.
8. Storage-layout and upgrade compatibility review.
9. No privileged mint, burn, freeze or upgrade without documented governance.
10. No production deployment without independent security review.

## Soroban
Use Soroban-native interfaces and an SDK supported by the target network/toolchain. Solidity/EVM bytecode and EVM Chainlink contracts are not assumed executable on Soroban. Cross-environment adapters require explicit services or verified contracts with signed messages, expiry, nonce and replay protection.

## OUSD gate
A test token must be labeled TEST_ONLY and make no USD-backing claim. A redeemable stable asset requires issuer specification, reserve evidence, custody/legal review, redemption tests and oracle risk policy.

## CI gates
Formatting, linting, unit/integration tests, static analysis, dependency audit, secret scanning, reproducible WASM build, interface compatibility checks, testnet deployment simulation and post-deployment verification.
