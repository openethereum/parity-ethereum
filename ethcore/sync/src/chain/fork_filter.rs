//! This module contains a wrapper that connects this codebase with `ethereum-forkid` crate which provides `FORK_ID`
//! to support Ethereum network protocol, version 64 and above.

// Re-export ethereum-forkid crate contents here.
pub use ethereum_forkid::{BlockNumber, ForkId, RejectReason};

use ethcore::client::ChainInfo;
use ethereum_forkid::ForkFilter;

/// Wrapper around fork filter that provides integration with `ForkFilter`.
pub struct ForkFilterApi {
    inner: ForkFilter,
}

impl ForkFilterApi {
    /// Create `ForkFilterApi` from `ChainInfo` and an `Iterator` over the hard forks.
    pub fn new<C: ?Sized + ChainInfo, I: IntoIterator<Item = BlockNumber>>(
        client: &C,
        forks: I,
    ) -> Self {
        let chain_info = client.chain_info();
        let genesis_hash = primitive_types07::H256::from_slice(&chain_info.genesis_hash.0);
        Self {
            inner: ForkFilter::new(chain_info.best_block_number, genesis_hash, forks),
        }
    }

    #[cfg(test)]
    /// Dummy version of ForkFilterApi with no forks.
    pub fn new_dummy<C: ?Sized + ChainInfo>(client: &C) -> Self {
        let chain_info = client.chain_info();
        Self {
            inner: ForkFilter::new(
                chain_info.best_block_number,
                primitive_types07::H256::from_slice(&chain_info.genesis_hash.0),
                vec![],
            ),
        }
    }

    fn update_head<C: ?Sized + ChainInfo>(&mut self, client: &C) {
        self.inner.set_head(client.chain_info().best_block_number);
    }

    /// Wrapper for `ForkFilter::current`
    pub fn current<C: ?Sized + ChainInfo>(&mut self, client: &C) -> ForkId {
        self.update_head(client);
        self.inner.current()
    }

    /// Wrapper for `ForkFilter::is_compatible`
    pub fn is_compatible<C: ?Sized + ChainInfo>(
        &mut self,
        client: &C,
        fork_id: ForkId,
    ) -> Result<(), RejectReason> {
        self.update_head(client);
        self.inner.is_compatible(fork_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethcore::{client::TestBlockChainClient, ethereum, spec::Spec};

    fn test_spec<F: Fn() -> Spec>(spec_builder: F, forks: Vec<BlockNumber>) {
        let spec = (spec_builder)();
        let genesis_hash = spec.genesis_header().hash();
        let spec_forks = spec.hard_forks.clone();
        let client = TestBlockChainClient::new_with_spec(spec);

        assert_eq!(
            ForkFilterApi::new(&client, spec_forks).inner,
            ForkFilter::new(
                0,
                primitive_types07::H256::from_slice(&genesis_hash.0),
                forks
            )
        );
    }

    #[test]
    fn ethereum_spec() {
        test_spec(
            || ethereum::new_foundation(&String::new()),
            vec![
                1_150_000, 1_920_000, 2_463_000, 2_675_000, 4_370_000, 7_280_000, 9_069_000,
                9_200_000,
            ],
        )
    }

    #[test]
    fn ropsten_spec() {
        test_spec(
            || ethereum::new_ropsten(&String::new()),
            vec![10, 1_700_000, 4_230_000, 4_939_394, 6_485_846, 7_117_117],
        )
    }

    #[test]
    fn rinkeby_spec() {
        test_spec(
            || ethereum::new_rinkeby(&String::new()),
            vec![1, 2, 3, 1_035_301, 3_660_663, 4_321_234, 5_435_345],
        )
    }

    #[test]
    fn goerli_spec() {
        test_spec(|| ethereum::new_goerli(&String::new()), vec![1_561_651])
    }
}
