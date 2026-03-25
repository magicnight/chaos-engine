// SPDX-License-Identifier: MIT
// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title ChaosPredictionMarket
 * @author ChaosDevOps@BKK&Estonia
 * @notice On-chain prediction market with LMSR pricing, powered by CHAOS Engine
 * @dev
 * - On-chain LMSR (Logarithmic Market Scoring Rule) — no off-chain trust
 * - Buy AND sell shares with slippage protection
 * - Pro-rata payout from deposited pool — contract always solvent
 * - Full event logging for off-chain indexing
 * - Market duration capped at MAX_MARKET_DURATION
 * - Per-market and per-tx share caps prevent overflow
 * - Emergency withdrawal for stuck tokens
 *
 * Security notes:
 * - Owner is a single EOA; for production, transfer to TimelockController or multisig
 * - Token pause() blocks claimWinnings — document this to users
 * - Pro-rata integer division may leave dust (< 1 wei per claimer)
 * - MEV/sandwich attacks mitigated by maxCost/minProceeds slippage params
 *
 * Part of the CHAOS Engine — Connected Human-Augmented OSINT Suite
 * https://github.com/magicnight/chaos-engine
 */
contract ChaosPredictionMarket is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    IERC20 public immutable token;

    uint256 public constant MAX_QUESTION_LENGTH = 500;
    uint256 public constant MAX_MARKET_DURATION = 365 days;
    uint256 public constant MAX_SHARES_PER_TX = 1_000_000 * 10 ** 18;
    uint256 public constant MAX_TOTAL_SHARES = 100_000_000 * 10 ** 18; // per side per market

    // LMSR liquidity parameter (scaled by 1e18 for fixed-point math)
    uint256 public constant LMSR_B = 100 * 10 ** 18;
    // Fixed-point scale
    uint256 private constant SCALE = 10 ** 18;

    enum MarketStatus { Open, Closed, ResolvedYes, ResolvedNo, Cancelled }
    enum Side { Yes, No }

    struct Market {
        string question;
        uint256 closeTime;
        MarketStatus status;
        uint256 yesShares; // total outstanding YES shares (scaled 1e18)
        uint256 noShares;  // total outstanding NO shares (scaled 1e18)
        uint256 totalDeposited;
        address creator;
    }

    struct Position {
        uint256 yesShares;
        uint256 noShares;
        uint256 totalCost;
        bool claimed;
    }

    uint256 public marketCount;
    mapping(uint256 => Market) public markets;
    mapping(uint256 => mapping(address => Position)) public positions;
    mapping(address => bool) public approvedCreators;

    event MarketCreated(uint256 indexed marketId, string question, uint256 closeTime, address indexed creator);
    event SharesPurchased(uint256 indexed marketId, address indexed trader, Side side, uint256 shares, uint256 cost);
    event SharesSold(uint256 indexed marketId, address indexed trader, Side side, uint256 shares, uint256 proceeds);
    event MarketClosed(uint256 indexed marketId);
    event MarketResolved(uint256 indexed marketId, MarketStatus result);
    event WinningsClaimed(uint256 indexed marketId, address indexed trader, uint256 payout);
    event MarketCancelled(uint256 indexed marketId);
    event CreatorApprovalChanged(address indexed creator, bool approved);
    event EmergencyWithdraw(address indexed token_, uint256 amount, address indexed to);

    constructor(address _token) Ownable(msg.sender) {
        require(_token != address(0), "Invalid token address");
        token = IERC20(_token);
    }

    // -----------------------------------------------------------------------
    // Modifiers
    // -----------------------------------------------------------------------

    modifier onlyCreator() {
        require(msg.sender == owner() || approvedCreators[msg.sender], "Not authorized");
        _;
    }

    modifier marketExists(uint256 marketId) {
        require(marketId < marketCount, "Market does not exist");
        _;
    }

    // -----------------------------------------------------------------------
    // On-chain LMSR Math (fixed-point, overflow-safe)
    // -----------------------------------------------------------------------

    /**
     * @dev Approximate exp(x) for x in fixed-point (scaled by SCALE).
     * Uses range reduction + Taylor 12 terms for high precision.
     * exp(x) = 2^k * exp(r) where r = x - k * ln2, |r| < ln2
     */
    function _expApprox(int256 x) internal pure returns (uint256) {
        if (x > 40 * int256(SCALE)) return type(uint256).max / 2;
        if (x < -40 * int256(SCALE)) return 1;

        int256 ln2 = 693147180559945309; // ln(2) * 1e18
        int256 k = x / ln2;
        int256 r = x - k * ln2;

        // Taylor series for exp(r) with 12 terms
        int256 s = int256(SCALE);
        int256 term = s;
        int256 result = s;

        for (uint256 i = 1; i <= 12; i++) {
            term = (term * r) / (int256(i) * s);
            result += term;
        }

        if (result <= 0) return 1;

        uint256 expR = uint256(result);
        if (k >= 0) {
            uint256 shift = uint256(k);
            if (shift > 80) return type(uint256).max / 2;
            expR = expR << shift;
        } else {
            uint256 shift = uint256(-k);
            if (shift > 80) return 1;
            expR = expR >> shift;
        }

        return expR > 0 ? expR : 1;
    }

    /**
     * @dev LMSR cost function: C(q) = b * ln(exp(qYes/b) + exp(qNo/b))
     * Fast-path: when both shares are 0, returns b * ln(2).
     */
    function _lmsrCostFunction(uint256 qYes, uint256 qNo) internal pure returns (uint256) {
        // Fast path: initial state avoids expensive exp/ln
        if (qYes == 0 && qNo == 0) {
            // b * ln(2) = LMSR_B * 0.693... = 100e18 * 693147180559945309 / 1e18
            return (LMSR_B * 693147180559945309) / SCALE;
        }

        int256 exponentYes = int256((qYes * SCALE) / LMSR_B);
        int256 exponentNo = int256((qNo * SCALE) / LMSR_B);

        uint256 expYes = _expApprox(exponentYes);
        uint256 expNo = _expApprox(exponentNo);

        uint256 sum = expYes + expNo;
        uint256 logResult = _lnApprox(sum);

        return (LMSR_B * logResult) / SCALE;
    }

    /**
     * @dev Approximate ln(x) where x is scaled by SCALE.
     * Range reduction to [SCALE, 2*SCALE) then artanh series.
     * Loop bounded to 128 iterations to prevent gas exhaustion.
     */
    function _lnApprox(uint256 x) internal pure returns (uint256) {
        require(x > 0, "ln(0) undefined");
        if (x == SCALE) return 0;

        uint256 result = 0;
        uint256 y = x;
        uint256 ln2 = 693147180559945309;

        uint256 maxIter = 128;
        for (uint256 i = 0; i < maxIter && y >= 2 * SCALE; i++) {
            y = y / 2;
            result += ln2;
        }
        for (uint256 i = 0; i < maxIter && y < SCALE; i++) {
            y = y * 2;
            if (result >= ln2) {
                result -= ln2;
            } else {
                return 0;
            }
        }

        // artanh series: ln(y/SCALE) = 2 * (t + t^3/3 + t^5/5 + t^7/7)
        uint256 t = ((y - SCALE) * SCALE) / (y + SCALE);
        uint256 t2 = (t * t) / SCALE;

        uint256 term = t;
        uint256 series = term;
        term = (term * t2) / SCALE;
        series += term / 3;
        term = (term * t2) / SCALE;
        series += term / 5;
        term = (term * t2) / SCALE;
        series += term / 7;

        result += 2 * series;
        return result;
    }

    /**
     * @dev Calculate the cost to buy `deltaShares` on `side`.
     */
    function calculateBuyCost(
        uint256 marketId,
        Side side,
        uint256 deltaShares
    ) public view marketExists(marketId) returns (uint256) {
        Market storage m = markets[marketId];
        uint256 costBefore = _lmsrCostFunction(m.yesShares, m.noShares);

        uint256 newYes = side == Side.Yes ? m.yesShares + deltaShares : m.yesShares;
        uint256 newNo = side == Side.No ? m.noShares + deltaShares : m.noShares;
        uint256 costAfter = _lmsrCostFunction(newYes, newNo);

        return costAfter > costBefore ? costAfter - costBefore : 0;
    }

    /**
     * @dev Calculate proceeds from selling `deltaShares` on `side`.
     */
    function calculateSellProceeds(
        uint256 marketId,
        Side side,
        uint256 deltaShares
    ) public view marketExists(marketId) returns (uint256) {
        Market storage m = markets[marketId];

        if (side == Side.Yes) {
            require(m.yesShares >= deltaShares, "Exceeds total YES shares");
        } else {
            require(m.noShares >= deltaShares, "Exceeds total NO shares");
        }

        uint256 costBefore = _lmsrCostFunction(m.yesShares, m.noShares);

        uint256 newYes = side == Side.Yes ? m.yesShares - deltaShares : m.yesShares;
        uint256 newNo = side == Side.No ? m.noShares - deltaShares : m.noShares;
        uint256 costAfter = _lmsrCostFunction(newYes, newNo);

        return costBefore > costAfter ? costBefore - costAfter : 0;
    }

    /**
     * @dev Current YES price (0 to SCALE).
     * Returns SCALE/2 (0.5) when both sides have 0 shares (fair starting price).
     */
    function getYesPrice(uint256 marketId) public view marketExists(marketId) returns (uint256) {
        Market storage m = markets[marketId];
        int256 diff = int256(m.yesShares) - int256(m.noShares);
        int256 scaled = (diff * int256(SCALE)) / int256(LMSR_B);

        uint256 expVal = _expApprox(scaled);
        return (expVal * SCALE) / (SCALE + expVal);
    }

    // -----------------------------------------------------------------------
    // Admin functions
    // -----------------------------------------------------------------------

    function setApprovedCreator(address creator, bool approved) external onlyOwner {
        require(creator != address(0), "Invalid address");
        approvedCreators[creator] = approved;
        emit CreatorApprovalChanged(creator, approved);
    }

    function createMarket(string calldata question, uint256 closeTime) external onlyCreator returns (uint256) {
        require(bytes(question).length > 0 && bytes(question).length <= MAX_QUESTION_LENGTH, "Invalid question length");
        require(closeTime > block.timestamp, "Close time must be in the future");
        require(closeTime <= block.timestamp + MAX_MARKET_DURATION, "Market duration too long");

        uint256 marketId = marketCount++;
        markets[marketId] = Market({
            question: question,
            closeTime: closeTime,
            status: MarketStatus.Open,
            yesShares: 0,
            noShares: 0,
            totalDeposited: 0,
            creator: msg.sender
        });

        emit MarketCreated(marketId, question, closeTime, msg.sender);
        return marketId;
    }

    function closeMarket(uint256 marketId) external onlyOwner marketExists(marketId) {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open, "Market not open");
        market.status = MarketStatus.Closed;
        emit MarketClosed(marketId);
    }

    function resolveMarket(uint256 marketId, bool yesWins) external onlyOwner marketExists(marketId) {
        Market storage market = markets[marketId];
        require(
            market.status == MarketStatus.Closed ||
            (market.status == MarketStatus.Open && block.timestamp >= market.closeTime),
            "Market must be closed or past close time"
        );

        market.status = yesWins ? MarketStatus.ResolvedYes : MarketStatus.ResolvedNo;
        emit MarketResolved(marketId, market.status);
    }

    function cancelMarket(uint256 marketId) external onlyOwner marketExists(marketId) {
        Market storage market = markets[marketId];
        require(
            market.status == MarketStatus.Open || market.status == MarketStatus.Closed,
            "Can only cancel open/closed markets"
        );
        market.status = MarketStatus.Cancelled;
        emit MarketCancelled(marketId);
    }

    /**
     * @notice Emergency: withdraw tokens accidentally sent to this contract.
     * @dev Cannot withdraw the primary market token while any market has deposits,
     *      to prevent rug-pulling active markets.
     */
    function emergencyWithdraw(address tokenAddr, uint256 amount, address to) external onlyOwner {
        require(to != address(0), "Invalid recipient");
        // If withdrawing the market token, only allow excess beyond all deposits
        if (tokenAddr == address(token)) {
            uint256 contractBalance = token.balanceOf(address(this));
            uint256 committed = _totalCommittedTokens();
            require(amount <= contractBalance - committed, "Cannot withdraw committed funds");
        }
        IERC20(tokenAddr).safeTransfer(to, amount);
        emit EmergencyWithdraw(tokenAddr, amount, to);
    }

    /**
     * @dev Sum of totalDeposited across all non-finalized markets.
     */
    function _totalCommittedTokens() internal view returns (uint256 total) {
        for (uint256 i = 0; i < marketCount; i++) {
            MarketStatus s = markets[i].status;
            if (s == MarketStatus.Open || s == MarketStatus.Closed ||
                s == MarketStatus.ResolvedYes || s == MarketStatus.ResolvedNo ||
                s == MarketStatus.Cancelled) {
                total += markets[i].totalDeposited;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Trading (on-chain LMSR pricing)
    // -----------------------------------------------------------------------

    /**
     * @notice Buy shares with on-chain LMSR pricing. maxCost provides slippage protection.
     */
    function buyShares(
        uint256 marketId,
        Side side,
        uint256 shares,
        uint256 maxCost
    ) external nonReentrant marketExists(marketId) {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open, "Market not open");
        require(block.timestamp < market.closeTime, "Market closed");
        require(shares > 0 && shares <= MAX_SHARES_PER_TX, "Invalid share amount");

        // Enforce per-market total shares cap
        if (side == Side.Yes) {
            require(market.yesShares + shares <= MAX_TOTAL_SHARES, "Exceeds market YES share cap");
        } else {
            require(market.noShares + shares <= MAX_TOTAL_SHARES, "Exceeds market NO share cap");
        }

        uint256 cost = calculateBuyCost(marketId, side, shares);
        require(cost > 0, "Cost must be > 0");
        require(cost <= maxCost, "Exceeds max cost (slippage)");

        token.safeTransferFrom(msg.sender, address(this), cost);

        if (side == Side.Yes) {
            market.yesShares += shares;
        } else {
            market.noShares += shares;
        }
        market.totalDeposited += cost;

        Position storage pos = positions[marketId][msg.sender];
        if (side == Side.Yes) {
            pos.yesShares += shares;
        } else {
            pos.noShares += shares;
        }
        pos.totalCost += cost;

        emit SharesPurchased(marketId, msg.sender, side, shares, cost);
    }

    /**
     * @notice Sell shares back to the market. minProceeds provides slippage protection.
     */
    function sellShares(
        uint256 marketId,
        Side side,
        uint256 shares,
        uint256 minProceeds
    ) external nonReentrant marketExists(marketId) {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open, "Market not open");
        require(block.timestamp < market.closeTime, "Market closed");
        require(shares > 0, "Must sell > 0 shares");

        Position storage pos = positions[marketId][msg.sender];
        if (side == Side.Yes) {
            require(pos.yesShares >= shares, "Insufficient YES shares");
        } else {
            require(pos.noShares >= shares, "Insufficient NO shares");
        }

        uint256 proceeds = calculateSellProceeds(marketId, side, shares);
        require(proceeds >= minProceeds, "Below min proceeds (slippage)");
        require(proceeds <= market.totalDeposited, "Insufficient market liquidity");

        if (side == Side.Yes) {
            market.yesShares -= shares;
            pos.yesShares -= shares;
        } else {
            market.noShares -= shares;
            pos.noShares -= shares;
        }
        market.totalDeposited -= proceeds;

        token.safeTransfer(msg.sender, proceeds);

        emit SharesSold(marketId, msg.sender, side, shares, proceeds);
    }

    // -----------------------------------------------------------------------
    // Resolution & Payout
    // -----------------------------------------------------------------------

    /**
     * @notice Claim winnings after resolution. Payout is pro-rata from totalDeposited.
     * @dev Integer division may leave dust (< 1 wei per claimer). This is by design —
     *      the alternative (tracking remaining pool) adds complexity and gas.
     */
    function claimWinnings(uint256 marketId) external nonReentrant marketExists(marketId) {
        Market storage market = markets[marketId];
        Position storage pos = positions[marketId][msg.sender];
        require(!pos.claimed, "Already claimed");

        uint256 payout = 0;

        if (market.status == MarketStatus.ResolvedYes) {
            require(pos.yesShares > 0, "No winning shares");
            payout = (pos.yesShares * market.totalDeposited) / market.yesShares;
        } else if (market.status == MarketStatus.ResolvedNo) {
            require(pos.noShares > 0, "No winning shares");
            payout = (pos.noShares * market.totalDeposited) / market.noShares;
        } else if (market.status == MarketStatus.Cancelled) {
            require(pos.totalCost > 0, "Nothing to refund");
            payout = pos.totalCost;
            if (payout > market.totalDeposited) {
                payout = market.totalDeposited;
            }
        } else {
            revert("Market not resolved or cancelled");
        }

        require(payout > 0, "Nothing to claim");

        pos.yesShares = 0;
        pos.noShares = 0;
        pos.totalCost = 0;
        pos.claimed = true;

        token.safeTransfer(msg.sender, payout);
        emit WinningsClaimed(marketId, msg.sender, payout);
    }

    // -----------------------------------------------------------------------
    // View functions
    // -----------------------------------------------------------------------

    function getPosition(uint256 marketId, address trader) external view marketExists(marketId) returns (
        uint256 yesShares, uint256 noShares, uint256 totalCost
    ) {
        Position storage pos = positions[marketId][trader];
        return (pos.yesShares, pos.noShares, pos.totalCost);
    }

    function getMarket(uint256 marketId) external view marketExists(marketId) returns (
        string memory question, uint256 closeTime, MarketStatus status,
        uint256 totalYesShares, uint256 totalNoShares, uint256 totalDeposited
    ) {
        Market storage m = markets[marketId];
        return (m.question, m.closeTime, m.status, m.yesShares, m.noShares, m.totalDeposited);
    }
}
