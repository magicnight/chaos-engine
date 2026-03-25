// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title CrucixToken (CRUX)
 * @dev BEP-20 token for the NewsPredict prediction market
 * - Initial supply minted to deployer
 * - Owner can mint additional tokens (for rewards/airdrops)
 * - Burnable (for market resolution mechanics)
 */
contract CrucixToken is ERC20, ERC20Burnable, Ownable {
    uint8 private constant _decimals = 18;

    constructor(uint256 initialSupply) ERC20("Crucix", "CRUX") Ownable(msg.sender) {
        _mint(msg.sender, initialSupply * 10 ** _decimals);
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function decimals() public pure override returns (uint8) {
        return _decimals;
    }
}
