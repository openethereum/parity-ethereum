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
	// Should remain 0 because `reportBenign` is no longer used.
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

	// Removes a validator from the list.
	function reportMalicious(address validator) public {
		validators[indices[validator]] = validators[validators.length-1];
		delete indices[validator];
		delete validators[validators.length-1];
		validators.length--;
	}

	// Benign validator behaviour report. Kept here for regression testing.
	function reportBenign(address validator) public {
		disliked = validator;
	}

	// Checks if `emitInitiateChange` can be called.
	function emitInitiateChangeCallable() pure public returns (bool) {
		return true;
	}

	// Emits an `InitiateChange` event in production code. Does nothing in the test.
	function emitInitiateChange() pure public {}

	// Applies a validator set change in production code. Does nothing in the test.
	function finalizeChange() pure public {}
}
