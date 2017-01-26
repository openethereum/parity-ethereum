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

#[derive(Copy, Clone, Default)]
pub struct InstructionInfo {
	pub name: &'static str,
	pub additional: usize,
	pub args: usize,
	pub ret: usize,
	pub side_effects: bool,
	pub tier: GasPriceTier
}

impl InstructionInfo {
	pub fn new(name: &'static str, additional: usize, args: usize, ret: usize, side_effects: bool, tier: GasPriceTier) -> Self {
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

lazy_static! {
	pub static ref INSTRUCTIONS: [InstructionInfo; 0x100] = {
		let mut arr = [InstructionInfo::default(); 0x100];
		arr[STOP as usize] =			InstructionInfo::new("STOP",			0, 0, 0, true, GasPriceTier::Zero);
		arr[ADD as usize] = 			InstructionInfo::new("ADD",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[SUB as usize] = 			InstructionInfo::new("SUB",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[MUL as usize] = 			InstructionInfo::new("MUL",				0, 2, 1, false, GasPriceTier::Low);
		arr[DIV as usize] = 			InstructionInfo::new("DIV",				0, 2, 1, false, GasPriceTier::Low);
		arr[SDIV as usize] =			InstructionInfo::new("SDIV",			0, 2, 1, false, GasPriceTier::Low);
		arr[MOD as usize] = 			InstructionInfo::new("MOD",				0, 2, 1, false, GasPriceTier::Low);
		arr[SMOD as usize] =			InstructionInfo::new("SMOD",			0, 2, 1, false, GasPriceTier::Low);
		arr[EXP as usize] = 			InstructionInfo::new("EXP",				0, 2, 1, false, GasPriceTier::Special);
		arr[NOT as usize] = 			InstructionInfo::new("NOT",				0, 1, 1, false, GasPriceTier::VeryLow);
		arr[LT as usize] =				InstructionInfo::new("LT",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[GT as usize] =				InstructionInfo::new("GT",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[SLT as usize] = 			InstructionInfo::new("SLT",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[SGT as usize] = 			InstructionInfo::new("SGT",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[EQ as usize] =				InstructionInfo::new("EQ",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[ISZERO as usize] =			InstructionInfo::new("ISZERO",			0, 1, 1, false, GasPriceTier::VeryLow);
		arr[AND as usize] = 			InstructionInfo::new("AND",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[OR as usize] =				InstructionInfo::new("OR",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[XOR as usize] = 			InstructionInfo::new("XOR",				0, 2, 1, false, GasPriceTier::VeryLow);
		arr[BYTE as usize] =			InstructionInfo::new("BYTE",			0, 2, 1, false, GasPriceTier::VeryLow);
		arr[ADDMOD as usize] =			InstructionInfo::new("ADDMOD",			0, 3, 1, false, GasPriceTier::Mid);
		arr[MULMOD as usize] =			InstructionInfo::new("MULMOD",			0, 3, 1, false, GasPriceTier::Mid);
		arr[SIGNEXTEND as usize] =		InstructionInfo::new("SIGNEXTEND",		0, 2, 1, false, GasPriceTier::Low);
		arr[SHA3 as usize] =			InstructionInfo::new("SHA3",			0, 2, 1, false, GasPriceTier::Special);
		arr[ADDRESS as usize] = 		InstructionInfo::new("ADDRESS",			0, 0, 1, false, GasPriceTier::Base);
		arr[BALANCE as usize] = 		InstructionInfo::new("BALANCE",			0, 1, 1, false, GasPriceTier::Special);
		arr[ORIGIN as usize] =			InstructionInfo::new("ORIGIN",			0, 0, 1, false, GasPriceTier::Base);
		arr[CALLER as usize] =			InstructionInfo::new("CALLER",			0, 0, 1, false, GasPriceTier::Base);
		arr[CALLVALUE as usize] =		InstructionInfo::new("CALLVALUE",		0, 0, 1, false, GasPriceTier::Base);
		arr[CALLDATALOAD as usize] =	InstructionInfo::new("CALLDATALOAD",	0, 1, 1, false, GasPriceTier::VeryLow);
		arr[CALLDATASIZE as usize] =	InstructionInfo::new("CALLDATASIZE",	0, 0, 1, false, GasPriceTier::Base);
		arr[CALLDATACOPY as usize] =	InstructionInfo::new("CALLDATACOPY",	0, 3, 0, true, GasPriceTier::VeryLow);
		arr[CODESIZE as usize] =		InstructionInfo::new("CODESIZE",		0, 0, 1, false, GasPriceTier::Base);
		arr[CODECOPY as usize] =		InstructionInfo::new("CODECOPY",		0, 3, 0, true, GasPriceTier::VeryLow);
		arr[GASPRICE as usize] =		InstructionInfo::new("GASPRICE",		0, 0, 1, false, GasPriceTier::Base);
		arr[EXTCODESIZE as usize] = 	InstructionInfo::new("EXTCODESIZE",		0, 1, 1, false, GasPriceTier::Special);
		arr[EXTCODECOPY as usize] = 	InstructionInfo::new("EXTCODECOPY",		0, 4, 0, true, GasPriceTier::Special);
		arr[BLOCKHASH as usize] =		InstructionInfo::new("BLOCKHASH",		0, 1, 1, false, GasPriceTier::Ext);
		arr[COINBASE as usize] =		InstructionInfo::new("COINBASE",		0, 0, 1, false, GasPriceTier::Base);
		arr[TIMESTAMP as usize] =		InstructionInfo::new("TIMESTAMP",		0, 0, 1, false, GasPriceTier::Base);
		arr[NUMBER as usize] =			InstructionInfo::new("NUMBER",			0, 0, 1, false, GasPriceTier::Base);
		arr[DIFFICULTY as usize] =		InstructionInfo::new("DIFFICULTY",		0, 0, 1, false, GasPriceTier::Base);
		arr[GASLIMIT as usize] =		InstructionInfo::new("GASLIMIT",		0, 0, 1, false, GasPriceTier::Base);
		arr[POP as usize] = 			InstructionInfo::new("POP",				0, 1, 0, false, GasPriceTier::Base);
		arr[MLOAD as usize] =			InstructionInfo::new("MLOAD",			0, 1, 1, false, GasPriceTier::VeryLow);
		arr[MSTORE as usize] =			InstructionInfo::new("MSTORE",			0, 2, 0, true, GasPriceTier::VeryLow);
		arr[MSTORE8 as usize] = 		InstructionInfo::new("MSTORE8",			0, 2, 0, true, GasPriceTier::VeryLow);
		arr[SLOAD as usize] =			InstructionInfo::new("SLOAD",			0, 1, 1, false, GasPriceTier::Special);
		arr[SSTORE as usize] =			InstructionInfo::new("SSTORE",			0, 2, 0, true, GasPriceTier::Special);
		arr[JUMP as usize] =			InstructionInfo::new("JUMP",			0, 1, 0, true, GasPriceTier::Mid);
		arr[JUMPI as usize] =			InstructionInfo::new("JUMPI",			0, 2, 0, true, GasPriceTier::High);
		arr[PC as usize] =				InstructionInfo::new("PC",				0, 0, 1, false, GasPriceTier::Base);
		arr[MSIZE as usize] =			InstructionInfo::new("MSIZE",			0, 0, 1, false, GasPriceTier::Base);
		arr[GAS as usize] = 			InstructionInfo::new("GAS",				0, 0, 1, false, GasPriceTier::Base);
		arr[JUMPDEST as usize] =		InstructionInfo::new("JUMPDEST",		0, 0, 0, true, GasPriceTier::Special);
		arr[PUSH1 as usize] =			InstructionInfo::new("PUSH1",			1, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH2 as usize] =			InstructionInfo::new("PUSH2",			2, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH3 as usize] =			InstructionInfo::new("PUSH3",			3, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH4 as usize] =			InstructionInfo::new("PUSH4",			4, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH5 as usize] =			InstructionInfo::new("PUSH5",			5, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH6 as usize] =			InstructionInfo::new("PUSH6",			6, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH7 as usize] =			InstructionInfo::new("PUSH7",			7, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH8 as usize] =			InstructionInfo::new("PUSH8",			8, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH9 as usize] =			InstructionInfo::new("PUSH9",			9, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH10 as usize] =			InstructionInfo::new("PUSH10",			10, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH11 as usize] =			InstructionInfo::new("PUSH11",			11, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH12 as usize] =			InstructionInfo::new("PUSH12",			12, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH13 as usize] =			InstructionInfo::new("PUSH13",			13, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH14 as usize] =			InstructionInfo::new("PUSH14",			14, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH15 as usize] =			InstructionInfo::new("PUSH15",			15, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH16 as usize] =			InstructionInfo::new("PUSH16",			16, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH17 as usize] =			InstructionInfo::new("PUSH17",			17, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH18 as usize] =			InstructionInfo::new("PUSH18",			18, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH19 as usize] =			InstructionInfo::new("PUSH19",			19, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH20 as usize] =			InstructionInfo::new("PUSH20",			20, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH21 as usize] =			InstructionInfo::new("PUSH21",			21, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH22 as usize] =			InstructionInfo::new("PUSH22",			22, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH23 as usize] =			InstructionInfo::new("PUSH23",			23, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH24 as usize] =			InstructionInfo::new("PUSH24",			24, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH25 as usize] =			InstructionInfo::new("PUSH25",			25, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH26 as usize] =			InstructionInfo::new("PUSH26",			26, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH27 as usize] =			InstructionInfo::new("PUSH27",			27, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH28 as usize] =			InstructionInfo::new("PUSH28",			28, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH29 as usize] =			InstructionInfo::new("PUSH29",			29, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH30 as usize] =			InstructionInfo::new("PUSH30",			30, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH31 as usize] =			InstructionInfo::new("PUSH31",			31, 0, 1, false, GasPriceTier::VeryLow);
		arr[PUSH32 as usize] =			InstructionInfo::new("PUSH32",			32, 0, 1, false, GasPriceTier::VeryLow);
		arr[DUP1 as usize] =			InstructionInfo::new("DUP1",			0, 1, 2, false, GasPriceTier::VeryLow);
		arr[DUP2 as usize] =			InstructionInfo::new("DUP2",			0, 2, 3, false, GasPriceTier::VeryLow);
		arr[DUP3 as usize] =			InstructionInfo::new("DUP3",			0, 3, 4, false, GasPriceTier::VeryLow);
		arr[DUP4 as usize] =			InstructionInfo::new("DUP4",			0, 4, 5, false, GasPriceTier::VeryLow);
		arr[DUP5 as usize] =			InstructionInfo::new("DUP5",			0, 5, 6, false, GasPriceTier::VeryLow);
		arr[DUP6 as usize] =			InstructionInfo::new("DUP6",			0, 6, 7, false, GasPriceTier::VeryLow);
		arr[DUP7 as usize] =			InstructionInfo::new("DUP7",			0, 7, 8, false, GasPriceTier::VeryLow);
		arr[DUP8 as usize] =			InstructionInfo::new("DUP8",			0, 8, 9, false, GasPriceTier::VeryLow);
		arr[DUP9 as usize] =			InstructionInfo::new("DUP9",			0, 9, 10, false, GasPriceTier::VeryLow);
		arr[DUP10 as usize] =			InstructionInfo::new("DUP10",			0, 10, 11, false, GasPriceTier::VeryLow);
		arr[DUP11 as usize] =			InstructionInfo::new("DUP11",			0, 11, 12, false, GasPriceTier::VeryLow);
		arr[DUP12 as usize] =			InstructionInfo::new("DUP12",			0, 12, 13, false, GasPriceTier::VeryLow);
		arr[DUP13 as usize] =			InstructionInfo::new("DUP13",			0, 13, 14, false, GasPriceTier::VeryLow);
		arr[DUP14 as usize] =			InstructionInfo::new("DUP14",			0, 14, 15, false, GasPriceTier::VeryLow);
		arr[DUP15 as usize] =			InstructionInfo::new("DUP15",			0, 15, 16, false, GasPriceTier::VeryLow);
		arr[DUP16 as usize] =			InstructionInfo::new("DUP16",			0, 16, 17, false, GasPriceTier::VeryLow);
		arr[SWAP1 as usize] =			InstructionInfo::new("SWAP1",			0, 2, 2, false, GasPriceTier::VeryLow);
		arr[SWAP2 as usize] =			InstructionInfo::new("SWAP2",			0, 3, 3, false, GasPriceTier::VeryLow);
		arr[SWAP3 as usize] =			InstructionInfo::new("SWAP3",			0, 4, 4, false, GasPriceTier::VeryLow);
		arr[SWAP4 as usize] =			InstructionInfo::new("SWAP4",			0, 5, 5, false, GasPriceTier::VeryLow);
		arr[SWAP5 as usize] =			InstructionInfo::new("SWAP5",			0, 6, 6, false, GasPriceTier::VeryLow);
		arr[SWAP6 as usize] =			InstructionInfo::new("SWAP6",			0, 7, 7, false, GasPriceTier::VeryLow);
		arr[SWAP7 as usize] =			InstructionInfo::new("SWAP7",			0, 8, 8, false, GasPriceTier::VeryLow);
		arr[SWAP8 as usize] =			InstructionInfo::new("SWAP8",			0, 9, 9, false, GasPriceTier::VeryLow);
		arr[SWAP9 as usize] =			InstructionInfo::new("SWAP9",			0, 10, 10, false, GasPriceTier::VeryLow);
		arr[SWAP10 as usize] =			InstructionInfo::new("SWAP10",			0, 11, 11, false, GasPriceTier::VeryLow);
		arr[SWAP11 as usize] =			InstructionInfo::new("SWAP11",			0, 12, 12, false, GasPriceTier::VeryLow);
		arr[SWAP12 as usize] =			InstructionInfo::new("SWAP12",			0, 13, 13, false, GasPriceTier::VeryLow);
		arr[SWAP13 as usize] =			InstructionInfo::new("SWAP13",			0, 14, 14, false, GasPriceTier::VeryLow);
		arr[SWAP14 as usize] =			InstructionInfo::new("SWAP14",			0, 15, 15, false, GasPriceTier::VeryLow);
		arr[SWAP15 as usize] =			InstructionInfo::new("SWAP15",			0, 16, 16, false, GasPriceTier::VeryLow);
		arr[SWAP16 as usize] =			InstructionInfo::new("SWAP16",			0, 17, 17, false, GasPriceTier::VeryLow);
		arr[LOG0 as usize] =			InstructionInfo::new("LOG0",			0, 2, 0, true, GasPriceTier::Special);
		arr[LOG1 as usize] =			InstructionInfo::new("LOG1",			0, 3, 0, true, GasPriceTier::Special);
		arr[LOG2 as usize] =			InstructionInfo::new("LOG2",			0, 4, 0, true, GasPriceTier::Special);
		arr[LOG3 as usize] =			InstructionInfo::new("LOG3",			0, 5, 0, true, GasPriceTier::Special);
		arr[LOG4 as usize] =			InstructionInfo::new("LOG4",			0, 6, 0, true, GasPriceTier::Special);
		arr[CREATE as usize] =			InstructionInfo::new("CREATE",			0, 3, 1, true, GasPriceTier::Special);
		arr[CALL as usize] =			InstructionInfo::new("CALL",			0, 7, 1, true, GasPriceTier::Special);
		arr[CALLCODE as usize] =		InstructionInfo::new("CALLCODE",		0, 7, 1, true, GasPriceTier::Special);
		arr[RETURN as usize] =			InstructionInfo::new("RETURN",			0, 2, 0, true, GasPriceTier::Zero);
		arr[DELEGATECALL as usize] =	InstructionInfo::new("DELEGATECALL",	0, 6, 1, true, GasPriceTier::Special);
		arr[SUICIDE as usize] = 		InstructionInfo::new("SUICIDE",			0, 1, 0, true, GasPriceTier::Special);
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

