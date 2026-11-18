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
