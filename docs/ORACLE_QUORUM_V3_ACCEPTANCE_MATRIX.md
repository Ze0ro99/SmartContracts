# Oracle Quorum V3 — Security and Acceptance Test Matrix

Status: review specification only. This document does not claim that the cases below are implemented, passing, or deployed.

Contract: `contracts/pirc_214_oracle_quorum_v3`

## Current implementation contract

The current reference implementation provides:
- Admin-authenticated initialization, reporter enable/disable, and feed configuration.
- A maximum of 32 reporter addresses.
- Authenticated reporter submissions with positive prices, exact unit matching, and strictly increasing per-reporter sequence numbers.
- Ledger timestamps and a configurable maximum observation age.
- Median aggregation followed by a maximum-deviation filter and a second median.
- Integer-only arithmetic with checked multiplication in deviation comparisons.

The existing unit test exercises a three-reporter median, one outlier rejection, and a quorum of two. It is a useful baseline, not complete security coverage.

## Required acceptance matrix

| ID | Area | Scenario | Expected result |
|---|---|---|---|
| OQ-001 | Initialization | Initialize twice | Second call returns `AlreadyInitialized` |
| OQ-002 | Admin auth | Non-admin changes reporter list | `Unauthorized`; state unchanged |
| OQ-003 | Reporter auth | Disabled or unknown reporter submits | `Unauthorized`; no observation/event |
| OQ-004 | Reporter bound | Enable a 33rd distinct reporter | `InvalidPolicy`; reporter set unchanged |
| OQ-005 | Feed policy | Quorum is zero or greater than 32 | `InvalidPolicy` |
| OQ-006 | Feed policy | Max age is zero, deviation exceeds 10,000 bps, or unit is empty | `InvalidPolicy` |
| OQ-007 | Price | Submit zero price | `InvalidPrice` |
| OQ-008 | Units | Observation unit differs from feed unit | `UnitMismatch`; no observation |
| OQ-009 | Sequence | Submit equal or lower sequence for same feed/reporter | `StaleSequence`; previous observation preserved |
| OQ-010 | Feed isolation | Submit to one feed and read another | No cross-feed observation |
| OQ-011 | Freshness | Observation age equals max age | Accepted as fresh |
| OQ-012 | Freshness | Observation age exceeds max age by one second | Rejected; quorum recalculated |
| OQ-013 | Freshness | Observation timestamp is later than ledger timestamp | Rejected |
| OQ-014 | Quorum | Fewer fresh observations than quorum | `InsufficientQuorum` |
| OQ-015 | Deviation | One value exceeds configured deviation | Outlier rejected; accepted median returned if quorum remains |
| OQ-016 | Deviation | Filtering leaves fewer than quorum | `InsufficientQuorum` |
| OQ-017 | Median | Odd and even accepted sample counts | Correct deterministic median; even median floors without overflow |
| OQ-018 | Arithmetic | Difference × 10,000 overflows u128 | `ArithmeticError`, never wrap |
| OQ-019 | Arithmetic | Reference × max deviation overflows u128 | `ArithmeticError`, never wrap |
| OQ-020 | Reporter lifecycle | Disable a reporter after it has submitted | Its old observation is excluded from consensus |
| OQ-021 | Reporter lifecycle | Re-enable a reporter | Only a fresh valid observation can contribute |
| OQ-022 | Events | Successful policy, reporter, and price changes | Event payloads match the committed state |
| OQ-023 | Non-mutation | Every rejected submission/configuration | Storage remains unchanged |
| OQ-024 | Resource bounds | Maximum 32 reporters, maximum-length feed/unit strings | Execution remains within measured Soroban resource budget |

## Important implementation review notes

1. The current contract stores reporter addresses in an instance vector and observations/policies in persistent storage. Tests must cover the 32-reporter boundary and the storage-expiration/TTL policy before production use.
2. The even-sized median uses an overflow-safe integer average. Preserve floor semantics in off-chain reference implementations so UI, indexer, and contract results cannot disagree.
3. The deviation calculation deliberately uses checked multiplication. Extreme `u128` values must return `ArithmeticError`; do not replace checked operations with saturating or wrapping arithmetic.
4. The consensus timestamp is the oldest fresh observation encountered before deviation filtering. Document this as a conservative freshness timestamp, or change the interface and tests if consumers need the oldest accepted observation instead.
5. The current contract is an oracle aggregation primitive. It does not authenticate external market sources, prove truth, establish a fiat redemption promise, or validate the economic meaning of a feed.

## Release gates

A release candidate must not be marked ready until:
- All OQ-001 through OQ-024 tests are implemented and passing.
- A deterministic off-chain reference test suite matches contract outputs for randomized bounded samples.
- Soroban CPU/memory/instruction budgets are recorded at the maximum reporter count.
- Admin and reporter authorization paths are reviewed independently.
- Deployment evidence includes the exact network, contract ID, WASM hash, transaction ID, and read-back of initialized admin, reporters, and feed policy.

## Explicit non-claims

This matrix is a test plan. It is not evidence of a deployed contract, a live Pi Testnet connection, verified price sources, proof of reserves, or production financial readiness.