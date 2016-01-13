//! VM Instructions list and utility functions

pub type Instruction = u8;

pub fn is_push (i : Instruction) -> bool {
	i >= PUSH1 && i <= PUSH32
}

pub fn get_push_bytes (i : Instruction) -> usize {
	// TODO [todr] range checking?
	(i - PUSH1 + 1) as usize
}

#[test]
fn test_get_push_bytes() {
	assert_eq!(get_push_bytes(PUSH1), 1);
	assert_eq!(get_push_bytes(PUSH3), 3);
	assert_eq!(get_push_bytes(PUSH32), 32);
}

pub fn get_dup_position (i: Instruction) -> usize {
	// TODO [todr] range checking?
	(i - DUP1) as usize
}

#[test]
fn test_get_dup_position() {
	assert_eq!(get_dup_position(DUP1), 1);
	assert_eq!(get_dup_position(DUP5), 5);
	assert_eq!(get_dup_position(DUP10), 10);
}

pub fn get_swap_position (i : Instruction) -> usize {
	// TODO [todr] range checking?
	(i - SWAP1 + 1) as usize
}

#[test]
fn test_get_swap_position() {
	assert_eq!(get_swap_position(SWAP1), 1);
	assert_eq!(get_swap_position(SWAP5), 5);
	assert_eq!(get_swap_position(SWAP10), 10);
}

pub fn get_log_topics (i: Instruction) -> usize {
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
	ZeroTier,
	/// 2 Quick
	BaseTier,
	/// 3 Fastest
	VeryLowTier,
	/// 5 Fast
	LowTier,
	/// 8 Mid
	MidTier,
	/// 10 Slow
	HighTier,
	/// 20 Ext
	ExtTier,
	/// Multiparam or otherwise special
	SpecialTier,
	/// Invalid
	InvalidTier
}

pub fn get_tier_idx (tier: GasPriceTier) -> usize {
	match tier {
		GasPriceTier::ZeroTier => 0,
		GasPriceTier::BaseTier => 1,
		GasPriceTier::VeryLowTier => 2,
		GasPriceTier::LowTier => 3,
		GasPriceTier::MidTier => 4,
		GasPriceTier::HighTier => 5,
		GasPriceTier::ExtTier => 6,
		GasPriceTier::SpecialTier => 7,
		GasPriceTier::InvalidTier => 8
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

pub fn get_info (instruction: Instruction) -> InstructionInfo {
	match instruction {
		STOP => 		InstructionInfo::new("STOP",			0, 0, 0, true, GasPriceTier::ZeroTier),
		ADD => 			InstructionInfo::new("ADD",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		SUB => 			InstructionInfo::new("SUB",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		MUL => 			InstructionInfo::new("MUL",			0, 2, 1, false, GasPriceTier::LowTier),
		DIV => 			InstructionInfo::new("DIV",			0, 2, 1, false, GasPriceTier::LowTier),
		SDIV => 		InstructionInfo::new("SDIV",			0, 2, 1, false, GasPriceTier::LowTier),
		MOD => 			InstructionInfo::new("MOD",			0, 2, 1, false, GasPriceTier::LowTier),
		SMOD => 		InstructionInfo::new("SMOD",			0, 2, 1, false, GasPriceTier::LowTier),
		EXP => 			InstructionInfo::new("EXP",			0, 2, 1, false, GasPriceTier::SpecialTier),
		NOT => 			InstructionInfo::new("NOT",			0, 1, 1, false, GasPriceTier::VeryLowTier),
		LT => 			InstructionInfo::new("LT",				0, 2, 1, false, GasPriceTier::VeryLowTier),
		GT => 			InstructionInfo::new("GT",				0, 2, 1, false, GasPriceTier::VeryLowTier),
		SLT => 			InstructionInfo::new("SLT",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		SGT => 			InstructionInfo::new("SGT",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		EQ => 			InstructionInfo::new("EQ",				0, 2, 1, false, GasPriceTier::VeryLowTier),
		ISZERO => 		InstructionInfo::new("ISZERO",			0, 1, 1, false, GasPriceTier::VeryLowTier),
		AND => 			InstructionInfo::new("AND",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		OR => 			InstructionInfo::new("OR",				0, 2, 1, false, GasPriceTier::VeryLowTier),
		XOR => 			InstructionInfo::new("XOR",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		BYTE => 		InstructionInfo::new("BYTE",			0, 2, 1, false, GasPriceTier::VeryLowTier),
		ADDMOD => 		InstructionInfo::new("ADDMOD",			0, 3, 1, false, GasPriceTier::MidTier),
		MULMOD => 		InstructionInfo::new("MULMOD",			0, 3, 1, false, GasPriceTier::MidTier),
		SIGNEXTEND => 	InstructionInfo::new("SIGNEXTEND",		0, 2, 1, false, GasPriceTier::LowTier),
		SHA3 => 		InstructionInfo::new("SHA3",			0, 2, 1, false, GasPriceTier::SpecialTier),
		ADDRESS => 		InstructionInfo::new("ADDRESS",		0, 0, 1, false, GasPriceTier::BaseTier),
		BALANCE => 		InstructionInfo::new("BALANCE",		0, 1, 1, false, GasPriceTier::ExtTier),
		ORIGIN => 		InstructionInfo::new("ORIGIN",			0, 0, 1, false, GasPriceTier::BaseTier),
		CALLER => 		InstructionInfo::new("CALLER",			0, 0, 1, false, GasPriceTier::BaseTier),
		CALLVALUE => 	InstructionInfo::new("CALLVALUE",		0, 0, 1, false, GasPriceTier::BaseTier),
		CALLDATALOAD => InstructionInfo::new("CALLDATALOAD",	0, 1, 1, false, GasPriceTier::VeryLowTier),
		CALLDATASIZE => InstructionInfo::new("CALLDATASIZE",	0, 0, 1, false, GasPriceTier::BaseTier),
		CALLDATACOPY => InstructionInfo::new("CALLDATACOPY",	0, 3, 0, true, GasPriceTier::VeryLowTier),
		CODESIZE => 	InstructionInfo::new("CODESIZE",		0, 0, 1, false, GasPriceTier::BaseTier),
		CODECOPY => 	InstructionInfo::new("CODECOPY",		0, 3, 0, true, GasPriceTier::VeryLowTier),
		GASPRICE => 	InstructionInfo::new("GASPRICE",		0, 0, 1, false, GasPriceTier::BaseTier),
		EXTCODESIZE => 	InstructionInfo::new("EXTCODESIZE",	0, 1, 1, false, GasPriceTier::ExtTier),
		EXTCODECOPY => 	InstructionInfo::new("EXTCODECOPY",	0, 4, 0, true, GasPriceTier::ExtTier),
		BLOCKHASH => 	InstructionInfo::new("BLOCKHASH",		0, 1, 1, false, GasPriceTier::ExtTier),
		COINBASE => 	InstructionInfo::new("COINBASE",		0, 0, 1, false, GasPriceTier::BaseTier),
		TIMESTAMP => 	InstructionInfo::new("TIMESTAMP",		0, 0, 1, false, GasPriceTier::BaseTier),
		NUMBER => 		InstructionInfo::new("NUMBER",			0, 0, 1, false, GasPriceTier::BaseTier),
		DIFFICULTY => 	InstructionInfo::new("DIFFICULTY",		0, 0, 1, false, GasPriceTier::BaseTier),
		GASLIMIT => 	InstructionInfo::new("GASLIMIT",		0, 0, 1, false, GasPriceTier::BaseTier),
		POP => 			InstructionInfo::new("POP",			0, 1, 0, false, GasPriceTier::BaseTier),
		MLOAD => 		InstructionInfo::new("MLOAD",			0, 1, 1, false, GasPriceTier::VeryLowTier),
		MSTORE => 		InstructionInfo::new("MSTORE",			0, 2, 0, true, GasPriceTier::VeryLowTier),
		MSTORE8 => 		InstructionInfo::new("MSTORE8",		0, 2, 0, true, GasPriceTier::VeryLowTier),
		SLOAD => 		InstructionInfo::new("SLOAD",			0, 1, 1, false, GasPriceTier::SpecialTier),
		SSTORE => 		InstructionInfo::new("SSTORE",			0, 2, 0, true, GasPriceTier::SpecialTier),
		JUMP => 		InstructionInfo::new("JUMP",			0, 1, 0, true, GasPriceTier::MidTier),
		JUMPI => 		InstructionInfo::new("JUMPI",			0, 2, 0, true, GasPriceTier::HighTier),
		PC => 			InstructionInfo::new("PC",				0, 0, 1, false, GasPriceTier::BaseTier),
		MSIZE => 		InstructionInfo::new("MSIZE",			0, 0, 1, false, GasPriceTier::BaseTier),
		GAS => 			InstructionInfo::new("GAS",			0, 0, 1, false, GasPriceTier::BaseTier),
		JUMPDEST => 	InstructionInfo::new("JUMPDEST",		0, 0, 0, true, GasPriceTier::SpecialTier),
		PUSH1 => 		InstructionInfo::new("PUSH1",			1, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH2 => 		InstructionInfo::new("PUSH2",			2, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH3 => 		InstructionInfo::new("PUSH3",			3, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH4 => 		InstructionInfo::new("PUSH4",			4, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH5 => 		InstructionInfo::new("PUSH5",			5, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH6 => 		InstructionInfo::new("PUSH6",			6, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH7 => 		InstructionInfo::new("PUSH7",			7, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH8 => 		InstructionInfo::new("PUSH8",			8, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH9 => 		InstructionInfo::new("PUSH9",			9, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH10 => 		InstructionInfo::new("PUSH10",			10, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH11 => 		InstructionInfo::new("PUSH11",			11, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH12 => 		InstructionInfo::new("PUSH12",			12, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH13 => 		InstructionInfo::new("PUSH13",			13, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH14 => 		InstructionInfo::new("PUSH14",			14, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH15 => 		InstructionInfo::new("PUSH15",			15, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH16 => 		InstructionInfo::new("PUSH16",			16, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH17 => 		InstructionInfo::new("PUSH17",			17, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH18 => 		InstructionInfo::new("PUSH18",			18, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH19 => 		InstructionInfo::new("PUSH19",			19, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH20 => 		InstructionInfo::new("PUSH20",			20, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH21 => 		InstructionInfo::new("PUSH21",			21, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH22 => 		InstructionInfo::new("PUSH22",			22, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH23 => 		InstructionInfo::new("PUSH23",			23, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH24 => 		InstructionInfo::new("PUSH24",			24, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH25 => 		InstructionInfo::new("PUSH25",			25, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH26 => 		InstructionInfo::new("PUSH26",			26, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH27 => 		InstructionInfo::new("PUSH27",			27, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH28 => 		InstructionInfo::new("PUSH28",			28, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH29 => 		InstructionInfo::new("PUSH29",			29, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH30 => 		InstructionInfo::new("PUSH30",			30, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH31 => 		InstructionInfo::new("PUSH31",			31, 0, 1, false, GasPriceTier::VeryLowTier),
		PUSH32 => 		InstructionInfo::new("PUSH32",			32, 0, 1, false, GasPriceTier::VeryLowTier),
		DUP1 => 		InstructionInfo::new("DUP1",			0, 1, 2, false, GasPriceTier::VeryLowTier),
		DUP2 => 		InstructionInfo::new("DUP2",			0, 2, 3, false, GasPriceTier::VeryLowTier),
		DUP3 => 		InstructionInfo::new("DUP3",			0, 3, 4, false, GasPriceTier::VeryLowTier),
		DUP4 => 		InstructionInfo::new("DUP4",			0, 4, 5, false, GasPriceTier::VeryLowTier),
		DUP5 => 		InstructionInfo::new("DUP5",			0, 5, 6, false, GasPriceTier::VeryLowTier),
		DUP6 => 		InstructionInfo::new("DUP6",			0, 6, 7, false, GasPriceTier::VeryLowTier),
		DUP7 => 		InstructionInfo::new("DUP7",			0, 7, 8, false, GasPriceTier::VeryLowTier),
		DUP8 => 		InstructionInfo::new("DUP8",			0, 8, 9, false, GasPriceTier::VeryLowTier),
		DUP9 => 		InstructionInfo::new("DUP9",			0, 9, 10, false, GasPriceTier::VeryLowTier),
		DUP10 => 		InstructionInfo::new("DUP10",			0, 10, 11, false, GasPriceTier::VeryLowTier),
		DUP11 => 		InstructionInfo::new("DUP11",			0, 11, 12, false, GasPriceTier::VeryLowTier),
		DUP12 => 		InstructionInfo::new("DUP12",			0, 12, 13, false, GasPriceTier::VeryLowTier),
		DUP13 => 		InstructionInfo::new("DUP13",			0, 13, 14, false, GasPriceTier::VeryLowTier),
		DUP14 => 		InstructionInfo::new("DUP14",			0, 14, 15, false, GasPriceTier::VeryLowTier),
		DUP15 => 		InstructionInfo::new("DUP15",			0, 15, 16, false, GasPriceTier::VeryLowTier),
		DUP16 => 		InstructionInfo::new("DUP16",			0, 16, 17, false, GasPriceTier::VeryLowTier),
		SWAP1 => 		InstructionInfo::new("SWAP1",			0, 2, 2, false, GasPriceTier::VeryLowTier),
		SWAP2 => 		InstructionInfo::new("SWAP2",			0, 3, 3, false, GasPriceTier::VeryLowTier),
		SWAP3 => 		InstructionInfo::new("SWAP3",			0, 4, 4, false, GasPriceTier::VeryLowTier),
		SWAP4 => 		InstructionInfo::new("SWAP4",			0, 5, 5, false, GasPriceTier::VeryLowTier),
		SWAP5 => 		InstructionInfo::new("SWAP5",			0, 6, 6, false, GasPriceTier::VeryLowTier),
		SWAP6 => 		InstructionInfo::new("SWAP6",			0, 7, 7, false, GasPriceTier::VeryLowTier),
		SWAP7 => 		InstructionInfo::new("SWAP7",			0, 8, 8, false, GasPriceTier::VeryLowTier),
		SWAP8 => 		InstructionInfo::new("SWAP8",			0, 9, 9, false, GasPriceTier::VeryLowTier),
		SWAP9 => 		InstructionInfo::new("SWAP9",			0, 10, 10, false, GasPriceTier::VeryLowTier),
		SWAP10 => 		InstructionInfo::new("SWAP10",			0, 11, 11, false, GasPriceTier::VeryLowTier),
		SWAP11 => 		InstructionInfo::new("SWAP11",			0, 12, 12, false, GasPriceTier::VeryLowTier),
		SWAP12 => 		InstructionInfo::new("SWAP12",			0, 13, 13, false, GasPriceTier::VeryLowTier),
		SWAP13 => 		InstructionInfo::new("SWAP13",			0, 14, 14, false, GasPriceTier::VeryLowTier),
		SWAP14 => 		InstructionInfo::new("SWAP14",			0, 15, 15, false, GasPriceTier::VeryLowTier),
		SWAP15 => 		InstructionInfo::new("SWAP15",			0, 16, 16, false, GasPriceTier::VeryLowTier),
		SWAP16 => 		InstructionInfo::new("SWAP16",			0, 17, 17, false, GasPriceTier::VeryLowTier),
		LOG0 => 		InstructionInfo::new("LOG0",			0, 2, 0, true, GasPriceTier::SpecialTier),
		LOG1 => 		InstructionInfo::new("LOG1",			0, 3, 0, true, GasPriceTier::SpecialTier),
		LOG2 => 		InstructionInfo::new("LOG2",			0, 4, 0, true, GasPriceTier::SpecialTier),
		LOG3 => 		InstructionInfo::new("LOG3",			0, 5, 0, true, GasPriceTier::SpecialTier),
		LOG4 => 		InstructionInfo::new("LOG4",			0, 6, 0, true, GasPriceTier::SpecialTier),
		CREATE => 		InstructionInfo::new("CREATE",			0, 3, 1, true, GasPriceTier::SpecialTier),
		CALL => 		InstructionInfo::new("CALL",			0, 7, 1, true, GasPriceTier::SpecialTier),
		CALLCODE => 	InstructionInfo::new("CALLCODE",		0, 7, 1, true, GasPriceTier::SpecialTier),
		RETURN => 		InstructionInfo::new("RETURN",			0, 2, 0, true, GasPriceTier::ZeroTier),
		DELEGATECALL => InstructionInfo::new("DELEGATECALL",	0, 6, 1, true, GasPriceTier::SpecialTier),
		SUICIDE => 		InstructionInfo::new("SUICIDE",		0, 1, 0, true, GasPriceTier::ZeroTier),
		_ => panic!(format!("Undefined instruction: {}", instruction))
	}
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
pub const	PUSH2: Instruction =  0x61; //< place 2 byte item on stack
pub const	PUSH3: Instruction =  0x62; //< place 3 byte item on stack
pub const	PUSH4: Instruction =  0x63; //< place 4 byte item on stack
pub const	PUSH5: Instruction =  0x64; //< place 5 byte item on stack
pub const	PUSH6: Instruction =  0x65; //< place 6 byte item on stack
pub const	PUSH7: Instruction =  0x66; //< place 7 byte item on stack
pub const	PUSH8: Instruction =  0x67; //< place 8 byte item on stack
pub const	PUSH9: Instruction =  0x68; //< place 9 byte item on stack
pub const	PUSH10: Instruction =  0x69; //< place 10 byte item on stack
pub const	PUSH11: Instruction =  0x6a; //< place 11 byte item on stack
pub const	PUSH12: Instruction =  0x6b; //< place 12 byte item on stack
pub const	PUSH13: Instruction =  0x6c; //< place 13 byte item on stack
pub const	PUSH14: Instruction =  0x6d; //< place 14 byte item on stack
pub const	PUSH15: Instruction =  0x6e; //< place 15 byte item on stack
pub const	PUSH16: Instruction =  0x6f; //< place 16 byte item on stack
pub const	PUSH17: Instruction =  0x70; //< place 17 byte item on stack
pub const	PUSH18: Instruction =  0x71; //< place 18 byte item on stack
pub const	PUSH19: Instruction =  0x72; //< place 19 byte item on stack
pub const	PUSH20: Instruction =  0x73; //< place 20 byte item on stack
pub const	PUSH21: Instruction =  0x74; //< place 21 byte item on stack
pub const	PUSH22: Instruction =  0x75; //< place 22 byte item on stack
pub const	PUSH23: Instruction =  0x76; //< place 23 byte item on stack
pub const	PUSH24: Instruction =  0x77; //< place 24 byte item on stack
pub const	PUSH25: Instruction =  0x78; //< place 25 byte item on stack
pub const	PUSH26: Instruction =  0x79; //< place 26 byte item on stack
pub const	PUSH27: Instruction =  0x7a; //< place 27 byte item on stack
pub const	PUSH28: Instruction =  0x7b; //< place 28 byte item on stack
pub const	PUSH29: Instruction =  0x7c; //< place 29 byte item on stack
pub const	PUSH30: Instruction =  0x7d; //< place 30 byte item on stack
pub const	PUSH31: Instruction =  0x7e; //< place 31 byte item on stack
pub const	PUSH32: Instruction =  0x7f; //< place 32 byte item on stack

pub const	DUP1: Instruction =  0x80;		//< copies the highest item in the stack to the top of the stack
pub const	DUP2: Instruction =  0x81; //< copies the second highest item in the stack to the top of the stack
pub const	DUP3: Instruction =  0x82; //< copies the third highest item in the stack to the top of the stack
pub const	DUP4: Instruction =  0x83; //< copies the 4th highest item in the stack to the top of the stack
pub const	DUP5: Instruction =  0x84; //< copies the 5th highest item in the stack to the top of the stack
pub const	DUP6: Instruction =  0x85; //< copies the 6th highest item in the stack to the top of the stack
pub const	DUP7: Instruction =  0x86; //< copies the 7th highest item in the stack to the top of the stack
pub const	DUP8: Instruction =  0x87; //< copies the 8th highest item in the stack to the top of the stack
pub const	DUP9: Instruction =  0x88; //< copies the 9th highest item in the stack to the top of the stack
pub const	DUP10: Instruction =  0x89; //< copies the 10th highest item in the stack to the top of the stack
pub const	DUP11: Instruction =  0x8a; //< copies the 11th highest item in the stack to the top of the stack
pub const	DUP12: Instruction =  0x8b; //< copies the 12th highest item in the stack to the top of the stack
pub const	DUP13: Instruction =  0x8c; //< copies the 13th highest item in the stack to the top of the stack
pub const	DUP14: Instruction =  0x8d; //< copies the 14th highest item in the stack to the top of the stack
pub const	DUP15: Instruction =  0x8e; //< copies the 15th highest item in the stack to the top of the stack
pub const	DUP16: Instruction =  0x8f; //< copies the 16th highest item in the stack to the top of the stack

pub const	SWAP1: Instruction =  0x90;		//< swaps the highest and second highest value on the stack
pub const	SWAP2: Instruction =  0x91; //< swaps the highest and third highest value on the stack
pub const	SWAP3: Instruction =  0x92; //< swaps the highest and 4th highest value on the stack
pub const	SWAP4: Instruction =  0x93; //< swaps the highest and 5th highest value on the stack
pub const	SWAP5: Instruction =  0x94; //< swaps the highest and 6th highest value on the stack
pub const	SWAP6: Instruction =  0x95; //< swaps the highest and 7th highest value on the stack
pub const	SWAP7: Instruction =  0x96; //< swaps the highest and 8th highest value on the stack
pub const	SWAP8: Instruction =  0x97; //< swaps the highest and 9th highest value on the stack
pub const	SWAP9: Instruction =  0x98; //< swaps the highest and 10th highest value on the stack
pub const	SWAP10: Instruction =  0x99; //< swaps the highest and 11th highest value on the stack
pub const	SWAP11: Instruction =  0x9a; //< swaps the highest and 12th highest value on the stack
pub const	SWAP12: Instruction =  0x9b; //< swaps the highest and 13th highest value on the stack
pub const	SWAP13: Instruction =  0x9c; //< swaps the highest and 14th highest value on the stack
pub const	SWAP14: Instruction =  0x9d; //< swaps the highest and 15th highest value on the stack
pub const	SWAP15: Instruction =  0x9e; //< swaps the highest and 16th highest value on the stack
pub const	SWAP16: Instruction =  0x9f; //< swaps the highest and 17th highest value on the stack

pub const	LOG0: Instruction =  0xa0;		//< Makes a log entry; no topics.
pub const	LOG1: Instruction =  0xa1; //< Makes a log entry; 1 topic.
pub const	LOG2: Instruction =  0xa2; //< Makes a log entry; 2 topics.
pub const	LOG3: Instruction =  0xa3; //< Makes a log entry; 3 topics.
pub const	LOG4: Instruction =  0xa4; //< Makes a log entry; 4 topics.

pub const	CREATE: Instruction =  0xf0;		//< create a new account with associated code
pub const	CALL: Instruction =  0xf1; //< message-call into an account
pub const	CALLCODE: Instruction =  0xf2; //< message-call with another account's code only
pub const	RETURN: Instruction =  0xf3; //< halt execution returning output data
pub const	DELEGATECALL: Instruction =  0xf4; //< like CALLCODE but keeps caller's value and sender
pub const	SUICIDE: Instruction =  0xff;		//< halt execution and register account for later deletion

