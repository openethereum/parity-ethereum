## Parity-Ethereum [v2.4.9](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.9)

Parity Ethereum v2.4.9-stable is a security update which addresses servo/rust-smallvec#148


## Parity-Ethereum [v2.4.8](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.8)

Parity-Ethereum 2.4.8-stable is a bugfix release that improves performance and stability.

* Blockchain: fix reset chain
* State tests: treat empty accounts the same as non-existant accounts (EIP 1052)
* Aura: fix Timestamp Overflow
* Networking: support discovery-only peers (geth bootnodes)
* Snapshotting: fix unclean shutdown while snappshotting is under way


## Parity-Ethereum [v2.4.7](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.7)

Parity-Ethereum 2.4.7-stable is a bugfix release that improves performance and stability.

Among others, it enables the _Atlantis_ hardfork on **Morden** and **Kotti** Classic networks.


## Parity-Ethereum [v2.4.6](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.6)

Parity-Ethereum 2.4.6-stable is a bugfix release that improves performance and stability.

Among others, it enables the Petersburg hardfork on **Rinkeby** and **POA-Core** Network, as well as the **Kovan** Network community hardfork.


## Parity-Ethereum [v2.4.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.5)

Parity-Ethereum 2.4.5-stable is a bugfix release that improves performance and stability. This release improves memory optimizations around timestamp handling and stabilizes the 2.4 release branch.

As of today, Parity-Ethereum 2.3 reaches end of life and everyone is encouraged to upgrade.


## Parity-Ethereum [v2.4.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.4)

Parity-Ethereum 2.4.4-beta is a bugfix release that improves performance and stability. This patch release removes the dead chain configs for Easthub and Ethereum Social.


## Parity-Ethereum [v2.4.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.3)

Parity-Ethereum 2.4.3-beta is a bugfix release that improves performance and stability. This patch release contains a critical bug fix where serving light clients previously led to client crashes. Upgrading is highly recommended.


## Parity-Ethereum [v2.4.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.2)

Parity-Ethereum 2.4.2-beta is a bugfix release that improves performance and stability.


## Parity-Ethereum [v2.4.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.1)

Parity-Ethereum 2.4.1-beta is a bugfix release that improves performance and stability.


## Parity-Ethereum [v2.4.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.0)

Parity-Ethereum 2.4.0-beta is our trifortnightly minor version release coming with a lot of new features as well as bugfixes and performance improvements.

Notable changes:
- Account management is now deprecated (#10213)
- Local accounts can now be specified via CLI (#9960)
- Chains can now be reset to a particular block via CLI (#9782)
- Ethash now additionally implements ProgPoW (#9762) 
- The `eip1283DisableTransition` flag was added to revert EIP-1283 (#10214)


