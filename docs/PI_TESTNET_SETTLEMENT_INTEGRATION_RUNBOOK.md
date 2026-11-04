# Pi Testnet Settlement Integration Runbook

## Purpose

This runbook deploys only the canonical pirc_economic_settlement Soroban contract to Pi Testnet and exercises its accounting state machine with Test Pi network fees.

The workflow is manual-only. It does not deploy to Pi Mainnet, transfer real Pi, redeem assets, or assert any market value. The accounting values passed to the contract are synthetic test units.

## Required setup

1. Open the GitHub repository Environment named pi-testnet (or create it).
2. Add an Environment secret named PI_TESTNET_DEPLOYER_SECRET containing a dedicated Pi Testnet deployer secret. Never use a Mainnet wallet secret.
3. Fund that test identity with Test Pi through the Pi Testnet wallet/faucet. Test Pi has no monetary value.
4. Prefer enabling required reviewers for the pi-testnet Environment.
5. Confirm the Horizon root reports the Pi Testnet network passphrase and the RPC health/latest-ledger probes succeed before deployment.

## Run

1. Open Actions → Pi Testnet Settlement Integration (Manual).
2. Select Run workflow.
3. Set confirm_pi_testnet_deploy to true only when a Testnet deployment is intended.
4. Review the Environment approval if protection is enabled.

## What the workflow checks

- Builds and runs unit tests for the canonical settlement contract.
- Verifies Pi Testnet RPC health and latest-ledger read access.
- Deploys the WASM contract to Pi Testnet.
- Initializes the contract and enables a temporary test operator.
- Proposes and approves a synthetic accounting period.
- Records a conserved allocation.
- Confirms duplicate allocations and over-allocation are rejected.
- Confirms double settlement and post-settlement mutation are rejected.
- Revokes the operator and confirms further proposals are rejected.
- Reads back the settled period, allocation and deployed contract instance.

## Network separation

- Pi Testnet: the target for this integration workflow.
- Stellar Testnet: used separately for Soroban compatibility tests and reference behavior.
- Pi Mainnet: never targeted by this workflow.

A successful run is evidence of a specific test deployment and state read-back on Pi Testnet. It is not a production audit, proof of reserves, token redemption, or Pi Core Team approval.
