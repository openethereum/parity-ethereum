// Source for the test AuRa validator set contract.
//
// The bytecode of this contract is included in `validator_contract.json` as the
// constructor of address `0x0000..0005`.

pragma solidity ^0.5.0;

contract TestList {
	address[] public validators = [
		0x7d577a597B2742b498Cb5Cf0C26cDCD726d39E6e,
		0x82A978B3f5962A5b0957d9ee9eEf472EE55B42F1
	];

	mapping(address => uint) indices;
	mapping(bytes32 => address[]) maliceReported;
	address public disliked;

	event InitiateChange(bytes32 indexed parentHash, address[] newSet);

	constructor() public {
		for (uint i = 0; i < validators.length; i++) {
			indices[validators[i]] = i;
		}
	}

	// Called on every block to update node validator list.
	function getValidators() view public returns (address[] memory) {
		return validators;
	}

	function setValidators(address[] memory _validators) public {
		validators = _validators;
		emit InitiateChange(blockhash(block.number - 1), validators);
	}

	// Removes a validator from the list.
	function reportMalicious(address validator, uint256 blockNum, bytes calldata) external {
		maliceReported[keccak256(abi.encode(validator, blockNum))].push(msg.sender);
	    if (validators[indices[validator]] == validator) {
		    validators[indices[validator]] = validators[validators.length-1];
		    delete indices[validator];
		    delete validators[validators.length-1];
		    validators.length--;
	    }
	}

	// Returns the list of all validators that reported the given validator as malicious for the given block.
	function maliceReportedForBlock(address validator, uint256 blockNum) public view returns(address[] memory) {
        return maliceReported[keccak256(abi.encode(validator, blockNum))];
	}

	// Benign validator behaviour report. Kept here for regression testing.
	function reportBenign(address validator, uint256) public {
		disliked = validator;
	}

	// Checks if `emitInitiateChange` can be called.
	function emitInitiateChangeCallable() view public returns (bool) {
		return block.number > 0;
	}

	// Checks if a validator has been removed.
	function isValidatorBanned(address validator) view public returns (bool) {
		return validators[indices[validator]] != validator;
	}

	// Emits an `InitiateChange` event.
	function emitInitiateChange() public {
		emit InitiateChange(blockhash(block.number - 1), validators);
	}

	// Applies a validator set change in production code. Does nothing in the test.
	function finalizeChange() pure public {}
}
