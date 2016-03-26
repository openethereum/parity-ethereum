// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

/// Returns number of bytes to read for `PUSHN` instruction
/// PUSH1 -> 1
pub fn get_push_bytes(i: Instruction) -> usize {
	assert!(is_push(i), "Only for PUSH instructions.");
	(i - PUSH1 + 1) as usize
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

/// Returns number of topcis to take from stack
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

#[derive(PartialEq)]
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

pub struct InstructionInfo {
	pub name: &'static str,
	pub additional: usize,
	pub args: usize,
	pub ret: usize,
	pub side_effects: bool,
	pub tier: GasPriceTier
}
impl InstructionInfo {
	pub fn new(name: &'static str, additional: usize, args: usize, ret: usize, side_effects: bool, tier: GasPriceTier) -> InstructionInfo {
		InstructionInfo {
			name: name,
			additional: additional,
			args: args,
			ret: ret,
			side_effects: side_effects,
			tier: tier
		}
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
/// Return details about specific instruction
pub fn get_info (instruction: Instruction) -> InstructionInfo {
	match instruction {
		STOP => 		InstructionInfo::new("STOP",			0, 0, 0, true, GasPriceTier::Zero),
		ADD => 			InstructionInfo::new("ADD",				0, 2, 1, false, GasPriceTier::VeryLow),
		SUB => 			InstructionInfo::new("SUB",				0, 2, 1, false, GasPriceTier::VeryLow),
		MUL => 			InstructionInfo::new("MUL",				0, 2, 1, false, GasPriceTier::Low),
		DIV => 			InstructionInfo::new("DIV",				0, 2, 1, false, GasPriceTier::Low),
		SDIV => 		InstructionInfo::new("SDIV",			0, 2, 1, false, GasPriceTier::Low),
		MOD => 			InstructionInfo::new("MOD",				0, 2, 1, false, GasPriceTier::Low),
		SMOD => 		InstructionInfo::new("SMOD",			0, 2, 1, false, GasPriceTier::Low),
		EXP => 			InstructionInfo::new("EXP",				0, 2, 1, false, GasPriceTier::Special),
		NOT => 			InstructionInfo::new("NOT",				0, 1, 1, false, GasPriceTier::VeryLow),
		LT => 			InstructionInfo::new("LT",				0, 2, 1, false, GasPriceTier::VeryLow),
		GT => 			InstructionInfo::new("GT",				0, 2, 1, false, GasPriceTier::VeryLow),
		SLT => 			InstructionInfo::new("SLT",				0, 2, 1, false, GasPriceTier::VeryLow),
		SGT => 			InstructionInfo::new("SGT",				0, 2, 1, false, GasPriceTier::VeryLow),
		EQ => 			InstructionInfo::new("EQ",				0, 2, 1, false, GasPriceTier::VeryLow),
		ISZERO => 		InstructionInfo::new("ISZERO",			0, 1, 1, false, GasPriceTier::VeryLow),
		AND => 			InstructionInfo::new("AND",				0, 2, 1, false, GasPriceTier::VeryLow),
		OR => 			InstructionInfo::new("OR",				0, 2, 1, false, GasPriceTier::VeryLow),
		XOR => 			InstructionInfo::new("XOR",				0, 2, 1, false, GasPriceTier::VeryLow),
		BYTE => 		InstructionInfo::new("BYTE",			0, 2, 1, false, GasPriceTier::VeryLow),
		ADDMOD => 		InstructionInfo::new("ADDMOD",			0, 3, 1, false, GasPriceTier::Mid),
		MULMOD => 		InstructionInfo::new("MULMOD",			0, 3, 1, false, GasPriceTier::Mid),
		SIGNEXTEND => 	InstructionInfo::new("SIGNEXTEND",		0, 2, 1, false, GasPriceTier::Low),
		SHA3 => 		InstructionInfo::new("SHA3",			0, 2, 1, false, GasPriceTier::Special),
		ADDRESS => 		InstructionInfo::new("ADDRESS",			0, 0, 1, false, GasPriceTier::Base),
		BALANCE => 		InstructionInfo::new("BALANCE",			0, 1, 1, false, GasPriceTier::Ext),
		ORIGIN => 		InstructionInfo::new("ORIGIN",			0, 0, 1, false, GasPriceTier::Base),
		CALLER => 		InstructionInfo::new("CALLER",			0, 0, 1, false, GasPriceTier::Base),
		CALLVALUE => 	InstructionInfo::new("CALLVALUE",		0, 0, 1, false, GasPriceTier::Base),
		CALLDATALOAD => InstructionInfo::new("CALLDATALOAD",	0, 1, 1, false, GasPriceTier::VeryLow),
		CALLDATASIZE => InstructionInfo::new("CALLDATASIZE",	0, 0, 1, false, GasPriceTier::Base),
		CALLDATACOPY => InstructionInfo::new("CALLDATACOPY",	0, 3, 0, true, GasPriceTier::VeryLow),
		CODESIZE => 	InstructionInfo::new("CODESIZE",		0, 0, 1, false, GasPriceTier::Base),
		CODECOPY => 	InstructionInfo::new("CODECOPY",		0, 3, 0, true, GasPriceTier::VeryLow),
		GASPRICE => 	InstructionInfo::new("GASPRICE",		0, 0, 1, false, GasPriceTier::Base),
		EXTCODESIZE => 	InstructionInfo::new("EXTCODESIZE",		0, 1, 1, false, GasPriceTier::Ext),
		EXTCODECOPY => 	InstructionInfo::new("EXTCODECOPY",		0, 4, 0, true, GasPriceTier::Ext),
		BLOCKHASH => 	InstructionInfo::new("BLOCKHASH",		0, 1, 1, false, GasPriceTier::Ext),
		COINBASE => 	InstructionInfo::new("COINBASE",		0, 0, 1, false, GasPriceTier::Base),
		TIMESTAMP => 	InstructionInfo::new("TIMESTAMP",		0, 0, 1, false, GasPriceTier::Base),
		NUMBER => 		InstructionInfo::new("NUMBER",			0, 0, 1, false, GasPriceTier::Base),
		DIFFICULTY => 	InstructionInfo::new("DIFFICULTY",		0, 0, 1, false, GasPriceTier::Base),
		GASLIMIT => 	InstructionInfo::new("GASLIMIT",		0, 0, 1, false, GasPriceTier::Base),
		POP => 			InstructionInfo::new("POP",				0, 1, 0, false, GasPriceTier::Base),
		MLOAD => 		InstructionInfo::new("MLOAD",			0, 1, 1, false, GasPriceTier::VeryLow),
		MSTORE => 		InstructionInfo::new("MSTORE",			0, 2, 0, true, GasPriceTier::VeryLow),
		MSTORE8 => 		InstructionInfo::new("MSTORE8",			0, 2, 0, true, GasPriceTier::VeryLow),
		SLOAD => 		InstructionInfo::new("SLOAD",			0, 1, 1, false, GasPriceTier::Special),
		SSTORE => 		InstructionInfo::new("SSTORE",			0, 2, 0, true, GasPriceTier::Special),
		JUMP => 		InstructionInfo::new("JUMP",			0, 1, 0, true, GasPriceTier::Mid),
		JUMPI => 		InstructionInfo::new("JUMPI",			0, 2, 0, true, GasPriceTier::High),
		PC => 			InstructionInfo::new("PC",				0, 0, 1, false, GasPriceTier::Base),
		MSIZE => 		InstructionInfo::new("MSIZE",			0, 0, 1, false, GasPriceTier::Base),
		GAS => 			InstructionInfo::new("GAS",				0, 0, 1, false, GasPriceTier::Base),
		JUMPDEST => 	InstructionInfo::new("JUMPDEST",		0, 0, 0, true, GasPriceTier::Special),
		PUSH1 => 		InstructionInfo::new("PUSH1",			1, 0, 1, false, GasPriceTier::VeryLow),
		PUSH2 => 		InstructionInfo::new("PUSH2",			2, 0, 1, false, GasPriceTier::VeryLow),
		PUSH3 => 		InstructionInfo::new("PUSH3",			3, 0, 1, false, GasPriceTier::VeryLow),
		PUSH4 => 		InstructionInfo::new("PUSH4",			4, 0, 1, false, GasPriceTier::VeryLow),
		PUSH5 => 		InstructionInfo::new("PUSH5",			5, 0, 1, false, GasPriceTier::VeryLow),
		PUSH6 => 		InstructionInfo::new("PUSH6",			6, 0, 1, false, GasPriceTier::VeryLow),
		PUSH7 => 		InstructionInfo::new("PUSH7",			7, 0, 1, false, GasPriceTier::VeryLow),
		PUSH8 => 		InstructionInfo::new("PUSH8",			8, 0, 1, false, GasPriceTier::VeryLow),
		PUSH9 => 		InstructionInfo::new("PUSH9",			9, 0, 1, false, GasPriceTier::VeryLow),
		PUSH10 => 		InstructionInfo::new("PUSH10",			10, 0, 1, false, GasPriceTier::VeryLow),
		PUSH11 => 		InstructionInfo::new("PUSH11",			11, 0, 1, false, GasPriceTier::VeryLow),
		PUSH12 => 		InstructionInfo::new("PUSH12",			12, 0, 1, false, GasPriceTier::VeryLow),
		PUSH13 => 		InstructionInfo::new("PUSH13",			13, 0, 1, false, GasPriceTier::VeryLow),
		PUSH14 => 		InstructionInfo::new("PUSH14",			14, 0, 1, false, GasPriceTier::VeryLow),
		PUSH15 => 		InstructionInfo::new("PUSH15",			15, 0, 1, false, GasPriceTier::VeryLow),
		PUSH16 => 		InstructionInfo::new("PUSH16",			16, 0, 1, false, GasPriceTier::VeryLow),
		PUSH17 => 		InstructionInfo::new("PUSH17",			17, 0, 1, false, GasPriceTier::VeryLow),
		PUSH18 => 		InstructionInfo::new("PUSH18",			18, 0, 1, false, GasPriceTier::VeryLow),
		PUSH19 => 		InstructionInfo::new("PUSH19",			19, 0, 1, false, GasPriceTier::VeryLow),
		PUSH20 => 		InstructionInfo::new("PUSH20",			20, 0, 1, false, GasPriceTier::VeryLow),
		PUSH21 => 		InstructionInfo::new("PUSH21",			21, 0, 1, false, GasPriceTier::VeryLow),
		PUSH22 => 		InstructionInfo::new("PUSH22",			22, 0, 1, false, GasPriceTier::VeryLow),
		PUSH23 => 		InstructionInfo::new("PUSH23",			23, 0, 1, false, GasPriceTier::VeryLow),
		PUSH24 => 		InstructionInfo::new("PUSH24",			24, 0, 1, false, GasPriceTier::VeryLow),
		PUSH25 => 		InstructionInfo::new("PUSH25",			25, 0, 1, false, GasPriceTier::VeryLow),
		PUSH26 => 		InstructionInfo::new("PUSH26",			26, 0, 1, false, GasPriceTier::VeryLow),
		PUSH27 => 		InstructionInfo::new("PUSH27",			27, 0, 1, false, GasPriceTier::VeryLow),
		PUSH28 => 		InstructionInfo::new("PUSH28",			28, 0, 1, false, GasPriceTier::VeryLow),
		PUSH29 => 		InstructionInfo::new("PUSH29",			29, 0, 1, false, GasPriceTier::VeryLow),
		PUSH30 => 		InstructionInfo::new("PUSH30",			30, 0, 1, false, GasPriceTier::VeryLow),
		PUSH31 => 		InstructionInfo::new("PUSH31",			31, 0, 1, false, GasPriceTier::VeryLow),
		PUSH32 => 		InstructionInfo::new("PUSH32",			32, 0, 1, false, GasPriceTier::VeryLow),
		DUP1 => 		InstructionInfo::new("DUP1",			0, 1, 2, false, GasPriceTier::VeryLow),
		DUP2 => 		InstructionInfo::new("DUP2",			0, 2, 3, false, GasPriceTier::VeryLow),
		DUP3 => 		InstructionInfo::new("DUP3",			0, 3, 4, false, GasPriceTier::VeryLow),
		DUP4 => 		InstructionInfo::new("DUP4",			0, 4, 5, false, GasPriceTier::VeryLow),
		DUP5 => 		InstructionInfo::new("DUP5",			0, 5, 6, false, GasPriceTier::VeryLow),
		DUP6 => 		InstructionInfo::new("DUP6",			0, 6, 7, false, GasPriceTier::VeryLow),
		DUP7 => 		InstructionInfo::new("DUP7",			0, 7, 8, false, GasPriceTier::VeryLow),
		DUP8 => 		InstructionInfo::new("DUP8",			0, 8, 9, false, GasPriceTier::VeryLow),
		DUP9 => 		InstructionInfo::new("DUP9",			0, 9, 10, false, GasPriceTier::VeryLow),
		DUP10 => 		InstructionInfo::new("DUP10",			0, 10, 11, false, GasPriceTier::VeryLow),
		DUP11 => 		InstructionInfo::new("DUP11",			0, 11, 12, false, GasPriceTier::VeryLow),
		DUP12 => 		InstructionInfo::new("DUP12",			0, 12, 13, false, GasPriceTier::VeryLow),
		DUP13 => 		InstructionInfo::new("DUP13",			0, 13, 14, false, GasPriceTier::VeryLow),
		DUP14 => 		InstructionInfo::new("DUP14",			0, 14, 15, false, GasPriceTier::VeryLow),
		DUP15 => 		InstructionInfo::new("DUP15",			0, 15, 16, false, GasPriceTier::VeryLow),
		DUP16 => 		InstructionInfo::new("DUP16",			0, 16, 17, false, GasPriceTier::VeryLow),
		SWAP1 => 		InstructionInfo::new("SWAP1",			0, 2, 2, false, GasPriceTier::VeryLow),
		SWAP2 => 		InstructionInfo::new("SWAP2",			0, 3, 3, false, GasPriceTier::VeryLow),
		SWAP3 => 		InstructionInfo::new("SWAP3",			0, 4, 4, false, GasPriceTier::VeryLow),
		SWAP4 => 		InstructionInfo::new("SWAP4",			0, 5, 5, false, GasPriceTier::VeryLow),
		SWAP5 => 		InstructionInfo::new("SWAP5",			0, 6, 6, false, GasPriceTier::VeryLow),
		SWAP6 => 		InstructionInfo::new("SWAP6",			0, 7, 7, false, GasPriceTier::VeryLow),
		SWAP7 => 		InstructionInfo::new("SWAP7",			0, 8, 8, false, GasPriceTier::VeryLow),
		SWAP8 => 		InstructionInfo::new("SWAP8",			0, 9, 9, false, GasPriceTier::VeryLow),
		SWAP9 => 		InstructionInfo::new("SWAP9",			0, 10, 10, false, GasPriceTier::VeryLow),
		SWAP10 => 		InstructionInfo::new("SWAP10",			0, 11, 11, false, GasPriceTier::VeryLow),
		SWAP11 => 		InstructionInfo::new("SWAP11",			0, 12, 12, false, GasPriceTier::VeryLow),
		SWAP12 => 		InstructionInfo::new("SWAP12",			0, 13, 13, false, GasPriceTier::VeryLow),
		SWAP13 => 		InstructionInfo::new("SWAP13",			0, 14, 14, false, GasPriceTier::VeryLow),
		SWAP14 => 		InstructionInfo::new("SWAP14",			0, 15, 15, false, GasPriceTier::VeryLow),
		SWAP15 => 		InstructionInfo::new("SWAP15",			0, 16, 16, false, GasPriceTier::VeryLow),
		SWAP16 => 		InstructionInfo::new("SWAP16",			0, 17, 17, false, GasPriceTier::VeryLow),
		LOG0 => 		InstructionInfo::new("LOG0",			0, 2, 0, true, GasPriceTier::Special),
		LOG1 => 		InstructionInfo::new("LOG1",			0, 3, 0, true, GasPriceTier::Special),
		LOG2 => 		InstructionInfo::new("LOG2",			0, 4, 0, true, GasPriceTier::Special),
		LOG3 => 		InstructionInfo::new("LOG3",			0, 5, 0, true, GasPriceTier::Special),
		LOG4 => 		InstructionInfo::new("LOG4",			0, 6, 0, true, GasPriceTier::Special),
		CREATE => 		InstructionInfo::new("CREATE",			0, 3, 1, true, GasPriceTier::Special),
		CALL => 		InstructionInfo::new("CALL",			0, 7, 1, true, GasPriceTier::Special),
		CALLCODE => 	InstructionInfo::new("CALLCODE",		0, 7, 1, true, GasPriceTier::Special),
		RETURN => 		InstructionInfo::new("RETURN",			0, 2, 0, true, GasPriceTier::Zero),
		DELEGATECALL => InstructionInfo::new("DELEGATECALL",	0, 6, 1, true, GasPriceTier::Special),
		SUICIDE => 		InstructionInfo::new("SUICIDE",			0, 1, 0, true, GasPriceTier::Zero),
		_ => InstructionInfo::new("INVALID_INSTRUCTION", 0, 0, 0, false, GasPriceTier::Invalid)
	}
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
/// halt execution and register account for later deletion
pub const SUICIDE: Instruction = 0xff;

