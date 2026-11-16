# Pi Testnet token release matrix and deployment gates

**Target network:** Pi Testnet1 only  
**Application public account supplied by operator:** `GD6HJ6WAXF4I7EW7M3SHOKRALOOPSSYSNVLCJ66PBFZRBZKYMPQFNMP4`  
**Release status:** PREPARATION — no deployment, initialization, mint, transfer, swap or liquidity action is asserted by this document.

## 1. Asset and contract separation

Every fungible token must have its own Soroban contract instance, issuer/admin policy, metadata, decimals, maximum supply, total supply, balances, and evidence record. Never merge token balances or reuse one token's supply cap for another. PiGold is treated as the ecosystem liquidity coordinator/orchestrator, not as a fungible token by assumption; its pool accounting must preserve per-asset reserves and must not double-count the same reserve.

| Logical layer / asset | Contract implementation candidate | Current status | Release gate |
|---|---|---|---|
| PiCash | `pirc_test_asset` template | UNMAPPED / NOT DEPLOYED | Confirm canonical symbol, metadata, decimals, cap, admin and purpose |
| PiBank | `pirc_test_asset` template | UNMAPPED / NOT DEPLOYED | Confirm whether it is a token or banking/service module; define custody boundary |
| PiGovernance | `pirc_test_asset` template only if a fungible governance token is intended | UNMAPPED / NOT DEPLOYED | Confirm voting rights, transferability, delegation and governance contract |
| PiYellow | `pirc_test_asset` template | UNMAPPED / NOT DEPLOYED | Confirm canonical asset identity and economics |
| PiOrange | `pirc_test_asset` template | UNMAPPED / NOT DEPLOYED | Confirm canonical asset identity and economics |
| Indigo / Purple | `pirc_test_asset` template | UNMAPPED / NOT DEPLOYED | Confirm whether these are separate tokens or alternate layer names; never alias silently |
| OUSD (test asset) | `ousd_test_token` | SOURCE EXISTS / NOT DEPLOYED | Test-only; no USD peg, backing, redemption, reserve or parity claim |
| PiGold | `pigold_coordinator` | REGISTRY / ROUTE VALIDATOR SOURCE | Not a token issuer, custodian, AMM or proof of live liquidity |

This matrix is intentionally not a deployment manifest with guessed symbols or supplies. An unconfirmed identity remains UNMAPPED and cannot be minted.

## 2. Reusable permissioned test-token implementation

`contracts/pirc_test_asset` is the configurable test asset template:
- one-time `initialize(admin, name, symbol, decimals, max_supply)`;
- decimals constrained to 0–18;
- admin-authorized `mint`, fixed per-instance supply cap;
- authenticated `transfer`, `approve`, `transfer_from`;
- `burn` and `burn_from` (where implemented by the contract);
- checked integer arithmetic and emitted asset events.

Each token requires a distinct deployed instance and independent initialization. The OUSD reference is separate and fixed at 7 decimals / fixed cap; it must not be treated as a generic stablecoin.

## 3. Mandatory build and source validation

From the SmartContracts repository root:

```bash
cargo test --manifest-path contracts/pirc_test_asset/Cargo.toml
cargo build --manifest-path contracts/pirc_test_asset/Cargo.toml --target wasm32-unknown-unknown --release
cargo test --manifest-path contracts/ousd_test_token/Cargo.toml
cargo build --manifest-path contracts/ousd_test_token/Cargo.toml --target wasm32-unknown-unknown --release
cargo test --manifest-path contracts/pigold_coordinator/Cargo.toml
cargo build --manifest-path contracts/pigold_coordinator/Cargo.toml --target wasm32-unknown-unknown --release
cargo test --manifest-path contracts/pidex_amm/Cargo.toml
cargo build --manifest-path contracts/pidex_amm/Cargo.toml --target wasm32-unknown-unknown --release
```

Record Rust toolchain, Soroban SDK lockfile, test output, artifact SHA-256, artifact byte size and source commit. A successful local build does not prove deployment.

## 4. Per-instance release record (required before deployment)

Create one signed-off record per token instance containing:
- canonical asset ID, display name, symbol, exact decimals and maximum supply in atomic units;
- purpose and legal/economic claims explicitly disallowed;
- issuer/admin public account and authorization policy;
- emergency pause / admin rotation policy (if supported; do not assume);
- independent contract instance ID and network passphrase;
- source commit, WASM SHA-256 and Soroban code hash;
- deployment transaction hash, ledger sequence and closed-ledger proof;
- initialization transaction hash and read-back of name, symbol, decimals, cap, admin and total supply;
- initial mint amount, recipient, transaction hash, resulting supply and balance read-back;
- evidence timestamp, source endpoint and status.

The supplied public account is an identity candidate only. Its signer authorization, admin role, balance and control of any existing contract remain UNKNOWN until independently read back and tested through an explicitly authorized Testnet transaction.

## 5. Required release sequence

1. Resolve the logical layer mapping and approve token parameters for every asset.
2. Run tests, Wasm builds, formatting and static/security checks; review all diffs.
3. Verify Pi Testnet1 Horizon and Soroban RPC endpoints, network identity/passphrase, latest closed ledger and RPC health. Abort on mismatch, HTTP error, stale ledger or unknown network.
4. Fund the test account only through the official Testnet faucet if needed; verify the resulting balance by read-only Horizon. Never use Mainnet funds.
5. Deploy one contract instance at a time to Pi Testnet1; capture transaction and ledger evidence.
6. Read back code hash, contract instance, initialization state, metadata, admin and cap.
7. Mint only a small, explicitly approved test amount per token instance; verify event, total supply and recipient balance. Keep each token's balances isolated.
8. Verify transfer/allowance/expiry/burn behavior in tests and, if explicitly approved, with small Testnet transactions.
9. Register each verified asset in the registry with its own metadata hash, issuer and decimals; do not register unknown or simulated assets as verified.
10. Connect PiDEX pools only after both token instances and their independent balances are verified. Keep pool reserves per asset pair; do not treat UI liquidity as on-chain liquidity.
11. Enable application gateways only after API/network identity, contract IDs, schema, event indexing, replay protection, idempotency and failure states pass.
12. Publish an evidence bundle and mark status VERIFIED only after independent read-back.

## 6. Gate statuses

- `SOURCE_ONLY`: code exists in a repository.
- `BUILT_TESTED`: pinned source builds and tests pass.
- `DEPLOYED_UNINITIALIZED`: deployment transaction and code hash verified; no initialized token state yet.
- `INITIALIZED_UNMINTED`: metadata/admin/cap read back; total supply zero.
- `MINTED_TESTNET`: mint transaction, event, supply and recipient balance independently read back.
- `INTEGRATED_TESTNET`: registry, wallet, indexer, gateway and relevant pool tests pass against the same Testnet network.
- `UNKNOWN` / `BLOCKED`: evidence missing, stale, mismatched or inaccessible.

Never infer a higher status from a lower one. UI simulations and local ledger fixtures remain SIMULATED.

## 7. Explicit safety boundary

This release preparation does not authorize Mainnet operations, production issuance, real-money claims, custody, asset conversion, bridge transfers, automated minting, liquidity deposits or swaps. No secret seed, private key, mnemonic or signer material belongs in GitHub, Lovable, frontend code, logs or chat. Use isolated operator-controlled signing only after the review gate and explicit per-transaction approval.

## 8. Known gaps that block "ready"

- New operator account has not yet been independently confirmed on Pi Testnet1.
- No contract IDs / WASM hashes / deployment transactions have been independently verified for the listed token contracts on Pi Testnet1.
- Logical layer-to-token mapping, symbols, decimals, supply caps and admin policy are not fully approved.
- Oracle quorum source independence and feed policy require validation before economic settlement.
- PiGold coordinator source is a registry/route validator; it does not custody assets, execute swaps or prove reserves.
- PiDEX AMM source is not proof of deployed pools or actual liquidity.
- Application gateway and Lovable synchronization remain subject to current project/workspace access and CI evidence.

Until these gaps close, the system is **prepared for controlled Testnet validation**, not fully launched or mint-ready.
