# OUSD Test Token Contract

This crate is a test-only Soroban token reference for exercising the PiRC asset registry and PiDEX UI. It does not create a USD peg, reserve claim, redemption right or official Pi Network asset.

## Interface

- `initialize(admin)`
- `mint(admin, to, amount)` — admin-only, fixed supply cap
- `transfer(from, to, amount)`
- `approve(from, spender, amount, expiration_ledger)`
- `allowance(from, spender)`
- `transfer_from(spender, from, to, amount)`
- `burn(from, amount)`
- `burn_from(spender, from, amount)`
- `balance(id)`, `total_supply()`, `decimals()`, `name()`, `symbol()`, `admin()`

Decimals are fixed at 7 in this reference contract. The maximum supply is fixed in atomic units. Both are software parameters, not economic guarantees.

## Build and test

```bash
cargo test --manifest-path contracts/ousd_test_token/Cargo.toml
cargo build --manifest-path contracts/ousd_test_token/Cargo.toml --target wasm32-unknown-unknown --release
```

## Deployment boundary

No contract ID is published because this crate is not deployed. Do not use a production admin key. Deployment is permitted only on an explicitly selected test network after the build, interface, authorization and readback checks pass. Minting is admin-controlled. No redemption, reserves, price oracle or USD parity mechanism is implemented.
