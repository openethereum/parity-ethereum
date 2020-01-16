// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Tests for block RLP encoding

use snapshot::test_helpers::AbridgedBlock;

use bytes::Bytes;
use ethereum_types::{H256, U256, Address};
use common_types::{
	transaction::{Action, Transaction},
	block::Block,
	view,
	views::BlockView,
};

fn encode_block(b: &Block) -> Bytes {
	b.rlp_bytes()
}

#[test]
fn empty_block_abridging() {
	let b = Block::default();
	let receipts_root = b.header.receipts_root().clone();
	let encoded = encode_block(&b);

	let abridged = AbridgedBlock::from_block_view(&view!(BlockView, &encoded));
	assert_eq!(abridged.to_block(H256::zero(), 0, receipts_root).unwrap(), b);
}

#[test]
#[should_panic]
fn wrong_number() {
	let b = Block::default();
	let receipts_root = b.header.receipts_root().clone();
	let encoded = encode_block(&b);

	let abridged = AbridgedBlock::from_block_view(&view!(BlockView, &encoded));
	assert_eq!(abridged.to_block(H256::zero(), 2, receipts_root).unwrap(), b);
}

#[test]
fn with_transactions() {
	let mut b = Block::default();

	let t1 = Transaction {
		action: Action::Create,
		nonce: U256::from(42),
		gas_price: U256::from(3000),
		gas: U256::from(50_000),
		value: U256::from(1),
		data: b"Hello!".to_vec()
	}.fake_sign(Address::from_low_u64_be(0x69));

	let t2 = Transaction {
		action: Action::Create,
		nonce: U256::from(88),
		gas_price: U256::from(12345),
		gas: U256::from(300000),
		value: U256::from(1000000000),
		data: "Eep!".into(),
	}.fake_sign(Address::from_low_u64_be(0x55));

	b.transactions.push(t1.into());
	b.transactions.push(t2.into());

	let receipts_root = b.header.receipts_root().clone();
	b.header.set_transactions_root(triehash::ordered_trie_root(
		b.transactions.iter().map(::rlp::encode)
	));

	let encoded = encode_block(&b);

	let abridged = AbridgedBlock::from_block_view(&view!(BlockView, &encoded[..]));
	assert_eq!(abridged.to_block(H256::zero(), 0, receipts_root).unwrap(), b);
}
