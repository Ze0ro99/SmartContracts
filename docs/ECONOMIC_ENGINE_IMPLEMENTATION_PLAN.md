# Soroban Economic Engine Implementation Plan

**Status:** Planned work; not a completed contract implementation.

## 1. Contract boundary

Implement a dedicated settlement contract rather than copying the generic asset template into a numbered crate. The contract owns period state, authorized settlement operators, replay protection, approved accounting inputs, and deterministic allocations. It must not infer market prices or revenue from token balances.

## 2. Proposed state model

- `Admin`: contract administrator.
- `Operator(Address)`: explicitly authorized settlement operator.
- `Policy`: versioned policy parameters and accepted asset identifiers.
- `Period(BytesN<32>)`: period ID, asset ID, collected revenue, deductions, distributable amount, state, and policy version.
- `Allocation(period_id, participant_id)`: atomic amount and eligibility evidence hash.
- `Settled(period_id)`: replay guard and final settlement digest.

Avoid storing unbounded participant arrays in one contract value. Define bounded batch sizes and deterministic pagination or Merkle commitments where appropriate. If a Merkle root is used, specify proof format and data availability requirements.

## 3. State machine

`DRAFT -> PROPOSED -> APPROVED -> SETTLED`

Terminal/recovery states:
- `REJECTED`: validation or policy failure.
- `CANCELLED`: authorized cancellation before settlement.
- `DISPUTED`: settlement held pending an explicit dispute process.

A period may transition to SETTLED only once. No transition from SETTLED back to an editable state is permitted. Corrections require a new adjustment period with a reference to the original settlement.

## 4. Accounting invariants

1. All amounts are non-negative integer atomic units of one declared asset per period.
2. Sum of deductions must not exceed collected revenue.
3. Distributable equals revenue minus all deductions exactly.
4. Sum of allocations equals distributable exactly when the allocation policy requires full allocation.
5. Participant identifiers are unique and canonically ordered for tie-breaking.
6. Weights are non-negative integers; positive distributions require a positive total weight.
7. A period ID is immutable after proposal approval.
8. Duplicate period settlement is rejected.
9. Every privileged transition requires authorization.
10. Events and stored state must reconcile.

## 5. Required public methods (proposed ABI)

- `initialize(admin)`
- `set_operator(admin, operator, enabled)`
- `propose_period(operator, period_id, asset_id, revenue, deductions, policy_version, input_digest)`
- `approve_period(approver, period_id)`
- `record_allocation(operator, period_id, participant_id, amount, evidence_hash)`
- `settle_period(operator, period_id, settlement_digest)`
- `get_period(period_id)`
- `get_allocation(period_id, participant_id)`

Final ABI must be reviewed against Soroban storage, event-size, resource, and authorization constraints before implementation.

## 6. Test plan

Use `fixtures/economic-engine-vectors.json` as the canonical source. Add contract-level tests for:
- every golden vector;
- underfunded periods;
- zero revenue and zero allocation;
- invalid asset/scale and duplicate participant IDs;
- negative values represented through invalid signed input types;
- maximum integer/resource boundaries;
- unauthorized admin/operator/approver;
- duplicate initialization and duplicate settlement;
- period mutation after approval;
- event/state mismatch;
- storage TTL and upgrade migration.

## 7. Oracle and reserve dependencies

Settlement must not directly trust an unverified external price. If policy needs a market quote, require a separate feed ID, quote unit, observation time, max age, source quorum and signed evidence. Reserve attestations must carry auditor identity, report hash, as-of time, unit/scale, scope and exception status. A reserve attestation is not an automatic payment authorization.

## 8. Delivery sequence

1. Create a dedicated crate with pinned workspace SDK.
2. Implement state types, errors, authorization helpers and one-time initialization.
3. Implement period proposal and immutable approval.
4. Implement allocation conservation and replay guard.
5. Port shared fixture cases into Soroban tests.
6. Add adversarial and boundary tests.
7. Add CI build/test and artifact checksum.
8. Conduct independent contract review.
9. Only then deploy to a named Testnet under an explicit allowlist and capture transaction/readback evidence.
