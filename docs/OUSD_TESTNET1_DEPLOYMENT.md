# OUSD Testnet 1 deployment runbook

This runbook deploys only the OUSD test token and reserve controller to Pi Testnet 1. It does not deploy to Pi Mainnet and does not establish USD backing, a peg, or a redemption guarantee.

## Network configuration

Pi's published Testnet RPC endpoint is:

```text
https://rpc.testnet.minepi.com
```

Network passphrase:

```text
Pi Testnet
```

First perform a read-only health check:

```bash
curl -fsS https://rpc.testnet.minepi.com \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

Do not substitute the Horizon REST URL `https://api.testnet.minepi.com` for the Soroban RPC endpoint.

## Local prerequisites

- Rust toolchain and Cargo.
- `wasm32-unknown-unknown` target (this workspace uses Soroban SDK 22.0.11).
- Stellar CLI installed locally.
- A funded Pi Testnet account imported into the local Stellar CLI key store.

Never paste a secret seed or private key into ChatGPT, GitHub issues, repository files, browser JavaScript, or Supabase. Keep the signer in the local Stellar CLI key store.

Check the account alias and public address:

```bash
stellar keys address <LOCAL_KEY_ALIAS>
```

The administrator, issuer, reserve verifier and payout verifier must be four distinct public addresses. The deployment signer should be the administrator or an account authorized to sign the administrator's invocations.

## Build the exact workspace contracts

From the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo fmt --all -- --check
cargo test --locked -p ousd-test-token
cargo test --locked -p ousd-reserve-controller
cargo build --locked --release --target wasm32-unknown-unknown -p ousd-test-token
cargo build --locked --release --target wasm32-unknown-unknown -p ousd-reserve-controller
```

Expected artifacts:

- `target/wasm32-unknown-unknown/release/ousd_test_token.wasm`
- `target/wasm32-unknown-unknown/release/ousd_reserve_controller.wasm`

Record each artifact's SHA-256 locally. The on-chain WASM hash must also be read back after deployment; a local hash alone is not proof of deployment.

## Deployment sequence

Use the local key alias and the Pi Testnet RPC URL/passphrase for every command. The CLI supports `--rpc-url` and `--network-passphrase`; do not use its default Stellar Testnet network for Pi Testnet.

1. Deploy `ousd_test_token.wasm`; record the returned token contract ID.
2. Deploy `ousd_reserve_controller.wasm`; record the returned controller contract ID.
3. Read each contract's generated invocation help/spec before submitting initialization calls.
4. Initialize the token with the administrator address.
5. Allowlist the controller as the token minter using `set_minter(admin, controller, true)`.
6. Initialize the controller with the administrator, issuer, reserve verifier and payout verifier addresses.
7. Configure the token contract once using `configure_token(admin, token_contract_id)`.
8. Read back the token admin, minter authorization, controller roles, configured token ID, token supply and contract code hashes.

Example deploy shape (replace only the local alias and artifact path; keep the Pi Testnet endpoint and passphrase):

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/ousd_test_token.wasm \
  --source-account <LOCAL_KEY_ALIAS> \
  --rpc-url https://rpc.testnet.minepi.com \
  --network-passphrase "Pi Testnet"
```

Repeat for the controller WASM. The returned contract IDs must be saved in a local deployment record and independently read back before updating the private Supabase registry.

## First reserve and lifecycle test

Do not attest a reserve from an arbitrary wallet balance or from an LP share balance alone. A reserve verifier must first validate the source and evidence off-chain, then submit the reserve attestation on Testnet.

After a verified reserve record exists:

1. Issue a small test liability.
2. Confirm the beneficiary receives the exact OUSD atomic amount.
3. Approve the controller for that exact amount.
4. Request redemption and confirm the tokens move into controller escrow.
5. Have the payout verifier attest an exact-amount external test settlement.
6. Confirm settlement; the controller burns the escrowed tokens atomically.
7. Read back total supply, balances, liability state, reserve allocation and emitted events.

For 7-decimal OUSD, the controller maps USD micros to token atomic units with checked multiplication by 10. The test token has a fixed supply cap of 1,000,000,000 OUSD.

## Registry and dashboard update

Only after both contracts are deployed and their IDs and code hashes are read back should the private row in `private.ousd_contract_registry` be updated to `deployed_unverified`. Set `active_testnet` only after readback, role verification, token supply checks and event reconciliation succeed.

Do not mark reserve rows `verified` unless the configured reserve verifier has independently verified the source. Evidence hashes are audit references, not bank statements or proof of settlement.

The public OUSD dashboard remains read-only. It must never be used to mint, redeem, transfer, or alter the registry.

## Stop conditions

Stop without submitting if any of the following occurs:

- RPC health check fails or reports a different network.
- The CLI network passphrase is not exactly `Pi Testnet`.
- The contract build artifacts do not match the tested source commit.
- Any role address is duplicated or not controlled by the intended verifier.
- The token controller is not allowlisted as a minter.
- A readback does not match the local deployment record.
- A reserve source, payout evidence, or issuer authority is unverified.

No Mainnet deployment is included in this runbook.
