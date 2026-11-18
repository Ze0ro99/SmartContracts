# OUSD Test Reserve & Liability Controller

**Status:** Testnet-only reference contract; not deployed or production-ready.

## Scope

This contract records a role-attested accounting lifecycle:
1. A reserve verifier records a settled-USD reserve attestation and evidence hash.
2. A separate issuer allocates part of that reserve to a beneficiary-bound liability.
3. The beneficiary requests redemption.
4. The beneficiary approves the controller, then requests redemption; OUSD moves into the controller's escrow.
5. A separate payout verifier records an exact-amount settled-USD payout attestation.
6. In the same Soroban transaction as the payout state transition, the controller burns the escrowed OUSD and updates reserve/liability accounting.

All amounts are integer USD micros (1 USD = 1,000,000 micros). The test token has 7 decimals, so token atomic units = USD micros × 10. Checked arithmetic and the token supply cap reject amounts that cannot be represented.

## Critical boundaries

- No USD custody, bank connection, payment-provider connection, Visa, Banxa, Onramper, or external payment execution.
- Evidence hashes are audit references; they do not prove that bank balances, payments, or documents are genuine.
- Reserve and payout verifiers are trusted attestors. Their authority and independence require governance outside this contract.
- The controller can mint only through an explicitly allowlisted minter address on the configured OUSD test token. The controller escrow collects the beneficiary's approved tokens before redemption and burns them atomically when the payout verifier confirms settlement.
- Token allowance, balance, and supply failures revert the state transition. The burnable backlog is always zero because there is no deferred burn workflow.
- Reserve allocation is consumed when a liability is issued and is not returned to available reserve after payout.
- A quote, market price, pending payment, or failed payment cannot create a settled reserve or paid liability.
- Source code is not evidence of deployment, USD backing, peg, redemption rights, or live liquidity.

The administrator, issuer, reserve verifier, and payout verifier must be four distinct addresses. Roles are immutable in this initial test-only version. The token administrator must also authorize the controller contract as a minter before OUSD issuance can succeed. Configure the token exactly once on the controller.

## Validation

```bash
cargo metadata --locked --format-version 1
cargo test -p ousd-reserve-controller --locked
cargo check -p ousd-reserve-controller --locked
cargo build -p ousd-reserve-controller --locked --release --target wasm32-unknown-unknown
```

Do not deploy or connect this controller to a token issuer before independent security review, verifier governance, and a separate token-burn integration design are approved.


## Storage and TTL policy

- Small global configuration, roles and aggregate counters remain in instance storage.
- Reserve records, liability records and evidence replay markers use persistent storage so record growth is not constrained by the contract-instance entry size.
- Persistent records are renewed to the network maximum TTL whenever they are read or written.
- Soroban persistent entries can expire into the Expired State Stack (ESS). Before a call that touches an expired record or replay marker, the transaction footprint must restore the required keys. A successful source test does not prove that a production operator has a working restoration process.
- The application must maintain a record-to-key index and an operator runbook for restoring all related reserve, liability and evidence keys before invoking state transitions. Never treat a missing/unrestored evidence marker as proof that an evidence hash is unused.
- OUSD minting and redemption are now connected to the test token through an atomic contract-call boundary. This is not a production USD-backed stablecoin implementation; reserve attestations remain trusted claims and external payout is performed outside Soroban.
- Persistent records and replay markers require ESS restoration when expired. The app must maintain an auditable key index and restore all required keys before state transitions.
