// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use miner::banning_queue::{BanningTransactionQueue};
use util::{H256, RwLock, Mutex, Condvar};
use std::sync::{Arc};

#[derive(PartialEq)]
pub enum QueueReservationStatus {
    Ready,
    NotReady,
    Dropped,
}

pub struct QueueReservation {
    queue: Arc<RwLock<BanningTransactionQueue>>,
    cond: Arc<(Mutex<()>, Condvar)>,
    hash: H256,
}

impl QueueReservation {
    pub fn new(
        queue: Arc<RwLock<BanningTransactionQueue>>,
        cond: Arc<(Mutex<()>, Condvar)>,
        hash: H256,
    ) -> QueueReservation {
        QueueReservation {
            queue: queue,
            cond: cond,
            hash: hash,
        }
    }

    pub fn hash(&self) -> H256 {
        self.hash
    }
}

impl Drop for QueueReservation {
    fn drop(&mut self) {
        if let Some(mut queue) = self.queue.try_write() {
            queue.drop_reserved(&self.hash);
        } else {
            warn!("Waiting for write lock to drop QueueReservation");
            self.queue.write().drop_reserved(&self.hash);
        }
        let &(_, ref cvar) = &*self.cond;
        cvar.notify_all();
    }
}