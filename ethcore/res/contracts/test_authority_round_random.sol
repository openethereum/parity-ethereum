pragma solidity 0.5.10;

/// @dev Randomness test contract based on https://github.com/poanetwork/posdao-contracts.
/// Generates and stores random numbers in a RANDAO manner and accumulates a random seed.
contract Random {
    mapping(uint256 => mapping(address => bytes32)) public hashes;
    mapping(uint256 => mapping(address => bytes)) public ciphers;
    mapping(uint256 => mapping(address => uint256)) public secrets;
    uint256 public value;

    /// @dev Called by the validator's node to store a hash and a cipher of the validator's secret on each collection
    /// round. The validator's node must use its mining address to call this function.
    /// This function can only be called once per collection round (during the `commits phase`).
    /// @param _secretHash The Keccak-256 hash of the validator's secret.
    /// @param _cipher The cipher of the validator's secret. Can be used by the node to decrypt and reveal.
    function commitHash(bytes32 _secretHash, bytes calldata _cipher) external {
        require(block.coinbase == msg.sender);
        require(_isCommitPhase(block.number - 1));
        uint256 round = _collectRound(block.number - 1);
        require(!isCommitted(round, msg.sender));
        hashes[round][msg.sender] = _secretHash;
        ciphers[round][msg.sender] = _cipher;
    }

    /// @dev Called by the validator's node to XOR its secret with the current random seed.
    /// The validator's node must use its mining address to call this function.
    /// This function can only be called once per collection round (during the `reveals phase`).
    /// @param _number The validator's secret.
    function revealNumber(uint256 _number) external {
        require(block.coinbase == msg.sender);
        require(_isRevealPhase(block.number - 1));
        uint256 round = _collectRound(block.number - 1);
        require(!sentReveal(round, msg.sender));
        require(hashes[round][msg.sender] == keccak256(abi.encodePacked(_number)));
        secrets[round][msg.sender] = _number;
        value ^= _number;
    }

	/// @dev Returns the Keccak-256 hash and cipher of the validator's secret for the specified collection round
    /// and the specified validator stored by the validator through the `commitHash` function.
    /// @param _collectRound The serial number of the collection round for which hash and cipher should be retrieved.
    /// @param _miningAddress The mining address of validator.
    function getCommitAndCipher(
        uint256 _collectRound,
        address _miningAddress
    ) public view returns(bytes32, bytes memory) {
        return (hashes[_collectRound][_miningAddress], ciphers[_collectRound][_miningAddress]);
    }

    /// @dev Returns a boolean flag indicating whether the specified validator has committed their secret's hash for the
    /// specified collection round.
    /// @param _collectRound The serial number of the collection round for which the checkup should be done.
    /// @param _miningAddress The mining address of the validator.
    function isCommitted(uint256 _collectRound, address _miningAddress) public view returns(bool) {
        return hashes[_collectRound][_miningAddress] != bytes32(0);
    }

    /// @dev Returns a boolean flag indicating whether the current phase of the current collection round
    /// is a `commits phase`. Used by the validator's node to determine if it should commit the hash of
    /// the secret during the current collection round.
    function isCommitPhase() public view returns(bool) {
        return _isCommitPhase(block.number);
    }

    /// @dev Returns a boolean flag indicating whether the current phase of the current collection round
    /// is a `reveals phase`. Used by the validator's node to determine if it should reveal the secret during
    /// the current collection round.
    function isRevealPhase() public view returns(bool) {
        return _isRevealPhase(block.number);
    }

    /// @dev Returns a boolean flag of whether the specified validator has revealed their secret for the
    /// specified collection round.
    /// @param _collectRound The serial number of the collection round for which the checkup should be done.
    /// @param _miningAddress The mining address of the validator.
    function sentReveal(uint256 _collectRound, address _miningAddress) public view returns(bool) {
        return secrets[_collectRound][_miningAddress] != uint256(0);
    }

    /// @dev Returns the current collect round number.
    function currentCollectRound() public view returns(uint256) {
        return _collectRound(block.number);
    }

    /// @dev Returns the current random value.
    function getValue() public view returns(uint256) {
        return value;
    }

    function _collectRound(uint256 blockNumber) private pure returns(uint256) {
        return blockNumber / 6;
    }

    function _isCommitPhase(uint256 blockNumber) private pure returns(bool) {
        return blockNumber % 6 < 3;
    }

    function _isRevealPhase(uint256 blockNumber) private pure returns(bool) {
        return blockNumber % 6 >= 3;
    }
}

