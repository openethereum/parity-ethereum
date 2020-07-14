use ethereum_types::{Address, H256};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};
use types::BlockNumber;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub hash: H256,
    pub number: BlockNumber,
    pub recents: BTreeMap<BlockNumber, Address>,
    pub signers: BTreeSet<Address>,
    pub tally: BTreeMap<Address, u64>,
    pub votes: Vec<()>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub inturn_percent: u8,
    pub num_blocks: u64,
    pub sealer_activity: BTreeMap<Address, u64>,
}