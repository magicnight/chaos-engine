// SPDX-License-Identifier: MIT
// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";

/**
 * @title ChaosToken (CHAOS)
 * @author ChaosDevOps@BKK&Estonia
 * @notice BEP-20 utility token for the CHAOS Engine prediction market platform
 * @dev
 * - Initial supply minted to deployer
 * - Owner can mint additional tokens up to MAX_SUPPLY
 * - Burnable (for market resolution mechanics)
 * - Pausable (for emergency situations)
 *
 * Part of the CHAOS Engine — Connected Human-Augmented OSINT Suite
 * https://github.com/magicnight/chaos-engine
 */
contract ChaosToken is ERC20, ERC20Burnable, Ownable, Pausable {
    uint8 private constant _decimals = 18;
    uint256 public constant MAX_SUPPLY = 10_000_000_000 * 10 ** 18; // 10 billion cap

    constructor(uint256 initialSupply) ERC20("C.H.A.O.S.", "CHAOS") Ownable(msg.sender) {
        require(initialSupply * 10 ** _decimals <= MAX_SUPPLY, "Exceeds max supply");
        _mint(msg.sender, initialSupply * 10 ** _decimals);
    }

    function mint(address to, uint256 amount) public onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }

    function decimals() public pure override returns (uint8) {
        return _decimals;
    }

    function pause() external onlyOwner {
        _pause();
    }

    function unpause() external onlyOwner {
        _unpause();
    }

    function _update(address from, address to, uint256 value) internal override whenNotPaused {
        super._update(from, to, value);
    }
}
