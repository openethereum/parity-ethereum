use std::collections::HashMap;
use util::uint::*;
use util::hash::*;

pub struct Account {
	balance: U256,
	code: Vec<u8>,
	nonce: U256,
	storage: HashMap<U256, U256>
}

impl Account {
	pub fn new_with_balance(balance: U256) -> Account {
		Account {
			balance: balance,
			code: vec![],
			nonce: U256::from(0u8),
			storage: HashMap::new()
		}
	}

	pub fn balance(&self) -> &U256 { &self.balance }
	pub fn code(&self) -> &[u8] { &self.code }
	pub fn nonce(&self) -> &U256 { &self.nonce }
	pub fn storage(&self) -> &HashMap<U256, U256> { &self.storage }
}

