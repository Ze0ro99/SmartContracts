# PiRC-101 Monetary Controller (Soroban Reference)

## Purpose

This contract connects a validated Pi/USD feed from the PiRC-214 Oracle Quorum contract to the PiRC-101 live IPPR reference and separately governed QWF state.

It exposes an informational reference snapshot only. It does not mint REF, issue credit, alter native Pi, custody collateral, convert wallet balances, authorize payments, or settle commerce.

## Equations

IPPR is re-based on every accepted oracle tick:

`IPPR_t = median(PI/USD_t) × 10,000,000`

QWF is a separate adaptive state, initially configured around the 10,000,000 base and bounded by governance-approved MIN_QWF / MAX_QWF.

The reflexive constraint is:

`Φ = (L_n × IPPR) / QWF`

The contract returns Phi in parts per million using integer arithmetic. If no fresh approved velocity/TVL observation exists, Phi is returned as unavailable.

## Oracle dependency

- Contract client: `pirc_214_oracle_quorum_v3`
- Feed ID: configured at initialization (for example `PI/USD`)
- Unit: configured at initialization (for example `NANO_USD_PER_PI`)
- The Oracle Quorum contract enforces reporter authorization, source quorum, sequence monotonicity, freshness and deviation.

## QWF governance

QWF bounds and adjustments require the configured admin/governance authorization, a strictly increasing policy version, an evidence hash, published network velocity and TVL inputs, and a per-update adjustment limit of 1,000 bps. The contract deliberately does not infer an adjustment rate from unverified caller inputs or run an autonomous monetary policy.

## Validation

`cargo test -p pirc-101-monetary`

Tests cover live oracle-linked IPPR, QWF/IPPR separation, Phi calculation, governance adjustment, QWF bounds, and invalid metric/adjustment rejection.

## Deployment status

Reference implementation only. No Testnet deployment, contract ID, transaction, or live feed binding is claimed by this crate. Mainnet is out of scope and disabled.
