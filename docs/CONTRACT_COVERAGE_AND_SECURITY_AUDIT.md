# Smart Contract Coverage and Security Audit

## Repository facts
The default branch contains 106 Rust files, but Git blob analysis identifies only four distinct Rust source contents. In particular, one implementation body is repeated across 60 numbered contract files, another across 25, and a third across 20. The numbered directory structure therefore overstates implementation diversity.

## Findings from inspected source

### Identity template reuse
`contracts/pirc_1/src/lib.rs`, `contracts/pirc_2/src/lib.rs` and `contracts/pirc_101/src/lib.rs` share the same source blob. The code stores a DID string and metadata hash keyed by address, but does not define DID method semantics, document resolution, update/revocation policy, controller authorization, key rotation or schema validation.

### Generic asset template reuse
`contracts/pirc_207/src/lib.rs`, `contracts/pirc_214/src/lib.rs`, `contracts/pirc_225/src/lib.rs` and `contracts/pirc_260/src/lib.rs` share the same source blob. The generic code:
- stores an Admin value but does not use it to authorize registration;
- accepts arbitrary metadata strings without schema/hash validation;
- permits any authenticated caller to register assets;
- panics on a missing owner through `unwrap()`;
- has no lifecycle events, pause state, issuer verification or governance;
- does not implement oracle, proof-of-reserves or registry-v3 behavior.

### Build configuration
The root workspace uses `soroban-sdk = 22.0.11` and `contracts/*` members. The integration branch should preserve this SDK pin until compatibility tests justify an upgrade.

## New reference implementations on this integration branch

- `contracts/pirc_207_registry_v3`: admin-controlled asset metadata registry with one-time initialization and explicit registration/status controls.
- `contracts/pirc_214_oracle_v2`: reporter allowlist, authenticated updates, monotonic sequence, ledger timestamp and typed feed record. It is a single-reporter observation store, not yet a decentralized quorum oracle.
- `contracts/pirc_225_por_v2`: admin-controlled auditor allowlist and per-asset reserve/liability attestations in common atomic units. It records auditor attestations; it does not independently prove physical custody.

These are reference baselines requiring compilation, tests and independent review. They are not deployed and must not be used to make production financial claims.

## Required coverage workflow
For each standard, record: normative spec path and revision; acceptance criteria; contract crate; public interface; storage schema; authorization matrix; invariants; unit tests; integration tests; threat model; deployment profile; transaction ID; deployed code hash; verification timestamp.

A folder named `pirc_N` or a successful build does not by itself satisfy this checklist.
