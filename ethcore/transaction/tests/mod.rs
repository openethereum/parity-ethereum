extern crate ethcore_transaction;
extern crate ethereum_types;
extern crate ethkey;
#[cfg(test)]
extern crate heapsize;
extern crate rustc_hex;

mod tests {
	use super::*;
	use ethcore_transaction::*;
	use ethereum_types::U256;

	#[test]
	fn heapsize_should_match_std() {
		assert_eq!(8, std::mem::size_of_val(&5u64));
		assert_eq!(4, std::mem::size_of_val(&5u32));
		assert_eq!(1, std::mem::size_of_val(&5u8));

		use heapsize::HeapSizeOf;
		let bytes: Vec<u8> = vec![1, 15, 31, 63, 127, 255];
		let u32s: Vec<u32> = vec![1, 15, 31, 63, 127, 255];
		let u64s: Vec<u64> = vec![1, 15, 31, 63, 127, 255];

		let z: Vec<u64> = Vec::new();
		assert_eq!(0, z.heap_size_of_children());
		assert_eq!(0, std::mem::size_of_val(&*z));

		let exp8 = 6;
		let exp32 = 24;
		let exp64 = 48;

		assert_eq!(exp8, std::mem::size_of_val(&*bytes));
		assert_eq!(exp32, std::mem::size_of_val(&*u32s));
		assert_eq!(exp64, std::mem::size_of_val(&*u64s));

		let byte_slice: &[u8] = &[1, 15, 31, 63, 127, 255];
		let u32_slice: &[u32] = &[1, 15, 31, 63, 127, 255];
		let u64_slice: &[u64] = &[1, 15, 31, 63, 127, 255];
		assert_eq!(exp8, std::mem::size_of_val(&*byte_slice));
		assert_eq!(exp32, std::mem::size_of_val(&*u32_slice));
		assert_eq!(exp64, std::mem::size_of_val(&*u64_slice));

		let bytes: Vec<u8> = ::rustc_hex::FromHex::from_hex("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap();
		let ut: UnverifiedTransaction =
			rlp::decode(&bytes).expect("decoding UnverifiedTransaction failed");

		let t: Transaction = (*ut).clone();
		assert_eq!(t.data, b"");
		assert_eq!(0, std::mem::size_of_val(&*t.data));
		assert_eq!(0, t.heap_size_of_children());
		assert_eq!(0, ut.heap_size_of_children());
		assert_eq!(0, ut.heap_size_of_data());

		let tr = Transaction {
			action: Action::Create,
			nonce: U256::from(42),
			gas_price: U256::from(3000),
			gas: U256::from(50_000),
			value: U256::from(1),
			data: b"Hello!".to_vec(),
		};
		assert_eq!(tr.data, b"Hello!");
		assert_eq!(6, std::mem::size_of_val(&*tr.data));
		// assert_eq!(6, tr.heap_size_of_children());
		assert_eq!(6, tr.heap_size_of_data());

		use ethkey::{Generator, Random};
		let key = Random.generate().unwrap();
		let hash = t.hash(Some(0));
		let sig = ::ethkey::sign(&key.secret(), &hash).unwrap();
		let u = t.with_signature(sig, Some(0));
		let signed = SignedTransaction::new(u).unwrap();
		assert_eq!(0, std::mem::size_of_val(&*signed.data));
		assert_eq!(0, signed.heap_size_of_children());
		assert_eq!(0, signed.heap_size_of_data());
	}
}
