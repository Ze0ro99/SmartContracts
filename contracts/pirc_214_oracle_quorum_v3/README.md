# PiRC-214 Oracle Quorum V3 (Experimental)

**Status:** Draft Soroban implementation. It is not deployed, audited, or approved for settlement.

## Scope

This crate adds a feed-scoped multi-reporter observation store and a deterministic quorum read path while preserving the existing pirc-214-oracle-v2 reference crate.

- Admin-controlled reporter allowlist.
- Feed-specific quorum, maximum observation age, unit and maximum deviation policy.
- Per-reporter, per-feed monotonic sequence checks.
- Ledger timestamping at submission.
- Freshness filtering, median calculation and deviation filtering.
- Explicit failure when the fresh or accepted observation count is below quorum.
- Integer-only price arithmetic; no floating point.

## Important trust boundaries

A quorum counts distinct Soroban addresses, not necessarily independent organizations or data sources. Governance must establish source independence, ownership separation, key custody and conflict-of-interest controls before configuring a production feed.

The reporter's Soroban authorization proves control of the authorized account for the invocation; it does not prove that the submitted price is true. The contract does not fetch CEX/DEX/Chainlink data, verify external exchange signatures, validate market depth, or establish legal rights to an asset.

The unit field is an exact string match, not a full canonical asset-pair registry. Integrators must bind each feed ID to a canonical base asset, quote asset, scale, network and policy version. The current contract has a maximum of 32 registered reporter addresses to bound iteration costs.

## Required follow-up before Testnet use

1. Compile and run this crate under the pinned workspace Soroban SDK.
2. Add negative tests for stale sequence, disabled reporter, wrong unit, stale observations, below-quorum and arithmetic boundaries.
3. Add property/fuzz tests for median and deviation boundaries.
4. Benchmark worst-case 32-reporter execution and storage footprint.
5. Add governance-controlled policy versioning and safe policy migration.
6. Independently review source independence, reporter rotation, emergency pause and upgrade policy.
7. Deploy only to an explicitly approved Testnet, then record transaction, WASM hash, contract ID and read-back evidence.

No Mainnet signing, deployment or settlement authorization is included.
