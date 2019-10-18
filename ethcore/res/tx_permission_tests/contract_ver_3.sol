pragma solidity ^0.4.20;

// Adapted from https://gist.github.com/VladLupashevskyi/84f18eabb1e4afadf572cf92af3e7e7f
// and: https://github.com/poanetwork/posdao-contracts/blob/master/contracts/TxPermission.sol

contract TxPermission {
    /// Allowed transaction types mask
    uint32 constant None = 0;
    uint32 constant All = 0xffffffff;
    uint32 constant Basic = 0x01;
    uint32 constant Call = 0x02;
    uint32 constant Create = 0x04;
    uint32 constant Private = 0x08;

    /// Contract name
    function contractName() public constant returns (string) {
        return "TX_PERMISSION_CONTRACT";
    }

    /// Contract name hash
    function contractNameHash() public constant returns (bytes32) {
        return keccak256(contractName());
    }

    /// Contract version
    function contractVersion() public constant returns (uint256) {
        return 3;
    }

    /// @dev Defines the allowed transaction types which may be initiated by the specified sender with
    /// the specified gas price and data. Used by the Parity engine each time a transaction is about to be
    /// included into a block. See https://wiki.parity.io/Permissioning.html#how-it-works-1
    /// @param _sender Transaction sender address.
    /// @param _to Transaction recipient address. If creating a contract, the `_to` address is zero.
    /// @param _value Transaction amount in wei.
    /// @param _gasPrice Gas price in wei for the transaction.
    /// @param _data Transaction data.
    /// @return `uint32 typesMask` - Set of allowed transactions for `_sender` depending on tx `_to` address,
    /// `_gasPrice`, and `_data`. The result is represented as a set of flags:
    /// 0x01 - basic transaction (e.g. ether transferring to user wallet);
    /// 0x02 - contract call;
    /// 0x04 - contract creation;
    /// 0x08 - private transaction.
    /// `bool cache` - If `true` is returned, the same permissions will be applied from the same
    /// `_sender` without calling this contract again.
	function allowedTxTypes(
        address _sender,
        address _to,
        uint256 _value,
        uint256 _gasPrice,
        bytes memory _data
    )
        public
        view
        returns(uint32 typesMask, bool cache)
    {
		if (_gasPrice > 0 || _data.length < 4) return (All, false);
		return (None, false);
    }
}
