# Implementation Gates and Security Matrix

## Contract status
The repository contains a Rust/Soroban workspace with many numbered contract crates. This document defines the acceptance gates required before any crate is described as production-ready.

## Required gates per contract
1. Specification: pinned PiRC proposal ID, revision, normative behavior and explicit non-goals.
2. Interface: typed public functions, events, storage schema, error codes and upgrade policy.
3. Authorization: one-time initialization, role separation, least privilege, multisig for critical authority, explicit caller authentication.
4. Arithmetic: checked add/subtract/multiply/divide, positive amount checks, decimals/units, deterministic rounding and supply bounds.
5. State machine: explicit transitions, replay protection, expiry and idempotency.
6. Asset safety: issuer/contract identity, network domain, trust state, metadata integrity and freeze/pause policy.
7. Tests: unit, integration, property/invariant, fuzz and adversarial tests.
8. Build: pinned toolchain, lockfile, reproducible WASM and artifact SHA-256.
9. Deployment: network passphrase, RPC, contract ID, deploy tx, initialization tx, WASM hash and live invocation.
10. Security: independent review and remediation tied to exact artifact hash.

## Findings from inspected default-branch examples

### `contracts/pirc_103/src/lib.rs`
- `initialize` can overwrite the Admin value; initialization is not one-time.
- `mint` does not reject zero/negative amounts and uses unchecked `balance + amount`.
- `transfer` does not reject zero/negative amounts and uses unchecked balance arithmetic.
- Subscription writes a plan string but has no expiry, payment, allowance or entitlement verification.
- No events are emitted for mint, transfer or subscription changes.

### `contracts/pirc_260/src/lib.rs`
- `initialize` can overwrite Admin.
- `register_asset` does not enforce registry roles, metadata schema, uniqueness policy beyond key overwrite, or lifecycle status.
- `transfer_asset` unwraps an absent owner, which can trap on unknown asset IDs.
- No explicit pause, events, asset provenance or metadata integrity proof is present in the inspected file.

These are source-level observations, not a full audit of every branch or crate.

## Soroban-specific compatibility
- Pin SDK version by crate and target protocol; do not infer protocol support from a comment.
- EVM/Solidity contracts and EVM Chainlink consumers do not execute natively as Soroban contracts.
- Cross-network calls require a documented relayer/adapter, authenticated messages, nonce, expiry, replay protection and independent validation.
- Storage durability, TTL extension, instance/persistent storage, resource budgets and host-function availability must be tested on the target network.

## PiDEX AMM acceptance criteria
- Validate reserve non-zero and checked arithmetic for every state update.
- Prove conservation and constant-product properties with property tests.
- Enforce min-out, max price impact, deadlines and circuit-breaker policy.
- Verify token transfers and balance deltas rather than assuming transfer success equals intended accounting.
- Prevent share inflation/dilution, rounding extraction and unauthorized liquidity withdrawals.
- Emit deterministic deposit, withdrawal, swap and pause events.
- Test reentrancy/callback assumptions and malicious token behavior.
- Do not enable optional BN254/ZK mode until verifier address, code hash and proof tests are verified.

## OUSD / treasury acceptance
A TEST_ONLY token can be used for integration tests. A redeemable USD-linked asset requires an identified issuer, enforceable redemption terms, custody and reserve evidence, independent attestations, insolvency treatment, sanctions/compliance controls, oracle/depeg policy, mint/burn controls and audited contracts. No reserve or redemption claim may be inferred from a symbol or token contract.

## CI policy
Every PR must run formatting, lint, tests, clippy, dependency audit, secret scanning, WASM build, ABI compatibility and testnet integration where configured. Production release requires an immutable release manifest containing spec revision, source commit, artifact hashes, SDK/toolchain versions, test results, deployment identifiers and reviewer approvals.
