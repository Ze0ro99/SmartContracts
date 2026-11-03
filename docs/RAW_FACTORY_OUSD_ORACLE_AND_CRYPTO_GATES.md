# RAW Factory, OUSD, Oracle and Cryptographic Execution Gates

**Status:** Implementation requirements for reference contracts. Not deployment authorization.

## 1. Contract factory boundary

A factory may deploy only a governance-approved, immutable template identified by a code hash and schema version. Required inputs: asset ID, issuer, metadata commitment, network, template ID, idempotency key and policy version. Required outputs: deployment transaction, child contract ID, code hash, registry receipt and read-back evidence.

The current RAW factory reference in the PiRC omni-factory branch only emits an event and returns a product ID. The JavaScript SDK returns a placeholder transaction string. Neither is a working deployer. A production factory requires an authorized deployer/relayer, upload and install flow, deterministic salt, deployment fee controls, retry-safe state machine, registry authorization and post-deployment verification. Do not claim "one registration = one contract" until these steps work on a supported network.

## 2. RAW evidence is not a token by itself

The contract must distinguish asset metadata from legal rights. A token may represent a specific right (title, claim, warehouse receipt, fractional beneficial interest, access or service entitlement) only when issuer authority and legal enforceability are documented. Store content hashes and evidence references on-chain; keep personal data and bulky documents off-chain under access controls.

Physical sensor events must use provisioned device keys and challenge-response:
- unique device identity and certificate chain;
- unpredictable nonce and expiry;
- signed payload binding device, asset, location scope, reading and timestamp;
- signature verification and revocation checks;
- replay detection and independent attestation;
- dispute and correction events.

A hash is an integrity commitment, not an authenticity signature. A QR/NFC identifier is not proof of title or possession. The inspected mock scanner uses SHA-256 over user/product/time and the inspected possession contract returns `true`; these must remain test-only until cryptographic verification is implemented.

## 3. OUSD: three distinct product choices

1. **Internal unit of account:** records a stable accounting unit for platform goods/services; not transferable or redeemable unless terms say so.
2. **Basket-backed utility credit:** represents a claim on specified goods/services or a weighted basket; needs enforceable delivery and substitution rules.
3. **USD-redeemable stablecoin:** requires a legal issuer, segregated high-quality reserves, custody, liabilities, daily reconciliation, independent attestations, redemption at disclosed terms, liquidity, insolvency and jurisdictional compliance.

Do not call options 1 or 2 a USD stablecoin. Do not represent an algorithmic target or oracle price as backing.

The current `ousd_test_token` reference has seven decimals, a fixed code-level cap and admin mint/burn. It does not implement collateral, reserves, redemption, peg, proof-of-reserves or liquidation. Keep it test-only. Minting should not be enabled on production networks.

## 4. Reserve and liability model

For a redeemable instrument, define:
`ReserveCoverage = eligible_haircut_reserves / outstanding_redeemable_liabilities`

Eligibility must include asset quality, custody control, encumbrances, maturity, currency mismatch, liquidity haircut, legal availability and concentration limits. Do not count internal platform goods, forecast revenue or another unredeemable token as cash reserves without an explicit legally enforceable valuation and liquidation policy.

Require:
- segregated asset and liability ledgers;
- issuance cap linked to verified eligible reserves;
- deterministic mint/burn authorization;
- redemption queue and settlement SLA;
- independent attestations and exception reporting;
- circuit breakers for stale oracle, depeg, custody failure or insolvency;
- user-facing disclosures of issuer, claim, redemption fees and jurisdiction.

## 5. Oracle architecture

Treat each feed as a typed tuple:
`(asset_id, network_id, quote_unit, source, price_atomic, decimals, timestamp, sequence, confidence, evidence_hash)`.

A valid aggregation requires independent sources, quorum, freshness/heartbeat, deviation bound, outlier rejection, venue quality and a documented fallback. A single allowlisted reporter is a data publisher, not a decentralized oracle network. An AI score or proof hash is not a price attestation.

Chainlink AggregatorV3 integration is chain-specific. Verify the exact chain, feed proxy, decimals, heartbeat, round completeness, updatedAt, access restrictions and market coverage. Do not assume an EVM feed contract is callable from Soroban. Use a separately audited adapter/relayer with signed source evidence if the target network has no native feed integration.

## 6. PiDEX arithmetic

Never use floating-point values in contract or accounting paths. Use integer atomic units and checked operations. For a constant-product pool with fee denominator D and fee F:
`amount_in_after_fee = floor(amount_in × (D-F) / D)`
`amount_out = floor(amount_in_after_fee × reserve_out / (reserve_in + amount_in_after_fee))`

Check zero input, zero reserves, overflow, minimum output, price impact, fee accounting, pool pause, token authorization, reserve conservation and rounding dust. The constant-product invariant is an AMM pricing rule, not a stable price guarantee.

CEX and DEX prices are not interchangeable:
- CEX: order-book depth and executable VWAP for a defined notional, plus venue/custody/withdrawal status.
- DEX: reserve state, fee, route, block/ledger, slippage and MEV risk.
- A composite index must disclose weighting and exclude stale/thin/manipulated sources.

## 7. Post-quantum crypto agility

Separate transport encryption, signatures, key agreement, hashing and ledger consensus. Use algorithm identifiers and key versions in signed records. Plan hybrid migration and dual verification where supported. Evaluate NIST FIPS 203 ML-KEM, FIPS 204 ML-DSA and FIPS 205 SLH-DSA against implementation maturity, memory, transaction size, CPU and runtime limits. The project must not claim PQ protection merely because a branch contains a `quantum` name, a SHA-256 hash, or a string saying "quantum-secured".

On-chain PQ verification requires actual target-network/runtime support and measured resource budgets. Until then, off-chain PQ-signed attestations with on-chain hashes can provide evidence anchoring, but do not make the ledger itself PQ-secure.

## 8. Differential geometry and risk engines

Differential geometry may describe state-dependent risk and liquidity surfaces. Keep the model outside token supply rules unless independently validated. Publish state variables, metric tensor, units, calibration data, boundary conditions, numerical stability, stress scenarios and error bounds. No geometric multiplier may create units or redemption rights.

## 9. Required adversarial test suites

- Unauthorized factory deploy, template substitution, duplicate asset ID and idempotency replay.
- Fake scanner, invalid signature, revoked device, expired nonce and cross-asset replay.
- OUSD cap overflow, unauthorized mint/burn, allowance expiry, supply conservation and pause.
- Oracle stale, quorum loss, source correlation, outlier, wrong network/asset/decimals and sequence rollback.
- AMM overflow, rounding, dust, zero reserves, sandwich/slippage and paused pool.
- Reserve shortfall, custody outage, redemption run, depeg and insolvency.
- PQ key rotation, downgrade, mixed algorithm, invalid signature and oversized payload.

## 10. Release gates

No production deployment until source code, contract ID, code hash, network, signer governance, audit, test results, oracle configuration, reserve evidence (if applicable), legal issuer terms, monitoring and rollback are independently verified.
