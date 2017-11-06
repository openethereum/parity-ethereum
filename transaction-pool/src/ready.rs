use {VerifiedTransaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
	Stalled,
	Ready,
	Future,
}

impl From<bool> for Readiness {
	fn from(b: bool) -> Self {
		if b { Readiness::Ready } else { Readiness::Future }
	}
}

pub trait Ready {
	/// Returns true if transaction is ready to be included in pending block,
	/// given all previous transactions that were ready are included.
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness;
}

impl<F> Ready for F where F: FnMut(&VerifiedTransaction) -> Readiness {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness {
		(*self)(tx)
	}
}
