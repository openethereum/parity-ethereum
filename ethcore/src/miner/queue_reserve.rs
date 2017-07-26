


pub struct TransactionQueueReservation {
    tx_queue: Arc<RwLock<TransactionQueue>>,
    tx_hash: H256,
}

impl TransactionQueueReservation {
    pub fn fill(
        &self,
        transaction: SignedTransaction,
        origin: TransactionOrigin,
        time: QueuingInstant,
        condition: Option<Condition>,
        details_provider: &TransactionDetailsProvider,
    ) -> Result<TransactionImportResult, Error> {
        let tx_queue = self.tx_queue.write();
        self.tx_queue.add(
            transaction,
            origin,
            time,
            condition,
            details_provider)
    }

    pub fn fill_with_banlist(
        &mut self,
        transaction: SignedTransaction,
        time: QueuingInstant,
        details_provider: &TransactionQueueDetailsProvider,
    ) -> Result<TransactionImportResult, Error> {
        let tx_queue = self.tx_queue.write();
        self.tx_queue.add_with_banlist(
            transaction,
            time,
            details_provider)
    }
}