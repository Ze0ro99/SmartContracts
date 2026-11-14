# PiGold Coordinator (Testnet candidate)

This crate is the first on-chain registry and route-validation boundary for PiGold.

## Implemented
- Admin-controlled registration of pair-specific pool identities.
- Unique reserve-accounting namespace enforcement at registration.
- Route validation across enabled pools, including pair continuity and duplicate-pool rejection.
- Admin disable switch to exclude a pool from new routes.
- Read-only pool metadata retrieval.

## Explicitly not implemented
- No token custody or token transfer.
- No AMM swap invocation or multi-hop execution.
- No oracle trust or price assertion.
- No reserve/liability proof, supply verification, or asset mint/burn.
- No deployment or Testnet transaction evidence.

A registered pool is metadata only. The UI must require separate deployment, WASM and state readback evidence before marking an asset or pool verified. Pool adapter execution, quote comparison, authorization delegation and oracle policy require separate reviewed changes.
