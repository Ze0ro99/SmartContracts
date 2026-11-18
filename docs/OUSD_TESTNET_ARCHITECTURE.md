# OUSD Testnet Architecture and Capability Specification

**Status:** Proposed testnet architecture. This document does not assert that OUSD is issued, backed, redeemable, deployed, or integrated with a payment provider.

## 1. Purpose and scope

OUSD is a new, separate test asset for exercising reserve accounting, issuance controls, liabilities, redemption, payout evidence, and reconciliation. It supplements the existing Pi ecosystem layers; it does not replace RED, ORANGE, YELLOW, GREEN, BLUE/PiBank, INDIGO, PURPLE, or PiGold.

OUSD is not native Pi, not an internal GCV claim, and not automatically equivalent to one US dollar. Any USD-denominated test accounting is a test convention until independent reserve verification, governance approval, legal review, and operational redemption capability exist.

## 2. What the supplied Testnet account snapshot establishes

The supplied account snapshot for `GB7EKKXHOCCMVN5SQNJ6IVILY53KSVH2DPMJ66P6272CWCJO7DUJMFVZ` reports:
- balances in the existing GOLD, GREEN, ORANGE, PURPLE, RED, and YELLOW assets;
- 10 PGOLD units issued by a different issuer address;
- multiple liquidity-pool-share positions; and
- a native-asset balance.

This is useful evidence that the ecosystem already has multi-asset and pool-share activity to integrate with. It does **not** establish OUSD issuance (no OUSD trustline/balance is present in the supplied snapshot), USD cash reserves, pool composition, redeemable value, control of issuer keys, or ownership of the account by the project operator. LP-share balances are not reserve balances; the underlying pool reserves and withdrawal conditions must be read and valued independently. A pool containing ecosystem tokens is not a USD reserve.

## 3. Asset identity and isolation

- Create OUSD as a distinct Testnet asset with its own issuer, distribution/authorization policy, trustline policy, and audit identity.
- Never reuse an existing layer's asset code, issuer, reserve ledger, or pool balance to represent OUSD.
- Record the network passphrase/network identifier, issuer public key, asset code, asset precision, contract IDs, and deployment ledger in a versioned deployment manifest.
- Use explicit states: `PROPOSED`, `TESTNET_DEPLOYED`, `RESERVE_VERIFIED`, `MINT_ENABLED`, `REDEMPTION_ENABLED`, `PAUSED`, and `RETIRED`. A state transition requires a recorded governance decision and evidence.

## 4. Logical architecture

| Component | Responsibility | Must not do |
|---|---|---|
| OUSD Asset / Token Adapter | Testnet asset identity, authorization and supply observation | Infer backing from price or LP shares |
| Reserve Registry | Record reserve lots, currency, amount, custodian reference, evidence digest, verifier, timestamps and status | Treat a hash as proof of bank funds |
| Reserve Attestation Gateway | Accept independently verified, finalized USD inflow evidence | Accept quotes, pending transfers, screenshots alone, or market prices as settled cash |
| Issuance Controller | Enforce reserve capacity, caps, roles, idempotency and mint authorization | Mint before finalized reserve approval |
| Liability Ledger | Track beneficiary, amount, reserve lot allocation, status and audit references | Double-count reserve capacity |
| Redemption Controller | Accept beneficiary-authorized redemption requests and enforce state transitions | Mark pending/failed payments as paid |
| Payout Verifier | Confirm exact settled payout evidence | Verify its own issuance or reserve record |
| Burn Adapter | Burn/retire only the exact OUSD amount after confirmed payout and reconciliation | Burn on request alone or burn unrelated balances |
| Risk & Circuit Breaker | Caps, concentration limits, stale-evidence checks, pause controls | Autonomously change roles or release funds |
| Reconciliation & Event Indexer | Reconcile token supply, reserve liabilities, payouts, burns and ledger events | Invent missing events or silently repair mismatches |
| Governance Registry | Approve roles, caps, supported reserve types, policy versions and upgrades | Allow one key to unilaterally attest, issue and pay |

The current source includes a test-only OUSD token and reserve/liability controller with allowlisted minting, beneficiary-approved escrow, and atomic token burn on payout-attestation finalization. This is an on-chain test workflow only: it does not custody USD, connect to a bank or payment rail, or independently prove off-chain reserve/payout evidence.

## 5. Reserve policy and eligible collateral

### Eligible for a future USD reserve (only after independent verification)
- Finalized USD cash held by a named, approved custodian or regulated account, evidenced through an approved independent verification process.
- A settled inbound USD payment only after the receiving institution confirms finality and the verifier reconciles it to the reserve account.

### Not eligible as USD reserve by default
- Pi, XLM, GOLD, GREEN, ORANGE, RED, YELLOW, PURPLE, PGOLD, or any other ecosystem token.
- AMM/DEX liquidity-pool shares or the notional value of pool positions.
- Unsettled transfers, pending payout instructions, quotes, oracle prices, market capitalization, GCV references, screenshots without independent confirmation, or unverified attestations.

Non-USD collateral could be considered only under a separately approved collateralized product with explicit asset denomination, haircut, custody, liquidation, volatility, and insolvency rules. It must not be silently counted as USD.

## 6. Accounting model

All reserve/liability accounting uses integer USD micros:
- `1 USD = 1,000,000 USD micros`
- no floating-point arithmetic;
- token atomic units are separate from USD micros and require a versioned conversion rule.

For each reserve lot (r):
- `attested_usd_micros[r]`
- `available_usd_micros[r]`
- `allocated_usd_micros[r]`
- `paid_out_usd_micros[r]`
- `released_usd_micros[r]` (only where a valid cancellation/release policy permits it)

Core invariants:
1. `attested = available + allocated + paid_out + released` for each lot, subject to explicitly modeled adjustments.
2. `available >= 0`, `allocated >= 0`, `paid_out >= 0`, and all arithmetic is checked.
3. Total outstanding liabilities must reconcile to the token supply or a documented, versioned liability mapping. Do not assume 1:1 without a policy and atomic mint/burn adapter.
4. A reserve evidence digest, liability evidence digest, payout digest, and transaction id are unique within their own domain and replay-protected.
5. Every state transition is idempotent and emits an event with a stable identifier.
6. No reserve lot may be allocated beyond its verified available balance.
7. Reconciliation mismatch pauses new issuance and redemption finalization pending review; it must not fabricate balancing entries.

## 7. Issuance and redemption lifecycle

### Issuance
1. Reserve funds arrive and reach final settlement outside the chain.
2. Independent reserve verifier checks custodian statement/API evidence and reconciliation.
3. Reserve Registry records a unique reserve lot and verification metadata.
4. Issuance Controller checks role separation, evidence freshness, limits, available reserve and idempotency.
5. In one atomic Soroban transaction, allocate the reserve lot, create the beneficiary liability, and mint the exact authorized OUSD amount through the configured token contract. The controller must be explicitly allowlisted as a minter on that token.
6. Reconcile minted atomic units against liability units and emit a verifiable issuance event.

The current test implementation performs token minting and liability accounting in the same transaction; a failed token call must revert the whole invocation. Deployment readiness still requires readback of token supply, balances, contract IDs and code hashes.

### Redemption
1. Beneficiary authenticates a redemption request and approves the controller to transfer the exact token amount.
2. The controller moves the liability to `REDEMPTION_PENDING` and escrows the approved OUSD.
3. A payment operator executes payout outside the chain.
4. An independent payout verifier attests the exact settled amount. The contract cannot inspect the bank or payment rail itself.
5. In one atomic Soroban transaction, the controller burns the exact escrowed OUSD, updates reserve and outstanding-liability accounting, records payout evidence, and marks the liability `PAID`.
6. Reconciliation independently confirms token supply reduction, reserve accounting, contract events and ledger state.

If payout is pending, failed, reversed, partial, wrong-currency, or mismatched, the liability remains outstanding and no burn is authorized. Partial payouts require an explicit partial-redemption state machine; they must not be rounded or silently treated as full settlement.

## 8. Roles and key separation

At minimum separate:
- governance/admin;
- OUSD issuer/minter;
- reserve verifier;
- payout verifier;
- payment operator;
- emergency guardian;
- auditor/read-only observer.

No single role may both attest reserve and attest payout for the same liability. High-impact changes require threshold/multisig authorization and timelock where supported. Emergency pause may stop new issuance; unpause, role changes, upgrades and reserve policy changes require stronger governance. Never store signing secrets in repository files, frontend code, logs, or chat.

## 9. Limits, monitoring and circuit breakers

Configurable testnet controls:
- per-transaction and per-day issuance caps;
- total outstanding cap;
- reserve concentration and stale-attestation limits;
- per-beneficiary and per-custodian exposure caps;
- minimum reserve coverage threshold;
- duplicate/replay detection;
- emergency pause for issuance and separately for redemption finalization;
- event-to-ledger reconciliation and alerting.

A market/oracle feed may inform monitoring or a separately labeled market quote. It must never create reserve, mint authority, or a redemption guarantee.

## 10. Events and data model

Every state-changing operation should emit versioned events with:
- `schema_version`, `event_id`, `network_id`, `contract_id`, `ledger_sequence`, `ledger_timestamp`;
- `reserve_id` and/or `liability_id`;
- amount and currency/asset identifier;
- actor role and public address;
- evidence digest and external reference identifier (never private bank credentials);
- previous and next state;
- policy version and idempotency key.

The indexer should persist raw event identity, cursor/ledger checkpoint, decoded payload, processing status and reconciliation result. Use unique constraints on `(network_id, contract_id, event_id)` and on source ledger/transaction/operation identity. Ingestion is at-least-once and idempotent; never claim complete synchronization until cursor continuity and ledger reconciliation are verified.

## 11. Required test matrix before any Testnet1 deployment

- role separation and unauthorized callers;
- invalid and zero/negative amounts;
- reserve capacity exhaustion and concurrent allocation;
- duplicate reserve/liability/payout evidence;
- stale or revoked reserve attestations;
- exact currency and exact payout amount;
- beneficiary mismatch;
- pending, failed, reversed and partial payout;
- repeated redemption, payout and burn;
- token mint/burn atomicity and rollback behavior;
- overflow and precision conversion boundaries;
- storage TTL, archival, migration and recovery;
- pause/unpause, caps, timelocks and role rotation;
- event replay, cursor gaps, duplicate ingestion and reorg/finality policy;
- reconciliation of reserve lots, outstanding liabilities, token supply and burned supply.

## 12. Deployment gates

No Testnet1 deployment or mint is authorized by this specification. Before deployment:
1. complete independent contract/security review, including negative-path, role-separation, replay, TTL, archival and atomic rollback tests;
2. finalize reserve/payout verifier governance, independent key custody and signer separation;
3. pin dependencies and pass locked Rust tests, WASM build, static analysis and secret scan on the exact release commit;
4. verify Pi Testnet 1 RPC health, network passphrase, deployment identities, contract IDs and WASM hashes using read-only queries;
5. publish a versioned deployment manifest and reconciliation dashboard;
6. obtain explicit authorization for the exact Testnet 1 deployment, administrator, issuer/verifier roles and mint limits;
7. deploy only to Testnet 1, then read back contract identity, token supply, reserve/liability state and emitted events;
8. keep all USD backing, peg and redemption claims disabled unless independently evidenced and legally/operationally approved.

## 13. Evidence labels

Use only these labels in dashboards and reports:
- `VERIFIED`: independently confirmed on-chain state plus validated network/ledger evidence;
- `OBSERVED`: read-only API or user-supplied snapshot, not independently reconciled;
- `ATTESTED`: trusted verifier submitted an evidence reference; underlying external fact remains trust-dependent;
- `SIMULATED`: test fixture or local model only;
- `UNKNOWN`: evidence missing or conflicting;
- `BLOCKED`: required approval, test, integration or external verification is absent.

The supplied account snapshot is `OBSERVED`. It is not proof of OUSD backing or a USD reserve.
