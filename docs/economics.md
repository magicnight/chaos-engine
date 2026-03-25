# CHAOS Prediction Market — Economic Model

## Overview

The CHAOS prediction market uses **LMSR (Logarithmic Market Scoring Rule)**, an automated market maker algorithm invented by Robin Hanson, for fully on-chain pricing. All pricing, trading, and payouts are automatic — no order book, no human market maker.

---

## Core LMSR Formulas

```
Cost Function:  C(qYes, qNo) = b × ln(exp(qYes/b) + exp(qNo/b))

Buy Cost     = C(state_after) - C(state_before)
Sell Proceeds = C(state_before) - C(state_after)

YES Price    = exp(qYes/b) / (exp(qYes/b) + exp(qNo/b))
             = 1 / (1 + exp(-(qYes - qNo) / b))    ← sigmoid function
```

### Parameters

| Parameter | Contract Value | Meaning |
|-----------|---------------|---------|
| `qYes` | `market.yesShares` | Total outstanding YES shares |
| `qNo` | `market.noShares` | Total outstanding NO shares |
| `b` | `LMSR_B = 100e18` | **Liquidity parameter** — larger = more stable prices, harder to manipulate |

---

## How Prices Move

```
Initial:      qYes=0,   qNo=0    → YES = 50.0%,  NO = 50.0%
After buy YES: qYes=50,  qNo=0    → YES = 62.2%,  NO = 37.8%
More YES:     qYes=200, qNo=0    → YES = 88.1%,  NO = 11.9%
```

**Buy more → price rises → subsequent buys cost more.** This is the LMSR auto-pricing mechanism.

---

## Token Flow

```
User Wallet (CHAOS tokens)
    │
    ├─ buyShares()  ──→ tokens into contract ──→ totalDeposited increases
    │                                            yesShares/noShares increases
    │                                            price auto-adjusts up
    │
    ├─ sellShares() ←── tokens from contract ←── totalDeposited decreases
    │                                             yesShares/noShares decreases
    │                                             price auto-adjusts down
    │
    └─ claimWinnings() ←── tokens from contract ←── pro-rata from pool
```

---

## Market Lifecycle

```
Create (Open) → Trading → Close (Closed) → Resolve (Resolved) → Claim
     │              │           │              │                   │
     │ CHAOS sweep  │ closeTime │ owner calls  │ users call        │
     │ auto-creates │ expires   │ resolveMarket│ claimWinnings     │
     │              │           │              │                   │
     │ buyShares()  │           │ Cancel path: │                   │
     │ sellShares() │           │ cancelMarket │                   │
     │              │           │ → full refund │                   │
```

---

## Automation Level

| Step | Automatic? | Mechanism |
|------|-----------|-----------|
| **Market creation** | Yes | CHAOS sweep → 12 rules + LLM → `/api/market-seeds` → DB |
| **Pricing** | Yes | On-chain LMSR, zero human intervention |
| **Price movement** | Yes | Auto-adjusts after every trade |
| **Market close** | Yes | `closeTime` expiry blocks new trades |
| **Market resolution** | Semi-auto | `/api/auto-resolve` checks CHAOS data, triggered by SSE sweep-hook |
| **Payout** | User-triggered | User calls `claimWinnings()`, calculation fully automatic |

---

## Worked Example

```
Market: "Will BTC exceed $75K this week?"

1. Initial state
   pool = 0 CHAOS, YES = 50%, NO = 50%

2. Alice buys 100 YES shares
   LMSR cost = 54.3 CHAOS
   pool = 54.3, YES = 62%, NO = 38%

3. Bob buys 200 NO shares
   LMSR cost = 92.1 CHAOS
   pool = 146.4, YES = 45%, NO = 55%

4. Carol buys 50 YES shares
   LMSR cost = 21.8 CHAOS
   pool = 168.2, YES = 48%, NO = 52%

5. Alice sells 50 YES shares (changes her mind)
   LMSR proceeds = 24.1 CHAOS
   pool = 144.1, YES = 44%, NO = 56%

6. Expiry: CHAOS data shows BTC = $72K → Resolved NO

7. Payout:
   Bob holds 200 NO shares (total NO = 200)
   payout = (200/200) × 144.1 = 144.1 CHAOS ← Bob takes entire pool

   Alice: 50 remaining YES shares → lost
   Carol: 50 YES shares → lost
```

---

## Pricing Economics

### 1. Buy-Sell Spread (Market Maker Profit)

```
Buy 100 shares:  cost ≈ 54 CHAOS
Sell 100 shares: proceeds ≈ 53.8 CHAOS
Spread ≈ 0.2 CHAOS (slippage/spread)
```

### 2. Non-Linear Cost for Large Orders (Manipulation Resistance)

```
Buy 10 shares:   cost ≈ 5 CHAOS    (avg price 0.50)
Buy 100 shares:  cost ≈ 54 CHAOS   (avg price 0.54)
Buy 1000 shares: cost ≈ 762 CHAOS  (avg price 0.76)
→ Whales cannot cheaply manipulate prices
```

### 3. Infinite Liquidity

Even at 99% price, buying is still possible — just extremely expensive (approaching 1:1 token per share).

---

## Liquidity Parameter `b = 100`

- **Maximum market maker loss** = `b × ln(2)` ≈ 69.3 tokens
- This is the "cost" of providing initial liquidity to the market
- Larger `b` → more stable prices, harder to manipulate with single trades
- Current `b=100e18` is suitable for small-to-medium prediction markets

---

## Dual Trading System

NewsPredict runs **two parallel trading systems**:

```
┌─────────────────────────────────────────────┐
│         NewsPredict Trading Layer            │
├─────────────────────┬───────────────────────┤
│   Virtual Trading    │   On-Chain Trading     │
│                     │                       │
│ POST /api/trades    │ buyShares() on-chain   │
│ trade-executor.ts   │ ChaosPredictionMarket  │
│ PostgreSQL records  │ BSC blockchain records  │
│ Virtual balance ($) │ CHAOS tokens (real)     │
│ LMSR (TypeScript)   │ LMSR (Solidity)        │
│                     │                       │
│ Purpose: free trial │ Purpose: real stakes    │
│ No wallet needed    │ Web3 wallet required    │
└─────────────────────┴───────────────────────┘

OrderPanel UI toggle: [Virtual Credits] [On-Chain (CHAOS)]
```

---

## Revenue Model

The current contracts have **zero built-in fees**. All tokens flow between users. Revenue options for the future:

| Approach | Complexity | Description |
|----------|-----------|-------------|
| Trade fee | Low | 0.5-2% on buyShares/sellShares to treasury |
| Creation fee | Low | Require deposit to createMarket |
| Resolution fee | Low | Small deduction on claimWinnings |
| Token deflation | Built-in | ChaosToken is burnable, reducing supply |

Currently running **zero-fee model** for early user acquisition.

---

## Solvency Guarantee

The pro-rata payout model ensures the contract **never becomes insolvent**:

```
Total payouts = Σ (user_shares / total_winning_shares × totalDeposited)
             = (totalDeposited / total_winning_shares) × Σ user_shares
             = (totalDeposited / total_winning_shares) × total_winning_shares
             = totalDeposited ✓
```

The contract always holds exactly enough tokens to pay all winners.

---

## Smart Contract Security

Four rounds of security audits completed. Key protections:

| Protection | Implementation |
|-----------|---------------|
| Reentrancy | `nonReentrant` on all state-changing functions |
| Integer overflow | Solidity 0.8.24 built-in + `MAX_TOTAL_SHARES` cap |
| Price manipulation | On-chain LMSR, no off-chain trust |
| Double-claim | `claimed` flag per position |
| Slippage | `maxCost` (buy) / `minProceeds` (sell) |
| Gas DoS | Bounded loops (128 max iterations) |
| Emergency | `emergencyWithdraw` with committed-funds protection |
| Pausable | Token-level pause for emergencies |
