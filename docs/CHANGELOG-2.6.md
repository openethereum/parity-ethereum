## Parity-Ethereum [v2.6.8](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.8)

Parity Ethereum v2.6.8-beta is a security release. Valid blocks with manipulated transactions (added/replaced) cause the client to stall.

The full list of included changes:
* Make sure to not mark block header hash as invalid if only the body is wrong (#11356)

## Parity-Ethereum [v2.6.7](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.7)

Parity Ethereum v2.6.7-beta is a patch release that adds Istanbul hardfork
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
* SecretStore database migration to v4 (#11322) 

## Parity-Ethereum [v2.6.6](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.6)

Parity Ethereum v2.6.6-beta is an emergency patch release that adds the missing
eip1344_transition for mainnet - Users are advised to update as soon as possible
to prevent any issues with the imminent Istanbul hardfork

The full list of included changes:
* [chainspec]: add `eip1344_transition` for istanbul (#11301)

## Parity-Ethereum [v2.6.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.5)

Parity Ethereum v2.6.5-beta is a patch release that adds block numbers for activating the Istanbul hardfork on mainnet, as well as a large number of various bugfixes, QoL changes, some code cleanup/refactoring and other miscellaneous changes.

This release removes legacy aliases for the mainnet. If you specify `--chain homestead`, `--chain frontier` or `--chain byzantium`, this will need to be changed to one of: `--chain eth`, `--chain ethereum`, `--chain foundation` or `--chain mainnet`.

This release includes important changes to how snapshots are produced. The size of the Ethereum account state means that producing a snapshot takes a long while; most nodes today are not able to finish before the relevant state is pruned. Starting with v2.6.5, pruning is paused while a snapshot is underway, hopefully fixing the current dearth of recent snapshots. The downside to this is that memory usage goes up while a snapshot is produced.

The full list of included changes:

* [CI] check evmbin build (#11096)
* Correct EIP-712 encoding (#11092)
* [client]: Fix for incorrectly dropped consensus messages (#11082) (#11086)
* Update hardcoded headers (foundation, classic, kovan, xdai, ewc, ...) (#11053)
* Add cargo-remote dir to .gitignore (?)
* Update light client headers: ropsten 6631425 foundation 8798209 (#11201)
* Update list of bootnodes for xDai chain (#11236)
* ethcore/res: add mordor testnet configuration (#11200)
* [chain specs]: activate Istanbul on mainnet (#11228)
* [builtin]: support multiple prices and activations in chain spec (#11039)
* [receipt]: add sender & receiver to RichReceipts (#11179)
* [ethcore/builtin]: do not panic in blake2pricer on short input (#11180)
* Made ecrecover implementation trait public (#11188)
* Fix docker centos build (#11226)
* Update MIX bootnodes. (#11203)
* Insert explicit warning into the panic hook (#11225)
* Use provided usd-per-eth value if an endpoint is specified (#11209)
* Cleanup stratum a bit (#11161)
* Add Constantinople EIPs to the dev (instant_seal) config (#10809) (already backported)
* util Host: fix a double Read Lock bug in fn Host::session_readable() (#11175)
* ethcore client: fix a double Read Lock bug in fn Client::logs() (#11172)
* Type annotation for next_key() matching of json filter options (#11192)
* Upgrade jsonrpc to latest (#11206)
* [dependencies]: jsonrpc 14.0.1 (#11183)
* Upgrade to jsonrpc v14 (#11151)
* Switching sccache from local to Redis (#10971)
* Snapshot restoration overhaul (#11219)
* Add new line after writing block to hex file. (#10984)
* Pause pruning while snapshotting (#11178)
* Change how RPCs eth_call and eth_estimateGas handle "Pending" (#11127)
* Fix block detail updating (#11015)
* Make InstantSeal Instant again #11186
* Filter out some bad ropsten warp snapshots (#11247)
* Allow default block parameter to be blockHash (#10932)

## Parity-Ethereum [v2.6.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.4)

Parity Ethereum v2.6.4-stable is a patch release that adds the block numbers for activating the Istanbul hardfork on test networks: Ropsten, Görli, Rinkeby and Kovan.

A full list of included changes:

* ethcore/res: activate Istanbul on Ropsten, Görli, Rinkeby, Kovan (#11068)
* cleanup json crate (#11027)
* [json-spec] make blake2 pricing spec more readable (#11034)
* Update JSON tests to d4f86ecf4aa7c (#11054)

## Parity-Ethereum [v2.6.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.3)

Parity Ethereum v2.6.3-stable is a patch release that improves security, stability and performance.

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
* Fix fork choice (#10837)
* Fix compilation on recent nightlies (#10991)
* Don't build rpc with ethcore test-helpers (#11048)
* EIP 1884 Re-pricing of trie-size dependent operations  (#10992)
* Implement EIP-1283 reenable transition, EIP-1706 and EIP-2200  (#10191)

## Parity-Ethereum [v2.6.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.2)

Parity Ethereum v2.6.2-stable is a bugfix release that fixes a potential DoS attack in the trace_call RPC method. This is a critical upgrade for anyone running Parity nodes with RPC exposed to the public internet (and highly recommended for anyone else). For details see this blog post.

## Parity-Ethereum [v2.6.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.1)

Parity-Ethereum 2.6.1-beta is a patch release that improves stability.

This release includes:
  * Allow specifying hostnames for node URLs
  * Fix a bug where archive nodes were losing peers
  * Add support for Energy Web Foundations new chains 'Volta' and 'EWC', and remove their deprecated 'Tobalaba' chain.

The full list of included changes:
  * Add support for Energy Web Foundation's new chains (#10957)
  * Kaspersky AV whitelisting (#10919)
  * Avast whitelist script (#10900)
  * Docker images renaming (#10863)
  * Remove excessive warning (#10831)
  * Allow --nat extip:your.host.here.org (#10830)
  * When updating the client or when called from RPC, sleep should mean sleep (#10814)
  * added new ropsten-bootnode and removed old one (#10794)
  * ethkey no longer uses byteorder (#10786)
  * docs: Update Readme with TOC, Contributor Guideline. Update Cargo package descriptions (#10652)

## Parity-Ethereum [v2.6.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.6.0)

Parity-Ethereum 2.6.0-beta is a minor release that stabilizes the 2.6 branch by
marking it as a beta release.

This release includes:
  * Major refactoring of the codebase
  * Many bugfixes
  * Significant improvements to logging, error and warning message clarity.
  * SecretStore: remove support of old database formats (#10757)
      * This is a potentially breaking change if you have not upgraded for
          quite some time.

 As of today, Parity-Ethereum 2.4 reaches end of life, and everyone is
 encouraged to upgrade.

The full list of included changes:
* update jsonrpc to 12.0 ([#10841](https://github.com/paritytech/parity-ethereum/pull/10841))
* Move more code into state-account ([#10840](https://github.com/paritytech/parity-ethereum/pull/10840))
* Extract AccountDB to account-db ([#10839](https://github.com/paritytech/parity-ethereum/pull/10839))
* Extricate PodAccount and state Account to own crates ([#10838](https://github.com/paritytech/parity-ethereum/pull/10838))
* Fix fork choice ([#10837](https://github.com/paritytech/parity-ethereum/pull/10837))
* tests: Relates to #10655: Test instructions for Readme ([#10835](https://github.com/paritytech/parity-ethereum/pull/10835))
* idiomatic changes to PodState ([#10834](https://github.com/paritytech/parity-ethereum/pull/10834))
* Break circular dependency between Client and Engine (part 1) ([#10833](https://github.com/paritytech/parity-ethereum/pull/10833))
* Remove excessive warning ([#10831](https://github.com/paritytech/parity-ethereum/pull/10831))
* Allow --nat extip:your.host.here.org ([#10830](https://github.com/paritytech/parity-ethereum/pull/10830))
* ethcore does not use byteorder ([#10829](https://github.com/paritytech/parity-ethereum/pull/10829))
* Fix typo in README.md ([#10828](https://github.com/paritytech/parity-ethereum/pull/10828))
* Update wordlist to v1.3 ([#10823](https://github.com/paritytech/parity-ethereum/pull/10823))
* bump `smallvec 0.6.10` to fix vulnerability ([#10822](https://github.com/paritytech/parity-ethereum/pull/10822))
* removed additional_params method ([#10818](https://github.com/paritytech/parity-ethereum/pull/10818))
* Improve logging when remote peer is unknown ([#10817](https://github.com/paritytech/parity-ethereum/pull/10817))
* replace memzero with zeroize crate ([#10816](https://github.com/paritytech/parity-ethereum/pull/10816))
* When updating the client or when called from RPC, sleep should mean sleep ([#10814](https://github.com/paritytech/parity-ethereum/pull/10814))
* Don't reimplement the logic from the Default impl ([#10813](https://github.com/paritytech/parity-ethereum/pull/10813))
* refactor: whisper: Add type aliases and update rustdocs in message.rs ([#10812](https://github.com/paritytech/parity-ethereum/pull/10812))
* test: whisper/cli `add invalid pool size test depending on processor` ([#10811](https://github.com/paritytech/parity-ethereum/pull/10811))
* Add Constantinople EIPs to the dev (instant_seal) config ([#10809](https://github.com/paritytech/parity-ethereum/pull/10809))
* fix spurious test failure ([#10808](https://github.com/paritytech/parity-ethereum/pull/10808))
* revert temp changes to .gitlab-ci.yml ([#10807](https://github.com/paritytech/parity-ethereum/pull/10807))
* removed redundant fmt::Display implementations ([#10806](https://github.com/paritytech/parity-ethereum/pull/10806))
* removed EthEngine alias ([#10805](https://github.com/paritytech/parity-ethereum/pull/10805))
* ethcore-bloom-journal updated to 2018 ([#10804](https://github.com/paritytech/parity-ethereum/pull/10804))
* Fix a few typos and unused warnings. ([#10803](https://github.com/paritytech/parity-ethereum/pull/10803))
* updated price-info to edition 2018 ([#10801](https://github.com/paritytech/parity-ethereum/pull/10801))
* updated parity-local-store to edition 2018 ([#10800](https://github.com/paritytech/parity-ethereum/pull/10800))
* updated project to ansi_term 0.11 ([#10799](https://github.com/paritytech/parity-ethereum/pull/10799))
* ethcore-light uses bincode 1.1 ([#10798](https://github.com/paritytech/parity-ethereum/pull/10798))
* ethcore-network-devp2p uses igd 0.9 ([#10797](https://github.com/paritytech/parity-ethereum/pull/10797))
* Better logging when backfilling ancient blocks fail ([#10796](https://github.com/paritytech/parity-ethereum/pull/10796))
* added new ropsten-bootnode and removed old one ([#10794](https://github.com/paritytech/parity-ethereum/pull/10794))
* Removed machine abstraction from ethcore ([#10791](https://github.com/paritytech/parity-ethereum/pull/10791))
* Removed redundant ethcore-service error type ([#10788](https://github.com/paritytech/parity-ethereum/pull/10788))
* Cleanup unused vm dependencies ([#10787](https://github.com/paritytech/parity-ethereum/pull/10787))
* ethkey no longer uses byteorder ([#10786](https://github.com/paritytech/parity-ethereum/pull/10786))
* Updated blooms-db to rust 2018 and removed redundant deps ([#10785](https://github.com/paritytech/parity-ethereum/pull/10785))
* Treat empty account the same as non-exist accounts in EIP-1052 ([#10775](https://github.com/paritytech/parity-ethereum/pull/10775))
* Do not drop the peer with None difficulty ([#10772](https://github.com/paritytech/parity-ethereum/pull/10772))
* EIP-1702: Generalized Account Versioning Scheme ([#10771](https://github.com/paritytech/parity-ethereum/pull/10771))
* Move Engine::register_client to be before other I/O handler registration ([#10767](https://github.com/paritytech/parity-ethereum/pull/10767))
* ethcore/res: activate atlantis classic hf on block 8772000 ([#10766](https://github.com/paritytech/parity-ethereum/pull/10766))
* Updated Bn128PairingImpl to use optimized batch pairing  ([#10765](https://github.com/paritytech/parity-ethereum/pull/10765))
* Remove unused code ([#10762](https://github.com/paritytech/parity-ethereum/pull/10762))
* Initialize private tx logger only if private tx functionality is enabled ([#10758](https://github.com/paritytech/parity-ethereum/pull/10758))
* SecretStore: remove support of old database formats ([#10757](https://github.com/paritytech/parity-ethereum/pull/10757))
* Enable aesni ([#10756](https://github.com/paritytech/parity-ethereum/pull/10756))
* updater: fix static id hashes initialization ([#10755](https://github.com/paritytech/parity-ethereum/pull/10755))
* Use fewer threads for snapshotting ([#10752](https://github.com/paritytech/parity-ethereum/pull/10752))
* Die error_chain, die ([#10747](https://github.com/paritytech/parity-ethereum/pull/10747))
* Fix deprectation warnings on nightly ([#10746](https://github.com/paritytech/parity-ethereum/pull/10746))
* Improve logging and cleanup in miner around block sealing ([#10745](https://github.com/paritytech/parity-ethereum/pull/10745))
* Add a way to signal shutdown to snapshotting threads ([#10744](https://github.com/paritytech/parity-ethereum/pull/10744))
* fix docker tags for publishing ([#10741](https://github.com/paritytech/parity-ethereum/pull/10741))
* refactor: Fix indentation in ethjson ([#10740](https://github.com/paritytech/parity-ethereum/pull/10740))
* Log validator set changes in EpochManager ([#10734](https://github.com/paritytech/parity-ethereum/pull/10734))
* Print warnings when using dangerous settings for ValidatorSet ([#10733](https://github.com/paritytech/parity-ethereum/pull/10733))
* ethcore: enable ECIP-1054 for classic ([#10731](https://github.com/paritytech/parity-ethereum/pull/10731))
* Stop breaking out of loop if a non-canonical hash is found ([#10729](https://github.com/paritytech/parity-ethereum/pull/10729))
* Removed secret_store folder ([#10722](https://github.com/paritytech/parity-ethereum/pull/10722))
* Revert "enable lto for release builds (#10717)" ([#10721](https://github.com/paritytech/parity-ethereum/pull/10721))
* fix: aura don't add `SystemTime::now()` ([#10720](https://github.com/paritytech/parity-ethereum/pull/10720))
* Use RUSTFLAGS to set the optimization level ([#10719](https://github.com/paritytech/parity-ethereum/pull/10719))
* enable lto for release builds ([#10717](https://github.com/paritytech/parity-ethereum/pull/10717))
* [devp2p] Update to 2018 edition ([#10716](https://github.com/paritytech/parity-ethereum/pull/10716))
* [devp2p] Don't use `rust-crypto` ([#10714](https://github.com/paritytech/parity-ethereum/pull/10714))
* [devp2p] Fix warnings and re-org imports ([#10710](https://github.com/paritytech/parity-ethereum/pull/10710))
* DevP2p: Get node IP address and udp port from Socket, if not included in PING packet ([#10705](https://github.com/paritytech/parity-ethereum/pull/10705))
* introduce MissingParent Error, fixes #10699 ([#10700](https://github.com/paritytech/parity-ethereum/pull/10700))
* Refactor Clique stepping ([#10691](https://github.com/paritytech/parity-ethereum/pull/10691))
* add_sync_notifier in EthPubSubClient holds on to a Client for too long ([#10689](https://github.com/paritytech/parity-ethereum/pull/10689))
* Fix compiler warning (that will become an error) ([#10683](https://github.com/paritytech/parity-ethereum/pull/10683))
* Don't panic if extra_data is longer than VANITY_LENGTH ([#10682](https://github.com/paritytech/parity-ethereum/pull/10682))
* Remove annoying compiler warnings ([#10679](https://github.com/paritytech/parity-ethereum/pull/10679))
* Remove support for hardware wallets ([#10678](https://github.com/paritytech/parity-ethereum/pull/10678))
* [CI] allow cargo audit to fail ([#10676](https://github.com/paritytech/parity-ethereum/pull/10676))
* new image ([#10673](https://github.com/paritytech/parity-ethereum/pull/10673))
* Upgrade ethereum types ([#10670](https://github.com/paritytech/parity-ethereum/pull/10670))
* Reset blockchain properly ([#10669](https://github.com/paritytech/parity-ethereum/pull/10669))
* fix: Move PR template into .github/ folder ([#10663](https://github.com/paritytech/parity-ethereum/pull/10663))
* docs: evmbin - Update Rust docs ([#10658](https://github.com/paritytech/parity-ethereum/pull/10658))
* refactor: Related #9459 - evmbin: replace untyped json! macro with fully typed serde serialization using Rust structs ([#10657](https://github.com/paritytech/parity-ethereum/pull/10657))
* docs: Add PR template ([#10654](https://github.com/paritytech/parity-ethereum/pull/10654))
* docs: Add ProgPoW Rust docs to ethash module ([#10653](https://github.com/paritytech/parity-ethereum/pull/10653))
* docs: Update Readme with TOC, Contributor Guideline. Update Cargo package descriptions ([#10652](https://github.com/paritytech/parity-ethereum/pull/10652))
* Upgrade to parity-crypto 0.4 ([#10650](https://github.com/paritytech/parity-ethereum/pull/10650))
* fix(compilation warnings) ([#10649](https://github.com/paritytech/parity-ethereum/pull/10649))
* [whisper] Move needed aes_gcm crypto in-crate ([#10647](https://github.com/paritytech/parity-ethereum/pull/10647))
* Update publishing ([#10644](https://github.com/paritytech/parity-ethereum/pull/10644))
* ci: publish docs debug ([#10638](https://github.com/paritytech/parity-ethereum/pull/10638))
* Fix publish docs ([#10635](https://github.com/paritytech/parity-ethereum/pull/10635))
* Fix rinkeby petersburg fork ([#10632](https://github.com/paritytech/parity-ethereum/pull/10632))
* Update kovan.json to switch Kovan validator set to POA Consensus Contracts ([#10628](https://github.com/paritytech/parity-ethereum/pull/10628))
* [ethcore] remove error_chain ([#10616](https://github.com/paritytech/parity-ethereum/pull/10616))
* Remove unused import ([#10615](https://github.com/paritytech/parity-ethereum/pull/10615))
* Adds parity_getRawBlockByNumber, parity_submitRawBlock ([#10609](https://github.com/paritytech/parity-ethereum/pull/10609))
* adds rpc error message for --no-ancient-blocks ([#10608](https://github.com/paritytech/parity-ethereum/pull/10608))
* Constantinople HF on POA Core ([#10606](https://github.com/paritytech/parity-ethereum/pull/10606))
* Clique: zero-fill extradata when the supplied value is less than 32 bytes in length ([#10605](https://github.com/paritytech/parity-ethereum/pull/10605))
* evm: add some mulmod benches ([#10600](https://github.com/paritytech/parity-ethereum/pull/10600))
* sccache logs to stdout ([#10596](https://github.com/paritytech/parity-ethereum/pull/10596))
* update bootnodes ([#10595](https://github.com/paritytech/parity-ethereum/pull/10595))
* Merge `Notifier` and `TransactionsPoolNotifier` ([#10591](https://github.com/paritytech/parity-ethereum/pull/10591))
* fix(whisper): change expiry `unix_time + ttl + work` ([#10587](https://github.com/paritytech/parity-ethereum/pull/10587))
* fix(evmbin): make benches compile again ([#10586](https://github.com/paritytech/parity-ethereum/pull/10586))
* fix issue with compilation when 'slow-blocks' feature enabled ([#10585](https://github.com/paritytech/parity-ethereum/pull/10585))
* Allow CORS requests in Secret Store API ([#10584](https://github.com/paritytech/parity-ethereum/pull/10584))
* CI improvements ([#10579](https://github.com/paritytech/parity-ethereum/pull/10579))
* ethcore: improve timestamp handling ([#10574](https://github.com/paritytech/parity-ethereum/pull/10574))
* Update Issue Template to direct security issue to email ([#10562](https://github.com/paritytech/parity-ethereum/pull/10562))
* version: bump master to 2.6 ([#10560](https://github.com/paritytech/parity-ethereum/pull/10560))
* fix(light cull): poll light cull instead of timer ([#10559](https://github.com/paritytech/parity-ethereum/pull/10559))
* Watch transactions pool ([#10558](https://github.com/paritytech/parity-ethereum/pull/10558))
* Add SealingState; don't prepare block when not ready. ([#10529](https://github.com/paritytech/parity-ethereum/pull/10529))
* Explicitly enable or disable Stratum in config file (Issue 9785) ([#10521](https://github.com/paritytech/parity-ethereum/pull/10521))
* Add filtering capability to `parity_pendingTransactions` (issue 8269) ([#10506](https://github.com/paritytech/parity-ethereum/pull/10506))
* Remove calls to heapsize ([#10432](https://github.com/paritytech/parity-ethereum/pull/10432))
* RPC: Implements eth_subscribe("syncing") ([#10311](https://github.com/paritytech/parity-ethereum/pull/10311))
* SecretStore: non-blocking wait of session completion ([#10303](https://github.com/paritytech/parity-ethereum/pull/10303))
* Node table limiting and cache for node filter ([#10288](https://github.com/paritytech/parity-ethereum/pull/10288))
* SecretStore: expose restore_key_public in HTTP API ([#10241](https://github.com/paritytech/parity-ethereum/pull/10241))
* Trivial journal for private transactions ([#10056](https://github.com/paritytech/parity-ethereum/pull/10056))

## Previous releases

- [CHANGELOG-2.5](docs/CHANGELOG-2.5.md) (_stable_)
- [CHANGELOG-2.4](docs/CHANGELOG-2.4.md) (EOL: 2019-07-08)
- [CHANGELOG-2.3](docs/CHANGELOG-2.3.md) (EOL: 2019-04-09)
- [CHANGELOG-2.2](docs/CHANGELOG-2.2.md) (EOL: 2019-02-25)
- [CHANGELOG-2.1](docs/CHANGELOG-2.1.md) (EOL: 2019-01-16)
- [CHANGELOG-2.0](docs/CHANGELOG-2.0.md) (EOL: 2018-11-15)
- [CHANGELOG-1.11](docs/CHANGELOG-1.11.md) (EOL: 2018-09-19)
- [CHANGELOG-1.10](docs/CHANGELOG-1.10.md) (EOL: 2018-07-18)
- [CHANGELOG-1.9](docs/CHANGELOG-1.9.md) (EOL: 2018-05-09)
- [CHANGELOG-1.8](docs/CHANGELOG-1.8.md) (EOL: 2018-03-22)
- [CHANGELOG-1.7](docs/CHANGELOG-1.7.md) (EOL: 2018-01-25)
- [CHANGELOG-1.6](docs/CHANGELOG-1.6.md) (EOL: 2017-10-15)
- [CHANGELOG-1.5](docs/CHANGELOG-1.5.md) (EOL: 2017-07-28)
- [CHANGELOG-1.4](docs/CHANGELOG-1.4.md) (EOL: 2017-03-13)
- [CHANGELOG-1.3](docs/CHANGELOG-1.3.md) (EOL: 2017-01-19)
- [CHANGELOG-1.2](docs/CHANGELOG-1.2.md) (EOL: 2016-11-07)
- [CHANGELOG-1.1](docs/CHANGELOG-1.1.md) (EOL: 2016-08-12)
- [CHANGELOG-1.0](docs/CHANGELOG-1.0.md) (EOL: 2016-06-24)
- [CHANGELOG-0.9](docs/CHANGELOG-0.9.md) (EOL: 2016-05-02)
