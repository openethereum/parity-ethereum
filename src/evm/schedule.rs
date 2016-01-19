//! Cost schedule and other parameterisations for the EVM.

/// Definition of the cost schedule and other parameterisations for the EVM.
pub struct Schedule {
	/// TODO [Gav Wood] Please document me
	pub exceptional_failed_code_deposit: bool,
	/// TODO [Gav Wood] Please document me
	pub have_delegate_call: bool,
	/// TODO [Tomusdrw] Please document me
	pub stack_limit: usize,
	/// TODO [Gav Wood] Please document me
	pub max_depth: usize,
	/// TODO [Gav Wood] Please document me
	pub tier_step_gas: [usize; 8],
	/// TODO [Gav Wood] Please document me
	pub exp_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub exp_byte_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sha3_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sha3_word_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sload_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sstore_set_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sstore_reset_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub sstore_refund_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub jumpdest_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub log_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub log_data_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub log_topic_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub create_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub call_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub call_stipend: usize,
	/// TODO [Gav Wood] Please document me
	pub call_value_transfer_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub call_new_account_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub suicide_refund_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub memory_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub quad_coeff_div: usize,
	/// TODO [Gav Wood] Please document me
	pub create_data_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub tx_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub tx_create_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub tx_data_zero_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub tx_data_non_zero_gas: usize,
	/// TODO [Gav Wood] Please document me
	pub copy_gas: usize,
}

impl Schedule {
	/// Schedule for the Frontier-era of the Ethereum main net.
	pub fn new_frontier() -> Schedule {
		Self::new(false, false, 21000)
	}

	/// Schedule for the Homestead-era of the Ethereum main net.
	pub fn new_homestead() -> Schedule {
		Self::new(true, true, 53000)
	}

	fn new(efcd: bool, hdc: bool, tcg: usize) -> Schedule {
		Schedule{
			exceptional_failed_code_deposit: efcd,
			have_delegate_call: hdc,
			stack_limit: 1024,
			max_depth: 1024,
			tier_step_gas: [0, 2, 3, 5, 8, 10, 20, 0],
			exp_gas: 10,
			exp_byte_gas: 10,
			sha3_gas: 30,
			sha3_word_gas: 6,
			sload_gas: 50,
			sstore_set_gas: 20000,
			sstore_reset_gas: 5000,
			sstore_refund_gas: 15000,
			jumpdest_gas: 1,
			log_gas: 375,
			log_data_gas: 8,
			log_topic_gas: 375,
			create_gas: 32000,
			call_gas: 40,
			call_stipend: 2300,
			call_value_transfer_gas: 9000,
			call_new_account_gas: 25000,
			suicide_refund_gas: 24000,
			memory_gas: 3,
			quad_coeff_div: 512,
			create_data_gas: 200,
			tx_gas: 21000,
			tx_create_gas: tcg,
			tx_data_zero_gas: 4,
			tx_data_non_zero_gas: 68,
			copy_gas: 3,	
		}
	}
}
