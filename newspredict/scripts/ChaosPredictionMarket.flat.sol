// Sources flattened with hardhat v3.2.0 https://hardhat.org

// SPDX-License-Identifier: MIT

// File npm/@openzeppelin/contracts@5.6.1/utils/Context.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.0.1) (utils/Context.sol)

pragma solidity ^0.8.20;

/**
 * @dev Provides information about the current execution context, including the
 * sender of the transaction and its data. While these are generally available
 * via msg.sender and msg.data, they should not be accessed in such a direct
 * manner, since when dealing with meta-transactions the account sending and
 * paying for execution may not be the actual sender (as far as an application
 * is concerned).
 *
 * This contract is only required for intermediate, library-like contracts.
 */
abstract contract Context {
    function _msgSender() internal view virtual returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual returns (bytes calldata) {
        return msg.data;
    }

    function _contextSuffixLength() internal view virtual returns (uint256) {
        return 0;
    }
}


// File npm/@openzeppelin/contracts@5.6.1/access/Ownable.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.0.0) (access/Ownable.sol)

pragma solidity ^0.8.20;

/**
 * @dev Contract module which provides a basic access control mechanism, where
 * there is an account (an owner) that can be granted exclusive access to
 * specific functions.
 *
 * The initial owner is set to the address provided by the deployer. This can
 * later be changed with {transferOwnership}.
 *
 * This module is used through inheritance. It will make available the modifier
 * `onlyOwner`, which can be applied to your functions to restrict their use to
 * the owner.
 */
abstract contract Ownable is Context {
    address private _owner;

    /**
     * @dev The caller account is not authorized to perform an operation.
     */
    error OwnableUnauthorizedAccount(address account);

    /**
     * @dev The owner is not a valid owner account. (eg. `address(0)`)
     */
    error OwnableInvalidOwner(address owner);

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /**
     * @dev Initializes the contract setting the address provided by the deployer as the initial owner.
     */
    constructor(address initialOwner) {
        if (initialOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(initialOwner);
    }

    /**
     * @dev Throws if called by any account other than the owner.
     */
    modifier onlyOwner() {
        _checkOwner();
        _;
    }

    /**
     * @dev Returns the address of the current owner.
     */
    function owner() public view virtual returns (address) {
        return _owner;
    }

    /**
     * @dev Throws if the sender is not the owner.
     */
    function _checkOwner() internal view virtual {
        if (owner() != _msgSender()) {
            revert OwnableUnauthorizedAccount(_msgSender());
        }
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` functions. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby disabling any functionality that is only available to the owner.
     */
    function renounceOwnership() public virtual onlyOwner {
        _transferOwnership(address(0));
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Can only be called by the current owner.
     */
    function transferOwnership(address newOwner) public virtual onlyOwner {
        if (newOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(newOwner);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Internal function without access restriction.
     */
    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}


// File npm/@openzeppelin/contracts@5.6.1/token/ERC20/IERC20.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.4.0) (token/ERC20/IERC20.sol)

pragma solidity >=0.4.16;

/**
 * @dev Interface of the ERC-20 standard as defined in the ERC.
 */
interface IERC20 {
    /**
     * @dev Emitted when `value` tokens are moved from one account (`from`) to
     * another (`to`).
     *
     * Note that `value` may be zero.
     */
    event Transfer(address indexed from, address indexed to, uint256 value);

    /**
     * @dev Emitted when the allowance of a `spender` for an `owner` is set by
     * a call to {approve}. `value` is the new allowance.
     */
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /**
     * @dev Returns the value of tokens in existence.
     */
    function totalSupply() external view returns (uint256);

    /**
     * @dev Returns the value of tokens owned by `account`.
     */
    function balanceOf(address account) external view returns (uint256);

    /**
     * @dev Moves a `value` amount of tokens from the caller's account to `to`.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transfer(address to, uint256 value) external returns (bool);

    /**
     * @dev Returns the remaining number of tokens that `spender` will be
     * allowed to spend on behalf of `owner` through {transferFrom}. This is
     * zero by default.
     *
     * This value changes when {approve} or {transferFrom} are called.
     */
    function allowance(address owner, address spender) external view returns (uint256);

    /**
     * @dev Sets a `value` amount of tokens as the allowance of `spender` over the
     * caller's tokens.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * IMPORTANT: Beware that changing an allowance with this method brings the risk
     * that someone may use both the old and the new allowance by unfortunate
     * transaction ordering. One possible solution to mitigate this race
     * condition is to first reduce the spender's allowance to 0 and set the
     * desired value afterwards:
     * https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
     *
     * Emits an {Approval} event.
     */
    function approve(address spender, uint256 value) external returns (bool);

    /**
     * @dev Moves a `value` amount of tokens from `from` to `to` using the
     * allowance mechanism. `value` is then deducted from the caller's
     * allowance.
     *
     * Returns a boolean value indicating whether the operation succeeded.
     *
     * Emits a {Transfer} event.
     */
    function transferFrom(address from, address to, uint256 value) external returns (bool);
}


// File npm/@openzeppelin/contracts@5.6.1/utils/introspection/IERC165.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.4.0) (utils/introspection/IERC165.sol)

pragma solidity >=0.4.16;

/**
 * @dev Interface of the ERC-165 standard, as defined in the
 * https://eips.ethereum.org/EIPS/eip-165[ERC].
 *
 * Implementers can declare support of contract interfaces, which can then be
 * queried by others ({ERC165Checker}).
 *
 * For an implementation, see {ERC165}.
 */
interface IERC165 {
    /**
     * @dev Returns true if this contract implements the interface defined by
     * `interfaceId`. See the corresponding
     * https://eips.ethereum.org/EIPS/eip-165#how-interfaces-are-identified[ERC section]
     * to learn more about how these ids are created.
     *
     * This function call must use less than 30 000 gas.
     */
    function supportsInterface(bytes4 interfaceId) external view returns (bool);
}


// File npm/@openzeppelin/contracts@5.6.1/interfaces/IERC165.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.4.0) (interfaces/IERC165.sol)

pragma solidity >=0.4.16;


// File npm/@openzeppelin/contracts@5.6.1/interfaces/IERC20.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.4.0) (interfaces/IERC20.sol)

pragma solidity >=0.4.16;


// File npm/@openzeppelin/contracts@5.6.1/interfaces/IERC1363.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.4.0) (interfaces/IERC1363.sol)

pragma solidity >=0.6.2;


/**
 * @title IERC1363
 * @dev Interface of the ERC-1363 standard as defined in the https://eips.ethereum.org/EIPS/eip-1363[ERC-1363].
 *
 * Defines an extension interface for ERC-20 tokens that supports executing code on a recipient contract
 * after `transfer` or `transferFrom`, or code on a spender contract after `approve`, in a single transaction.
 */
interface IERC1363 is IERC20, IERC165 {
    /*
     * Note: the ERC-165 identifier for this interface is 0xb0202a11.
     * 0xb0202a11 ===
     *   bytes4(keccak256('transferAndCall(address,uint256)')) ^
     *   bytes4(keccak256('transferAndCall(address,uint256,bytes)')) ^
     *   bytes4(keccak256('transferFromAndCall(address,address,uint256)')) ^
     *   bytes4(keccak256('transferFromAndCall(address,address,uint256,bytes)')) ^
     *   bytes4(keccak256('approveAndCall(address,uint256)')) ^
     *   bytes4(keccak256('approveAndCall(address,uint256,bytes)'))
     */

    /**
     * @dev Moves a `value` amount of tokens from the caller's account to `to`
     * and then calls {IERC1363Receiver-onTransferReceived} on `to`.
     * @param to The address which you want to transfer to.
     * @param value The amount of tokens to be transferred.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function transferAndCall(address to, uint256 value) external returns (bool);

    /**
     * @dev Moves a `value` amount of tokens from the caller's account to `to`
     * and then calls {IERC1363Receiver-onTransferReceived} on `to`.
     * @param to The address which you want to transfer to.
     * @param value The amount of tokens to be transferred.
     * @param data Additional data with no specified format, sent in call to `to`.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function transferAndCall(address to, uint256 value, bytes calldata data) external returns (bool);

    /**
     * @dev Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism
     * and then calls {IERC1363Receiver-onTransferReceived} on `to`.
     * @param from The address which you want to send tokens from.
     * @param to The address which you want to transfer to.
     * @param value The amount of tokens to be transferred.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function transferFromAndCall(address from, address to, uint256 value) external returns (bool);

    /**
     * @dev Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism
     * and then calls {IERC1363Receiver-onTransferReceived} on `to`.
     * @param from The address which you want to send tokens from.
     * @param to The address which you want to transfer to.
     * @param value The amount of tokens to be transferred.
     * @param data Additional data with no specified format, sent in call to `to`.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function transferFromAndCall(address from, address to, uint256 value, bytes calldata data) external returns (bool);

    /**
     * @dev Sets a `value` amount of tokens as the allowance of `spender` over the
     * caller's tokens and then calls {IERC1363Spender-onApprovalReceived} on `spender`.
     * @param spender The address which will spend the funds.
     * @param value The amount of tokens to be spent.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function approveAndCall(address spender, uint256 value) external returns (bool);

    /**
     * @dev Sets a `value` amount of tokens as the allowance of `spender` over the
     * caller's tokens and then calls {IERC1363Spender-onApprovalReceived} on `spender`.
     * @param spender The address which will spend the funds.
     * @param value The amount of tokens to be spent.
     * @param data Additional data with no specified format, sent in call to `spender`.
     * @return A boolean value indicating whether the operation succeeded unless throwing.
     */
    function approveAndCall(address spender, uint256 value, bytes calldata data) external returns (bool);
}


// File npm/@openzeppelin/contracts@5.6.1/token/ERC20/utils/SafeERC20.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.5.0) (token/ERC20/utils/SafeERC20.sol)

pragma solidity ^0.8.20;


/**
 * @title SafeERC20
 * @dev Wrappers around ERC-20 operations that throw on failure (when the token
 * contract returns false). Tokens that return no value (and instead revert or
 * throw on failure) are also supported, non-reverting calls are assumed to be
 * successful.
 * To use this library you can add a `using SafeERC20 for IERC20;` statement to your contract,
 * which allows you to call the safe operations as `token.safeTransfer(...)`, etc.
 */
library SafeERC20 {
    /**
     * @dev An operation with an ERC-20 token failed.
     */
    error SafeERC20FailedOperation(address token);

    /**
     * @dev Indicates a failed `decreaseAllowance` request.
     */
    error SafeERC20FailedDecreaseAllowance(address spender, uint256 currentAllowance, uint256 requestedDecrease);

    /**
     * @dev Transfer `value` amount of `token` from the calling contract to `to`. If `token` returns no value,
     * non-reverting calls are assumed to be successful.
     */
    function safeTransfer(IERC20 token, address to, uint256 value) internal {
        if (!_safeTransfer(token, to, value, true)) {
            revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Transfer `value` amount of `token` from `from` to `to`, spending the approval given by `from` to the
     * calling contract. If `token` returns no value, non-reverting calls are assumed to be successful.
     */
    function safeTransferFrom(IERC20 token, address from, address to, uint256 value) internal {
        if (!_safeTransferFrom(token, from, to, value, true)) {
            revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Variant of {safeTransfer} that returns a bool instead of reverting if the operation is not successful.
     */
    function trySafeTransfer(IERC20 token, address to, uint256 value) internal returns (bool) {
        return _safeTransfer(token, to, value, false);
    }

    /**
     * @dev Variant of {safeTransferFrom} that returns a bool instead of reverting if the operation is not successful.
     */
    function trySafeTransferFrom(IERC20 token, address from, address to, uint256 value) internal returns (bool) {
        return _safeTransferFrom(token, from, to, value, false);
    }

    /**
     * @dev Increase the calling contract's allowance toward `spender` by `value`. If `token` returns no value,
     * non-reverting calls are assumed to be successful.
     *
     * IMPORTANT: If the token implements ERC-7674 (ERC-20 with temporary allowance), and if the "client"
     * smart contract uses ERC-7674 to set temporary allowances, then the "client" smart contract should avoid using
     * this function. Performing a {safeIncreaseAllowance} or {safeDecreaseAllowance} operation on a token contract
     * that has a non-zero temporary allowance (for that particular owner-spender) will result in unexpected behavior.
     */
    function safeIncreaseAllowance(IERC20 token, address spender, uint256 value) internal {
        uint256 oldAllowance = token.allowance(address(this), spender);
        forceApprove(token, spender, oldAllowance + value);
    }

    /**
     * @dev Decrease the calling contract's allowance toward `spender` by `requestedDecrease`. If `token` returns no
     * value, non-reverting calls are assumed to be successful.
     *
     * IMPORTANT: If the token implements ERC-7674 (ERC-20 with temporary allowance), and if the "client"
     * smart contract uses ERC-7674 to set temporary allowances, then the "client" smart contract should avoid using
     * this function. Performing a {safeIncreaseAllowance} or {safeDecreaseAllowance} operation on a token contract
     * that has a non-zero temporary allowance (for that particular owner-spender) will result in unexpected behavior.
     */
    function safeDecreaseAllowance(IERC20 token, address spender, uint256 requestedDecrease) internal {
        unchecked {
            uint256 currentAllowance = token.allowance(address(this), spender);
            if (currentAllowance < requestedDecrease) {
                revert SafeERC20FailedDecreaseAllowance(spender, currentAllowance, requestedDecrease);
            }
            forceApprove(token, spender, currentAllowance - requestedDecrease);
        }
    }

    /**
     * @dev Set the calling contract's allowance toward `spender` to `value`. If `token` returns no value,
     * non-reverting calls are assumed to be successful. Meant to be used with tokens that require the approval
     * to be set to zero before setting it to a non-zero value, such as USDT.
     *
     * NOTE: If the token implements ERC-7674, this function will not modify any temporary allowance. This function
     * only sets the "standard" allowance. Any temporary allowance will remain active, in addition to the value being
     * set here.
     */
    function forceApprove(IERC20 token, address spender, uint256 value) internal {
        if (!_safeApprove(token, spender, value, false)) {
            if (!_safeApprove(token, spender, 0, true)) revert SafeERC20FailedOperation(address(token));
            if (!_safeApprove(token, spender, value, true)) revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Performs an {ERC1363} transferAndCall, with a fallback to the simple {ERC20} transfer if the target has no
     * code. This can be used to implement an {ERC721}-like safe transfer that relies on {ERC1363} checks when
     * targeting contracts.
     *
     * Reverts if the returned value is other than `true`.
     */
    function transferAndCallRelaxed(IERC1363 token, address to, uint256 value, bytes memory data) internal {
        if (to.code.length == 0) {
            safeTransfer(token, to, value);
        } else if (!token.transferAndCall(to, value, data)) {
            revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Performs an {ERC1363} transferFromAndCall, with a fallback to the simple {ERC20} transferFrom if the target
     * has no code. This can be used to implement an {ERC721}-like safe transfer that relies on {ERC1363} checks when
     * targeting contracts.
     *
     * Reverts if the returned value is other than `true`.
     */
    function transferFromAndCallRelaxed(
        IERC1363 token,
        address from,
        address to,
        uint256 value,
        bytes memory data
    ) internal {
        if (to.code.length == 0) {
            safeTransferFrom(token, from, to, value);
        } else if (!token.transferFromAndCall(from, to, value, data)) {
            revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Performs an {ERC1363} approveAndCall, with a fallback to the simple {ERC20} approve if the target has no
     * code. This can be used to implement an {ERC721}-like safe transfer that rely on {ERC1363} checks when
     * targeting contracts.
     *
     * NOTE: When the recipient address (`to`) has no code (i.e. is an EOA), this function behaves as {forceApprove}.
     * Oppositely, when the recipient address (`to`) has code, this function only attempts to call {ERC1363-approveAndCall}
     * once without retrying, and relies on the returned value to be true.
     *
     * Reverts if the returned value is other than `true`.
     */
    function approveAndCallRelaxed(IERC1363 token, address to, uint256 value, bytes memory data) internal {
        if (to.code.length == 0) {
            forceApprove(token, to, value);
        } else if (!token.approveAndCall(to, value, data)) {
            revert SafeERC20FailedOperation(address(token));
        }
    }

    /**
     * @dev Imitates a Solidity `token.transfer(to, value)` call, relaxing the requirement on the return value: the
     * return value is optional (but if data is returned, it must not be false).
     *
     * @param token The token targeted by the call.
     * @param to The recipient of the tokens
     * @param value The amount of token to transfer
     * @param bubble Behavior switch if the transfer call reverts: bubble the revert reason or return a false boolean.
     */
    function _safeTransfer(IERC20 token, address to, uint256 value, bool bubble) private returns (bool success) {
        bytes4 selector = IERC20.transfer.selector;

        assembly ("memory-safe") {
            let fmp := mload(0x40)
            mstore(0x00, selector)
            mstore(0x04, and(to, shr(96, not(0))))
            mstore(0x24, value)
            success := call(gas(), token, 0, 0x00, 0x44, 0x00, 0x20)
            // if call success and return is true, all is good.
            // otherwise (not success or return is not true), we need to perform further checks
            if iszero(and(success, eq(mload(0x00), 1))) {
                // if the call was a failure and bubble is enabled, bubble the error
                if and(iszero(success), bubble) {
                    returndatacopy(fmp, 0x00, returndatasize())
                    revert(fmp, returndatasize())
                }
                // if the return value is not true, then the call is only successful if:
                // - the token address has code
                // - the returndata is empty
                success := and(success, and(iszero(returndatasize()), gt(extcodesize(token), 0)))
            }
            mstore(0x40, fmp)
        }
    }

    /**
     * @dev Imitates a Solidity `token.transferFrom(from, to, value)` call, relaxing the requirement on the return
     * value: the return value is optional (but if data is returned, it must not be false).
     *
     * @param token The token targeted by the call.
     * @param from The sender of the tokens
     * @param to The recipient of the tokens
     * @param value The amount of token to transfer
     * @param bubble Behavior switch if the transfer call reverts: bubble the revert reason or return a false boolean.
     */
    function _safeTransferFrom(
        IERC20 token,
        address from,
        address to,
        uint256 value,
        bool bubble
    ) private returns (bool success) {
        bytes4 selector = IERC20.transferFrom.selector;

        assembly ("memory-safe") {
            let fmp := mload(0x40)
            mstore(0x00, selector)
            mstore(0x04, and(from, shr(96, not(0))))
            mstore(0x24, and(to, shr(96, not(0))))
            mstore(0x44, value)
            success := call(gas(), token, 0, 0x00, 0x64, 0x00, 0x20)
            // if call success and return is true, all is good.
            // otherwise (not success or return is not true), we need to perform further checks
            if iszero(and(success, eq(mload(0x00), 1))) {
                // if the call was a failure and bubble is enabled, bubble the error
                if and(iszero(success), bubble) {
                    returndatacopy(fmp, 0x00, returndatasize())
                    revert(fmp, returndatasize())
                }
                // if the return value is not true, then the call is only successful if:
                // - the token address has code
                // - the returndata is empty
                success := and(success, and(iszero(returndatasize()), gt(extcodesize(token), 0)))
            }
            mstore(0x40, fmp)
            mstore(0x60, 0)
        }
    }

    /**
     * @dev Imitates a Solidity `token.approve(spender, value)` call, relaxing the requirement on the return value:
     * the return value is optional (but if data is returned, it must not be false).
     *
     * @param token The token targeted by the call.
     * @param spender The spender of the tokens
     * @param value The amount of token to transfer
     * @param bubble Behavior switch if the transfer call reverts: bubble the revert reason or return a false boolean.
     */
    function _safeApprove(IERC20 token, address spender, uint256 value, bool bubble) private returns (bool success) {
        bytes4 selector = IERC20.approve.selector;

        assembly ("memory-safe") {
            let fmp := mload(0x40)
            mstore(0x00, selector)
            mstore(0x04, and(spender, shr(96, not(0))))
            mstore(0x24, value)
            success := call(gas(), token, 0, 0x00, 0x44, 0x00, 0x20)
            // if call success and return is true, all is good.
            // otherwise (not success or return is not true), we need to perform further checks
            if iszero(and(success, eq(mload(0x00), 1))) {
                // if the call was a failure and bubble is enabled, bubble the error
                if and(iszero(success), bubble) {
                    returndatacopy(fmp, 0x00, returndatasize())
                    revert(fmp, returndatasize())
                }
                // if the return value is not true, then the call is only successful if:
                // - the token address has code
                // - the returndata is empty
                success := and(success, and(iszero(returndatasize()), gt(extcodesize(token), 0)))
            }
            mstore(0x40, fmp)
        }
    }
}


// File npm/@openzeppelin/contracts@5.6.1/utils/StorageSlot.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.1.0) (utils/StorageSlot.sol)
// This file was procedurally generated from scripts/generate/templates/StorageSlot.js.

pragma solidity ^0.8.20;

/**
 * @dev Library for reading and writing primitive types to specific storage slots.
 *
 * Storage slots are often used to avoid storage conflict when dealing with upgradeable contracts.
 * This library helps with reading and writing to such slots without the need for inline assembly.
 *
 * The functions in this library return Slot structs that contain a `value` member that can be used to read or write.
 *
 * Example usage to set ERC-1967 implementation slot:
 * ```solidity
 * contract ERC1967 {
 *     // Define the slot. Alternatively, use the SlotDerivation library to derive the slot.
 *     bytes32 internal constant _IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;
 *
 *     function _getImplementation() internal view returns (address) {
 *         return StorageSlot.getAddressSlot(_IMPLEMENTATION_SLOT).value;
 *     }
 *
 *     function _setImplementation(address newImplementation) internal {
 *         require(newImplementation.code.length > 0);
 *         StorageSlot.getAddressSlot(_IMPLEMENTATION_SLOT).value = newImplementation;
 *     }
 * }
 * ```
 *
 * TIP: Consider using this library along with {SlotDerivation}.
 */
library StorageSlot {
    struct AddressSlot {
        address value;
    }

    struct BooleanSlot {
        bool value;
    }

    struct Bytes32Slot {
        bytes32 value;
    }

    struct Uint256Slot {
        uint256 value;
    }

    struct Int256Slot {
        int256 value;
    }

    struct StringSlot {
        string value;
    }

    struct BytesSlot {
        bytes value;
    }

    /**
     * @dev Returns an `AddressSlot` with member `value` located at `slot`.
     */
    function getAddressSlot(bytes32 slot) internal pure returns (AddressSlot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns a `BooleanSlot` with member `value` located at `slot`.
     */
    function getBooleanSlot(bytes32 slot) internal pure returns (BooleanSlot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns a `Bytes32Slot` with member `value` located at `slot`.
     */
    function getBytes32Slot(bytes32 slot) internal pure returns (Bytes32Slot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns a `Uint256Slot` with member `value` located at `slot`.
     */
    function getUint256Slot(bytes32 slot) internal pure returns (Uint256Slot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns a `Int256Slot` with member `value` located at `slot`.
     */
    function getInt256Slot(bytes32 slot) internal pure returns (Int256Slot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns a `StringSlot` with member `value` located at `slot`.
     */
    function getStringSlot(bytes32 slot) internal pure returns (StringSlot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns an `StringSlot` representation of the string storage pointer `store`.
     */
    function getStringSlot(string storage store) internal pure returns (StringSlot storage r) {
        assembly ("memory-safe") {
            r.slot := store.slot
        }
    }

    /**
     * @dev Returns a `BytesSlot` with member `value` located at `slot`.
     */
    function getBytesSlot(bytes32 slot) internal pure returns (BytesSlot storage r) {
        assembly ("memory-safe") {
            r.slot := slot
        }
    }

    /**
     * @dev Returns an `BytesSlot` representation of the bytes storage pointer `store`.
     */
    function getBytesSlot(bytes storage store) internal pure returns (BytesSlot storage r) {
        assembly ("memory-safe") {
            r.slot := store.slot
        }
    }
}


// File npm/@openzeppelin/contracts@5.6.1/utils/ReentrancyGuard.sol

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.5.0) (utils/ReentrancyGuard.sol)

pragma solidity ^0.8.20;

/**
 * @dev Contract module that helps prevent reentrant calls to a function.
 *
 * Inheriting from `ReentrancyGuard` will make the {nonReentrant} modifier
 * available, which can be applied to functions to make sure there are no nested
 * (reentrant) calls to them.
 *
 * Note that because there is a single `nonReentrant` guard, functions marked as
 * `nonReentrant` may not call one another. This can be worked around by making
 * those functions `private`, and then adding `external` `nonReentrant` entry
 * points to them.
 *
 * TIP: If EIP-1153 (transient storage) is available on the chain you're deploying at,
 * consider using {ReentrancyGuardTransient} instead.
 *
 * TIP: If you would like to learn more about reentrancy and alternative ways
 * to protect against it, check out our blog post
 * https://blog.openzeppelin.com/reentrancy-after-istanbul/[Reentrancy After Istanbul].
 *
 * IMPORTANT: Deprecated. This storage-based reentrancy guard will be removed and replaced
 * by the {ReentrancyGuardTransient} variant in v6.0.
 *
 * @custom:stateless
 */
abstract contract ReentrancyGuard {
    using StorageSlot for bytes32;

    // keccak256(abi.encode(uint256(keccak256("openzeppelin.storage.ReentrancyGuard")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant REENTRANCY_GUARD_STORAGE =
        0x9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f00;

    // Booleans are more expensive than uint256 or any type that takes up a full
    // word because each write operation emits an extra SLOAD to first read the
    // slot's contents, replace the bits taken up by the boolean, and then write
    // back. This is the compiler's defense against contract upgrades and
    // pointer aliasing, and it cannot be disabled.

    // The values being non-zero value makes deployment a bit more expensive,
    // but in exchange the refund on every call to nonReentrant will be lower in
    // amount. Since refunds are capped to a percentage of the total
    // transaction's gas, it is best to keep them low in cases like this one, to
    // increase the likelihood of the full refund coming into effect.
    uint256 private constant NOT_ENTERED = 1;
    uint256 private constant ENTERED = 2;

    /**
     * @dev Unauthorized reentrant call.
     */
    error ReentrancyGuardReentrantCall();

    constructor() {
        _reentrancyGuardStorageSlot().getUint256Slot().value = NOT_ENTERED;
    }

    /**
     * @dev Prevents a contract from calling itself, directly or indirectly.
     * Calling a `nonReentrant` function from another `nonReentrant`
     * function is not supported. It is possible to prevent this from happening
     * by making the `nonReentrant` function external, and making it call a
     * `private` function that does the actual work.
     */
    modifier nonReentrant() {
        _nonReentrantBefore();
        _;
        _nonReentrantAfter();
    }

    /**
     * @dev A `view` only version of {nonReentrant}. Use to block view functions
     * from being called, preventing reading from inconsistent contract state.
     *
     * CAUTION: This is a "view" modifier and does not change the reentrancy
     * status. Use it only on view functions. For payable or non-payable functions,
     * use the standard {nonReentrant} modifier instead.
     */
    modifier nonReentrantView() {
        _nonReentrantBeforeView();
        _;
    }

    function _nonReentrantBeforeView() private view {
        if (_reentrancyGuardEntered()) {
            revert ReentrancyGuardReentrantCall();
        }
    }

    function _nonReentrantBefore() private {
        // On the first call to nonReentrant, _status will be NOT_ENTERED
        _nonReentrantBeforeView();

        // Any calls to nonReentrant after this point will fail
        _reentrancyGuardStorageSlot().getUint256Slot().value = ENTERED;
    }

    function _nonReentrantAfter() private {
        // By storing the original value once again, a refund is triggered (see
        // https://eips.ethereum.org/EIPS/eip-2200)
        _reentrancyGuardStorageSlot().getUint256Slot().value = NOT_ENTERED;
    }

    /**
     * @dev Returns true if the reentrancy guard is currently set to "entered", which indicates there is a
     * `nonReentrant` function in the call stack.
     */
    function _reentrancyGuardEntered() internal view returns (bool) {
        return _reentrancyGuardStorageSlot().getUint256Slot().value == ENTERED;
    }

    function _reentrancyGuardStorageSlot() internal pure virtual returns (bytes32) {
        return REENTRANCY_GUARD_STORAGE;
    }
}


// File contracts/ChaosPredictionMarket.sol

// Original license: SPDX_License_Identifier: MIT
// Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.
pragma solidity ^0.8.24;




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

