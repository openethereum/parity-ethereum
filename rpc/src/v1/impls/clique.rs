use clique::{Clique as CliqueEngine, VoteType};
use ethcore::client::EngineInfo;
use std::{collections::HashMap, sync::Arc};
use ethereum_types::Address;
use jsonrpc_core::Result;

use v1::helpers::errors;
use v1::traits::Clique;

/// Clique RPC implementation
pub struct CliqueClient<E> where E: EngineInfo + Send + Sync + 'static {
    engine_info: Arc<E>,
}

impl<E> CliqueClient<E> where E: EngineInfo + Send + Sync + 'static {
    /// Creates new CliqueClient.
    pub fn new(engine_info: Arc<E>) -> Self {
        Self {
            engine_info
        }
    }

    fn engine(&self) -> Result<&CliqueEngine> {
        self.engine_info
            .engine()
            .downcast_ref::<CliqueEngine>()
            .ok_or_else(|| errors::internal("Not running Clique", ""))
    }
}

impl<E> Clique for CliqueClient<E> where E: EngineInfo + Send + Sync + 'static {
    fn proposals(&self) -> Result<HashMap<Address, bool>> {
        Ok(self.engine()?.proposals().into_iter().map(|(address, vote)| (address, match vote {
            VoteType::Add => true,
            VoteType::Remove => false,
        })).collect())
    }

    fn propose(&self, address: Address, auth: bool) -> Result<()> {
        let vote = if auth {
            VoteType::Add
        } else {
            VoteType::Remove
        };
        Ok(self.engine()?.vote(address, Some(vote)).map_err(|e| errors::internal("Failed to vote", e))?)
    }

    fn discard(&self, address: Address) -> Result<()> {
        Ok(self.engine()?.vote(address, None).map_err(|e| errors::internal("Failed to vote", e))?)
    }
}