// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use client::{Rpc, RpcError};
use ethereum_types::U256;
use futures::Canceled;
use rpc::signer::{ConfirmationRequest, TransactionCondition, TransactionModification};
use serde;
use serde_json::{to_value, Value as JsonValue};
use std::path::PathBuf;
use BoxFuture;

pub struct SignerRpc {
    rpc: Rpc,
}

impl SignerRpc {
    pub fn new(url: &str, authfile: &PathBuf) -> Result<Self, RpcError> {
        Ok(SignerRpc {
            rpc: Rpc::new(&url, authfile)?,
        })
    }

    pub fn requests_to_confirm(
        &mut self,
    ) -> BoxFuture<Result<Vec<ConfirmationRequest>, RpcError>, Canceled> {
        self.rpc.request("signer_requestsToConfirm", vec![])
    }

    pub fn confirm_request(
        &mut self,
        id: U256,
        new_gas: Option<U256>,
        new_gas_price: Option<U256>,
        new_condition: Option<Option<TransactionCondition>>,
        pwd: &str,
    ) -> BoxFuture<Result<U256, RpcError>, Canceled> {
        self.rpc.request(
            "signer_confirmRequest",
            vec![
                Self::to_value(&format!("{:#x}", id)),
                Self::to_value(&TransactionModification {
                    sender: None,
                    gas_price: new_gas_price,
                    gas: new_gas,
                    condition: new_condition,
                }),
                Self::to_value(&pwd),
            ],
        )
    }

    pub fn reject_request(&mut self, id: U256) -> BoxFuture<Result<bool, RpcError>, Canceled> {
        self.rpc.request(
            "signer_rejectRequest",
            vec![JsonValue::String(format!("{:#x}", id))],
        )
    }

    fn to_value<T: serde::Serialize>(v: &T) -> JsonValue {
        to_value(v).expect("Our types are always serializable; qed")
    }
}
