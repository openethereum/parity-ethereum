// Source for the test AuRa validator set contract. DO NOT USE IN PRODUCTION.
//
// Contains POSDAO features. The full POSDAO ValidatorSet contract production code is available at
// https://github.com/poanetwork/posdao-contracts/blob/master/contracts/ValidatorSetAuRa.sol
//
// The bytecode of this contract is included in `validator_contract.json` as the
// constructor of address `0x0000..0005`.

pragma solidity ^0.5.0;

contract TestValidatorSet {

	address public disliked; // contains the address of validator reported by `reportBenign`
	mapping(address => bool) public isValidatorBanned; // if the validator is banned by `reportMalicious`

	// The initial set of validators
	address[] public validators = [
		0x7d577a597B2742b498Cb5Cf0C26cDCD726d39E6e,
		0x82A978B3f5962A5b0957d9ee9eEf472EE55B42F1
	];

	// The mappings used by POSDAO features testing (see `reportMalicious` and `shouldValidatorReport` functions below)
	mapping(address => mapping(uint256 => address[])) private _maliceReportedForBlock;
	mapping(address => mapping(uint256 => mapping(address => bool))) private _maliceReportedForBlockMapped;
	mapping(address => uint256) private _validatorIndex;

	// The standard event to notify the engine about the validator set changing in the contract
	event InitiateChange(bytes32 indexed parentHash, address[] newSet);

	constructor() public {
		// Initialize validator indices to be able to correctly remove
		// a malicious validator from the validator set later
		for (uint i = 0; i < validators.length; i++) {
			_validatorIndex[validators[i]] = i;
		}
	}

	// Emits an `InitiateChange` event with the current (or new) validator set
	function emitInitiateChange() public {
		emit InitiateChange(blockhash(block.number - 1), validators);
	}

	// Applies a validator set change in production code. Does nothing in the test
	function finalizeChange() pure public {}

	// Benign validator behaviour report. Kept here for regression testing
	function reportBenign(address _validator, uint256) public {
		disliked = _validator;
	}

	// Removes a malicious validator from the list
	function reportMalicious(address _validator, uint256 _blockNum, bytes calldata) external {
		address reportingValidator = msg.sender;

		// Mark the `_validator` as reported by `reportingValidator` for the block `_blockNum`
		_maliceReportedForBlock[_validator][_blockNum].push(reportingValidator);
		_maliceReportedForBlockMapped[_validator][_blockNum][reportingValidator] = true;
		isValidatorBanned[_validator] = true;

		// If the passed validator is in the validator set
		if (validators[_validatorIndex[_validator]] == _validator) {
			// Remove the validator from the set
			validators[_validatorIndex[_validator]] = validators[validators.length - 1];
			delete _validatorIndex[_validator];
			delete validators[validators.length - 1];
			validators.length--;
		}
	}

	// Tests validator set changing and emitting the `InitiateChange` event
	function setValidators(address[] memory _validators) public {
		validators = _validators;
		emitInitiateChange();
	}

	// Checks if `emitInitiateChange` can be called (used by POSDAO tests)
	function emitInitiateChangeCallable() view public returns(bool) {
		return block.number > 0;
	}

	// Returns the current validator set
	function getValidators() public view returns(address[] memory) {
		return validators;
	}

	// Returns the list of all validators that reported the given validator
	// as malicious for the given block. Used by POSDAO tests
	function maliceReportedForBlock(address _validator, uint256 _blockNum) public view returns(address[] memory) {
		return _maliceReportedForBlock[_validator][_blockNum];
	}

	// Returns a boolean flag indicating whether the specified validator
	// should report about some validator's misbehaviour at the specified block.
	// Used by POSDAO tests.
	// `_reportingValidator` is the address of validator who reports.
	// `_maliciousValidator` is the address of malicious validator.
	// `_blockNumber` is the block number at which the malicious validator misbehaved.
	function shouldValidatorReport(
		address _reportingValidator,
		address _maliciousValidator,
		uint256 _blockNumber
	) public view returns(bool) {
		uint256 currentBlock = block.number;
		if (_blockNumber > currentBlock) {
			return false;
		}
		if (currentBlock > 100 && currentBlock - 100 > _blockNumber) {
			return false;
		}
		if (isValidatorBanned[_maliciousValidator]) {
			// We shouldn't report the malicious validator
			// as it has already been reported and banned
			return false;
		}
		// Return `false` if already reported by the same `_reportingValidator` for the same `_blockNumber`
		return !_maliceReportedForBlockMapped[_maliciousValidator][_blockNumber][_reportingValidator];
	}

}
