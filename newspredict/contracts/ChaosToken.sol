// SPDX-License-Identifier: MIT
// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ChaosToken (CHAOS)
 * @author ChaosDevOps@BKK&Estonia
 * @notice BEP-20 utility token for the CHAOS Engine prediction market platform
 * @dev
 * - Initial supply minted to deployer
 * - Owner can mint additional tokens (for rewards/airdrops)
 * - Burnable (for market resolution mechanics)
 *
 * Part of the CHAOS Engine — Connected Human-Augmented OSINT Suite
 * https://github.com/magicnight/chaos-engine
 */
contract ChaosToken is ERC20, ERC20Burnable, Ownable {
    uint8 private constant _decimals = 18;

    constructor(uint256 initialSupply) ERC20("CHAOS", "CHAOS") Ownable(msg.sender) {
        _mint(msg.sender, initialSupply * 10 ** _decimals);
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function decimals() public pure override returns (uint8) {
        return _decimals;
    }
}
