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
	pub fn balance(&self) -> &U256 { &self.balance }
	pub fn code(&self) -> &[u8] { &self.code }
	pub fn nonce(&self) -> &U256 { &self.nonce }
	pub fn storage(&self) -> &HashMap<U256, U256> { &self.storage }
}

pub struct AccountMap {
	accounts: HashMap<Address, Account>
}

impl AccountMap {
	pub fn accounts(&self) -> &HashMap<Address, Account> { &self.accounts }
}
