use rand::{self, distributions::Standard, Rng};
use rlp::Encodable;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use types::transaction::SignedTransaction;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub(super) struct Contribution {
	transactions: Vec<Vec<u8>>,
	timestamp: u64,
	/// Random data for on-chain randomness.
	///
	/// The invariant of `random_data.len()` == RANDOM_BYTES_PER_EPOCH **must** hold true.
	random_data: Vec<u8>,
}

/// Number of random bytes to generate per epoch.
///
/// Currently, we want twenty u32s worth of random data to generated on each epoch.
// TODO: Make this configurable somewhere.
const RANDOM_BYTES_PER_EPOCH: usize = 4 * 20;

/// Returns the current UNIX Epoch time, in seconds.
fn unix_now_secs() -> u64 {
	UNIX_EPOCH.elapsed().expect("Time not available").as_secs()
}

impl Contribution {
	pub fn new(txns: &Vec<Arc<SignedTransaction>>) -> Self {
		let ser_txns: Vec<_> = txns.iter().map(|txn| txn.rlp_bytes()).collect();
		let mut rng = rand::thread_rng();

		Contribution {
			transactions: ser_txns,
			timestamp: unix_now_secs(),
			random_data: rng
				.sample_iter(&Standard)
				.take(RANDOM_BYTES_PER_EPOCH)
				.collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::test_helpers::create_transaction;
	use rlp::{Decodable, Rlp};
	use std::sync::Arc;
	use types::transaction::SignedTransaction;

	#[test]
	fn test_contribution_serialization() {
		let mut pending: Vec<Arc<SignedTransaction>> = Vec::new();
		pending.push(Arc::new(create_transaction()));
		let contribution = super::Contribution::new(&pending);

		let deser_txns: Vec<_> = contribution
			.transactions
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
