// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.

// ChaosToken ABI
export const CHAOS_TOKEN_ABI = [
  'function name() view returns (string)',
  'function symbol() view returns (string)',
  'function decimals() view returns (uint8)',
  'function totalSupply() view returns (uint256)',
  'function MAX_SUPPLY() view returns (uint256)',
  'function balanceOf(address account) view returns (uint256)',
  'function approve(address spender, uint256 amount) returns (bool)',
  'function allowance(address owner, address spender) view returns (uint256)',
  'function transfer(address to, uint256 amount) returns (bool)',
  'function owner() view returns (address)',
  'function paused() view returns (bool)',
] as const;

// ChaosPredictionMarket ABI
export const CHAOS_MARKET_ABI = [
  // Trading
  'function buyShares(uint256 marketId, uint8 side, uint256 shares, uint256 maxCost)',
  'function sellShares(uint256 marketId, uint8 side, uint256 shares, uint256 minProceeds)',
  'function claimWinnings(uint256 marketId)',
  // View - pricing
  'function calculateBuyCost(uint256 marketId, uint8 side, uint256 deltaShares) view returns (uint256)',
  'function calculateSellProceeds(uint256 marketId, uint8 side, uint256 deltaShares) view returns (uint256)',
  'function getYesPrice(uint256 marketId) view returns (uint256)',
  // View - state
  'function getPosition(uint256 marketId, address trader) view returns (uint256 yesShares, uint256 noShares, uint256 totalCost)',
  'function getMarket(uint256 marketId) view returns (string question, uint256 closeTime, uint8 status, uint256 totalYesShares, uint256 totalNoShares, uint256 totalDeposited)',
  'function marketCount() view returns (uint256)',
  'function token() view returns (address)',
  'function owner() view returns (address)',
  // Constants
  'function LMSR_B() view returns (uint256)',
  'function MAX_TOTAL_SHARES() view returns (uint256)',
  'function MAX_SHARES_PER_TX() view returns (uint256)',
  // Events
  'event SharesPurchased(uint256 indexed marketId, address indexed trader, uint8 side, uint256 shares, uint256 cost)',
  'event SharesSold(uint256 indexed marketId, address indexed trader, uint8 side, uint256 shares, uint256 proceeds)',
  'event WinningsClaimed(uint256 indexed marketId, address indexed trader, uint256 payout)',
  'event MarketCreated(uint256 indexed marketId, string question, uint256 closeTime, address indexed creator)',
  'event MarketResolved(uint256 indexed marketId, uint8 result)',
  'event MarketClosed(uint256 indexed marketId)',
  'event MarketCancelled(uint256 indexed marketId)',
] as const;

// Contract addresses from deployment
export const CONTRACTS = {
  // BSC Testnet (chainId: 97)
  97: {
    token: process.env.NEXT_PUBLIC_CHAOS_TOKEN_TESTNET || '0x96be44c3CC84C05B76db83c18bd3d7bA538a433d',
    market: process.env.NEXT_PUBLIC_MARKET_CONTRACT_TESTNET || '0x244d85a0a174Ef420AF1b7133c475219C3e9429b',
  },
  // BSC Mainnet (chainId: 56)
  56: {
    token: process.env.NEXT_PUBLIC_CHAOS_TOKEN_MAINNET || '0xcE3fbb08D72BEd7F645F59FE0f031659b5B298c4',
    market: process.env.NEXT_PUBLIC_MARKET_CONTRACT_MAINNET || '0xAa7208Cf64078756fB58698fbE748DC3c9b4Cb88',
  },
} as Record<number, { token: string; market: string }>;
