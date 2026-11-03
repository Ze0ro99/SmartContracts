# Seven-Layer Contract Mapping

| Layer | Contract responsibilities | Verification gate |
|---|---|---|
| L1 Network and Settlement | Network-specific settlement interfaces and transaction references | Network ID, passphrase, RPC and finality verified |
| L2 Pi Cash and Payments | Payment intent and settlement state machine | Idempotency, refunds, replay protection, complete events |
| L3 Pi Bank and Treasury | Treasury ledger, reserve records, multisig operations | Segregated liabilities, reserve evidence and role tests |
| L4 Hybrid Assets and PiDEX | Asset registry, token interfaces, AMM/pool/router logic | Issuer verification, invariants and market safety |
| L5 Economic Intelligence | Fixed-point calculation libraries | Dimensional analysis, overflow and rounding properties |
| L6 Oracle and Interoperability | Oracle receiver, signed data, nonce/expiry and adapters | Quorum, freshness, deviation and replay tests |
| L7 Governance and Security | Roles, timelocks, emergency controls and proposal execution | Authorization matrix and adversarial tests |

Compilation alone does not establish production readiness. Each deployment needs a pinned specification, tested artifact hash, reviewed authority model and verified network deployment.
