use std::rc::Rc;
use std::cell::Cell;

use {U256, Address as Sender, VerifiedTransaction};

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
	nonce: U256,
	gas_price: U256,
	gas: U256,
	sender: Sender,
	insertion_id: Rc<Cell<u64>>,
}

impl TransactionBuilder {
	pub fn tx(&self) -> Self {
		self.clone()
	}

	pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
		self.nonce = nonce.into();
		self
	}

	pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
		self.gas_price = gas_price.into();
		self
	}

	pub fn sender<T: Into<Sender>>(mut self, sender: T) -> Self {
		self.sender = sender.into();
		self
	}

	pub fn new(self) -> VerifiedTransaction {
		let insertion_id = {
			let id = self.insertion_id.get() + 1;
			self.insertion_id.set(id);
			id
		};
		let hash = self.nonce ^ (U256::from(100) * self.gas_price) ^ (U256::from(100_000) * self.sender.low_u64().into());
		VerifiedTransaction {
			hash: hash.into(),
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas: 21_000.into(),
			sender: self.sender,
			insertion_id,
		}
	}
}
