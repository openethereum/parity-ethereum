#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub(super) struct Contribution {
	transactions: Vec<Vec<u8>>,
	timestamp: u64,
	/// Random data for on-chain randomness.
	///
	/// The invariant of `random_data.len()` == RANDOM_BYTES_PER_EPOCH **must** hold true.
	random_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
	use crate::test_helpers::create_transaction;
	use rlp::{Decodable, Encodable, Rlp};
	use std::sync::Arc;
	use types::transaction::SignedTransaction;

	#[test]
	fn test_contribution_serialization() {
		let mut pending: Vec<Arc<SignedTransaction>> = Vec::new();
		pending.push(Arc::new(create_transaction()));
		let ser_txns: Vec<_> = pending.iter().map(|txn| txn.rlp_bytes()).collect();

		let deser_txns: Vec<_> = ser_txns
			.iter()
			.filter_map(|ser_txn| Decodable::decode(&Rlp::new(ser_txn)).ok())
			.filter_map(|txn| SignedTransaction::new(txn).ok())
			.collect();

		assert_eq!(pending.len(), deser_txns.len());
		assert_eq!(
			pending.iter().nth(0).unwrap().as_ref(),
			deser_txns.iter().nth(0).unwrap()
		);
	}
}
