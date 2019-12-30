## Parity-Ethereum [v2.5.13](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.13)

Parity Ethereum v2.5.13-stable is a security release. Valid blocks with manipulated transactions (added/replaced) cause the client to stall.

The full list of included changes:
* Make sure to not mark block header hash as invalid if only the body is wrong (#11356)

## Parity-Ethereum [v2.5.12](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.12)

Parity Ethereum v2.5.12-stable is a patch release that adds Istanbul hardfork
block numbers for POA and xDai networks, implements ECIP-1056 and implements
EIP-2384/2387 - Muir Glacier.

The full list of included changes:
* Enable EIP-2384 for ice age hard fork (#11281)
* ethcore/res: activate agharta on classic 9573000 (#11331)
* Istanbul HF in xDai (2019-12-12) (#11299)
* Istanbul HF in POA Core (2019-12-19) (#11298)
* Istanbul HF in POA Sokol (2019-12-05) (#11282)
* Activate ecip-1061 on kotti and mordor (#11338)
* Enable basic verification of local transactions (#11332)
* Disallow EIP-86 style null signatures for transactions outside tests (#11335)


## Parity-Ethereum [v2.5.11](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.11)

Parity Ethereum v2.5.11-stable is an emergency patch release that adds the missing
eip1344_transition for mainnet - Users are advised to update as soon as possible
to prevent any issues with the imminent Istanbul hardfork

The full list of included changes:
- [chainspec]: add `eip1344_transition` for istanbul (#11301)

## Parity-Ethereum [v2.5.10](https://github.com/paritytech/parity-ethereum/releases/tag/2.5.10)

Parity Ethereum v2.5.10-stable is a patch release that adds block numbers for
activating the Istanbul hardfork on mainnet, as well as a large number of
various bugfixes, QoL changes, some code cleanup/refactoring and other
miscellaneous changes.

This release removes legacy aliases for the mainnet. If you specify `--chain homestead`, `--chain frontier` or `--chain byzantium`, this will need to be changed to one of: `--chain eth`, `--chain ethereum`, `--chain foundation` or `--chain mainnet`.

The full list of included changes:

* ropsten #6631425 foundation #8798209 (#11201)
* [stable] builtin, istanbul and mordor testnet backports (#11234)
  * ethcore-builtin (#10850)
  * [builtin]: support `multiple prices and activations` in chain spec (#11039)
  * [chain specs]: activate `Istanbul` on mainnet (#11228)
  * ethcore/res: add mordor testnet configuration (#11200)
* Update list of bootnodes for xDai chain (#11236)
* ethcore: remove `test-helper feat` from build (#11047)
* Secret store: fix Instant::now() related race in net_keep_alive (#11155) (#11159)
* [stable]: backport #10691 and #10683 (#11143)
  * Fix compiler warning (that will become an error) (#10683)
  * Refactor Clique stepping (#10691)
* Add Constantinople eips to the dev (instant_seal) config (#10809)
* Add cargo-remote dir to .gitignore (?)
* Insert explicit warning into the panic hook (#11225)
* Fix docker centos build (#11226)
* Update MIX bootnodes. (#11203)
* Use provided usd-per-eth value if an endpoint is specified (#11209)
* Add new line after writing block to hex file. (#10984)
* Type annotation for next_key() matching of json filter options (#11192) (but no `FilterOption` in 2.5 so…)
* Upgrade jsonrpc to latest (#11206)
* [CI] check evmbin build (#11096)
* Correct EIP-712 encoding (#11092)
* [client]: Fix for incorrectly dropped consensus messages (#11086)
* Fix block detail updating (#11015)
* Switching sccache from local to Redis (#10971)
* Made ecrecover implementation trait public (#11188)
* [dependencies]: jsonrpc `14.0.1` (#11183)
* [receipt]: add `sender` & `receiver` to `RichReceipts` (#11179)
* [ethcore/builtin]: do not panic in blake2pricer on short input (#11180)
* util Host: fix a double Read Lock bug in fn Host::session_readable() (#11175)
* ethcore client: fix a double Read Lock bug in fn Client::logs() (#11172)
* Change how RPCs eth_call and eth_estimateGas handle "Pending" (#11127)
* Cleanup stratum a bit (#11161)
* Upgrade to jsonrpc v14 (#11151)
* SecretStore: expose restore_key_public in HTTP API (#10241)

## Parity-Ethereum [v2.5.9](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.9)

Parity Ethereum v2.5.9-stable is a patch release that adds the block numbers for activating the Istanbul hardfork on test networks: Ropsten, Görli, Rinkeby and Kovan.

The full list of included changes:

* ethcore/res: activate Istanbul on Ropsten, Görli, Rinkeby, Kovan (#11068)
* [json-spec] make blake2 pricing spec more readable (#11034)

## Parity-Ethereum [v2.5.8](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.8)

Parity Ethereum v2.5.8-stable is a patch release that improves security, stability and performance.

* The most noteworthy improvement in this release is incorporating all the EIPs required for the Istanbul hard fork.
* This release also fixes certain security and performance issues, one of which was suspected to be consensus-threatening but turned out to be benign. Thanks to Martin Holst Swende and Felix Lange from the Ethereum Foundation for bringing the suspicious issue to our attention.

The full list of included changes:

* add more tx tests (#11038)
* Fix parallel transactions race-condition (#10995)
* Add blake2_f precompile (#11017)
* [trace] introduce trace failed to Ext (#11019)
* Edit publish-onchain.sh to use https (#11016)
* Fix deadlock in network-devp2p (#11013)
* EIP 1108: Reduce alt_bn128 precompile gas costs (#11008)
* xDai chain support and nodes list update (#10989)
* EIP 2028: transaction gas lowered from 68 to 16 (#10987)
* EIP-1344 Add CHAINID op-code (#10983)
* manual publish jobs for releases, no changes for nightlies (#10977)
* [blooms-db] Fix benchmarks (#10974)
* Verify transaction against its block during import (#10954)
* Better error message for rpc gas price errors (#10931)
* tx-pool: accept local tx with higher gas price when pool full (#10901)
* Fix fork choice (#10837)
* Cleanup unused vm dependencies (#10787)
* Fix compilation on recent nightlies (#10991)
* Don't build rpc with ethcore test-helpers (#11048) 
* EIP 1884 Re-pricing of trie-size dependent operations  (#10992)
* Implement EIP-1283 reenable transition, EIP-1706 and EIP-2200  (#10191)

## Parity-Ethereum [v2.5.7](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.7)

Parity Ethereum v2.5.7-stable is a bugfix release that fixes a potential DoS attack in the trace_call RPC method. This is a critical upgrade for anyone running Parity nodes with RPC exposed to the public internet (and highly recommended for anyone else). For details see this blog post.

## Parity-Ethereum [v2.5.6](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.6)

Parity-Ethereum v2.5.6-stable is a bugfix release that improves stability.

* Allow specifying hostnames for node URLs
* Fix a bug where archive nodes were losing peers

The full list of included changes:

* Kaspersky AV whitelisting (#10919)
* Avast whitelist script (#10900) 
* Docker images renaming (#10863) 
* Remove excessive warning (#10831) 
* Allow --nat extip:your.host.here.org (#10830) 
* When updating the client or when called from RPC, sleep should mean sleep (#10814)
* added new ropsten-bootnode and removed old one (#10794)
* ethkey no longer uses byteorder (#10786) 
* Do not drop the peer with None difficulty (#10772)
* docs: Update Readme with TOC, Contributor Guideline. Update Cargo package descriptions (#10652)

## Parity-Ethereum [v2.5.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.5)

Parity-Ethereum v2.5.5-stable is a minor release that improves performance and stability.
This release stabilises the 2.5 branch.

As of today, Parity-Ethereum 2.4 reaches end of life and everyone is
encouraged to upgrade.

## Parity-Ethereum [v2.5.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.5.4)

Parity Ethereum v2.5.4-beta is a security update that addresses servo/rust-smallvec#148

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

