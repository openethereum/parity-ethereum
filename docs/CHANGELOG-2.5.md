## Parity-Ethereum [v2.5.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.5)

Parity-Ethereum is a minor release that improves performance and stability.
This release stabilises the 2.5 branch.

As of today, Parity-Ethereum 2.4 reaches end of life and everyone is
encouraged to upgrade.

## Parity-Ethereum [v2.5.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.4)

Parity Ethereum v2.5.4-stable is a security update that addresses servo/rust-smallvec#148

The full list of included changes:

* cargo update -p smallvec ([#10822](https://github.com/paritytech/parity-ethereum/pull/10822))

## Parity-Ethereum [v2.5.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.3)

Parity-Ethereum 2.5.3-beta is a bugfix release that improves performance and stability.

* EthereumClassic: activate the Atlantis Hardfork
* Clique: fix time overflow
* State tests: treat empty accounts the same as non-existant accounts (EIP 1052)
* Networking: support discovery-only peers (geth bootnodes)
* Snapshotting: fix unclean shutdown while snappshotting is under way

The full list of included changes:

* ethcore/res: activate atlantis classic hf on block 8772000 ([#10766](https://github.com/paritytech/parity-ethereum/pull/10766))
* fix docker tags for publishing ([#10741](https://github.com/paritytech/parity-ethereum/pull/10741))
* fix: aura don't add `SystemTime::now()` ([#10720](https://github.com/paritytech/parity-ethereum/pull/10720))
* Treat empty account the same as non-exist accounts in EIP-1052 ([#10775](https://github.com/paritytech/parity-ethereum/pull/10775))
* DevP2p: Get node IP address and udp port from Socket, if not included in PING packet ([#10705](https://github.com/paritytech/parity-ethereum/pull/10705))
* Add a way to signal shutdown to snapshotting threads ([#10744](https://github.com/paritytech/parity-ethereum/pull/10744))

## Parity-Ethereum [v2.5.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.2)

Parity-Ethereum 2.5.2-beta is a bugfix release that improves performance and stability.

Among others, it enables the _Atlantis_ hardfork on **Morden** and **Kotti** Classic networks.

The full list of included changes:

* [CI] allow cargo audit to fail ([#10676](https://github.com/paritytech/parity-ethereum/pull/10676))
* Reset blockchain properly ([#10669](https://github.com/paritytech/parity-ethereum/pull/10669))
* new image ([#10673](https://github.com/paritytech/parity-ethereum/pull/10673))
* Update publishing ([#10644](https://github.com/paritytech/parity-ethereum/pull/10644))
* enable lto for release builds ([#10717](https://github.com/paritytech/parity-ethereum/pull/10717))
* Use RUSTFLAGS to set the optimization level ([#10719](https://github.com/paritytech/parity-ethereum/pull/10719))
* ethcore: enable ECIP-1054 for classic ([#10731](https://github.com/paritytech/parity-ethereum/pull/10731))

## Parity-Ethereum [v2.5.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.1)

Parity-Ethereum 2.5.1-beta is a bugfix release that improves performance and stability. 

Among others, it enables the Petersburg hardfork on **Rinkeby** and **POA-Core** Network, as well as the **Kovan** Network community hardfork.

The full list of included changes:

* ci: publish docs debug ([#10638](https://github.com/paritytech/parity-ethereum/pull/10638))

## Parity-Ethereum [v2.5.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.0)

Parity-Ethereum 2.5.0-beta is a minor release that improves performance and stabilizes the 2.5 branch by marking it as beta release. 

- This release adds support for the Clique consensus engine ([#9981](https://github.com/paritytech/parity-ethereum/pull/9981))
  - This enables Parity-Ethereum users to use the Görli, the Kotti Classic, and the legacy Rinkeby testnet. To get started try `parity --chain goerli`; note that light client support is currently not yet fully functional.
- This release removes the dead chain configs for Easthub and Ethereum Social ([#10531](https://github.com/paritytech/parity-ethereum/pull/10531))

As of today, Parity-Ethereum 2.3 reaches end of life and everyone is encouraged to upgrade.

The full list of included changes:

* fix(light cull): poll light cull instead of timer ([#10559](https://github.com/paritytech/parity-ethereum/pull/10559))

