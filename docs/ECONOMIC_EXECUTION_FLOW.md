# Soroban Economic Execution Flow

## Authorized settlement

```mermaid
sequenceDiagram
  participant API as Unity Sphere API
  participant POL as Policy/Evidence Verifier
  participant C as Soroban Contract
  participant N as Network
  API->>POL: Validate identity, asset, network, revenue, oracle evidence
  POL-->>API: Signed policy decision + evidence hash
  API->>C: Submit operation ID, period, amounts, policy hash
  C->>C: Authenticate caller and validate state machine
  C->>C: Checked integer arithmetic and invariant checks
  C->>N: Commit authorized state transition
  N-->>C: Finalized transaction
  C-->>API: Event and transaction reference
  API->>N: Read back state and supply/balance
  API-->>API: Reconcile expected vs observed state
```

## Supply-operation gate

```mermaid
flowchart TD
  A[Mint / burn / lock / release] --> B[Resolve exact asset + network]
  B --> C[Authenticate caller]
  C --> D[Check role and operation permission]
  D --> E[Check positive atomic amount and bounds]
  E --> F[Check cap / balance / liability / expiry]
  F --> G[Apply atomic state update]
  G --> H[Emit event]
  H --> I[Read-back and reconcile]
  B -->|Mismatch| X[Reject]
  C -->|Unauthorized| X
  D -->|Denied| X
  E -->|Invalid| X
  F -->|Invariant failure| X
```

## Period close and allocation

```mermaid
flowchart LR
  A[Collected revenue evidence] --> B[Costs, reserves, taxes, contingencies]
  B --> C[Positive surplus check]
  C --> D[Liquidity factor]
  D --> E[Eligibility snapshot]
  E --> F[Contribution weights and caps]
  F --> G[Integer allocation]
  G --> H[Independent approval]
  H --> I[Idempotent settlement]
  I --> J[Event + read-back]
```

No deployment, mint, burn or transfer is authorized by these diagrams. They specify the controls expected in implementation.

## PiRC-101 live market reference

The Soroban reference crate `contracts/pirc_101_monetary` consumes a typed median from `pirc_214_oracle_quorum_v3` and calculates `IPPR = median(PI/USD) * 10,000,000` on each snapshot. QWF is stored independently and changes only through an authorized, versioned adjustment with evidence hash, velocity and TVL inputs. It computes `Phi = (L_n * IPPR) / QWF` using integer arithmetic. The controller is reference-only: no REF mint, credit expansion, wallet conversion or payment is performed.
