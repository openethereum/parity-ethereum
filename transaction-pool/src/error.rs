use bigint::hash::H256;

error_chain! {
	errors {
		AlreadyImported(hash: H256) {
			description("transaction is already in the queue"),
			display("[{:?}] transaction already imported", hash)
		}
		TooCheapToEnter(hash: H256) {
			description("the pool is full and transaction is too cheap to replace any transaction"),
			display("[{:?}] transaction too cheap to enter the pool", hash)
		}
		TooCheapToReplace(old_hash: H256, hash: H256) {
			description("transaction is too cheap too replace existing transaction in the queue"),
			display("[{:?}] transaction too cheap to replace: {:?}", hash, old_hash)
		}
	}
}

#[cfg(test)]
impl PartialEq for ErrorKind {
	fn eq(&self, other: &Self) -> bool {
		use self::ErrorKind::*;

		match (self, other) {
			(&AlreadyImported(ref h1), &AlreadyImported(ref h2)) => h1 == h2,
			(&TooCheapToEnter(ref h1), &TooCheapToEnter(ref h2)) => h1 == h2,
			(&TooCheapToReplace(ref old1, ref new1), &TooCheapToReplace(ref old2, ref new2)) => old1 == old2 && new1 == new2,
			_ => false,
		}
	}
}
