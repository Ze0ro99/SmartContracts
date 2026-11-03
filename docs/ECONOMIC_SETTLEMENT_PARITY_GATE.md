# Economic Settlement Contract Parity Gate

## Purpose

This document defines the acceptance boundary between the PiRC reference economic engine and Soroban contracts. It is a test plan, not a claim that contract parity has already been achieved.

## Canonical arithmetic rules

- All money amounts are unsigned integer atomic units. Floating-point arithmetic is prohibited.
- The period ledger uses collected revenue, not forecasts or market capitalization.
- Subtract operating costs, statutory/tax obligations, refunds and due liabilities, reserve replenishment, and approved commitments before calculating distributable value.
- If deductions exceed collected revenue, reject settlement with `UNDERFUNDED_PERIOD`; never clamp a deficit to zero.
- Allocation must be deterministic and conserve value exactly. Equal fractional remainders are broken by ascending canonical participant identifier.
- Reserve coverage is expressed in basis points. A zero liability denominator returns an undefined result, not infinity.
- Supply growth is expressed in basis points and must define behavior for zero prior supply.

## Shared test vectors

The canonical fixture is `fixtures/economic-engine-vectors.json`. The PiRC Python reference implementation, Soroban contract tests, and Unity Sphere API tests must consume the same cases. Do not duplicate expected values in implementation-specific test files.

## Required contract tests

1. Happy path for each fixture.
2. Underfunded settlement rejection.
3. Zero revenue and zero prior-supply boundaries.
4. Zero-weight, duplicate-ID, unsorted-ID, and overflow inputs.
5. Allocation conservation: sum(allocations) == distributable.
6. Authorization: only configured settlement authority may settle or change policy.
7. Replay protection: a period ID can settle at most once.
8. Event/readback consistency: persisted period totals and emitted events agree.
9. Upgrade and storage-expiry behavior are explicitly tested for the pinned Soroban SDK.
10. Contract results match every shared fixture byte-for-byte after canonical serialization.

## Release gate

Do not mark the engine implemented until the fixture tests run in CI for the pinned Soroban target and the CI run is green. A passing Python reference test is not evidence of Soroban parity, network deployment, or economic backing.
