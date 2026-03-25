// Contract ABIs (simplified for frontend interaction)
export const CRUX_TOKEN_ABI = [
  'function balanceOf(address account) view returns (uint256)',
  'function approve(address spender, uint256 amount) returns (bool)',
  'function allowance(address owner, address spender) view returns (uint256)',
  'function transfer(address to, uint256 amount) returns (bool)',
  'function decimals() view returns (uint8)',
  'function symbol() view returns (string)',
] as const;

export const PREDICTION_MARKET_ABI = [
  'function buyShares(uint256 marketId, uint8 side, uint256 shares, uint256 maxCost)',
  'function claimWinnings(uint256 marketId)',
  'function getPosition(uint256 marketId, address trader) view returns (uint256 yesShares, uint256 noShares, uint256 totalCost)',
  'function getMarket(uint256 marketId) view returns (string question, uint256 closeTime, uint8 status, uint256 totalYesShares, uint256 totalNoShares, uint256 totalDeposited)',
  'event SharesPurchased(uint256 indexed marketId, address indexed trader, uint8 side, uint256 shares, uint256 cost)',
  'event WinningsClaimed(uint256 indexed marketId, address indexed trader, uint256 payout)',
] as const;

// Contract addresses (set after deployment)
export const CONTRACTS = {
  // BSC Testnet
  97: {
    token: process.env.NEXT_PUBLIC_CRUX_TOKEN_TESTNET || '',
    market: process.env.NEXT_PUBLIC_MARKET_CONTRACT_TESTNET || '',
  },
  // BSC Mainnet — using existing deployed token
  56: {
    token: process.env.NEXT_PUBLIC_CRUX_TOKEN_MAINNET || '0x5Bde264AD913192929f71c2A5253440fd01CBdf1',
    market: process.env.NEXT_PUBLIC_MARKET_CONTRACT_MAINNET || '',
  },
} as Record<number, { token: string; market: string }>;
