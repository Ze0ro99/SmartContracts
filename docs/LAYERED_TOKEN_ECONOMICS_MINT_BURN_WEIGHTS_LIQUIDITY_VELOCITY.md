# Layered Token Economics: Mint, Burn, Weights, Liquidity, Velocity and Units

**Status:** PROPOSED DESIGN / TESTNET1 SPECIFICATION — NOT AN ISSUANCE AUTHORIZATION  
**Scope:** PiCash, PiBank, PiGovernance, PiYellow, PiOrange, Indigo, Purple, OUSD test asset, PiGold coordinator  
**Safety:** No Mainnet action. No deployment, initialization, mint, burn, transfer, swap, pool funding or parameter activation is authorized by this document.

## 1. Non-negotiable accounting invariants

1. Every fungible asset has a distinct contract instance and independent metadata, decimals, cap, total supply, balances, issuer policy, mint/burn policy and evidence record.
2. For asset i, total supply is updated only by successful mint/burn:
   `S_i(next) = S_i + M_i - H_i`.
3. Always enforce `0 <= S_i <= C_i`; mint and burn amounts must be positive atomic integers; all arithmetic is checked.
4. Transfers and AMM swaps do not mint or burn the asset. Pool reserves are balances held by the pool contract and must be independently reconciled against token balances.
5. Never net one asset's reserves, supply, liabilities or fees against another asset. A route is a sequence of conversions, not a merged balance sheet.
6. PiGold is a registry and route coordinator only. It is not a token, custodian, issuer, AMM, oracle or proof of liquidity.
7. GCV/reference values are not cash redemption promises, official Pi Network policy or market-price guarantees.

## 2. Atomic units and decimals

Each asset chooses an immutable decimal count d_i at initialization, with the current generic test asset template allowing 0 <= d_i <= 18. One whole token is 10^d_i atomic units.

- Human amount x -> atomic amount = x * 10^d_i, only if exactly representable.
- Atomic amount a -> human amount = a / 10^d_i.
- Contract state, caps, balances, reserves, fees, shares and calculations use integers only.
- UI/API may display decimal strings but must never use binary floating-point for money arithmetic.
- Never assume all layers use 7 or 18 decimals. OUSD test token currently uses 7 decimals; that is a property of its source implementation, not a global standard.
- Store and expose the asset ID, contract ID, symbol, decimals and network identity together. Reject cross-network or mismatched-decimal operations.

## 3. Mint policy by layer

Mint is permissioned and policy-gated. A contract admin authorization is necessary but not sufficient: the application release controller must also validate an approved issuance record, policy version, remaining cap and idempotency key before preparing a mint.

| Layer | Proposed issuance basis | Required controls |
|---|---|---|
| PiCash | Approved commerce/reward budget tied to completed eligible activity | Per-epoch budget, recipient eligibility, cap, duplicate-event protection |
| PiBank | Only against a precisely defined service unit or liability model; never assume a deposit is backing | Define whether it is a token or service first; segregate customer funds and liabilities; independent reconciliation |
| PiGovernance | Fixed governance allocation and/or bounded participation rewards | Voting-rights policy, transferability/delegation rules, anti-sybil controls, epoch cap |
| PiYellow | Verified productive activity or utility delivery | Evidence ID, eligibility score, issuance budget and appeal/reversal process |
| PiOrange | Verified contribution/output under its own reward schedule | Evidence ID, contribution class, epoch cap and duplicate protection |
| Indigo | Independent allocation/reward schedule for Indigo utility | Separate issuer, cap, pool and accounting namespace |
| Purple | Independent allocation/reward schedule for Purple utility | Separate issuer, cap, pool and accounting namespace |
| OUSD test asset | Test-only issuance within its fixed source cap | No USD peg, reserve, redemption or backing claims |
| PiGold | No mint: coordinator only | Reject token issuance semantics |

A generic issuance budget can be calculated as:

`M_i,t = min(C_i - S_i,t, floor(B_i,t))`

where `B_i,t` is the approved epoch budget in atomic units. The budget may be derived from verified revenue and eligible utility:

`B_i,t = floor(alpha_i * R_i,t + beta_i * U_i,t)`

Here `R_i,t` is verified net distributable revenue allocated to the layer, `U_i,t` is independently verified eligible utility/contribution units, and `alpha_i,beta_i` are governance-approved coefficients. They MUST remain unset/disabled until the layer's unit, evidence source, budget period and coefficients are approved. Do not infer revenue from token price or raw trading volume.

Every issuance record must include asset/contract ID, epoch, policy version, amount, recipient, evidence references, approver, idempotency key and resulting supply read-back.

## 4. Burn policy by layer

Burn is irreversible supply reduction, not a transfer to a dead address. Only the holder or an explicitly approved allowance-based spender can burn. A protocol fee burn must be explicitly configured per asset and must not silently burn user principal.

Proposed burn sources:
- PiCash: optional portion of explicitly disclosed commerce/settlement fees.
- PiBank: only a disclosed service-unit consumption or fee, after the token/service decision is resolved.
- PiGovernance: no automatic burn for voting, abstention or proposal outcomes; only a separately approved policy.
- PiYellow and PiOrange: disclosed service/usage fees or expiry of non-transferable reward units only if the contract supports safe expiry semantics.
- Indigo and Purple: separate fee/usage burns within their own ledgers.
- OUSD test asset: test-only burn; no redemption inference.
- PiGold: no token burn.

For every burn:
`H_i,t = min(S_i,t, approved_burn_i,t)`
`S_i,t+1 = S_i,t - H_i,t`

Burn records include asset ID, source account, amount, reason code, policy version, transaction/event ID and post-burn supply. Never burn based on a UI estimate or a price oracle alone.

## 5. Normalized economic weights

Weights allocate a defined reward/fee/utility budget; they do not create money or guarantee market value. Calculate a score for each eligible participant or layer using bounded, independently measured inputs:

`raw_i = wU*U_i + wC*C_i + wL*L_i + wP*P_i - wR*Risk_i`

`weight_i = raw_i / sum(raw_j)` for eligible entries with positive raw scores.

Suggested dimensions:
- U: verified utility delivered / service consumption.
- C: verified contribution or productive work.
- L: time-weighted, independently reconciled liquidity depth, not a momentary deposit.
- P: provenance/identity/quality eligibility score.
- Risk: concentration, wash activity, oracle uncertainty, default or operational risk.

All inputs are normalized to documented [0,1] ranges before scoring. Coefficients must be non-negative, versioned and approved; risk coefficient is non-negative and subtractive. If denominator is zero, distribute nothing and return a controlled no-eligible-weight result. Apply per-account/per-layer caps and anti-sybil/concentration limits before normalization. Keep weights separate by layer; do not use one layer's score as another layer's balance.

## 6. Liquidity and AMM invariants

The current PiDEX AMM source uses a constant-product pool and a 30 basis-point swap fee. This is a source-code setting, not evidence of a deployed pool or an approved universal fee for every future pool.

For a pool with reserves x and y:
`k = x*y`
`amountInAfterFee = floor(amountIn*(10000-feeBps)/10000)`
`amountOut = floor(y*amountInAfterFee/(x+amountInAfterFee))`

The implementation must preserve checked integer arithmetic, positive reserves, min_out/slippage protection, and token transfers that match the reserve update. For every pool:
- pool identity is the ordered pair of contract IDs and network;
- reserve A and reserve B are independent atomic amounts with their own decimals;
- reconcile reserve state to actual token balances before enabling a route;
- LP shares and total shares are pool-specific;
- withdrawal is pro-rata and cannot exceed owned shares;
- pool fee, protocol fee, fee recipient and any burn split are separately configured;
- never count the same reserve in two pools or two accounting namespaces;
- disable routing on mismatch, stale oracle, paused pool or failed read-back.

PiGold may validate route connectivity and uniqueness, but it does not calculate custody balances or prove reserves. A route planner must read each AMM pool and verify every hop independently.

## 7. Market velocity and turnover

For asset i over a fixed interval T:
`V_i,T = eligible_transfer_volume_i,T / average_circulating_supply_i,T`

Annualized velocity (only if interval T is measured in days):
`V_i,annual = V_i,T * 365 / T_days`

Market turnover ratio:
`Turnover_i,T = eligible_market_trade_volume_i,T / average_circulating_market_value_i,T`

Report separately:
- on-chain transfer velocity;
- DEX swap turnover;
- CEX volume (only from identified, independently sourced venues);
- commerce/utility settlement volume;
- self-transfers, wash-trade suspects and pool-internal movements excluded or separately labelled.

Never add the same swap to both transfer and trade volume. Report supply denominator, time window, exclusions, data source and confidence. Velocity is a diagnostic metric, not a mint trigger by itself. A high velocity can reflect useful circulation or wash activity; a low velocity can reflect long-term holding or inactive utility.

## 8. Reference rates and oracle controls

The system must distinguish:
- internal reference/accounting unit;
- external market quote;
- executable pool quote after fee and slippage;
- actual realized settlement price.

Do not equate a GCV/reference coordinate with redemption value. Do not use a single venue or a single oracle reporter for issuance budgets. Require freshness windows, quorum/median, deviation bounds, source independence, signed observation IDs and circuit breakers. On missing/stale/conflicting feeds, freeze price-dependent calculations and keep ordinary token transfers governed by their own contract rules.

The previously discussed 10^7 multiplier and GCV figures are hypotheses in the economic model, not universal contract constants. Keep them outside token supply, AMM reserve and mint arithmetic unless a separately approved specification defines a bounded use.

## 9. Parameter registry: deliberately unset until approved

For each asset, the registry must contain:
- canonical name/symbol and purpose;
- contract ID and network passphrase;
- decimals d_i;
- maximum supply C_i in atomic units;
- current supply S_i read from chain;
- issuer/admin and rotation/recovery policy;
- issuance coefficients alpha_i/beta_i and epoch budget;
- burn fee/usage coefficients and fee split;
- weight coefficients and input normalization rules;
- pool IDs, pair assets, feeBps and reserve-accounting IDs;
- velocity/turnover measurement window and exclusion policy;
- oracle quorum, freshness and deviation limits;
- policy version, approval evidence and effective ledger/time.

Unknown values must remain null/disabled, not be filled with guessed constants. No mint-enabled status until all mandatory fields and authorization are verified.

## 10. Required test vectors

Implement unit/property/integration tests for:
1. Mint exactly to cap succeeds; one atomic unit over cap fails.
2. Mint zero, negative and overflow amounts fail without state change.
3. Unauthorized mint and unauthorized burn fail.
4. Burn cannot exceed holder balance or total supply.
5. burn_from requires valid unexpired allowance and reduces allowance exactly.
6. Independent token instances cannot share supply or metadata.
7. Decimal conversion round-trips exactly; excess precision is rejected.
8. Weight normalization sums to 1 within integer rounding policy; zero denominator allocates zero.
9. Negative/out-of-range weights and stale evidence are rejected.
10. AMM quote matches swap arithmetic exactly; min_out failure leaves all state unchanged.
11. Pool reserve read-back equals actual token contract balances.
12. Duplicate pool/reserve accounting IDs and disconnected routes are rejected.
13. Velocity excludes documented self/wash/duplicate volume and uses time-weighted average supply.
14. Stale/conflicting oracle observations trip the circuit breaker.
15. Idempotent issuance requests cannot mint twice.

## 11. Release gates

- SOURCE_ONLY: source exists.
- BUILT_TESTED: pinned source builds and tests pass.
- DEPLOYED_UNINITIALIZED: Testnet1 deployment and code hash verified.
- INITIALIZED_UNMINTED: metadata/admin/decimals/cap read back; supply is zero.
- MINTED_TESTNET: explicitly approved test mint and independent event/supply/balance read-back.
- POOLS_VERIFIED: actual token balances equal pool reserves and pool identity is verified.
- INTEGRATED_TESTNET: wallet, registry, API, indexer and route tests use the same Testnet1 network identity.

This specification is design guidance only. It does not claim these stages have been completed or authorize any transaction.
