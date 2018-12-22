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

use ethereum_types::{U256, H256};
use ethkey::{Random, Generator};
use rustc_hex::FromHex;
use transaction::{self, Transaction, SignedTransaction, UnverifiedTransaction};

use pool::{verifier, VerifiedTransaction};

#[derive(Clone)]
pub struct Tx {
	pub nonce: u64,
	pub gas: u64,
	pub gas_price: u64,
	pub multiplier: u64,
}

impl Default for Tx {
	fn default() -> Self {
		Tx {
			nonce: 123,
			gas: 21_000,
			gas_price: 1,
			multiplier: 0,
		}
	}
}

impl Tx {
	pub fn gas_price(gas_price: u64) -> Self {
		Tx {
			gas_price,
			..Default::default()
		}
	}

	pub fn signed(self) -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		self.unsigned().sign(keypair.secret(), None)
	}

	pub fn signed_pair(self) -> (SignedTransaction, SignedTransaction) {
		let (tx1, tx2, _) = self.signed_triple();
		(tx1, tx2)
	}

	pub fn signed_triple(mut self) -> (SignedTransaction, SignedTransaction, SignedTransaction) {
		let keypair = Random.generate().unwrap();
		let tx1 = self.clone().unsigned().sign(keypair.secret(), None);
		self.nonce += 1;
		let tx2 = self.clone().unsigned().sign(keypair.secret(), None);
		self.nonce += 1;
		let tx3 = self.unsigned().sign(keypair.secret(), None);

		(tx1, tx2, tx3)
	}

	/// Consecutive transactions where the gas_price is decided by the predicate
	/// P(i, nonce)
	pub fn signed_consecutive<P>(mut self, amount: usize, fun: P) -> Vec<SignedTransaction>
		where P: Fn(usize, u64) -> u64
	{
		let keypair = Random.generate().unwrap();
		(0..amount).map(|i| {
			self.gas_price = fun(i, self.nonce);
			let tx = self.clone().unsigned().sign(keypair.secret(), None);
			self.nonce += 1;
			tx
		}).collect::<Vec<SignedTransaction>>()
	}

	pub fn signed_replacement(mut self) -> (SignedTransaction, SignedTransaction) {
		let keypair = Random.generate().unwrap();
		let tx1 = self.clone().unsigned().sign(keypair.secret(), None);
		self.gas_price += 1;
		let tx2 = self.unsigned().sign(keypair.secret(), None);

		(tx1, tx2)
	}

	pub fn unsigned(self) -> Transaction {
		Transaction {
			action: transaction::Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: self.gas.into(),
			gas_price: self.gas_price.into(),
			nonce: self.nonce.into()
		}
	}

	pub fn big_one(self) -> SignedTransaction {
		let keypair = Random.generate().unwrap();
		let tx = Transaction {
			action: transaction::Action::Create,
			value: U256::from(100),
			data: include_str!("../res/big_transaction.data").from_hex().unwrap(),
			gas: self.gas.into(),
			gas_price: self.gas_price.into(),
			nonce: self.nonce.into()
		};
		tx.sign(keypair.secret(), None)
	}
}

#[derive(Default)]
pub struct TxBuilder {
	pub nonce: Option<u64>,
	pub gas: Option<u64>,
	pub gas_price: Option<u64>,
	pub multiplier: Option<u64>,
}

impl TxBuilder {
	pub fn build(&mut self) -> Tx {
		let mut tx = Tx::default();
		if let Some(n) = self.nonce {
			tx.nonce = n;
		}
		if let Some(g) = self.gas {
			tx.gas = g;
		}
		if let Some(gp) = self.gas_price {
			tx.gas_price = gp;
		}
		if let Some(multiplier) = self.multiplier {
			tx.multiplier = multiplier;
		}
		tx
	}

	pub fn nonce(&mut self, nonce: u64) -> &mut Self {
		self.nonce = Some(nonce);
		self
	}

	pub fn gas(&mut self, gas: u64) -> &mut Self {
		self.gas = Some(gas);
		self
	}
	
	pub fn gas_price(&mut self, gas_price: u64) -> &mut Self {
		self.gas_price = Some(gas_price);
		self
	}

	pub fn multiplier(&mut self, multiplier: u64) -> &mut Self {
		self.multiplier = Some(multiplier);
		self
	}
}

pub trait TxExt: Sized {
	type Out;
	type Verified;
	type Hash;

	fn hash(&self) -> Self::Hash;

	fn local(self) -> Self::Out;

	fn retracted(self) -> Self::Out;

	fn unverified(self) -> Self::Out;

	fn verified(self) -> Self::Verified;
}

impl<A, B, O, V, H> TxExt for (A, B) where
	A: TxExt<Out=O, Verified=V, Hash=H>,
	B: TxExt<Out=O, Verified=V, Hash=H>,
{
	type Out = (O, O);
	type Verified = (V, V);
	type Hash = (H, H);

	fn hash(&self) -> Self::Hash { (self.0.hash(), self.1.hash()) }
	fn local(self) -> Self::Out { (self.0.local(), self.1.local()) }
	fn retracted(self) -> Self::Out { (self.0.retracted(), self.1.retracted()) }
	fn unverified(self) -> Self::Out { (self.0.unverified(), self.1.unverified()) }
	fn verified(self) -> Self::Verified { (self.0.verified(), self.1.verified()) }
}

impl TxExt for SignedTransaction {
	type Out = verifier::Transaction;
	type Verified = VerifiedTransaction;
	type Hash = H256;

	fn hash(&self) -> Self::Hash {
		UnverifiedTransaction::hash(self)
	}

	fn local(self) -> Self::Out {
		verifier::Transaction::Local(self.into())
	}

	fn retracted(self) -> Self::Out {
		verifier::Transaction::Retracted(self.into())
	}

	fn unverified(self) -> Self::Out {
		verifier::Transaction::Unverified(self.into())
	}

	fn verified(self) -> Self::Verified {
		VerifiedTransaction::from_pending_block_transaction(self)
	}
}

impl TxExt for Vec<SignedTransaction> {
	type Out = Vec<verifier::Transaction>;
	type Verified = Vec<VerifiedTransaction>;
	type Hash = Vec<H256>;

	fn hash(&self) -> Self::Hash {
		self.iter().map(|tx| tx.hash()).collect()
	}

	fn local(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Local).collect()
	}

	fn retracted(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Retracted).collect()
	}

	fn unverified(self) -> Self::Out {
		self.into_iter().map(Into::into).map(verifier::Transaction::Unverified).collect()
	}

	fn verified(self) -> Self::Verified {
		self.into_iter().map(VerifiedTransaction::from_pending_block_transaction).collect()
	}
}

pub trait PairExt {
	type Type;

	fn into_vec(self) -> Vec<Self::Type>;
}

impl<A> PairExt for (A, A) {
	type Type = A;
	fn into_vec(self) -> Vec<A> {
		vec![self.0, self.1]
	}
}
