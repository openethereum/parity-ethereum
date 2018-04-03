// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

pub type Instruction = u8;

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

/// Virtual machine bytecode instruction.
/// halts execution
pub const STOP: Instruction = 0x00;
/// addition operation
pub const ADD: Instruction = 0x01;
/// mulitplication operation
pub const MUL: Instruction = 0x02;
/// subtraction operation
pub const SUB: Instruction = 0x03;
/// integer division operation
pub const DIV: Instruction = 0x04;
/// signed integer division operation
pub const SDIV: Instruction = 0x05;
/// modulo remainder operation
pub const MOD: Instruction = 0x06;
/// signed modulo remainder operation
pub const SMOD: Instruction = 0x07;
/// unsigned modular addition
pub const ADDMOD: Instruction = 0x08;
/// unsigned modular multiplication
pub const MULMOD: Instruction = 0x09;
/// exponential operation
pub const EXP: Instruction = 0x0a;
/// extend length of signed integer
pub const SIGNEXTEND: Instruction = 0x0b;

/// less-than comparision
pub const LT: Instruction = 0x10;
/// greater-than comparision
pub const GT: Instruction = 0x11;
/// signed less-than comparision
pub const SLT: Instruction = 0x12;
/// signed greater-than comparision
pub const SGT: Instruction = 0x13;
/// equality comparision
pub const EQ: Instruction = 0x14;
/// simple not operator
pub const ISZERO: Instruction = 0x15;
/// bitwise AND operation
pub const AND: Instruction = 0x16;
/// bitwise OR operation
pub const OR: Instruction = 0x17;
/// bitwise XOR operation
pub const XOR: Instruction = 0x18;
/// bitwise NOT opertation
pub const NOT: Instruction = 0x19;
/// retrieve single byte from word
pub const BYTE: Instruction = 0x1a;

/// compute SHA3-256 hash
pub const SHA3: Instruction = 0x20;

/// get address of currently executing account
pub const ADDRESS: Instruction = 0x30;
/// get balance of the given account
pub const BALANCE: Instruction = 0x31;
/// get execution origination address
pub const ORIGIN: Instruction = 0x32;
/// get caller address
pub const CALLER: Instruction = 0x33;
/// get deposited value by the instruction/transaction responsible for this execution
pub const CALLVALUE: Instruction = 0x34;
/// get input data of current environment
pub const CALLDATALOAD: Instruction = 0x35;
/// get size of input data in current environment
pub const CALLDATASIZE: Instruction = 0x36;
/// copy input data in current environment to memory
pub const CALLDATACOPY: Instruction = 0x37;
/// get size of code running in current environment
pub const CODESIZE: Instruction = 0x38;
/// copy code running in current environment to memory
pub const CODECOPY: Instruction = 0x39;
/// get price of gas in current environment
pub const GASPRICE: Instruction = 0x3a;
/// get external code size (from another contract)
pub const EXTCODESIZE: Instruction = 0x3b;
/// copy external code (from another contract)
pub const EXTCODECOPY: Instruction = 0x3c;
/// get the size of the return data buffer for the last call
pub const RETURNDATASIZE: Instruction = 0x3d;
/// copy return data buffer to memory
pub const RETURNDATACOPY: Instruction = 0x3e;

/// get hash of most recent complete block
pub const BLOCKHASH: Instruction = 0x40;
/// get the block's coinbase address
pub const COINBASE: Instruction = 0x41;
/// get the block's timestamp
pub const TIMESTAMP: Instruction = 0x42;
/// get the block's number
pub const NUMBER: Instruction = 0x43;
/// get the block's difficulty
pub const DIFFICULTY: Instruction = 0x44;
/// get the block's gas limit
pub const GASLIMIT: Instruction = 0x45;

/// remove item from stack
pub const POP: Instruction = 0x50;
/// load word from memory
pub const MLOAD: Instruction = 0x51;
/// save word to memory
pub const MSTORE: Instruction = 0x52;
/// save byte to memory
pub const MSTORE8: Instruction = 0x53;
/// load word from storage
pub const SLOAD: Instruction = 0x54;
/// save word to storage
pub const SSTORE: Instruction = 0x55;
/// alter the program counter
pub const JUMP: Instruction = 0x56;
/// conditionally alter the program counter
pub const JUMPI: Instruction = 0x57;
/// get the program counter
pub const PC: Instruction = 0x58;
/// get the size of active memory
pub const MSIZE: Instruction = 0x59;
/// get the amount of available gas
pub const GAS: Instruction = 0x5a;
/// set a potential jump destination
pub const JUMPDEST: Instruction = 0x5b;

/// place 1 byte item on stack
pub const PUSH1: Instruction = 0x60;
/// place 2 byte item on stack
pub const PUSH2: Instruction = 0x61;
/// place 3 byte item on stack
pub const PUSH3: Instruction = 0x62;
/// place 4 byte item on stack
pub const PUSH4: Instruction = 0x63;
/// place 5 byte item on stack
pub const PUSH5: Instruction = 0x64;
/// place 6 byte item on stack
pub const PUSH6: Instruction = 0x65;
/// place 7 byte item on stack
pub const PUSH7: Instruction = 0x66;
/// place 8 byte item on stack
pub const PUSH8: Instruction = 0x67;
/// place 9 byte item on stack
pub const PUSH9: Instruction = 0x68;
/// place 10 byte item on stack
pub const PUSH10: Instruction = 0x69;
/// place 11 byte item on stack
pub const PUSH11: Instruction = 0x6a;
/// place 12 byte item on stack
pub const PUSH12: Instruction = 0x6b;
/// place 13 byte item on stack
pub const PUSH13: Instruction = 0x6c;
/// place 14 byte item on stack
pub const PUSH14: Instruction = 0x6d;
/// place 15 byte item on stack
pub const PUSH15: Instruction = 0x6e;
/// place 16 byte item on stack
pub const PUSH16: Instruction = 0x6f;
/// place 17 byte item on stack
pub const PUSH17: Instruction = 0x70;
/// place 18 byte item on stack
pub const PUSH18: Instruction = 0x71;
/// place 19 byte item on stack
pub const PUSH19: Instruction = 0x72;
/// place 20 byte item on stack
pub const PUSH20: Instruction = 0x73;
/// place 21 byte item on stack
pub const PUSH21: Instruction = 0x74;
/// place 22 byte item on stack
pub const PUSH22: Instruction = 0x75;
/// place 23 byte item on stack
pub const PUSH23: Instruction = 0x76;
/// place 24 byte item on stack
pub const PUSH24: Instruction = 0x77;
/// place 25 byte item on stack
pub const PUSH25: Instruction = 0x78;
/// place 26 byte item on stack
pub const PUSH26: Instruction = 0x79;
/// place 27 byte item on stack
pub const PUSH27: Instruction = 0x7a;
/// place 28 byte item on stack
pub const PUSH28: Instruction = 0x7b;
/// place 29 byte item on stack
pub const PUSH29: Instruction = 0x7c;
/// place 30 byte item on stack
pub const PUSH30: Instruction = 0x7d;
/// place 31 byte item on stack
pub const PUSH31: Instruction = 0x7e;
/// place 32 byte item on stack
pub const PUSH32: Instruction = 0x7f;

/// copies the highest item in the stack to the top of the stack
pub const DUP1: Instruction = 0x80;
/// copies the second highest item in the stack to the top of the stack
pub const DUP2: Instruction = 0x81;
/// copies the third highest item in the stack to the top of the stack
pub const DUP3: Instruction = 0x82;
/// copies the 4th highest item in the stack to the top of the stack
pub const DUP4: Instruction = 0x83;
/// copies the 5th highest item in the stack to the top of the stack
pub const DUP5: Instruction = 0x84;
/// copies the 6th highest item in the stack to the top of the stack
pub const DUP6: Instruction = 0x85;
/// copies the 7th highest item in the stack to the top of the stack
pub const DUP7: Instruction = 0x86;
/// copies the 8th highest item in the stack to the top of the stack
pub const DUP8: Instruction = 0x87;
/// copies the 9th highest item in the stack to the top of the stack
pub const DUP9: Instruction = 0x88;
/// copies the 10th highest item in the stack to the top of the stack
pub const DUP10: Instruction = 0x89;
/// copies the 11th highest item in the stack to the top of the stack
pub const DUP11: Instruction = 0x8a;
/// copies the 12th highest item in the stack to the top of the stack
pub const DUP12: Instruction = 0x8b;
/// copies the 13th highest item in the stack to the top of the stack
pub const DUP13: Instruction = 0x8c;
/// copies the 14th highest item in the stack to the top of the stack
pub const DUP14: Instruction = 0x8d;
/// copies the 15th highest item in the stack to the top of the stack
pub const DUP15: Instruction = 0x8e;
/// copies the 16th highest item in the stack to the top of the stack
pub const DUP16: Instruction = 0x8f;

/// swaps the highest and second highest value on the stack
pub const SWAP1: Instruction = 0x90;
/// swaps the highest and third highest value on the stack
pub const SWAP2: Instruction = 0x91;
/// swaps the highest and 4th highest value on the stack
pub const SWAP3: Instruction = 0x92;
/// swaps the highest and 5th highest value on the stack
pub const SWAP4: Instruction = 0x93;
/// swaps the highest and 6th highest value on the stack
pub const SWAP5: Instruction = 0x94;
/// swaps the highest and 7th highest value on the stack
pub const SWAP6: Instruction = 0x95;
/// swaps the highest and 8th highest value on the stack
pub const SWAP7: Instruction = 0x96;
/// swaps the highest and 9th highest value on the stack
pub const SWAP8: Instruction = 0x97;
/// swaps the highest and 10th highest value on the stack
pub const SWAP9: Instruction = 0x98;
/// swaps the highest and 11th highest value on the stack
pub const SWAP10: Instruction = 0x99;
/// swaps the highest and 12th highest value on the stack
pub const SWAP11: Instruction = 0x9a;
/// swaps the highest and 13th highest value on the stack
pub const SWAP12: Instruction = 0x9b;
/// swaps the highest and 14th highest value on the stack
pub const SWAP13: Instruction = 0x9c;
/// swaps the highest and 15th highest value on the stack
pub const SWAP14: Instruction = 0x9d;
/// swaps the highest and 16th highest value on the stack
pub const SWAP15: Instruction = 0x9e;
/// swaps the highest and 17th highest value on the stack
pub const SWAP16: Instruction = 0x9f;

/// Makes a log entry; no topics.
pub const LOG0: Instruction = 0xa0;
/// Makes a log entry; 1 topic.
pub const LOG1: Instruction = 0xa1;
/// Makes a log entry; 2 topics.
pub const LOG2: Instruction = 0xa2;
/// Makes a log entry; 3 topics.
pub const LOG3: Instruction = 0xa3;
/// Makes a log entry; 4 topics.
pub const LOG4: Instruction = 0xa4;
/// Maximal number of topics for log instructions
pub const MAX_NO_OF_TOPICS : usize = 4;

/// create a new account with associated code
pub const CREATE: Instruction = 0xf0;
/// message-call into an account
pub const CALL: Instruction = 0xf1;
/// message-call with another account's code only
pub const CALLCODE: Instruction = 0xf2;
/// halt execution returning output data
pub const RETURN: Instruction = 0xf3;
/// like CALLCODE but keeps caller's value and sender
pub const DELEGATECALL: Instruction = 0xf4;
/// create a new account and set creation address to sha3(sender + sha3(init code)) % 2**160
pub const CREATE2: Instruction = 0xfb;
/// stop execution and revert state changes. Return output data.
pub const REVERT: Instruction = 0xfd;
/// like CALL but it does not take value, nor modify the state
pub const STATICCALL: Instruction = 0xfa;
/// halt execution and register account for later deletion
pub const SUICIDE: Instruction = 0xff;

