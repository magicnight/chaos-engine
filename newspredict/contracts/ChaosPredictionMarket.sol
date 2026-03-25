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
 * @notice On-chain prediction market powered by CHAOS Engine OSINT intelligence
 * @dev
 * - Markets created by owner (system) or approved creators
 * - LMSR pricing computed off-chain, shares tracked on-chain
 * - Resolution by owner with automated payout
 * - Uses CHAOS token (BEP-20) for all transactions
 *
 * Part of the CHAOS Engine — Connected Human-Augmented OSINT Suite
 * https://github.com/magicnight/chaos-engine
 */
contract ChaosPredictionMarket is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    IERC20 public token;

    enum MarketStatus { Open, Closed, ResolvedYes, ResolvedNo, Cancelled }
    enum Side { Yes, No }

    struct Market {
        string question;
        uint256 closeTime;
        MarketStatus status;
        uint256 totalYesShares;
        uint256 totalNoShares;
        uint256 totalDeposited;
        address creator;
    }

    struct Position {
        uint256 yesShares;
        uint256 noShares;
        uint256 totalCost;
    }

    uint256 public marketCount;
    mapping(uint256 => Market) public markets;
    mapping(uint256 => mapping(address => Position)) public positions;
    mapping(address => bool) public approvedCreators;

    event MarketCreated(uint256 indexed marketId, string question, uint256 closeTime, address creator);
    event SharesPurchased(uint256 indexed marketId, address indexed trader, Side side, uint256 shares, uint256 cost);
    event MarketResolved(uint256 indexed marketId, MarketStatus result);
    event WinningsClaimed(uint256 indexed marketId, address indexed trader, uint256 payout);
    event MarketCancelled(uint256 indexed marketId);

    constructor(address _token) Ownable(msg.sender) {
        token = IERC20(_token);
    }

    modifier onlyCreator() {
        require(msg.sender == owner() || approvedCreators[msg.sender], "Not authorized to create markets");
        _;
    }

    function setApprovedCreator(address creator, bool approved) external onlyOwner {
        approvedCreators[creator] = approved;
    }

    function createMarket(string calldata question, uint256 closeTime) external onlyCreator returns (uint256) {
        require(closeTime > block.timestamp, "Close time must be in the future");

        uint256 marketId = marketCount++;
        markets[marketId] = Market({
            question: question,
            closeTime: closeTime,
            status: MarketStatus.Open,
            totalYesShares: 0,
            totalNoShares: 0,
            totalDeposited: 0,
            creator: msg.sender
        });

        emit MarketCreated(marketId, question, closeTime, msg.sender);
        return marketId;
    }

    function buyShares(uint256 marketId, Side side, uint256 shares, uint256 maxCost) external nonReentrant {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open, "Market not open");
        require(block.timestamp < market.closeTime, "Market closed");
        require(shares > 0, "Must buy > 0 shares");
        require(maxCost > 0, "Max cost must be > 0");

        token.safeTransferFrom(msg.sender, address(this), maxCost);

        Position storage pos = positions[marketId][msg.sender];
        if (side == Side.Yes) {
            market.totalYesShares += shares;
            pos.yesShares += shares;
        } else {
            market.totalNoShares += shares;
            pos.noShares += shares;
        }
        pos.totalCost += maxCost;
        market.totalDeposited += maxCost;

        emit SharesPurchased(marketId, msg.sender, side, shares, maxCost);
    }

    function resolveMarket(uint256 marketId, bool yesWins) external onlyOwner {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open || market.status == MarketStatus.Closed, "Cannot resolve");

        market.status = yesWins ? MarketStatus.ResolvedYes : MarketStatus.ResolvedNo;
        emit MarketResolved(marketId, market.status);
    }

    function cancelMarket(uint256 marketId) external onlyOwner {
        Market storage market = markets[marketId];
        require(market.status == MarketStatus.Open, "Can only cancel open markets");
        market.status = MarketStatus.Cancelled;
        emit MarketCancelled(marketId);
    }

    function claimWinnings(uint256 marketId) external nonReentrant {
        Market storage market = markets[marketId];
        Position storage pos = positions[marketId][msg.sender];

        uint256 payout = 0;

        if (market.status == MarketStatus.ResolvedYes) {
            require(pos.yesShares > 0, "No winning shares");
            payout = pos.yesShares;
            pos.yesShares = 0;
        } else if (market.status == MarketStatus.ResolvedNo) {
            require(pos.noShares > 0, "No winning shares");
            payout = pos.noShares;
            pos.noShares = 0;
        } else if (market.status == MarketStatus.Cancelled) {
            payout = pos.totalCost;
            pos.yesShares = 0;
            pos.noShares = 0;
            pos.totalCost = 0;
        } else {
            revert("Market not resolved or cancelled");
        }

        require(payout > 0, "Nothing to claim");
        token.safeTransfer(msg.sender, payout);

        emit WinningsClaimed(marketId, msg.sender, payout);
    }

    function getPosition(uint256 marketId, address trader) external view returns (uint256 yesShares, uint256 noShares, uint256 totalCost) {
        Position storage pos = positions[marketId][trader];
        return (pos.yesShares, pos.noShares, pos.totalCost);
    }

    function getMarket(uint256 marketId) external view returns (
        string memory question, uint256 closeTime, MarketStatus status,
        uint256 totalYesShares, uint256 totalNoShares, uint256 totalDeposited
    ) {
        Market storage m = markets[marketId];
        return (m.question, m.closeTime, m.status, m.totalYesShares, m.totalNoShares, m.totalDeposited);
    }
}
