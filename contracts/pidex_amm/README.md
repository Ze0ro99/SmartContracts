# PiDEX AMM — Canonical Contract Crate

This crate is the canonical SmartContracts-owned source for the PiDEX Soroban constant-product AMM. The Unity Sphere repository may retain a historical mirror for application packaging, but new contract changes must be made here first and consumed through a pinned commit and generated contract specification.

## Scope

- Constant-product AMM reference implementation.
- Ratio-matched liquidity deposits and proportional withdrawals.
- Checked arithmetic and invariant checks.
- Configurable admin pause.
- Minimum-output slippage protection.
- Optional verified-commitment gate.

The commitment gate is transparent and does not hide assets, amounts, balances, addresses or events. The optional verifier is not a privacy layer or a proof of decentralized BN254 verification.

## Build and test

From the SmartContracts repository root:

```bash
cargo test --locked --manifest-path contracts/pidex_amm/Cargo.toml
cargo build --locked --manifest-path contracts/pidex_amm/Cargo.toml --target wasm32-unknown-unknown --release
sha256sum target/wasm32-unknown-unknown/release/pidex_amm.wasm
```

## Testnet deployment gates

Do not deploy until the PiRC Testnet1 profile has verified the exact network passphrase and protocol version. Record the source commit, lockfile, toolchain, WASM hash, deployment transaction, contract ID, code hash readback, token IDs, issuer/admin state, and functional test evidence.

The contract is not deployed by this PR. A build or unit test does not prove Testnet deployment, liquidity, issuer authorization, wallet signing, or live PiDEX trading.
