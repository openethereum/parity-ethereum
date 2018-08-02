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

macro_rules! enum_with_from_u8 {
	(
		$( #[$enum_attr:meta] )*
		pub enum $name:ident {
			$( $( #[$variant_attr:meta] )* $variant:ident = $discriminator:expr ),+,
		}
	) => {
		$( #[$enum_attr] )*
		pub enum $name {
			$( $( #[$variant_attr] )* $variant = $discriminator ),+,
		}

		impl $name {
			#[doc = "Convert from u8 to the given enum"]
			pub fn from_u8(value: u8) -> Option<Self> {
				match value {
					$( $discriminator => Some($variant) ),+,
					_ => None,
				}
			}
		}
	};
}

enum_with_from_u8! {
	#[doc = "Virtual machine bytecode instruction."]
	#[repr(u8)]
	#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
	pub enum Instruction {
		#[doc = "halts execution"]
		STOP = 0x00,
		#[doc = "addition operation"]
		ADD = 0x01,
		#[doc = "mulitplication operation"]
		MUL = 0x02,
		#[doc = "subtraction operation"]
		SUB = 0x03,
		#[doc = "integer division operation"]
		DIV = 0x04,
		#[doc = "signed integer division operation"]
		SDIV = 0x05,
		#[doc = "modulo remainder operation"]
		MOD = 0x06,
		#[doc = "signed modulo remainder operation"]
		SMOD = 0x07,
		#[doc = "unsigned modular addition"]
		ADDMOD = 0x08,
		#[doc = "unsigned modular multiplication"]
		MULMOD = 0x09,
		#[doc = "exponential operation"]
		EXP = 0x0a,
		#[doc = "extend length of signed integer"]
		SIGNEXTEND = 0x0b,

		#[doc = "less-than comparision"]
		LT = 0x10,
		#[doc = "greater-than comparision"]
		GT = 0x11,
		#[doc = "signed less-than comparision"]
		SLT = 0x12,
		#[doc = "signed greater-than comparision"]
		SGT = 0x13,
		#[doc = "equality comparision"]
		EQ = 0x14,
		#[doc = "simple not operator"]
		ISZERO = 0x15,
		#[doc = "bitwise AND operation"]
		AND = 0x16,
		#[doc = "bitwise OR operation"]
		OR = 0x17,
		#[doc = "bitwise XOR operation"]
		XOR = 0x18,
		#[doc = "bitwise NOT opertation"]
		NOT = 0x19,
		#[doc = "retrieve single byte from word"]
		BYTE = 0x1a,
		#[doc = "shift left operation"]
		SHL = 0x1b,
		#[doc = "logical shift right operation"]
		SHR = 0x1c,
		#[doc = "arithmetic shift right operation"]
		SAR = 0x1d,

		#[doc = "compute SHA3-256 hash"]
		SHA3 = 0x20,

		#[doc = "get address of currently executing account"]
		ADDRESS = 0x30,
		#[doc = "get balance of the given account"]
		BALANCE = 0x31,
		#[doc = "get execution origination address"]
		ORIGIN = 0x32,
		#[doc = "get caller address"]
		CALLER = 0x33,
		#[doc = "get deposited value by the instruction/transaction responsible for this execution"]
		CALLVALUE = 0x34,
		#[doc = "get input data of current environment"]
		CALLDATALOAD = 0x35,
		#[doc = "get size of input data in current environment"]
		CALLDATASIZE = 0x36,
		#[doc = "copy input data in current environment to memory"]
		CALLDATACOPY = 0x37,
		#[doc = "get size of code running in current environment"]
		CODESIZE = 0x38,
		#[doc = "copy code running in current environment to memory"]
		CODECOPY = 0x39,
		#[doc = "get price of gas in current environment"]
		GASPRICE = 0x3a,
		#[doc = "get external code size (from another contract)"]
		EXTCODESIZE = 0x3b,
		#[doc = "copy external code (from another contract)"]
		EXTCODECOPY = 0x3c,
		#[doc = "get the size of the return data buffer for the last call"]
		RETURNDATASIZE = 0x3d,
		#[doc = "copy return data buffer to memory"]
		RETURNDATACOPY = 0x3e,
		#[doc = "return the keccak256 hash of contract code"]
		EXTCODEHASH = 0x3f,

		#[doc = "get hash of most recent complete block"]
		BLOCKHASH = 0x40,
		#[doc = "get the block's coinbase address"]
		COINBASE = 0x41,
		#[doc = "get the block's timestamp"]
		TIMESTAMP = 0x42,
		#[doc = "get the block's number"]
		NUMBER = 0x43,
		#[doc = "get the block's difficulty"]
		DIFFICULTY = 0x44,
		#[doc = "get the block's gas limit"]
		GASLIMIT = 0x45,

		#[doc = "remove item from stack"]
		POP = 0x50,
		#[doc = "load word from memory"]
		MLOAD = 0x51,
		#[doc = "save word to memory"]
		MSTORE = 0x52,
		#[doc = "save byte to memory"]
		MSTORE8 = 0x53,
		#[doc = "load word from storage"]
		SLOAD = 0x54,
		#[doc = "save word to storage"]
		SSTORE = 0x55,
		#[doc = "alter the program counter"]
		JUMP = 0x56,
		#[doc = "conditionally alter the program counter"]
		JUMPI = 0x57,
		#[doc = "get the program counter"]
		PC = 0x58,
		#[doc = "get the size of active memory"]
		MSIZE = 0x59,
		#[doc = "get the amount of available gas"]
		GAS = 0x5a,
		#[doc = "set a potential jump destination"]
		JUMPDEST = 0x5b,

		#[doc = "place 1 byte item on stack"]
		PUSH1 = 0x60,
		#[doc = "place 2 byte item on stack"]
		PUSH2 = 0x61,
		#[doc = "place 3 byte item on stack"]
		PUSH3 = 0x62,
		#[doc = "place 4 byte item on stack"]
		PUSH4 = 0x63,
		#[doc = "place 5 byte item on stack"]
		PUSH5 = 0x64,
		#[doc = "place 6 byte item on stack"]
		PUSH6 = 0x65,
		#[doc = "place 7 byte item on stack"]
		PUSH7 = 0x66,
		#[doc = "place 8 byte item on stack"]
		PUSH8 = 0x67,
		#[doc = "place 9 byte item on stack"]
		PUSH9 = 0x68,
		#[doc = "place 10 byte item on stack"]
		PUSH10 = 0x69,
		#[doc = "place 11 byte item on stack"]
		PUSH11 = 0x6a,
		#[doc = "place 12 byte item on stack"]
		PUSH12 = 0x6b,
		#[doc = "place 13 byte item on stack"]
		PUSH13 = 0x6c,
		#[doc = "place 14 byte item on stack"]
		PUSH14 = 0x6d,
		#[doc = "place 15 byte item on stack"]
		PUSH15 = 0x6e,
		#[doc = "place 16 byte item on stack"]
		PUSH16 = 0x6f,
		#[doc = "place 17 byte item on stack"]
		PUSH17 = 0x70,
		#[doc = "place 18 byte item on stack"]
		PUSH18 = 0x71,
		#[doc = "place 19 byte item on stack"]
		PUSH19 = 0x72,
		#[doc = "place 20 byte item on stack"]
		PUSH20 = 0x73,
		#[doc = "place 21 byte item on stack"]
		PUSH21 = 0x74,
		#[doc = "place 22 byte item on stack"]
		PUSH22 = 0x75,
		#[doc = "place 23 byte item on stack"]
		PUSH23 = 0x76,
		#[doc = "place 24 byte item on stack"]
		PUSH24 = 0x77,
		#[doc = "place 25 byte item on stack"]
		PUSH25 = 0x78,
		#[doc = "place 26 byte item on stack"]
		PUSH26 = 0x79,
		#[doc = "place 27 byte item on stack"]
		PUSH27 = 0x7a,
		#[doc = "place 28 byte item on stack"]
		PUSH28 = 0x7b,
		#[doc = "place 29 byte item on stack"]
		PUSH29 = 0x7c,
		#[doc = "place 30 byte item on stack"]
		PUSH30 = 0x7d,
		#[doc = "place 31 byte item on stack"]
		PUSH31 = 0x7e,
		#[doc = "place 32 byte item on stack"]
		PUSH32 = 0x7f,

		#[doc = "copies the highest item in the stack to the top of the stack"]
		DUP1 = 0x80,
		#[doc = "copies the second highest item in the stack to the top of the stack"]
		DUP2 = 0x81,
		#[doc = "copies the third highest item in the stack to the top of the stack"]
		DUP3 = 0x82,
		#[doc = "copies the 4th highest item in the stack to the top of the stack"]
		DUP4 = 0x83,
		#[doc = "copies the 5th highest item in the stack to the top of the stack"]
		DUP5 = 0x84,
		#[doc = "copies the 6th highest item in the stack to the top of the stack"]
		DUP6 = 0x85,
		#[doc = "copies the 7th highest item in the stack to the top of the stack"]
		DUP7 = 0x86,
		#[doc = "copies the 8th highest item in the stack to the top of the stack"]
		DUP8 = 0x87,
		#[doc = "copies the 9th highest item in the stack to the top of the stack"]
		DUP9 = 0x88,
		#[doc = "copies the 10th highest item in the stack to the top of the stack"]
		DUP10 = 0x89,
		#[doc = "copies the 11th highest item in the stack to the top of the stack"]
		DUP11 = 0x8a,
		#[doc = "copies the 12th highest item in the stack to the top of the stack"]
		DUP12 = 0x8b,
		#[doc = "copies the 13th highest item in the stack to the top of the stack"]
		DUP13 = 0x8c,
		#[doc = "copies the 14th highest item in the stack to the top of the stack"]
		DUP14 = 0x8d,
		#[doc = "copies the 15th highest item in the stack to the top of the stack"]
		DUP15 = 0x8e,
		#[doc = "copies the 16th highest item in the stack to the top of the stack"]
		DUP16 = 0x8f,

		#[doc = "swaps the highest and second highest value on the stack"]
		SWAP1 = 0x90,
		#[doc = "swaps the highest and third highest value on the stack"]
		SWAP2 = 0x91,
		#[doc = "swaps the highest and 4th highest value on the stack"]
		SWAP3 = 0x92,
		#[doc = "swaps the highest and 5th highest value on the stack"]
		SWAP4 = 0x93,
		#[doc = "swaps the highest and 6th highest value on the stack"]
		SWAP5 = 0x94,
		#[doc = "swaps the highest and 7th highest value on the stack"]
		SWAP6 = 0x95,
		#[doc = "swaps the highest and 8th highest value on the stack"]
		SWAP7 = 0x96,
		#[doc = "swaps the highest and 9th highest value on the stack"]
		SWAP8 = 0x97,
		#[doc = "swaps the highest and 10th highest value on the stack"]
		SWAP9 = 0x98,
		#[doc = "swaps the highest and 11th highest value on the stack"]
		SWAP10 = 0x99,
		#[doc = "swaps the highest and 12th highest value on the stack"]
		SWAP11 = 0x9a,
		#[doc = "swaps the highest and 13th highest value on the stack"]
		SWAP12 = 0x9b,
		#[doc = "swaps the highest and 14th highest value on the stack"]
		SWAP13 = 0x9c,
		#[doc = "swaps the highest and 15th highest value on the stack"]
		SWAP14 = 0x9d,
		#[doc = "swaps the highest and 16th highest value on the stack"]
		SWAP15 = 0x9e,
		#[doc = "swaps the highest and 17th highest value on the stack"]
		SWAP16 = 0x9f,

		#[doc = "Makes a log entry, no topics."]
		LOG0 = 0xa0,
		#[doc = "Makes a log entry, 1 topic."]
		LOG1 = 0xa1,
		#[doc = "Makes a log entry, 2 topics."]
		LOG2 = 0xa2,
		#[doc = "Makes a log entry, 3 topics."]
		LOG3 = 0xa3,
		#[doc = "Makes a log entry, 4 topics."]
		LOG4 = 0xa4,

		#[doc = "create a new account with associated code"]
		CREATE = 0xf0,
		#[doc = "message-call into an account"]
		CALL = 0xf1,
		#[doc = "message-call with another account's code only"]
		CALLCODE = 0xf2,
		#[doc = "halt execution returning output data"]
		RETURN = 0xf3,
		#[doc = "like CALLCODE but keeps caller's value and sender"]
		DELEGATECALL = 0xf4,
		#[doc = "create a new account and set creation address to sha3(sender + sha3(init code)) % 2**160"]
		CREATE2 = 0xfb,
		#[doc = "stop execution and revert state changes. Return output data."]
		REVERT = 0xfd,
		#[doc = "like CALL but it does not take value, nor modify the state"]
		STATICCALL = 0xfa,
		#[doc = "halt execution and register account for later deletion"]
		SUICIDE = 0xff,
	}
}

impl Instruction {
	/// Returns true if given instruction is `PUSHN` instruction.
	pub fn is_push(&self) -> bool {
		*self >= PUSH1 && *self <= PUSH32
	}

	/// Returns number of bytes to read for `PUSHN` instruction
	/// PUSH1 -> 1
	pub fn push_bytes(&self) -> Option<usize> {
		if self.is_push() {
			Some(((*self as u8) - (PUSH1 as u8) + 1) as usize)
		} else {
			None
		}
	}


	/// Returns stack position of item to duplicate
	/// DUP1 -> 0
	pub fn dup_position(&self) -> Option<usize> {
		if *self >= DUP1 && *self <= DUP16 {
			Some(((*self as u8) - (DUP1 as u8)) as usize)
		} else {
			None
		}
	}


	/// Returns stack position of item to SWAP top with
	/// SWAP1 -> 1
	pub fn swap_position(&self) -> Option<usize> {
		if *self >= SWAP1 && *self <= SWAP16 {
			Some(((*self as u8) - (SWAP1 as u8) + 1) as usize)
		} else {
			None
		}
	}

	/// Returns number of topics to take from stack
	/// LOG0 -> 0
	pub fn log_topics(&self) -> Option<usize> {
		if *self >= LOG0 && *self <= LOG4 {
			Some(((*self as u8) - (LOG0 as u8)) as usize)
		} else {
			None
		}
	}

	/// Returns the instruction info.
	pub fn info(&self) -> &'static InstructionInfo {
		INSTRUCTIONS[*self as usize].as_ref().expect("A instruction is defined in Instruction enum, but it is not found in InstructionInfo struct; this indicates a logic failure in the code.")
	}
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
}

impl GasPriceTier {
	/// Returns the index in schedule for specific `GasPriceTier`
	pub fn idx(&self) -> usize {
		match self {
			&GasPriceTier::Zero => 0,
			&GasPriceTier::Base => 1,
			&GasPriceTier::VeryLow => 2,
			&GasPriceTier::Low => 3,
			&GasPriceTier::Mid => 4,
			&GasPriceTier::High => 5,
			&GasPriceTier::Ext => 6,
			&GasPriceTier::Special => 7,
		}
	}
}

/// EVM instruction information.
#[derive(Copy, Clone)]
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
	static ref INSTRUCTIONS: [Option<InstructionInfo>; 0x100] = {
		let mut arr = [None; 0x100];
		arr[STOP as usize] = Some(InstructionInfo::new("STOP", 0, 0, GasPriceTier::Zero));
		arr[ADD as usize] = Some(InstructionInfo::new("ADD", 2, 1, GasPriceTier::VeryLow));
		arr[SUB as usize] = Some(InstructionInfo::new("SUB", 2, 1, GasPriceTier::VeryLow));
		arr[MUL as usize] = Some(InstructionInfo::new("MUL", 2, 1, GasPriceTier::Low));
		arr[DIV as usize] = Some(InstructionInfo::new("DIV", 2, 1, GasPriceTier::Low));
		arr[SDIV as usize] = Some(InstructionInfo::new("SDIV", 2, 1, GasPriceTier::Low));
		arr[MOD as usize] = Some(InstructionInfo::new("MOD", 2, 1, GasPriceTier::Low));
		arr[SMOD as usize] = Some(InstructionInfo::new("SMOD", 2, 1, GasPriceTier::Low));
		arr[EXP as usize] = Some(InstructionInfo::new("EXP", 2, 1, GasPriceTier::Special));
		arr[NOT as usize] = Some(InstructionInfo::new("NOT", 1, 1, GasPriceTier::VeryLow));
		arr[LT as usize] = Some(InstructionInfo::new("LT", 2, 1, GasPriceTier::VeryLow));
		arr[GT as usize] = Some(InstructionInfo::new("GT", 2, 1, GasPriceTier::VeryLow));
		arr[SLT as usize] = Some(InstructionInfo::new("SLT", 2, 1, GasPriceTier::VeryLow));
		arr[SGT as usize] = Some(InstructionInfo::new("SGT", 2, 1, GasPriceTier::VeryLow));
		arr[EQ as usize] = Some(InstructionInfo::new("EQ", 2, 1, GasPriceTier::VeryLow));
		arr[ISZERO as usize] = Some(InstructionInfo::new("ISZERO", 1, 1, GasPriceTier::VeryLow));
		arr[AND as usize] = Some(InstructionInfo::new("AND", 2, 1, GasPriceTier::VeryLow));
		arr[OR as usize] = Some(InstructionInfo::new("OR", 2, 1, GasPriceTier::VeryLow));
		arr[XOR as usize] = Some(InstructionInfo::new("XOR", 2, 1, GasPriceTier::VeryLow));
		arr[BYTE as usize] = Some(InstructionInfo::new("BYTE", 2, 1, GasPriceTier::VeryLow));
		arr[SHL as usize] = Some(InstructionInfo::new("SHL", 2, 1, GasPriceTier::VeryLow));
		arr[SHR as usize] = Some(InstructionInfo::new("SHR", 2, 1, GasPriceTier::VeryLow));
		arr[SAR as usize] = Some(InstructionInfo::new("SAR", 2, 1, GasPriceTier::VeryLow));
		arr[ADDMOD as usize] = Some(InstructionInfo::new("ADDMOD", 3, 1, GasPriceTier::Mid));
		arr[MULMOD as usize] = Some(InstructionInfo::new("MULMOD", 3, 1, GasPriceTier::Mid));
		arr[SIGNEXTEND as usize] = Some(InstructionInfo::new("SIGNEXTEND", 2, 1, GasPriceTier::Low));
		arr[RETURNDATASIZE as usize] = Some(InstructionInfo::new("RETURNDATASIZE", 0, 1, GasPriceTier::Base));
		arr[RETURNDATACOPY as usize] = Some(InstructionInfo::new("RETURNDATACOPY", 3, 0, GasPriceTier::VeryLow));
		arr[SHA3 as usize] = Some(InstructionInfo::new("SHA3", 2, 1, GasPriceTier::Special));
		arr[ADDRESS as usize] = Some(InstructionInfo::new("ADDRESS", 0, 1, GasPriceTier::Base));
		arr[BALANCE as usize] = Some(InstructionInfo::new("BALANCE", 1, 1, GasPriceTier::Special));
		arr[ORIGIN as usize] = Some(InstructionInfo::new("ORIGIN", 0, 1, GasPriceTier::Base));
		arr[CALLER as usize] = Some(InstructionInfo::new("CALLER", 0, 1, GasPriceTier::Base));
		arr[CALLVALUE as usize] = Some(InstructionInfo::new("CALLVALUE", 0, 1, GasPriceTier::Base));
		arr[CALLDATALOAD as usize] = Some(InstructionInfo::new("CALLDATALOAD", 1, 1, GasPriceTier::VeryLow));
		arr[CALLDATASIZE as usize] = Some(InstructionInfo::new("CALLDATASIZE", 0, 1, GasPriceTier::Base));
		arr[CALLDATACOPY as usize] = Some(InstructionInfo::new("CALLDATACOPY", 3, 0, GasPriceTier::VeryLow));
		arr[EXTCODEHASH as usize] = Some(InstructionInfo::new("EXTCODEHASH", 1, 1, GasPriceTier::Special));
		arr[CODESIZE as usize] = Some(InstructionInfo::new("CODESIZE", 0, 1, GasPriceTier::Base));
		arr[CODECOPY as usize] = Some(InstructionInfo::new("CODECOPY", 3, 0, GasPriceTier::VeryLow));
		arr[GASPRICE as usize] = Some(InstructionInfo::new("GASPRICE", 0, 1, GasPriceTier::Base));
		arr[EXTCODESIZE as usize] = Some(InstructionInfo::new("EXTCODESIZE", 1, 1, GasPriceTier::Special));
		arr[EXTCODECOPY as usize] = Some(InstructionInfo::new("EXTCODECOPY", 4, 0, GasPriceTier::Special));
		arr[BLOCKHASH as usize] = Some(InstructionInfo::new("BLOCKHASH", 1, 1, GasPriceTier::Ext));
		arr[COINBASE as usize] = Some(InstructionInfo::new("COINBASE", 0, 1, GasPriceTier::Base));
		arr[TIMESTAMP as usize] = Some(InstructionInfo::new("TIMESTAMP", 0, 1, GasPriceTier::Base));
		arr[NUMBER as usize] = Some(InstructionInfo::new("NUMBER", 0, 1, GasPriceTier::Base));
		arr[DIFFICULTY as usize] = Some(InstructionInfo::new("DIFFICULTY", 0, 1, GasPriceTier::Base));
		arr[GASLIMIT as usize] = Some(InstructionInfo::new("GASLIMIT", 0, 1, GasPriceTier::Base));
		arr[POP as usize] = Some(InstructionInfo::new("POP", 1, 0, GasPriceTier::Base));
		arr[MLOAD as usize] = Some(InstructionInfo::new("MLOAD", 1, 1, GasPriceTier::VeryLow));
		arr[MSTORE as usize] = Some(InstructionInfo::new("MSTORE", 2, 0, GasPriceTier::VeryLow));
		arr[MSTORE8 as usize] = Some(InstructionInfo::new("MSTORE8", 2, 0, GasPriceTier::VeryLow));
		arr[SLOAD as usize] = Some(InstructionInfo::new("SLOAD", 1, 1, GasPriceTier::Special));
		arr[SSTORE as usize] = Some(InstructionInfo::new("SSTORE", 2, 0, GasPriceTier::Special));
		arr[JUMP as usize] = Some(InstructionInfo::new("JUMP", 1, 0, GasPriceTier::Mid));
		arr[JUMPI as usize] = Some(InstructionInfo::new("JUMPI", 2, 0, GasPriceTier::High));
		arr[PC as usize] = Some(InstructionInfo::new("PC", 0, 1, GasPriceTier::Base));
		arr[MSIZE as usize] = Some(InstructionInfo::new("MSIZE", 0, 1, GasPriceTier::Base));
		arr[GAS as usize] = Some(InstructionInfo::new("GAS", 0, 1, GasPriceTier::Base));
		arr[JUMPDEST as usize] = Some(InstructionInfo::new("JUMPDEST", 0, 0, GasPriceTier::Special));
		arr[PUSH1 as usize] = Some(InstructionInfo::new("PUSH1", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH2 as usize] = Some(InstructionInfo::new("PUSH2", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH3 as usize] = Some(InstructionInfo::new("PUSH3", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH4 as usize] = Some(InstructionInfo::new("PUSH4", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH5 as usize] = Some(InstructionInfo::new("PUSH5", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH6 as usize] = Some(InstructionInfo::new("PUSH6", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH7 as usize] = Some(InstructionInfo::new("PUSH7", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH8 as usize] = Some(InstructionInfo::new("PUSH8", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH9 as usize] = Some(InstructionInfo::new("PUSH9", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH10 as usize] = Some(InstructionInfo::new("PUSH10", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH11 as usize] = Some(InstructionInfo::new("PUSH11", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH12 as usize] = Some(InstructionInfo::new("PUSH12", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH13 as usize] = Some(InstructionInfo::new("PUSH13", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH14 as usize] = Some(InstructionInfo::new("PUSH14", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH15 as usize] = Some(InstructionInfo::new("PUSH15", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH16 as usize] = Some(InstructionInfo::new("PUSH16", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH17 as usize] = Some(InstructionInfo::new("PUSH17", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH18 as usize] = Some(InstructionInfo::new("PUSH18", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH19 as usize] = Some(InstructionInfo::new("PUSH19", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH20 as usize] = Some(InstructionInfo::new("PUSH20", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH21 as usize] = Some(InstructionInfo::new("PUSH21", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH22 as usize] = Some(InstructionInfo::new("PUSH22", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH23 as usize] = Some(InstructionInfo::new("PUSH23", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH24 as usize] = Some(InstructionInfo::new("PUSH24", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH25 as usize] = Some(InstructionInfo::new("PUSH25", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH26 as usize] = Some(InstructionInfo::new("PUSH26", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH27 as usize] = Some(InstructionInfo::new("PUSH27", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH28 as usize] = Some(InstructionInfo::new("PUSH28", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH29 as usize] = Some(InstructionInfo::new("PUSH29", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH30 as usize] = Some(InstructionInfo::new("PUSH30", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH31 as usize] = Some(InstructionInfo::new("PUSH31", 0, 1, GasPriceTier::VeryLow));
		arr[PUSH32 as usize] = Some(InstructionInfo::new("PUSH32", 0, 1, GasPriceTier::VeryLow));
		arr[DUP1 as usize] = Some(InstructionInfo::new("DUP1", 1, 2, GasPriceTier::VeryLow));
		arr[DUP2 as usize] = Some(InstructionInfo::new("DUP2", 2, 3, GasPriceTier::VeryLow));
		arr[DUP3 as usize] = Some(InstructionInfo::new("DUP3", 3, 4, GasPriceTier::VeryLow));
		arr[DUP4 as usize] = Some(InstructionInfo::new("DUP4", 4, 5, GasPriceTier::VeryLow));
		arr[DUP5 as usize] = Some(InstructionInfo::new("DUP5", 5, 6, GasPriceTier::VeryLow));
		arr[DUP6 as usize] = Some(InstructionInfo::new("DUP6", 6, 7, GasPriceTier::VeryLow));
		arr[DUP7 as usize] = Some(InstructionInfo::new("DUP7", 7, 8, GasPriceTier::VeryLow));
		arr[DUP8 as usize] = Some(InstructionInfo::new("DUP8", 8, 9, GasPriceTier::VeryLow));
		arr[DUP9 as usize] = Some(InstructionInfo::new("DUP9", 9, 10, GasPriceTier::VeryLow));
		arr[DUP10 as usize] = Some(InstructionInfo::new("DUP10", 10, 11, GasPriceTier::VeryLow));
		arr[DUP11 as usize] = Some(InstructionInfo::new("DUP11", 11, 12, GasPriceTier::VeryLow));
		arr[DUP12 as usize] = Some(InstructionInfo::new("DUP12", 12, 13, GasPriceTier::VeryLow));
		arr[DUP13 as usize] = Some(InstructionInfo::new("DUP13", 13, 14, GasPriceTier::VeryLow));
		arr[DUP14 as usize] = Some(InstructionInfo::new("DUP14", 14, 15, GasPriceTier::VeryLow));
		arr[DUP15 as usize] = Some(InstructionInfo::new("DUP15", 15, 16, GasPriceTier::VeryLow));
		arr[DUP16 as usize] = Some(InstructionInfo::new("DUP16", 16, 17, GasPriceTier::VeryLow));
		arr[SWAP1 as usize] = Some(InstructionInfo::new("SWAP1", 2, 2, GasPriceTier::VeryLow));
		arr[SWAP2 as usize] = Some(InstructionInfo::new("SWAP2", 3, 3, GasPriceTier::VeryLow));
		arr[SWAP3 as usize] = Some(InstructionInfo::new("SWAP3", 4, 4, GasPriceTier::VeryLow));
		arr[SWAP4 as usize] = Some(InstructionInfo::new("SWAP4", 5, 5, GasPriceTier::VeryLow));
		arr[SWAP5 as usize] = Some(InstructionInfo::new("SWAP5", 6, 6, GasPriceTier::VeryLow));
		arr[SWAP6 as usize] = Some(InstructionInfo::new("SWAP6", 7, 7, GasPriceTier::VeryLow));
		arr[SWAP7 as usize] = Some(InstructionInfo::new("SWAP7", 8, 8, GasPriceTier::VeryLow));
		arr[SWAP8 as usize] = Some(InstructionInfo::new("SWAP8", 9, 9, GasPriceTier::VeryLow));
		arr[SWAP9 as usize] = Some(InstructionInfo::new("SWAP9", 10, 10, GasPriceTier::VeryLow));
		arr[SWAP10 as usize] = Some(InstructionInfo::new("SWAP10", 11, 11, GasPriceTier::VeryLow));
		arr[SWAP11 as usize] = Some(InstructionInfo::new("SWAP11", 12, 12, GasPriceTier::VeryLow));
		arr[SWAP12 as usize] = Some(InstructionInfo::new("SWAP12", 13, 13, GasPriceTier::VeryLow));
		arr[SWAP13 as usize] = Some(InstructionInfo::new("SWAP13", 14, 14, GasPriceTier::VeryLow));
		arr[SWAP14 as usize] = Some(InstructionInfo::new("SWAP14", 15, 15, GasPriceTier::VeryLow));
		arr[SWAP15 as usize] = Some(InstructionInfo::new("SWAP15", 16, 16, GasPriceTier::VeryLow));
		arr[SWAP16 as usize] = Some(InstructionInfo::new("SWAP16", 17, 17, GasPriceTier::VeryLow));
		arr[LOG0 as usize] = Some(InstructionInfo::new("LOG0", 2, 0, GasPriceTier::Special));
		arr[LOG1 as usize] = Some(InstructionInfo::new("LOG1", 3, 0, GasPriceTier::Special));
		arr[LOG2 as usize] = Some(InstructionInfo::new("LOG2", 4, 0, GasPriceTier::Special));
		arr[LOG3 as usize] = Some(InstructionInfo::new("LOG3", 5, 0, GasPriceTier::Special));
		arr[LOG4 as usize] = Some(InstructionInfo::new("LOG4", 6, 0, GasPriceTier::Special));
		arr[CREATE as usize] = Some(InstructionInfo::new("CREATE", 3, 1, GasPriceTier::Special));
		arr[CALL as usize] = Some(InstructionInfo::new("CALL", 7, 1, GasPriceTier::Special));
		arr[CALLCODE as usize] = Some(InstructionInfo::new("CALLCODE", 7, 1, GasPriceTier::Special));
		arr[RETURN as usize] = Some(InstructionInfo::new("RETURN", 2, 0, GasPriceTier::Zero));
		arr[DELEGATECALL as usize] = Some(InstructionInfo::new("DELEGATECALL", 6, 1, GasPriceTier::Special));
		arr[STATICCALL as usize] = Some(InstructionInfo::new("STATICCALL", 6, 1, GasPriceTier::Special));
		arr[SUICIDE as usize] = Some(InstructionInfo::new("SUICIDE", 1, 0, GasPriceTier::Special));
		arr[CREATE2 as usize] = Some(InstructionInfo::new("CREATE2", 4, 1, GasPriceTier::Special));
		arr[REVERT as usize] = Some(InstructionInfo::new("REVERT", 2, 0, GasPriceTier::Zero));
		arr
	};
}

/// Maximal number of topics for log instructions
pub const MAX_NO_OF_TOPICS: usize = 4;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_push() {
		assert!(PUSH1.is_push());
		assert!(PUSH32.is_push());
		assert!(!DUP1.is_push());
	}

	#[test]
	fn test_get_push_bytes() {
		assert_eq!(PUSH1.push_bytes(), Some(1));
		assert_eq!(PUSH3.push_bytes(), Some(3));
		assert_eq!(PUSH32.push_bytes(), Some(32));
	}

	#[test]
	fn test_get_dup_position() {
		assert_eq!(DUP1.dup_position(), Some(0));
		assert_eq!(DUP5.dup_position(), Some(4));
		assert_eq!(DUP10.dup_position(), Some(9));
	}

	#[test]
	fn test_get_swap_position() {
		assert_eq!(SWAP1.swap_position(), Some(1));
		assert_eq!(SWAP5.swap_position(), Some(5));
		assert_eq!(SWAP10.swap_position(), Some(10));
	}

	#[test]
	fn test_get_log_topics() {
		assert_eq!(LOG0.log_topics(), Some(0));
		assert_eq!(LOG2.log_topics(), Some(2));
		assert_eq!(LOG4.log_topics(), Some(4));
	}
}
