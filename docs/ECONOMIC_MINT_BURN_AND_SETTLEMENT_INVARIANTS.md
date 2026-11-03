# Economic Mint, Burn, and Settlement Invariants

**Scope:** Soroban reference contracts in this repository. This document does not grant authority over native Pi or any external issuer's asset.

## 1. Asset-domain boundary

Every operation must bind to an explicit network domain, asset contract/issuer, asset identifier, decimals, and authorization policy. A contract for a project-issued token cannot mint or burn native Pi unless the native Pi protocol explicitly exposes that capability and authorizes the contract. Never model native Pi supply changes by editing application storage.

## 2. Operation semantics

- **Mint:** increases the supply of the specific contract-issued asset. Require issuer/admin authorization, one-time initialization, cap checks, checked arithmetic, and an auditable event. If the asset represents a liability, update liability accounting atomically.
- **Burn:** decreases the supply of the specific burnable asset. Define who may burn (holder, issuer, or both), reject zero amounts, check balance and supply, and emit an event.
- **Lock:** moves units into a restricted state or escrow with explicit release/expiry conditions. Locked balances remain part of total supply unless the asset specification says otherwise.
- **Unlock/release:** requires a verified condition and idempotency key; cannot release twice.
- **Internal credit retirement:** closes an application accounting credit. It is not a blockchain token burn and must be named distinctly.
- **Fee collection:** account separately from burn. A fee sent to a treasury is not burned; a fee burned must be reflected in supply read-back.

## 3. Required state invariants

For each asset:
`total_supply = sum(all spendable balances) + sum(all locked balances) + sum(all escrow balances)`
subject to explicitly documented excluded/burned states.

For each operation:
- Amount is a positive integer in atomic units.
- Add/subtract/multiply are checked; no floating point in consensus/accounting paths.
- Caller is authenticated and authorized for the exact operation.
- Network and asset identifiers match deployment configuration.
- Nonce/idempotency key prevents replay.
- State update and event emission are atomic.
- Failure leaves state unchanged.
- Period identifiers and settlement IDs are unique.
- Sum of allocations cannot exceed the approved distributable amount.
- Deductions exceeding collected revenue reject the period; they do not silently clamp.

## 4. Revenue-funded settlement

Inputs must distinguish assessed fees, invoiced amounts, collected amounts, refunds, chargebacks, costs, reserves and legally required obligations. Only reconciled collected revenue can fund distributions.

`surplus = collected_revenue - operating_costs - required_reserves - taxes - contingencies - prior_deficits`

If any required input is missing or surplus is negative, return a typed rejection. If positive, apply a bounded liquidity factor:
`LCR = eligible_liquid_assets / stressed_outflows`
`Phi = min(1, (LCR / LCR_target)^2)`
`allowed = floor(surplus × Phi)`

The contract must not accept an arbitrary caller-supplied `Phi` without a signed, versioned and independently validated evidence package. Prefer storing the approved period inputs and policy hash.

## 5. Fair allocation

`weight_i = eligibility_i × quality_i × reliability_i × min(weight_cap, (contribution_i / baseline)^alpha)`, where `0 < alpha < 1`.

`allocation_i = floor(allowed × weight_i / sum(weights))`

Distribute the remaining atomic units using largest remainder, with a deterministic tie-break. Enforce per-account caps, eligibility snapshots, unique participant keys, and replay protection. The contract must not trust self-reported contribution values.

## 6. Dual-value conversion boundary

External market quotes and internal reference values are different data types. A quote must include asset, network, market, currency, source, timestamp, observation hash, policy version and expiry. Reject stale, low-quorum, mismatched-network or manipulated observations. The contract must not interpret `S = P_ref / P_ext` as a redemption multiplier or multiply a user's token balance by S.

A sovereign ratio is a reporting index unless a separate legal issuer, reserve, custody, redemption terms and independent attestation establish a funded conversion obligation.

## 7. Events and audit fields

Emit stable events for initialization, mint, burn, lock, release, fee collection, settlement approval, settlement execution, rejection, pause and recovery. Include operation ID, asset/network domain, amount, policy version, evidence hash, period, actor and resulting state version. Do not place personal identity documents or secrets in events.

## 8. Adversarial tests

1. Unauthorized mint/burn/lock/release.
2. Duplicate initialization and admin replacement attempts.
3. Zero amount, overflow, underflow and maximum-supply boundary.
4. Duplicate operation IDs, replayed settlement and duplicate period.
5. Allocations over the approved amount and duplicate participants.
6. Missing revenue evidence, negative surplus and over-deduction.
7. Stale oracle, low quorum, outlier, wrong asset and wrong network.
8. Lock expiry, double release and pause/recovery.
9. Integer rounding conservation and largest-remainder deterministic tie-break.
10. Storage growth, TTL expiry, resource-budget and upgrade compatibility.

## 9. Release evidence

A release must pin PiRC spec revision, source commit, Rust/Soroban SDK, toolchain, lockfile, WASM hash, test results, network passphrase, contract ID, deploy transaction, initialization transaction, live read-back, reviewer and remediation status. A successful compile or unit test is not a security audit or production approval.
