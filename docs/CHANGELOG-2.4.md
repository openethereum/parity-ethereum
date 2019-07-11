## Parity-Ethereum [v2.4.9](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.9)

Parity Ethereum v2.4.9-stable is a security update which addresses servo/rust-smallvec#148

The full list of included changes:

* cargo update -p smallvec ([#10822](https://github.com/paritytech/parity-ethereum/pull/10822))

## Parity-Ethereum [v2.4.8](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.8)

Parity-Ethereum 2.4.8-stable is a bugfix release that improves performance and stability.

* Blockchain: fix reset chain
* State tests: treat empty accounts the same as non-existant accounts (EIP 1052)
* Aura: fix Timestamp Overflow
* Networking: support discovery-only peers (geth bootnodes)
* Snapshotting: fix unclean shutdown while snappshotting is under way

The full list of included changes:

* ethcore/res: activate atlantis classic hf on block 8772000 ([#10766](https://github.com/paritytech/parity-ethereum/pull/10766))
* fix docker tags for publishing ([#10741](https://github.com/paritytech/parity-ethereum/pull/10741))
* Reset blockchain properly ([#10669](https://github.com/paritytech/parity-ethereum/pull/10669))
* adds rpc error message for --no-ancient-blocks ([#10608](https://github.com/paritytech/parity-ethereum/pull/10608))
* Treat empty account the same as non-exist accounts in EIP-1052 ([#10775](https://github.com/paritytech/parity-ethereum/pull/10775))
* fix: aura don't add `SystemTime::now()` ([#10720](https://github.com/paritytech/parity-ethereum/pull/10720))
* DevP2p: Get node IP address and udp port from Socket, if not included in PING packet ([#10705](https://github.com/paritytech/parity-ethereum/pull/10705))
* Revert "fix: aura don't add `SystemTime::now()` ([#10720](https://github.com/paritytech/parity-ethereum/pull/10720))"
* Add a way to signal shutdown to snapshotting threads ([#10744](https://github.com/paritytech/parity-ethereum/pull/10744))

## Parity-Ethereum [v2.4.7](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.7)

Parity-Ethereum 2.4.7-stable is a bugfix release that improves performance and stability.

Among others, it enables the _Atlantis_ hardfork on **Morden** and **Kotti** Classic networks.

The full list of included changes:

* [CI] allow cargo audit to fail ([#10676](https://github.com/paritytech/parity-ethereum/pull/10676))
* new image ([#10673](https://github.com/paritytech/parity-ethereum/pull/10673))
* Update publishing ([#10644](https://github.com/paritytech/parity-ethereum/pull/10644))
* enable lto for release builds ([#10717](https://github.com/paritytech/parity-ethereum/pull/10717))
* Use RUSTFLAGS to set the optimization level ([#10719](https://github.com/paritytech/parity-ethereum/pull/10719))
* ethcore: enable ECIP-1054 for classic ([#10731](https://github.com/paritytech/parity-ethereum/pull/10731))

## Parity-Ethereum [v2.4.6](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.6)

Parity-Ethereum 2.4.6-stable is a bugfix release that improves performance and stability.

Among others, it enables the Petersburg hardfork on **Rinkeby** and **POA-Core** Network, as well as the **Kovan** Network community hardfork.

The full list of included changes:

* ci: publish docs debug ([#10638](https://github.com/paritytech/parity-ethereum/pull/10638))

## Parity-Ethereum [v2.4.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.5)

Parity-Ethereum 2.4.5-stable is a bugfix release that improves performance and stability. This release improves memory optimizations around timestamp handling and stabilizes the 2.4 release branch.

As of today, Parity-Ethereum 2.3 reaches end of life and everyone is encouraged to upgrade.

## Parity-Ethereum [v2.4.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.4)

Parity-Ethereum 2.4.4-beta is a bugfix release that improves performance and stability. This patch release removes the dead chain configs for Easthub and Ethereum Social.

The full list of included changes:

* fix(rpc-types): replace uint and hash with `ethereum_types v0.4` ([#10217](https://github.com/paritytech/parity-ethereum/pull/10217))
* chore(bump ethereum-types) ([#10396](https://github.com/paritytech/parity-ethereum/pull/10396))
* fix(light eth_gasPrice): ask network if not in cache ([#10535](https://github.com/paritytech/parity-ethereum/pull/10535))
* fix(light account response): update `tx_queue` ([#10545](https://github.com/paritytech/parity-ethereum/pull/10545))
* fix(bump dependencies) ([#10540](https://github.com/paritytech/parity-ethereum/pull/10540))
* tx-pool: check transaction readiness before replacing ([#10526](https://github.com/paritytech/parity-ethereum/pull/10526))
* fix #10390 ([#10391](https://github.com/paritytech/parity-ethereum/pull/10391))
* private-tx: replace error_chain ([#10510](https://github.com/paritytech/parity-ethereum/pull/10510))

## Parity-Ethereum [v2.4.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.3)

Parity-Ethereum 2.4.3-beta is a bugfix release that improves performance and stability. This patch release contains a critical bug fix where serving light clients previously led to client crashes. Upgrading is highly recommended.

The full list of included changes:

* Add additional request tests ([#10503](https://github.com/paritytech/parity-ethereum/pull/10503))

## Parity-Ethereum [v2.4.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.2)

Parity-Ethereum 2.4.2-beta is a bugfix release that improves performance and stability.

The full list of included changes:

* Сaching through docker volume ([#10477](https://github.com/paritytech/parity-ethereum/pull/10477))
* fix win&mac build ([#10486](https://github.com/paritytech/parity-ethereum/pull/10486))
* fix(extract `timestamp_checked_add` as lib) ([#10383](https://github.com/paritytech/parity-ethereum/pull/10383))

## Parity-Ethereum [v2.4.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.1)

Parity-Ethereum 2.4.1-beta is a bugfix release that improves performance and stability.

The full list of included changes:

* Implement parity_versionInfo & parity_setChain on LC; fix parity_setChain ([#10312](https://github.com/paritytech/parity-ethereum/pull/10312))
* CI publish to aws ([#10446](https://github.com/paritytech/parity-ethereum/pull/10446))
* CI aws git checkout ([#10451](https://github.com/paritytech/parity-ethereum/pull/10451))
* Revert "CI aws git checkout ([#10451](https://github.com/paritytech/parity-ethereum/pull/10451))" (#10456)
* Revert "CI aws git checkout ([#10451](https://github.com/paritytech/parity-ethereum/pull/10451))"
* Tests parallelized ([#10452](https://github.com/paritytech/parity-ethereum/pull/10452))
* Ensure static validator set changes are recognized ([#10467](https://github.com/paritytech/parity-ethereum/pull/10467))

## Parity-Ethereum [v2.4.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.4.0)

Parity-Ethereum 2.4.0-beta is our trifortnightly minor version release coming with a lot of new features as well as bugfixes and performance improvements.

Notable changes:
- Account management is now deprecated ([#10213](https://github.com/paritytech/parity-ethereum/pull/10213))
- Local accounts can now be specified via CLI ([#9960](https://github.com/paritytech/parity-ethereum/pull/9960))
- Chains can now be reset to a particular block via CLI ([#9782](https://github.com/paritytech/parity-ethereum/pull/9782))
- Ethash now additionally implements ProgPoW ([#9762](https://github.com/paritytech/parity-ethereum/pull/9762)) 
- The `eip1283DisableTransition` flag was added to revert EIP-1283 ([#10214](https://github.com/paritytech/parity-ethereum/pull/10214))

The full list of included changes:

* revert some changes, could be buggy ([#10399](https://github.com/paritytech/parity-ethereum/pull/10399))
* 10000 > 5000 ([#10422](https://github.com/paritytech/parity-ethereum/pull/10422))
* fix panic when logging directory does not exist, closes #10420 ([#10424](https://github.com/paritytech/parity-ethereum/pull/10424))
* fix underflow in pip, closes #10419 ([#10423](https://github.com/paritytech/parity-ethereum/pull/10423))
* ci: clean up gitlab-ci.yml leftovers from previous merge ([#10429](https://github.com/paritytech/parity-ethereum/pull/10429))
* Update hardcoded headers for Foundation, Ropsten, Kovan and Classic ([#10417](https://github.com/paritytech/parity-ethereum/pull/10417))

