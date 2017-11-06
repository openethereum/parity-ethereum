use {SharedTransaction, VerifiedTransaction};

pub trait Listener {
	fn added(&mut self, _tx: &SharedTransaction, _old: Option<&SharedTransaction>) {}
	fn rejected(&mut self, _tx: VerifiedTransaction) {}
	fn dropped(&mut self, _tx: &SharedTransaction) {}
	fn invalid(&mut self, _tx: &SharedTransaction) {}
	fn cancelled(&mut self, _tx: &SharedTransaction) {}
	fn mined(&mut self, _tx: &SharedTransaction) {}
}

pub struct NoopListener;
impl Listener for NoopListener {}
