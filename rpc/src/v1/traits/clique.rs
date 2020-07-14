
//! Clique rpc interface.
use ethereum_types::Address;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use std::collections::BTreeMap;
use v1::types::*;

/// Clique rpc interface.
#[rpc(server)]
pub trait Clique {
    /// Retrieves a snapshot of all clique state at a given block.
    #[rpc(name = "clique_getSnapshot")]
    fn get_snapshot(&self, block: BlockNumber) -> Result<CliqueSnapshot>;

    /// Retrieves the list of authorized signers at the specified block.
    #[rpc(name = "clique_getSigners")]
    fn get_signers(&self, block: BlockNumber) -> Result<Vec<Address>>;

    /// Returns the current proposals the node is voting on.
    #[rpc(name = "clique_proposals")]
    fn proposals(&self) -> Result<BTreeMap<Address, bool>>;

    /// Adds a new authorization proposal that the signer will attempt to push through. If the auth parameter is true, the local signer votes for the given address to be included in the set of authorized signers. With auth set to false, the vote is against the address.
    #[rpc(name = "clique_propose")]
    fn propose(&self, address: Address, auth: bool) -> Result<()>;

    /// This method drops a currently running proposal. The signer will not cast further votes (either for or against) the address.
    #[rpc(name = "clique_discard")]
    fn discard(&self, address: Address) -> Result<()>;

    /// This is a debugging method which returns statistics about signer activity for the last 64 blocks. The returned object contains the following fields:
    #[rpc(name = "clique_status")]
    fn status(&self) -> Result<CliqueStatus>;
}