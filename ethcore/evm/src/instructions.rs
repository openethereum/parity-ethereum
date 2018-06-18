// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! VM Instructions list and utility functions

pub use self::Instruction::*;

/// Virtual machine bytecode instruction.
#[repr(u8)]
#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
	/// halts execution
	STOP = 0x00,
	/// addition operation
	ADD = 0x01,
	/// mulitplication operation
	MUL = 0x02,
	/// subtraction operation
	SUB = 0x03,
	/// integer division operation
	DIV = 0x04,
	/// signed integer division operation
	SDIV = 0x05,
	/// modulo remainder operation
	MOD = 0x06,
	/// signed modulo remainder operation
	SMOD = 0x07,
	/// unsigned modular addition
	ADDMOD = 0x08,
	/// unsigned modular multiplication
	MULMOD = 0x09,
	/// exponential operation
	EXP = 0x0a,
	/// extend length of signed integer
	SIGNEXTEND = 0x0b,

	/// less-than comparision
	LT = 0x10,
	/// greater-than comparision
	GT = 0x11,
	/// signed less-than comparision
	SLT = 0x12,
	/// signed greater-than comparision
	SGT = 0x13,
	/// equality comparision
	EQ = 0x14,
	/// simple not operator
	ISZERO = 0x15,
	/// bitwise AND operation
	AND = 0x16,
	/// bitwise OR operation
	OR = 0x17,
	/// bitwise XOR operation
	XOR = 0x18,
	/// bitwise NOT opertation
	NOT = 0x19,
	/// retrieve single byte from word
	BYTE = 0x1a,
	/// shift left operation
	SHL = 0x1b,
	/// logical shift right operation
	SHR = 0x1c,
	/// arithmetic shift right operation
	SAR = 0x1d,

	/// compute SHA3-256 hash
	SHA3 = 0x20,

	/// get address of currently executing account
	ADDRESS = 0x30,
	/// get balance of the given account
	BALANCE = 0x31,
	/// get execution origination address
	ORIGIN = 0x32,
	/// get caller address
	CALLER = 0x33,
	/// get deposited value by the instruction/transaction responsible for this execution
	CALLVALUE = 0x34,
	/// get input data of current environment
	CALLDATALOAD = 0x35,
	/// get size of input data in current environment
	CALLDATASIZE = 0x36,
	/// copy input data in current environment to memory
	CALLDATACOPY = 0x37,
	/// get size of code running in current environment
	CODESIZE = 0x38,
	/// copy code running in current environment to memory
	CODECOPY = 0x39,
	/// get price of gas in current environment
	GASPRICE = 0x3a,
	/// get external code size (from another contract)
	EXTCODESIZE = 0x3b,
	/// copy external code (from another contract)
	EXTCODECOPY = 0x3c,
	/// get the size of the return data buffer for the last call
	RETURNDATASIZE = 0x3d,
	/// copy return data buffer to memory
	RETURNDATACOPY = 0x3e,

	/// get hash of most recent complete block
	BLOCKHASH = 0x40,
	/// get the block's coinbase address
	COINBASE = 0x41,
	/// get the block's timestamp
	TIMESTAMP = 0x42,
	/// get the block's number
	NUMBER = 0x43,
	/// get the block's difficulty
	DIFFICULTY = 0x44,
	/// get the block's gas limit
	GASLIMIT = 0x45,

	/// remove item from stack
	POP = 0x50,
	/// load word from memory
	MLOAD = 0x51,
	/// save word to memory
	MSTORE = 0x52,
	/// save byte to memory
	MSTORE8 = 0x53,
	/// load word from storage
	SLOAD = 0x54,
	/// save word to storage
	SSTORE = 0x55,
	/// alter the program counter
	JUMP = 0x56,
	/// conditionally alter the program counter
	JUMPI = 0x57,
	/// get the program counter
	PC = 0x58,
	/// get the size of active memory
	MSIZE = 0x59,
	/// get the amount of available gas
	GAS = 0x5a,
	/// set a potential jump destination
	JUMPDEST = 0x5b,

	/// place 1 byte item on stack
	PUSH1 = 0x60,
	/// place 2 byte item on stack
	PUSH2 = 0x61,
	/// place 3 byte item on stack
	PUSH3 = 0x62,
	/// place 4 byte item on stack
	PUSH4 = 0x63,
	/// place 5 byte item on stack
	PUSH5 = 0x64,
	/// place 6 byte item on stack
	PUSH6 = 0x65,
	/// place 7 byte item on stack
	PUSH7 = 0x66,
	/// place 8 byte item on stack
	PUSH8 = 0x67,
	/// place 9 byte item on stack
	PUSH9 = 0x68,
	/// place 10 byte item on stack
	PUSH10 = 0x69,
	/// place 11 byte item on stack
	PUSH11 = 0x6a,
	/// place 12 byte item on stack
	PUSH12 = 0x6b,
	/// place 13 byte item on stack
	PUSH13 = 0x6c,
	/// place 14 byte item on stack
	PUSH14 = 0x6d,
	/// place 15 byte item on stack
	PUSH15 = 0x6e,
	/// place 16 byte item on stack
	PUSH16 = 0x6f,
	/// place 17 byte item on stack
	PUSH17 = 0x70,
	/// place 18 byte item on stack
	PUSH18 = 0x71,
	/// place 19 byte item on stack
	PUSH19 = 0x72,
	/// place 20 byte item on stack
	PUSH20 = 0x73,
	/// place 21 byte item on stack
	PUSH21 = 0x74,
	/// place 22 byte item on stack
	PUSH22 = 0x75,
	/// place 23 byte item on stack
	PUSH23 = 0x76,
	/// place 24 byte item on stack
	PUSH24 = 0x77,
	/// place 25 byte item on stack
	PUSH25 = 0x78,
	/// place 26 byte item on stack
	PUSH26 = 0x79,
	/// place 27 byte item on stack
	PUSH27 = 0x7a,
	/// place 28 byte item on stack
	PUSH28 = 0x7b,
	/// place 29 byte item on stack
	PUSH29 = 0x7c,
	/// place 30 byte item on stack
	PUSH30 = 0x7d,
	/// place 31 byte item on stack
	PUSH31 = 0x7e,
	/// place 32 byte item on stack
	PUSH32 = 0x7f,

	/// copies the highest item in the stack to the top of the stack
	DUP1 = 0x80,
	/// copies the second highest item in the stack to the top of the stack
	DUP2 = 0x81,
	/// copies the third highest item in the stack to the top of the stack
	DUP3 = 0x82,
	/// copies the 4th highest item in the stack to the top of the stack
	DUP4 = 0x83,
	/// copies the 5th highest item in the stack to the top of the stack
	DUP5 = 0x84,
	/// copies the 6th highest item in the stack to the top of the stack
	DUP6 = 0x85,
	/// copies the 7th highest item in the stack to the top of the stack
	DUP7 = 0x86,
	/// copies the 8th highest item in the stack to the top of the stack
	DUP8 = 0x87,
	/// copies the 9th highest item in the stack to the top of the stack
	DUP9 = 0x88,
	/// copies the 10th highest item in the stack to the top of the stack
	DUP10 = 0x89,
	/// copies the 11th highest item in the stack to the top of the stack
	DUP11 = 0x8a,
	/// copies the 12th highest item in the stack to the top of the stack
	DUP12 = 0x8b,
	/// copies the 13th highest item in the stack to the top of the stack
	DUP13 = 0x8c,
	/// copies the 14th highest item in the stack to the top of the stack
	DUP14 = 0x8d,
	/// copies the 15th highest item in the stack to the top of the stack
	DUP15 = 0x8e,
	/// copies the 16th highest item in the stack to the top of the stack
	DUP16 = 0x8f,

	/// swaps the highest and second highest value on the stack
	SWAP1 = 0x90,
	/// swaps the highest and third highest value on the stack
	SWAP2 = 0x91,
	/// swaps the highest and 4th highest value on the stack
	SWAP3 = 0x92,
	/// swaps the highest and 5th highest value on the stack
	SWAP4 = 0x93,
	/// swaps the highest and 6th highest value on the stack
	SWAP5 = 0x94,
	/// swaps the highest and 7th highest value on the stack
	SWAP6 = 0x95,
	/// swaps the highest and 8th highest value on the stack
	SWAP7 = 0x96,
	/// swaps the highest and 9th highest value on the stack
	SWAP8 = 0x97,
	/// swaps the highest and 10th highest value on the stack
	SWAP9 = 0x98,
	/// swaps the highest and 11th highest value on the stack
	SWAP10 = 0x99,
	/// swaps the highest and 12th highest value on the stack
	SWAP11 = 0x9a,
	/// swaps the highest and 13th highest value on the stack
	SWAP12 = 0x9b,
	/// swaps the highest and 14th highest value on the stack
	SWAP13 = 0x9c,
	/// swaps the highest and 15th highest value on the stack
	SWAP14 = 0x9d,
	/// swaps the highest and 16th highest value on the stack
	SWAP15 = 0x9e,
	/// swaps the highest and 17th highest value on the stack
	SWAP16 = 0x9f,

	/// Makes a log entry, no topics.
	LOG0 = 0xa0,
	/// Makes a log entry, 1 topic.
	LOG1 = 0xa1,
	/// Makes a log entry, 2 topics.
	LOG2 = 0xa2,
	/// Makes a log entry, 3 topics.
	LOG3 = 0xa3,
	/// Makes a log entry, 4 topics.
	LOG4 = 0xa4,

	/// create a new account with associated code
	CREATE = 0xf0,
	/// message-call into an account
	CALL = 0xf1,
	/// message-call with another account's code only
	CALLCODE = 0xf2,
	/// halt execution returning output data
	RETURN = 0xf3,
	/// like CALLCODE but keeps caller's value and sender
	DELEGATECALL = 0xf4,
	/// create a new account and set creation address to sha3(sender + sha3(init code)) % 2**160
	CREATE2 = 0xfb,
	/// stop execution and revert state changes. Return output data.
	REVERT = 0xfd,
	/// like CALL but it does not take value, nor modify the state
	STATICCALL = 0xfa,
	/// halt execution and register account for later deletion
	SUICIDE = 0xff,
}

impl Instruction {
	pub fn from_u8(value: u8) -> Option<Instruction> {
		match value {
			0x00 => Instruction::STOP,
			0x01 => Instruction::ADD,
			0x02 => Instruction::MUL,
			0x03 => Instruction::SUB,
			0x04 => Instruction::DIV,
			0x05 => Instruction::SDIV,
			0x06 => Instruction::MOD,
			0x07 => Instruction::SMOD,
			0x08 => Instruction::ADDMOD,
			0x09 => Instruction::MULMOD,
			0x0a => Instruction::EXP,
			0x0b => Instruction::SIGNEXTEND,

			0x10 => Instruction::LT,
			0x11 => Instruction::GT,
			0x12 => Instruction::SLT,
			0x13 => Instruction::SGT,
			0x14 => Instruction::EQ,
			0x15 => Instruction::ISZERO,
			0x16 => Instruction::AND,
			0x17 => Instruction::OR,
			0x18 => Instruction::XOR,
			0x19 => Instruction::NOT,
			0x1a => Instruction::BYTE,
			0x1b => Instruction::SHL,
			0x1c => Instruction::SHR,
			0x1d => Instruction::SAR,

			0x20 => Instruction::SHA3,

			0x30 => Instruction::ADDRESS,
			0x31 => Instruction::BALANCE,
			0x32 => Instruction::ORIGIN,
			0x33 => Instruction::CALLER,
			0x34 => Instruction::CALLVALUE,
			0x35 => Instruction::CALLDATALOAD,
			0x36 => Instruction::CALLDATASIZE,
			0x37 => Instruction::CALLDATACOPY,
			0x38 => Instruction::CODESIZE,
			0x39 => Instruction::CODECOPY,
			0x3a => Instruction::GASPRICE,
			0x3b => Instruction::EXTCODESIZE,
			0x3c => Instruction::EXTCODECOPY,
			0x3d => Instruction::RETURNDATASIZE,
			0x3e => Instruction::RETURNDATACOPY,

			0x40 => Instruction::BLOCKHASH,
			0x41 => Instruction::COINBASE,
			0x42 => Instruction::TIMESTAMP,
			0x43 => Instruction::NUMBER,
			0x44 => Instruction::DIFFICULTY,
			0x45 => Instruction::GASLIMIT,

			0x50 => Instruction::POP,
			0x51 => Instruction::MLOAD,
			0x52 => Instruction::MSTORE,
			0x53 => Instruction::MSTORE8,
			0x54 => Instruction::SLOAD,
			0x55 => Instruction::SSTORE,
			0x56 => Instruction::JUMP,
			0x57 => Instruction::JUMPI,
			0x58 => Instruction::PC,
			0x59 => Instruction::MSIZE,
			0x5a => Instruction::GAS,
			0x5b => Instruction::JUMPDEST,

			0x60 => Instruction::PUSH1,
			0x61 => Instruction::PUSH2,
			0x62 => Instruction::PUSH3,
			0x63 => Instruction::PUSH4,
			0x64 => Instruction::PUSH5,
			0x65 => Instruction::PUSH6,
			0x66 => Instruction::PUSH7,
			0x67 => Instruction::PUSH8,
			0x68 => Instruction::PUSH9,
			0x69 => Instruction::PUSH10,
			0x6a => Instruction::PUSH11,
			0x6b => Instruction::PUSH12,
			0x6c => Instruction::PUSH13,
			0x6d => Instruction::PUSH14,
			0x6e => Instruction::PUSH15,
			0x6f => Instruction::PUSH16,
			0x70 => Instruction::PUSH17,
			0x71 => Instruction::PUSH18,
			0x72 => Instruction::PUSH19,
			0x73 => Instruction::PUSH20,
			0x74 => Instruction::PUSH21,
			0x75 => Instruction::PUSH22,
			0x76 => Instruction::PUSH23,
			0x77 => Instruction::PUSH24,
			0x78 => Instruction::PUSH25,
			0x79 => Instruction::PUSH26,
			0x7a => Instruction::PUSH27,
			0x7b => Instruction::PUSH28,
			0x7c => Instruction::PUSH29,
			0x7d => Instruction::PUSH30,
			0x7e => Instruction::PUSH31,
			0x7f => Instruction::PUSH32,

			0x80 => Instruction::DUP1,
			0x81 => Instruction::DUP2,
			0x82 => Instruction::DUP3,
			0x83 => Instruction::DUP4,
			0x84 => Instruction::DUP5,
			0x85 => Instruction::DUP6,
			0x86 => Instruction::DUP7,
			0x87 => Instruction::DUP8,
			0x88 => Instruction::DUP9,
			0x89 => Instruction::DUP10,
			0x8a => Instruction::DUP11,
			0x8b => Instruction::DUP12,
			0x8c => Instruction::DUP13,
			0x8d => Instruction::DUP14,
			0x8e => Instruction::DUP15,
			0x8f => Instruction::DUP16,

			0x90 => Instruction::SWAP1,
			0x91 => Instruction::SWAP2,
			0x92 => Instruction::SWAP3,
			0x93 => Instruction::SWAP4,
			0x94 => Instruction::SWAP5,
			0x95 => Instruction::SWAP6,
			0x96 => Instruction::SWAP7,
			0x97 => Instruction::SWAP8,
			0x98 => Instruction::SWAP9,
			0x99 => Instruction::SWAP10,
			0x9a => Instruction::SWAP11,
			0x9b => Instruction::SWAP12,
			0x9c => Instruction::SWAP13,
			0x9d => Instruction::SWAP14,
			0x9e => Instruction::SWAP15,
			0x9f => Instruction::SWAP16,

			0xa0 => Instruction::LOG0,
			0xa1 => Instruction::LOG1,
			0xa2 => Instruction::LOG2,
			0xa3 => Instruction::LOG3,
			0xa4 => Instruction::LOG4,

			0xf0 => Instruction::CREATE,
			0xf1 => Instruction::CALL,
			0xf2 => Instruction::CALLCODE,
			0xf3 => Instruction::RETURN,
			0xf4 => Instruction::DELEGATECALL,
			0xfb => Instruction::CREATE2,
			0xfd => Instruction::REVERT,
			0xfa => Instruction::STATICCALL,
			0xff => Instruction::SUICIDE,
		}
	}
}

/// Returns true if given instruction is `PUSHN` instruction.
pub fn is_push(i: Instruction) -> bool {
	i >= PUSH1 && i <= PUSH32
}

#[test]
fn test_is_push() {
	assert!(is_push(PUSH1));
	assert!(is_push(PUSH32));
	assert!(!is_push(DUP1));
}

/// Returns number of bytes to read for `PUSHN` instruction
/// PUSH1 -> 1
pub fn get_push_bytes(i: Instruction) -> usize {
	assert!(is_push(i), "Only for PUSH instructions.");
	(i - PUSH1 + 1) as usize
}

/// Returns number of bytes to read for `PUSHN` instruction or 0.
pub fn push_bytes(i: Instruction) -> usize {
	if is_push(i) {
		get_push_bytes(i)
	} else {
		0
	}
}

#[test]
fn test_get_push_bytes() {
	assert_eq!(get_push_bytes(PUSH1), 1);
	assert_eq!(get_push_bytes(PUSH3), 3);
	assert_eq!(get_push_bytes(PUSH32), 32);
}

/// Returns stack position of item to duplicate
/// DUP1 -> 0
pub fn get_dup_position(i: Instruction) -> usize {
	assert!(i >= DUP1 && i <= DUP16);
	(i - DUP1) as usize
}

#[test]
fn test_get_dup_position() {
	assert_eq!(get_dup_position(DUP1), 0);
	assert_eq!(get_dup_position(DUP5), 4);
	assert_eq!(get_dup_position(DUP10), 9);
}

/// Returns stack position of item to SWAP top with
/// SWAP1 -> 1
pub fn get_swap_position(i: Instruction) -> usize {
	assert!(i >= SWAP1 && i <= SWAP16);
	(i - SWAP1 + 1) as usize
}

#[test]
fn test_get_swap_position() {
	assert_eq!(get_swap_position(SWAP1), 1);
	assert_eq!(get_swap_position(SWAP5), 5);
	assert_eq!(get_swap_position(SWAP10), 10);
}

/// Returns number of topics to take from stack
/// LOG0 -> 0
pub fn get_log_topics (i: Instruction) -> usize {
	assert!(i >= LOG0 && i <= LOG4);
	(i - LOG0) as usize
}

#[test]
fn test_get_log_topics() {
	assert_eq!(get_log_topics(LOG0), 0);
	assert_eq!(get_log_topics(LOG2), 2);
	assert_eq!(get_log_topics(LOG4), 4);
}

#[derive(PartialEq, Clone, Copy)]
pub enum GasPriceTier {
	/// 0 Zero
	Zero,
	/// 2 Quick
	Base,
	/// 3 Fastest
	VeryLow,
	/// 5 Fast
	Low,
	/// 8 Mid
	Mid,
	/// 10 Slow
	High,
	/// 20 Ext
	Ext,
	/// Multiparam or otherwise special
	Special,
	/// Invalid
	Invalid
}

impl Default for GasPriceTier {
	fn default() -> Self {
		GasPriceTier::Invalid
	}
}

/// Returns the index in schedule for specific `GasPriceTier`
pub fn get_tier_idx (tier: GasPriceTier) -> usize {
	match tier {
		GasPriceTier::Zero => 0,
		GasPriceTier::Base => 1,
		GasPriceTier::VeryLow => 2,
		GasPriceTier::Low => 3,
		GasPriceTier::Mid => 4,
		GasPriceTier::High => 5,
		GasPriceTier::Ext => 6,
		GasPriceTier::Special => 7,
		GasPriceTier::Invalid => 8
	}
}

/// EVM instruction information.
#[derive(Copy, Clone, Default)]
pub struct InstructionInfo {
	/// Mnemonic name.
	pub name: &'static str,
	/// Number of stack arguments.
	pub args: usize,
	/// Number of returned stack items.
	pub ret: usize,
	/// Gas price tier.
	pub tier: GasPriceTier
}

impl InstructionInfo {
	/// Create new instruction info.
	pub fn new(name: &'static str, args: usize, ret: usize, tier: GasPriceTier) -> Self {
		InstructionInfo {
			name: name,
			args: args,
			ret: ret,
			tier: tier
		}
	}
}

lazy_static! {
	/// Static instruction table.
	pub static ref INSTRUCTIONS: [InstructionInfo; 0x100] = {
		let mut arr = [InstructionInfo::default(); 0x100];
		arr[STOP as usize] =			InstructionInfo::new("STOP",			0, 0, GasPriceTier::Zero);
		arr[ADD as usize] = 			InstructionInfo::new("ADD",				2, 1, GasPriceTier::VeryLow);
		arr[SUB as usize] = 			InstructionInfo::new("SUB",				2, 1, GasPriceTier::VeryLow);
		arr[MUL as usize] = 			InstructionInfo::new("MUL",				2, 1, GasPriceTier::Low);
		arr[DIV as usize] = 			InstructionInfo::new("DIV",				2, 1, GasPriceTier::Low);
		arr[SDIV as usize] =			InstructionInfo::new("SDIV",			2, 1, GasPriceTier::Low);
		arr[MOD as usize] = 			InstructionInfo::new("MOD",				2, 1, GasPriceTier::Low);
		arr[SMOD as usize] =			InstructionInfo::new("SMOD",			2, 1, GasPriceTier::Low);
		arr[EXP as usize] = 			InstructionInfo::new("EXP",				2, 1, GasPriceTier::Special);
		arr[NOT as usize] = 			InstructionInfo::new("NOT",				1, 1, GasPriceTier::VeryLow);
		arr[LT as usize] =				InstructionInfo::new("LT",				2, 1, GasPriceTier::VeryLow);
		arr[GT as usize] =				InstructionInfo::new("GT",				2, 1, GasPriceTier::VeryLow);
		arr[SLT as usize] = 			InstructionInfo::new("SLT",				2, 1, GasPriceTier::VeryLow);
		arr[SGT as usize] = 			InstructionInfo::new("SGT",				2, 1, GasPriceTier::VeryLow);
		arr[EQ as usize] =				InstructionInfo::new("EQ",				2, 1, GasPriceTier::VeryLow);
		arr[ISZERO as usize] =			InstructionInfo::new("ISZERO",			1, 1, GasPriceTier::VeryLow);
		arr[AND as usize] = 			InstructionInfo::new("AND",				2, 1, GasPriceTier::VeryLow);
		arr[OR as usize] =				InstructionInfo::new("OR",				2, 1, GasPriceTier::VeryLow);
		arr[XOR as usize] = 			InstructionInfo::new("XOR",				2, 1, GasPriceTier::VeryLow);
		arr[BYTE as usize] =			InstructionInfo::new("BYTE",			2, 1, GasPriceTier::VeryLow);
		arr[SHL as usize] =             InstructionInfo::new("SHL",             2, 1, GasPriceTier::VeryLow);
		arr[SHR as usize] =             InstructionInfo::new("SHR",             2, 1, GasPriceTier::VeryLow);
		arr[SAR as usize] =             InstructionInfo::new("SAR",             2, 1, GasPriceTier::VeryLow);
		arr[ADDMOD as usize] =			InstructionInfo::new("ADDMOD",			3, 1, GasPriceTier::Mid);
		arr[MULMOD as usize] =			InstructionInfo::new("MULMOD",			3, 1, GasPriceTier::Mid);
		arr[SIGNEXTEND as usize] =		InstructionInfo::new("SIGNEXTEND",		2, 1, GasPriceTier::Low);
		arr[RETURNDATASIZE as usize] =	InstructionInfo::new("RETURNDATASIZE",	0, 1, GasPriceTier::Base);
		arr[RETURNDATACOPY as usize] =	InstructionInfo::new("RETURNDATACOPY",	3, 0, GasPriceTier::VeryLow);
		arr[SHA3 as usize] =			InstructionInfo::new("SHA3",			2, 1, GasPriceTier::Special);
		arr[ADDRESS as usize] = 		InstructionInfo::new("ADDRESS",			0, 1, GasPriceTier::Base);
		arr[BALANCE as usize] = 		InstructionInfo::new("BALANCE",			1, 1, GasPriceTier::Special);
		arr[ORIGIN as usize] =			InstructionInfo::new("ORIGIN",			0, 1, GasPriceTier::Base);
		arr[CALLER as usize] =			InstructionInfo::new("CALLER",			0, 1, GasPriceTier::Base);
		arr[CALLVALUE as usize] =		InstructionInfo::new("CALLVALUE",		0, 1, GasPriceTier::Base);
		arr[CALLDATALOAD as usize] =	InstructionInfo::new("CALLDATALOAD",	1, 1, GasPriceTier::VeryLow);
		arr[CALLDATASIZE as usize] =	InstructionInfo::new("CALLDATASIZE",	0, 1, GasPriceTier::Base);
		arr[CALLDATACOPY as usize] =	InstructionInfo::new("CALLDATACOPY",	3, 0, GasPriceTier::VeryLow);
		arr[CODESIZE as usize] =		InstructionInfo::new("CODESIZE",		0, 1, GasPriceTier::Base);
		arr[CODECOPY as usize] =		InstructionInfo::new("CODECOPY",		3, 0, GasPriceTier::VeryLow);
		arr[GASPRICE as usize] =		InstructionInfo::new("GASPRICE",		0, 1, GasPriceTier::Base);
		arr[EXTCODESIZE as usize] = 	InstructionInfo::new("EXTCODESIZE",		1, 1, GasPriceTier::Special);
		arr[EXTCODECOPY as usize] = 	InstructionInfo::new("EXTCODECOPY",		4, 0, GasPriceTier::Special);
		arr[BLOCKHASH as usize] =		InstructionInfo::new("BLOCKHASH",		1, 1, GasPriceTier::Ext);
		arr[COINBASE as usize] =		InstructionInfo::new("COINBASE",		0, 1, GasPriceTier::Base);
		arr[TIMESTAMP as usize] =		InstructionInfo::new("TIMESTAMP",		0, 1, GasPriceTier::Base);
		arr[NUMBER as usize] =			InstructionInfo::new("NUMBER",			0, 1, GasPriceTier::Base);
		arr[DIFFICULTY as usize] =		InstructionInfo::new("DIFFICULTY",		0, 1, GasPriceTier::Base);
		arr[GASLIMIT as usize] =		InstructionInfo::new("GASLIMIT",		0, 1, GasPriceTier::Base);
		arr[POP as usize] = 			InstructionInfo::new("POP",				1, 0, GasPriceTier::Base);
		arr[MLOAD as usize] =			InstructionInfo::new("MLOAD",			1, 1, GasPriceTier::VeryLow);
		arr[MSTORE as usize] =			InstructionInfo::new("MSTORE",			2, 0, GasPriceTier::VeryLow);
		arr[MSTORE8 as usize] = 		InstructionInfo::new("MSTORE8",			2, 0, GasPriceTier::VeryLow);
		arr[SLOAD as usize] =			InstructionInfo::new("SLOAD",			1, 1, GasPriceTier::Special);
		arr[SSTORE as usize] =			InstructionInfo::new("SSTORE",			2, 0, GasPriceTier::Special);
		arr[JUMP as usize] =			InstructionInfo::new("JUMP",			1, 0, GasPriceTier::Mid);
		arr[JUMPI as usize] =			InstructionInfo::new("JUMPI",			2, 0, GasPriceTier::High);
		arr[PC as usize] =				InstructionInfo::new("PC",				0, 1, GasPriceTier::Base);
		arr[MSIZE as usize] =			InstructionInfo::new("MSIZE",			0, 1, GasPriceTier::Base);
		arr[GAS as usize] = 			InstructionInfo::new("GAS",				0, 1, GasPriceTier::Base);
		arr[JUMPDEST as usize] =		InstructionInfo::new("JUMPDEST",		0, 0, GasPriceTier::Special);
		arr[PUSH1 as usize] =			InstructionInfo::new("PUSH1",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH2 as usize] =			InstructionInfo::new("PUSH2",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH3 as usize] =			InstructionInfo::new("PUSH3",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH4 as usize] =			InstructionInfo::new("PUSH4",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH5 as usize] =			InstructionInfo::new("PUSH5",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH6 as usize] =			InstructionInfo::new("PUSH6",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH7 as usize] =			InstructionInfo::new("PUSH7",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH8 as usize] =			InstructionInfo::new("PUSH8",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH9 as usize] =			InstructionInfo::new("PUSH9",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH10 as usize] =			InstructionInfo::new("PUSH10",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH11 as usize] =			InstructionInfo::new("PUSH11",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH12 as usize] =			InstructionInfo::new("PUSH12",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH13 as usize] =			InstructionInfo::new("PUSH13",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH14 as usize] =			InstructionInfo::new("PUSH14",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH15 as usize] =			InstructionInfo::new("PUSH15",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH16 as usize] =			InstructionInfo::new("PUSH16",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH17 as usize] =			InstructionInfo::new("PUSH17",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH18 as usize] =			InstructionInfo::new("PUSH18",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH19 as usize] =			InstructionInfo::new("PUSH19",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH20 as usize] =			InstructionInfo::new("PUSH20",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH21 as usize] =			InstructionInfo::new("PUSH21",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH22 as usize] =			InstructionInfo::new("PUSH22",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH23 as usize] =			InstructionInfo::new("PUSH23",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH24 as usize] =			InstructionInfo::new("PUSH24",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH25 as usize] =			InstructionInfo::new("PUSH25",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH26 as usize] =			InstructionInfo::new("PUSH26",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH27 as usize] =			InstructionInfo::new("PUSH27",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH28 as usize] =			InstructionInfo::new("PUSH28",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH29 as usize] =			InstructionInfo::new("PUSH29",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH30 as usize] =			InstructionInfo::new("PUSH30",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH31 as usize] =			InstructionInfo::new("PUSH31",			0, 1, GasPriceTier::VeryLow);
		arr[PUSH32 as usize] =			InstructionInfo::new("PUSH32",			0, 1, GasPriceTier::VeryLow);
		arr[DUP1 as usize] =			InstructionInfo::new("DUP1",			1, 2, GasPriceTier::VeryLow);
		arr[DUP2 as usize] =			InstructionInfo::new("DUP2",			2, 3, GasPriceTier::VeryLow);
		arr[DUP3 as usize] =			InstructionInfo::new("DUP3",			3, 4, GasPriceTier::VeryLow);
		arr[DUP4 as usize] =			InstructionInfo::new("DUP4",			4, 5, GasPriceTier::VeryLow);
		arr[DUP5 as usize] =			InstructionInfo::new("DUP5",			5, 6, GasPriceTier::VeryLow);
		arr[DUP6 as usize] =			InstructionInfo::new("DUP6",			6, 7, GasPriceTier::VeryLow);
		arr[DUP7 as usize] =			InstructionInfo::new("DUP7",			7, 8, GasPriceTier::VeryLow);
		arr[DUP8 as usize] =			InstructionInfo::new("DUP8",			8, 9, GasPriceTier::VeryLow);
		arr[DUP9 as usize] =			InstructionInfo::new("DUP9",			9, 10, GasPriceTier::VeryLow);
		arr[DUP10 as usize] =			InstructionInfo::new("DUP10",			10, 11, GasPriceTier::VeryLow);
		arr[DUP11 as usize] =			InstructionInfo::new("DUP11",			11, 12, GasPriceTier::VeryLow);
		arr[DUP12 as usize] =			InstructionInfo::new("DUP12",			12, 13, GasPriceTier::VeryLow);
		arr[DUP13 as usize] =			InstructionInfo::new("DUP13",			13, 14, GasPriceTier::VeryLow);
		arr[DUP14 as usize] =			InstructionInfo::new("DUP14",			14, 15, GasPriceTier::VeryLow);
		arr[DUP15 as usize] =			InstructionInfo::new("DUP15",			15, 16, GasPriceTier::VeryLow);
		arr[DUP16 as usize] =			InstructionInfo::new("DUP16",			16, 17, GasPriceTier::VeryLow);
		arr[SWAP1 as usize] =			InstructionInfo::new("SWAP1",			2, 2, GasPriceTier::VeryLow);
		arr[SWAP2 as usize] =			InstructionInfo::new("SWAP2",			3, 3, GasPriceTier::VeryLow);
		arr[SWAP3 as usize] =			InstructionInfo::new("SWAP3",			4, 4, GasPriceTier::VeryLow);
		arr[SWAP4 as usize] =			InstructionInfo::new("SWAP4",			5, 5, GasPriceTier::VeryLow);
		arr[SWAP5 as usize] =			InstructionInfo::new("SWAP5",			6, 6, GasPriceTier::VeryLow);
		arr[SWAP6 as usize] =			InstructionInfo::new("SWAP6",			7, 7, GasPriceTier::VeryLow);
		arr[SWAP7 as usize] =			InstructionInfo::new("SWAP7",			8, 8, GasPriceTier::VeryLow);
		arr[SWAP8 as usize] =			InstructionInfo::new("SWAP8",			9, 9, GasPriceTier::VeryLow);
		arr[SWAP9 as usize] =			InstructionInfo::new("SWAP9",			10, 10, GasPriceTier::VeryLow);
		arr[SWAP10 as usize] =			InstructionInfo::new("SWAP10",			11, 11, GasPriceTier::VeryLow);
		arr[SWAP11 as usize] =			InstructionInfo::new("SWAP11",			12, 12, GasPriceTier::VeryLow);
		arr[SWAP12 as usize] =			InstructionInfo::new("SWAP12",			13, 13, GasPriceTier::VeryLow);
		arr[SWAP13 as usize] =			InstructionInfo::new("SWAP13",			14, 14, GasPriceTier::VeryLow);
		arr[SWAP14 as usize] =			InstructionInfo::new("SWAP14",			15, 15, GasPriceTier::VeryLow);
		arr[SWAP15 as usize] =			InstructionInfo::new("SWAP15",			16, 16, GasPriceTier::VeryLow);
		arr[SWAP16 as usize] =			InstructionInfo::new("SWAP16",			17, 17, GasPriceTier::VeryLow);
		arr[LOG0 as usize] =			InstructionInfo::new("LOG0",			2, 0, GasPriceTier::Special);
		arr[LOG1 as usize] =			InstructionInfo::new("LOG1",			3, 0, GasPriceTier::Special);
		arr[LOG2 as usize] =			InstructionInfo::new("LOG2",			4, 0, GasPriceTier::Special);
		arr[LOG3 as usize] =			InstructionInfo::new("LOG3",			5, 0, GasPriceTier::Special);
		arr[LOG4 as usize] =			InstructionInfo::new("LOG4",			6, 0, GasPriceTier::Special);
		arr[CREATE as usize] =			InstructionInfo::new("CREATE",			3, 1, GasPriceTier::Special);
		arr[CALL as usize] =			InstructionInfo::new("CALL",			7, 1, GasPriceTier::Special);
		arr[CALLCODE as usize] =		InstructionInfo::new("CALLCODE",		7, 1, GasPriceTier::Special);
		arr[RETURN as usize] =			InstructionInfo::new("RETURN",			2, 0, GasPriceTier::Zero);
		arr[DELEGATECALL as usize] =	InstructionInfo::new("DELEGATECALL",	6, 1, GasPriceTier::Special);
		arr[STATICCALL as usize] =		InstructionInfo::new("STATICCALL",		6, 1, GasPriceTier::Special);
		arr[SUICIDE as usize] = 		InstructionInfo::new("SUICIDE",			1, 0, GasPriceTier::Special);
		arr[CREATE2 as usize] = 		InstructionInfo::new("CREATE2",			3, 1, GasPriceTier::Special);
		arr[REVERT as usize] =			InstructionInfo::new("REVERT",			2, 0, GasPriceTier::Zero);
		arr
	};
}

/// Maximal number of topics for log instructions
pub const MAX_NO_OF_TOPICS : usize = 4;
