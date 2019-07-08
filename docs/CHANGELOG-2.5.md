## Parity-Ethereum [v2.5.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.5)

Parity-Ethereum is a minor release that improves performance and stability.
This release stabilises the 2.5 branch.

As of today, Parity-Ethereum 2.4 reaches end of life and everyone is
encouraged to upgrade.


## Parity-Ethereum [v2.5.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.4)

Parity Ethereum v2.5.4-stable is a security update that addresses servo/rust-smallvec#148


## Parity-Ethereum [v2.5.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.3)

Parity-Ethereum 2.5.3-beta is a bugfix release that improves performance and stability.

* EthereumClassic: activate the Atlantis Hardfork
* Clique: fix time overflow
* State tests: treat empty accounts the same as non-existant accounts (EIP 1052)
* Networking: support discovery-only peers (geth bootnodes)
* Snapshotting: fix unclean shutdown while snappshotting is under way


## Parity-Ethereum [v2.5.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.2)

Parity-Ethereum 2.5.2-beta is a bugfix release that improves performance and stability.

Among others, it enables the _Atlantis_ hardfork on **Morden** and **Kotti** Classic networks.


## Parity-Ethereum [v2.5.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.1)

Parity-Ethereum 2.5.1-beta is a bugfix release that improves performance and stability. 

Among others, it enables the Petersburg hardfork on **Rinkeby** and **POA-Core** Network, as well as the **Kovan** Network community hardfork.


## Parity-Ethereum [v2.5.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.0)

Parity-Ethereum 2.5.0-beta is a minor release that improves performance and stabilizes the 2.5 branch by marking it as beta release. 

- This release adds support for the Clique consensus engine (#9981)
  - This enables Parity-Ethereum users to use the Görli, the Kotti Classic, and the legacy Rinkeby testnet. To get started try `parity --chain goerli`; note that light client support is currently not yet fully functional.
- This release removes the dead chain configs for Easthub and Ethereum Social (#10531)

As of today, Parity-Ethereum 2.3 reaches end of life and everyone is encouraged to upgrade.


