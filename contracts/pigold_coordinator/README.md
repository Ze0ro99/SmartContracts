# PiGold Coordinator (Testnet candidate)

This crate is the first on-chain registry and route-validation boundary for PiGold.

## Implemented
- Admin-controlled registration of pair-specific pool identities.
- Unique reserve-accounting namespace enforcement at registration.
- Route validation across enabled pools, including pair continuity and duplicate-pool rejection.
- Explicit-input route validation (`validate_route_from`) so forward and reverse paths are unambiguous.
- Persistent pool and reserve-index TTL extension on writes and reads, plus instance TTL extension.
- Admin disable switch to exclude a pool from new routes.
- Read-only pool metadata retrieval.

## Explicitly not implemented
- No token custody or token transfer.
- No AMM swap invocation or multi-hop execution.
- No oracle trust or price assertion.
- No reserve/liability proof, supply verification, or asset mint/burn.
- No deployment or Testnet transaction evidence.

A registered pool is metadata only. The UI must require separate deployment, WASM and state readback evidence before marking an asset or pool verified. Pool adapter execution, quote comparison, authorization delegation and oracle policy require separate reviewed changes.

## Storage lifecycle
- Pool records and reserve-accounting indexes are stored in persistent Soroban storage and extended to the network maximum TTL when written or read.
- The contract rejects a pool read if its reserve-accounting index is missing, rather than silently treating the namespace as unused.
- Soroban persistent entries can still expire after long inactivity. Operators must restore expired footprints before calling the affected records; TTL extension is not a substitute for archival/restore operations.

## Route direction
- New clients should call `validate_route_from(asset_in, route)` and provide the actual input asset.
- `validate_route(route)` remains as a compatibility wrapper that assumes the first pool's `asset_a` is the input.
- Route validation does not execute a swap or move funds.
