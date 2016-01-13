//! VM Instructions list and utility functions

pub type Instruction = u8;

pub fn is_jump (i : Instruction) -> bool {
	i == JUMP || i == JUMPI || i == JUMPDEST
}

pub fn is_push (i : Instruction) -> bool {
	i >= PUSH1 && i <= PUSH32
}

pub fn get_push_bytes (i : Instruction) -> usize {
	// TODO [todr] range checking?
	(i - PUSH1 + 1) as usize
}

pub fn get_dup_position (i: Instruction) -> usize {
	// TODO [todr] range checking?
	(i - DUP1) as usize
}

pub fn get_swap_position (i : Instruction) -> usize {
	// TODO [todr] range checking?
	(i - SWAP1 + 1) as usize
}

pub fn get_log_topics (i: Instruction) -> usize {
	(i - LOG0) as usize
}

#[test]
fn test_get_push_bytes() {
	assert_eq!(get_push_bytes(PUSH1), 1);
	assert_eq!(get_push_bytes(PUSH3), 3);
	assert_eq!(get_push_bytes(PUSH32), 32);
}

// Virtual machine bytecode instruction.
pub const STOP: Instruction =  0x00; //< halts execution
pub const ADD: Instruction =  0x01; //< addition operation
pub const	MUL: Instruction =  0x02; //< mulitplication operation
pub const	SUB: Instruction =  0x03; //< subtraction operation
pub const	DIV: Instruction =  0x04; //< integer division operation
pub const	SDIV: Instruction =  0x05; //< signed integer division operation
pub const	MOD: Instruction =  0x06; //< modulo remainder operation
pub const	SMOD: Instruction =  0x07; //< signed modulo remainder operation
pub const	ADDMOD: Instruction =  0x08; //< unsigned modular addition
pub const	MULMOD: Instruction =  0x09; //< unsigned modular multiplication
pub const	EXP: Instruction =  0x0a; //< exponential operation
pub const	SIGNEXTEND: Instruction =  0x0b; //< extend length of signed integer

pub const	LT: Instruction =  0x10;			//< less-than comparision
pub const	GT: Instruction =  0x11; //< greater-than comparision
pub const	SLT: Instruction =  0x12; //< signed less-than comparision
pub const	SGT: Instruction =  0x13; //< signed greater-than comparision
pub const	EQ: Instruction =  0x14; //< equality comparision
pub const	ISZERO: Instruction =  0x15; //< simple not operator
pub const	AND: Instruction =  0x16; //< bitwise AND operation
pub const	OR: Instruction =  0x17; //< bitwise OR operation
pub const	XOR: Instruction =  0x18; //< bitwise XOR operation
pub const	NOT: Instruction =  0x19; //< bitwise NOT opertation
pub const	BYTE: Instruction =  0x1a; //< retrieve single byte from word

pub const	SHA3: Instruction =  0x20;		//< compute SHA3-256 hash

pub const	ADDRESS: Instruction =  0x30;		//< get address of currently executing account
pub const	BALANCE: Instruction =  0x31; //< get balance of the given account
pub const	ORIGIN: Instruction =  0x32; //< get execution origination address
pub const	CALLER: Instruction =  0x33; //< get caller address
pub const	CALLVALUE: Instruction =  0x34; //< get deposited value by the instruction/transaction responsible for this execution
pub const	CALLDATALOAD: Instruction =  0x35; //< get input data of current environment
pub const	CALLDATASIZE: Instruction =  0x36; //< get size of input data in current environment
pub const	CALLDATACOPY: Instruction =  0x37; //< copy input data in current environment to memory
pub const	CODESIZE: Instruction =  0x38; //< get size of code running in current environment
pub const	CODECOPY: Instruction =  0x39; //< copy code running in current environment to memory
pub const	GASPRICE: Instruction =  0x3a; //< get price of gas in current environment
pub const	EXTCODESIZE: Instruction =  0x3b; //< get external code size (from another contract)
pub const	EXTCODECOPY: Instruction =  0x3c; //< copy external code (from another contract)

pub const	BLOCKHASH: Instruction =  0x40;	//< get hash of most recent complete block
pub const	COINBASE: Instruction =  0x41; //< get the block's coinbase address
pub const	TIMESTAMP: Instruction =  0x42; //< get the block's timestamp
pub const	NUMBER: Instruction =  0x43; //< get the block's number
pub const	DIFFICULTY: Instruction =  0x44; //< get the block's difficulty
pub const	GASLIMIT: Instruction =  0x45; //< get the block's gas limit

pub const	POP: Instruction =  0x50;			//< remove item from stack
pub const	MLOAD: Instruction =  0x51; //< load word from memory
pub const	MSTORE: Instruction =  0x52; //< save word to memory
pub const	MSTORE8: Instruction =  0x53; //< save byte to memory
pub const	SLOAD: Instruction =  0x54; //< load word from storage
pub const	SSTORE: Instruction =  0x55; //< save word to storage
pub const	JUMP: Instruction =  0x56; //< alter the program counter
pub const	JUMPI: Instruction =  0x57; //< conditionally alter the program counter
pub const	PC: Instruction =  0x58; //< get the program counter
pub const	MSIZE: Instruction =  0x59; //< get the size of active memory
pub const	GAS: Instruction =  0x5a; //< get the amount of available gas
pub const	JUMPDEST: Instruction =  0x5b; //< set a potential jump destination

pub const	PUSH1: Instruction =  0x60;		//< place 1 byte item on stack
#[allow(dead_code)]
pub const	PUSH2: Instruction =  0x61; //< place 2 byte item on stack
#[allow(dead_code)]
pub const	PUSH3: Instruction =  0x62; //< place 3 byte item on stack
#[allow(dead_code)]
pub const	PUSH4: Instruction =  0x63; //< place 4 byte item on stack
#[allow(dead_code)]
pub const	PUSH5: Instruction =  0x64; //< place 5 byte item on stack
#[allow(dead_code)]
pub const	PUSH6: Instruction =  0x65; //< place 6 byte item on stack
#[allow(dead_code)]
pub const	PUSH7: Instruction =  0x66; //< place 7 byte item on stack
#[allow(dead_code)]
pub const	PUSH8: Instruction =  0x67; //< place 8 byte item on stack
#[allow(dead_code)]
pub const	PUSH9: Instruction =  0x68; //< place 9 byte item on stack
#[allow(dead_code)]
pub const	PUSH10: Instruction =  0x69; //< place 10 byte item on stack
#[allow(dead_code)]
pub const	PUSH11: Instruction =  0x6a; //< place 11 byte item on stack
#[allow(dead_code)]
pub const	PUSH12: Instruction =  0x6b; //< place 12 byte item on stack
#[allow(dead_code)]
pub const	PUSH13: Instruction =  0x6c; //< place 13 byte item on stack
#[allow(dead_code)]
pub const	PUSH14: Instruction =  0x6d; //< place 14 byte item on stack
#[allow(dead_code)]
pub const	PUSH15: Instruction =  0x6e; //< place 15 byte item on stack
#[allow(dead_code)]
pub const	PUSH16: Instruction =  0x6f; //< place 16 byte item on stack
#[allow(dead_code)]
pub const	PUSH17: Instruction =  0x70; //< place 17 byte item on stack
#[allow(dead_code)]
pub const	PUSH18: Instruction =  0x71; //< place 18 byte item on stack
#[allow(dead_code)]
pub const	PUSH19: Instruction =  0x72; //< place 19 byte item on stack
#[allow(dead_code)]
pub const	PUSH20: Instruction =  0x73; //< place 20 byte item on stack
#[allow(dead_code)]
pub const	PUSH21: Instruction =  0x74; //< place 21 byte item on stack
#[allow(dead_code)]
pub const	PUSH22: Instruction =  0x75; //< place 22 byte item on stack
#[allow(dead_code)]
pub const	PUSH23: Instruction =  0x76; //< place 23 byte item on stack
#[allow(dead_code)]
pub const	PUSH24: Instruction =  0x77; //< place 24 byte item on stack
#[allow(dead_code)]
pub const	PUSH25: Instruction =  0x78; //< place 25 byte item on stack
#[allow(dead_code)]
pub const	PUSH26: Instruction =  0x79; //< place 26 byte item on stack
#[allow(dead_code)]
pub const	PUSH27: Instruction =  0x7a; //< place 27 byte item on stack
#[allow(dead_code)]
pub const	PUSH28: Instruction =  0x7b; //< place 28 byte item on stack
#[allow(dead_code)]
pub const	PUSH29: Instruction =  0x7c; //< place 29 byte item on stack
#[allow(dead_code)]
pub const	PUSH30: Instruction =  0x7d; //< place 30 byte item on stack
#[allow(dead_code)]
pub const	PUSH31: Instruction =  0x7e; //< place 31 byte item on stack
pub const	PUSH32: Instruction =  0x7f; //< place 32 byte item on stack

pub const	DUP1: Instruction =  0x80;		//< copies the highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP2: Instruction =  0x81; //< copies the second highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP3: Instruction =  0x82; //< copies the third highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP4: Instruction =  0x83; //< copies the 4th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP5: Instruction =  0x84; //< copies the 5th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP6: Instruction =  0x85; //< copies the 6th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP7: Instruction =  0x86; //< copies the 7th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP8: Instruction =  0x87; //< copies the 8th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP9: Instruction =  0x88; //< copies the 9th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP10: Instruction =  0x89; //< copies the 10th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP11: Instruction =  0x8a; //< copies the 11th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP12: Instruction =  0x8b; //< copies the 12th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP13: Instruction =  0x8c; //< copies the 13th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP14: Instruction =  0x8d; //< copies the 14th highest item in the stack to the top of the stack
#[allow(dead_code)]
pub const	DUP15: Instruction =  0x8e; //< copies the 15th highest item in the stack to the top of the stack
pub const	DUP16: Instruction =  0x8f; //< copies the 16th highest item in the stack to the top of the stack

pub const	SWAP1: Instruction =  0x90;		//< swaps the highest and second highest value on the stack
#[allow(dead_code)]
pub const	SWAP2: Instruction =  0x91; //< swaps the highest and third highest value on the stack
#[allow(dead_code)]
pub const	SWAP3: Instruction =  0x92; //< swaps the highest and 4th highest value on the stack
#[allow(dead_code)]
pub const	SWAP4: Instruction =  0x93; //< swaps the highest and 5th highest value on the stack
#[allow(dead_code)]
pub const	SWAP5: Instruction =  0x94; //< swaps the highest and 6th highest value on the stack
#[allow(dead_code)]
pub const	SWAP6: Instruction =  0x95; //< swaps the highest and 7th highest value on the stack
#[allow(dead_code)]
pub const	SWAP7: Instruction =  0x96; //< swaps the highest and 8th highest value on the stack
#[allow(dead_code)]
pub const	SWAP8: Instruction =  0x97; //< swaps the highest and 9th highest value on the stack
#[allow(dead_code)]
pub const	SWAP9: Instruction =  0x98; //< swaps the highest and 10th highest value on the stack
#[allow(dead_code)]
pub const	SWAP10: Instruction =  0x99; //< swaps the highest and 11th highest value on the stack
#[allow(dead_code)]
pub const	SWAP11: Instruction =  0x9a; //< swaps the highest and 12th highest value on the stack
#[allow(dead_code)]
pub const	SWAP12: Instruction =  0x9b; //< swaps the highest and 13th highest value on the stack
#[allow(dead_code)]
pub const	SWAP13: Instruction =  0x9c; //< swaps the highest and 14th highest value on the stack
#[allow(dead_code)]
pub const	SWAP14: Instruction =  0x9d; //< swaps the highest and 15th highest value on the stack
#[allow(dead_code)]
pub const	SWAP15: Instruction =  0x9e; //< swaps the highest and 16th highest value on the stack
pub const	SWAP16: Instruction =  0x9f; //< swaps the highest and 17th highest value on the stack

pub const	LOG0: Instruction =  0xa0;		//< Makes a log entry; no topics.
#[allow(dead_code)]
pub const	LOG1: Instruction =  0xa1; //< Makes a log entry; 1 topic.
#[allow(dead_code)]
pub const	LOG2: Instruction =  0xa2; //< Makes a log entry; 2 topics.
#[allow(dead_code)]
pub const	LOG3: Instruction =  0xa3; //< Makes a log entry; 3 topics.
pub const	LOG4: Instruction =  0xa4; //< Makes a log entry; 4 topics.

pub const	CREATE: Instruction =  0xf0;		//< create a new account with associated code
pub const	CALL: Instruction =  0xf1; //< message-call into an account
pub const	CALLCODE: Instruction =  0xf2; //< message-call with another account's code only
pub const	RETURN: Instruction =  0xf3; //< halt execution returning output data
pub const	DELEGATECALL: Instruction =  0xf4; //< like CALLCODE but keeps caller's value and sender
pub const	SUICIDE: Instruction =  0xff;		//< halt execution and register account for later deletion

