# Contract Capability and Evidence Matrix

| Capability | Current source evidence | What it proves | What it does not prove | Gate |
|---|---|---|---|---|
| OUSD test token | `contracts/ousd_test_token` | Test token interface, admin mint/burn, fixed cap, 7 decimals | USD peg, reserve, redemption, stable value, deployment | Keep test-only; design issuer/reserve/liability model before production |
| Oracle V2 | `contracts/pirc_214_oracle_v2` | Authorized reporters, monotonic sequence, timestamped latest record | Independent multi-source quorum, Chainlink integration, manipulation resistance | Add source registry, quorum aggregation, heartbeat/deviation and tests |
| AI Oracle | `contracts/pirc_237_ai_oracle` | Allowlisted updater, bounded scores, sequence, model version and proof hash | Correct model inference, ZK proof verification, decentralized AI consensus | Treat hash as commitment; add verifiable evidence and model governance |
| PiDEX AMM | `contracts/pidex_amm` (Unity Sphere adapter/reference) | Soroban AMM reference implementation exists | Production liquidity, audited economics, CEX connection, peg | Verify code, tests, token auth, fixed-point arithmetic and deployment evidence |
| RAW Factory | PiRC `omni-factory-release` branch | Minimal factory scaffold emits event and returns product ID | Child contract deployment, registry finality, code hash verification | Implement deploy/install/initialize/read-back lifecycle |
| Physical possession | PiRC `omni-factory-release` branch | Interface accepts a user and signature-shaped value | Signature validation, device trust, title or physical possession | Replace unconditional success with cryptographic verification |
| PQ migration | PiRC/Unity `quantum-legacy` paths | Research/legacy hooks and readiness documentation | Production PQ algorithm support or ledger-wide quantum resistance | Crypto-agility plan and runtime compatibility proof |

## Status vocabulary

- `DOCUMENTED`: written specification exists.
- `SCAFFOLD`: code skeleton or placeholder exists.
- `SIMULATED`: local/mock behavior only.
- `TESTED`: tests pass in a named environment and commit.
- `TESTNET_VERIFIED`: deployed and read back on a named Testnet.
- `AUDITED`: independent security audit with scope and commit.
- `PRODUCTION_VERIFIED`: live deployment, ownership, code hash, monitoring and read-back evidence all verified.

Never promote a capability status based only on a branch name, file path, README claim, generated artifact or mocked test.
