# OUSD Test Reserve & Liability Controller

**Status:** Testnet-only reference contract; not deployed or production-ready.

## Scope

This contract records a role-attested accounting lifecycle:
1. A reserve verifier records a settled-USD reserve attestation and evidence hash.
2. A separate issuer allocates part of that reserve to a beneficiary-bound liability.
3. The beneficiary requests redemption.
4. A separate payout verifier records an exact-amount settled-USD payout attestation.
5. Only then does the liability become eligible for a separate token-burn workflow.

All amounts are integer USD micros (1 USD = 1,000,000 micros), not token atomic units.

## Critical boundaries

- No USD custody, bank connection, payment-provider connection, Visa, Banxa, Onramper, or external payment execution.
- Evidence hashes are audit references; they do not prove that bank balances, payments, or documents are genuine.
- Reserve and payout verifiers are trusted attestors. Their authority and independence require governance outside this contract.
- The controller does not mint or burn OUSD. The burn authorization view is a state gate only; a separate workflow must verify token balances and execute any burn.
- Reserve allocation is consumed when a liability is issued and is not returned to available reserve after payout.
- A quote, market price, pending payment, or failed payment cannot create a settled reserve or paid liability.
- Source code is not evidence of deployment, USD backing, peg, redemption rights, or live liquidity.

The administrator, issuer, reserve verifier, and payout verifier must be four distinct addresses. Roles are immutable in this initial test-only version.

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
- This reference still does not implement OUSD token minting or an atomic token burn. The burnable aggregate is an accounting indicator only and must not be used as authorization to burn outside a separately reviewed atomic integration.
