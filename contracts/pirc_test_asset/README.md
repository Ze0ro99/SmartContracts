# PiRC Test Asset Token

A configurable, permissioned Soroban token for Pi Testnet integration tests.

## Explicit scope

This is a test-only asset contract. It does not represent Pi, OUSD backing, a USD peg, reserves, redemption, or an official Pi Network asset. Every deployed instance must be labelled TEST ASSET in the registry and UI.

## Features

- Configurable name, symbol, decimals (0–18) and hard maximum supply.
- Admin-only minting with checked supply arithmetic.
- Authenticated transfer and burn, including allowance-backed delegated burn.
- Allowance approval with ledger expiration and delegated transfer.
- Checked balance/supply arithmetic and transfer/mint/burn/approval events.

## Build and test

From the SmartContracts repository root:

```bash
cargo test --locked --manifest-path contracts/pirc_test_asset/Cargo.toml
cargo build --locked --manifest-path contracts/pirc_test_asset/Cargo.toml --target wasm32-unknown-unknown --release
sha256sum target/wasm32-unknown-unknown/release/pirc_test_asset.wasm
```

## Deployment

Deploy only after the Pi Testnet1 network identity and protocol checks pass. Use a dedicated Testnet-only signer, initialize each instance with unique layer metadata and a bounded supply, and register the returned contract ID and code hash in the PiRC-207 registry.

Do not reuse a testnet token contract ID on Mainnet. Do not treat a deployed test asset as backed, redeemable or tradable until a separate verified AMM, wallet-signing and settlement path exists.
