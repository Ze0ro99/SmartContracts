# Smart Contract Security Test Matrix

This matrix defines minimum tests for reference contracts. A row marked REQUIRED is a release blocker until a passing test is linked to the exact commit.

| Area | Threat / failure | Required assertion | Evidence |
|---|---|---|---|
| Initialization | Admin takeover by reinitialization | Second initialize fails | Unit test |
| Authorization | Unauthorized caller changes state | Operation rejected; state unchanged | Unit test |
| Identity | Caller overwrites another identity | Owner/controller policy enforced | Unit test |
| Registry | Duplicate asset ID | Registration rejected | Unit test |
| Registry | Unbounded or malformed metadata | Hash/size/schema policy enforced | Unit test |
| Registry | Issuer spoofing | Issuer authorization and evidence checked | Unit test |
| Asset transfer | Missing owner causes panic | Typed error; no panic | Unit test |
| Oracle | Unauthorized reporter | Observation rejected | Unit test |
| Oracle | Replayed or stale sequence | Observation rejected | Unit test |
| Oracle | Stale/future observation | Feed freshness policy enforced | Unit + integration test |
| Oracle | One reporter masquerades as quorum | Quorum status not asserted without distinct sources | Integration test |
| Reserve | Unauthorized auditor | Attestation rejected | Unit test |
| Reserve | Old report presented as current | Freshness and expiry enforced | Unit/integration test |
| Reserve | Reserve and liability use different units | Attestation rejected | Unit test |
| Settlement | Deductions exceed revenue | Underfunded period rejected | Unit test |
| Settlement | Duplicate period submission | Replay rejected | Unit test |
| Settlement | Allocation rounding creates/destroys units | Sum allocations equals distributable | Property test |
| Settlement | Client controls distributable | Contract derives/checks authoritative inputs | Integration test |
| Storage | TTL expiry removes critical state | TTL renewal or explicit expiry behavior tested | Testnet/local test |
| Upgrade | Storage layout changes incompatibly | Migration or version rejection tested | Upgrade test |
| Events | Event differs from stored state | Event/readback reconciliation passes | Integration test |
| Arithmetic | Overflow or precision truncation | Checked arithmetic / typed error | Unit test |
| Operations | Unreviewed crate deployed | Explicit release allowlist required | Workflow test |

## Required test conventions

- Each test names the invariant it protects.
- Negative tests assert state remains unchanged.
- Do not use `mock_all_auths` as the only authorization test; include real auth-context or explicit unauthorized cases.
- Property tests should vary weights and amounts and assert conservation, monotonicity, and deterministic output.
- Pin SDK and toolchain versions. Capture compiler and test output in CI.
- A local unit test does not establish deployed behavior.
- A successful WASM build does not establish security, correctness, or economic backing.

## Current reference-module limitations

- Oracle V2 currently provides an authenticated single-reporter observation record with monotonic sequence; quorum aggregation and signed envelope verification remain separate work.
- Proof-of-Reserves V2 stores auditor attestations; it does not independently verify custody.
- Registry V3 provides an admin-controlled metadata record; it does not itself establish that an issuer or external asset is legitimate.
- Generic numbered contract templates are not evidence of standard-specific implementation.
