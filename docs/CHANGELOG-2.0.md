## Parity-Ethereum [v2.0.6](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.6) (2018-09-20)

Parity-Ethereum 2.0.6-stable is a release that does not improve performance and stability; no changes were made.

The full list of included changes:

- Backports to 2.0.6 stable ([#9600](https://github.com/paritytech/parity-ethereum/pull/9600))
- Ci: disable build cache for json-rpc-docs ([#9587](https://github.com/paritytech/parity-ethereum/pull/9587))

## Parity-Ethereum [v2.0.5](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.5) (2018-09-18)

Parity-Ethereum 2.0.5-stable is a bug-fix release to improve performance and stability.

Please, note:

- This release marks the 2.0 track of Parity-Ethereum as stable.
- This release contains a low-severity issue with the web-sockets ports. [#9545](https://github.com/paritytech/parity-ethereum/pull/9545)
- This release resolves a potential network fragmentation issue. [#9526](https://github.com/paritytech/parity-ethereum/pull/9526)
- The default `gas_floor_target` was increased to `8_000_000`, the default `gas_cap` to `10_000_000`.
- With this release, all versions of Parity Ethereum 1.x prior to 2.0 reached end of life.
- Users are urged to upgrade to 2.0.5-stable or 2.1.0-beta.

The full list of included changes:

- Backports for 2.0.5 stable ([#9519](https://github.com/paritytech/parity-ethereum/pull/9519))
  - Parity-version: mark 2.0.5 track stable
  - Deps: bump fs-swap to 0.2.4
  - Remove initial token for WS. ([#9545](https://github.com/paritytech/parity-ethereum/pull/9545))
  - Version: mark release critical
  - Increase Gas-floor-target and Gas Cap ([#9564](https://github.com/paritytech/parity-ethereum/pull/9564))
    - Gas-floor-target increased to 8M by default
    - Gas-cap increased to 10M by default
  - Improve P2P discovery ([#9526](https://github.com/paritytech/parity-ethereum/pull/9526))
    - Add `target` to Rust traces
    - Network-devp2p: Don't remove discovery peer in main sync
    - Network-p2p: Refresh discovery more often
    - Update Peer discovery protocol
    - Run discovery more often when not enough nodes connected
    - Start the first discovery early
    - Update fast discovery rate
    - Fix tests
    - Fix `ping` tests
    - Fixing remote Node address ; adding PingPong round
    - Fix tests: update new +1 PingPong round
    - Increase slow Discovery rate
      - Check in flight FindNode before pings
    - Add `deprecated` to deprecated_echo_hash
    - Refactor `discovery_round` branching
  - Net_version caches network_id to avoid redundant acquire of sync read lock ([#9544](https://github.com/paritytech/parity-ethereum/pull/9544))
    - Net_version caches network_id to avoid redundant acquire of sync read lock, [#8746](https://github.com/paritytech/parity-ethereum/issues/8746)
    - Use lower_hex display formatting for `net_peerCount` RPC method
- Update snapcraft.yaml ([#9530](https://github.com/paritytech/parity-ethereum/pull/9530))
  - Fix DEPRECATED `prepare`
  - Fix TODO https://bugs.launchpad.net/snapcraft/+bug/1778530

## Parity-Ethereum [v2.0.4](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.4) (2018-09-11)

Parity-Ethereum 2.0.4-beta is a bug-fix release to improve performance and stability:

- `eth_coinbase` now provides an actual account for light clients
- don't report skipped primaries when empty steps are enabled in proof-of-authority networks
- fix snapshot restoration failure on windows
- check warp sync status for `eth_getWorks`

The full list of included changes:

- Beta backports to 2.0.4 ([#9452](https://github.com/paritytech/parity-ethereum/pull/9452))
  - Parity-version: bump beta to 2.0.4
  - [Light/jsonrpc] Provide the actual account for `eth_coinbase` RPC and unify error handeling for light and full client ([#9383](https://github.com/paritytech/parity-ethereum/pull/9383))
    - Provide the actual `account` for eth_coinbase
    - The previous implementation always provided the `zero address` on `eth_coinbase` RPC. Now, instead the actual address is returned on success or an error when no account(s) is found!
    - Full client `eth_coinbase` return err
    - In the full-client return an error when no account is found instead of returning the `zero address`
    - Remove needless blocks on single import
    - Remove needless `static` lifetime on const
    - Fix `rpc_eth_author` test
  - Parity: print correct keys path on startup ([#9501](https://github.com/paritytech/parity-ethereum/pull/9501))
  - Aura: don't report skipped primaries when empty steps are enabled ([#9435](https://github.com/paritytech/parity-ethereum/pull/9435))
  - Only check warp syncing for eth_getWorks ([#9484](https://github.com/paritytech/parity-ethereum/pull/9484))
    - Only check warp syncing for eth_getWorks
    - Use SyncStatus::is_snapshot_syncing
  - Fix Snapshot restoration failure on Windows ([#9491](https://github.com/paritytech/parity-ethereum/pull/9491))
    - Close Blooms DB files before DB restoration
    - Address Grumbles

## Parity-Ethereum [v2.0.3](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.3) (2018-09-01)

Parity-Ethereum 2.0.3-beta is a bug-fix release to improve performance and stability. Hopefully. ;)

The full list of included changes:

- Beta backports for 2.0.3 ([#9229](https://github.com/paritytech/parity-ethereum/pull/9229))
  - parity-version: bump beta to 2.0.2
  - remove ssl from dockerfiles, closes [#8880](https://github.com/paritytech/parity-ethereum/issues/8880) ([#9195](https://github.com/paritytech/parity-ethereum/pull/9195))
  - snap: remove ssl dependencies from snapcraft definition ([#9222](https://github.com/paritytech/parity-ethereum/pull/9222))
  - parity-version: bump beta to 2.0.3
  - Remove all dapp permissions related settings ([#9120](https://github.com/paritytech/parity-ethereum/pull/9120))
    - Completely remove all dapps struct from rpc
    - Remove unused pub use
    - Remove dapp policy/permission func in ethcore
    - Remove all dapps settings from rpc
    - Fix rpc tests
    - Use both origin and user_agent
    - Address grumbles
    - Address grumbles
    - Fix tests
  - Check if synced when using eth_getWork ([#9193](https://github.com/paritytech/parity-ethereum/issues/9193)) ([#9210](https://github.com/paritytech/parity-ethereum/pull/9210))
    - Check if synced when using eth_getWork ([#9193](https://github.com/paritytech/parity-ethereum/issues/9193))
    - Don't use fn syncing
    - Fix identation
    - Fix typo
    - Don't check for warping
    - rpc: avoid calling queue_info twice on eth_getWork
    - Fix potential as_usize overflow when casting from U256 in miner ([#9221](https://github.com/paritytech/parity-ethereum/pull/9221))
    - Allow old blocks from peers with lower difficulty ([#9226](https://github.com/paritytech/parity-ethereum/pull/9226))
    - Previously we only allow downloading of old blocks if the peer difficulty was greater than our syncing difficulty. This change allows downloading of blocks from peers where the difficulty is greater then the last downloaded old block.
  - Update Dockerfile ([#9242](https://github.com/paritytech/parity-ethereum/pull/9242))
    - Update Dockerfile
    - fix Docker build
  - fix dockerfile paths: parity -> parity-ethereum ([#9248](https://github.com/paritytech/parity-ethereum/pull/9248))
  - Propagate transactions for next 4 blocks. ([#9265](https://github.com/paritytech/parity-ethereum/pull/9265))
    - Closes [#9255](https://github.com/paritytech/parity-ethereum/issues/9255)
    - This PR also removes the limit of max 64 transactions per packet, currently we only attempt to prevent the packet size to go over 8MB. This will only be the case for super-large transactions or high-block-gas-limit chains.
    - Patching this is important only for chains that have blocks that can fit more than 4k transactions (over 86M block gas limit)
    - For mainnet, we should actually see a tiny bit faster propagation since instead of computing 4k pending set, we only need `4 * 8M / 21k = 1523` transactions.
  - Update tobalaba.json ([#9313](https://github.com/paritytech/parity-ethereum/pull/9313))
  - Fix load share ([#9321](https://github.com/paritytech/parity-ethereum/pull/9321))
    - fix(light_sync): calculate `load_share` properly
    - refactor(api.rs): extract `light_params` fn, add test
    - style(api.rs): add trailing commas
  - ethcore: fix pow difficulty validation ([#9328](https://github.com/paritytech/parity-ethereum/pull/9328))
    - ethcore: fix pow difficulty validation
    - ethcore: validate difficulty is not zero
    - ethcore: add issue link to regression test
    - ethcore: fix tests
    - ethcore: move difficulty_to_boundary to ethash crate
    - ethcore: reuse difficulty_to_boundary and boundary_to_difficulty
    - ethcore: fix grumbles in difficulty_to_boundary_aux
  - Light client `Provide default nonce in transactions when it´s missing` ([#9370](https://github.com/paritytech/parity-ethereum/pull/9370))
    - Provide `default_nonce` in tx's when it's missing
    - When `nonce` is missing in a `EthTransaction` will cause it to fall in these cases provide `default_nonce` value instead!
    - Changed http:// to https:// on Yasm link ([#9369](https://github.com/paritytech/parity-ethereum/pull/9369))
    - Changed http:// to https:// on Yasm link in README.md
    - Address grumbles
  - ethcore: kovan: delay activation of strict score validation ([#9406](https://github.com/paritytech/parity-ethereum/pull/9406))
  - Better support for eth_getLogs in light mode ([#9186](https://github.com/paritytech/parity-ethereum/pull/9186))
    - Light client on-demand request for headers range.
    - Cache headers in HeaderWithAncestors response.
    - Also fulfills request locally if all headers are in cache.
    - LightFetch::logs fetches missing headers on demand.
    - LightFetch::logs limit the number of headers requested at a time.
    - LightFetch::logs refactor header fetching logic.
    - Enforce limit on header range length in light client logs request.
    - Fix light request tests after struct change.
    - Respond to review comments.
  - Add update docs script to CI ([#9219](https://github.com/paritytech/parity-ethereum/pull/9219))
    - Add update docs script to CI
    - Added a script to CI that will use the jsonrpc tool to update rpc documentation then commit and push those to the wiki repo.
    - fix gitlab ci lint
    - Only apply jsonrpc docs update on tags
    - Update gitlab-rpc-docs.sh
    - Copy correct parity repo to jsonrpc folder
    - Copy correct parity repo to jsonrpc folder before attempting to build docs since the CI runner clones the repo as parity and not parity-ethereum.
    - Fix JSONRPC docs CI job
    - Update remote config in wiki repo before pushing changes using a github token for authentication. Add message to wiki tag when pushing changes. Use project directory to correctly copy parity code base into the jsonrpc repo for doc generation.
    - Fix set_remote_wiki function call in CI
  - Prevent blockchain & miner racing when accessing pending block. ([#9310](https://github.com/paritytech/parity-ethereum/pull/9310))
    - Prevent blockchain & miner racing when accessing pending block.
    - Fix unavailability of pending block during reseal.
  - Prevent sync restart if import queue full ([#9381](https://github.com/paritytech/parity-ethereum/pull/9381))
  - Add POA Networks: Core and Sokol ([#9413](https://github.com/paritytech/parity-ethereum/pull/9413))
    - ethcore: add poa network and sokol chainspecs
    - rpc: simplify chain spec docs
    - cli: rearrange networks by main/test and size/range
    - parity: don't blacklist 0x00a328 on sokol testnet
    - parity: add sokol and poanet to params and clean up a bit, add tests
    - ethcore: add the poa networks and clean up a bit
    - ethcore: fix path to poacore chain spec
    - parity: rename poa networks to poacore and poasokol
    - parity: fix configuration tests
    - parity: fix parameter tests
    - ethcore: rename POA Core and POA Sokol
  - Update tobalaba.json ([#9419](https://github.com/paritytech/parity-ethereum/pull/9419))
  - Update hardcoded sync ([#9421](https://github.com/paritytech/parity-ethereum/pull/9421))
    - Update foundation hardcoded header to block 6219777
    - Update ropsten hardcoded header to block 3917825
    - Update kovan hardcoded header to block 8511489

## Parity-Ethereum [v2.0.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.1) (2018-07-27)

Parity-Ethereum 2.0.1-beta is a bug-fix release to improve performance and stability.

Note, authorities in PoA networks based on the Aura engine, should upgrade their nodes to 1.11.8-stable or 2.0.1-beta as this release includes a critical fix.

The full list of included changes:

- Backports to 2.0.1-beta ([#9145](https://github.com/paritytech/parity-ethereum/pull/9145))
  - Parity-version: bump beta to 2.0.1
  - Ci: update version strings for snaps ([#9160](https://github.com/paritytech/parity-ethereum/pull/9160))
  - Be more graceful on Aura difficulty validation ([#9164](https://github.com/paritytech/parity-ethereum/pull/9164))
    - Be more graceful on Aura difficulty validation
    - Test: rejects_step_backwards
    - Test: proposer_switching
    - Test: rejects_future_block
    - Test: reports_skipped
    - Test: verify_empty_seal_steps
  - Remove node-health ([#9119](https://github.com/paritytech/parity-ethereum/pull/9119))
    - Remove node-health
    - Remove ntp_servers
    - Add --ntp-servers as legacy instead of removing it
    - Add --ntp-servers to deprecated args
    - Remove unused stuff
    - Remove _legacy_ntp_servers
  - Parity: fix UserDefaults json parser ([#9189](https://github.com/paritytech/parity-ethereum/pull/9189))
    - Parity: fix UserDefaults json parser
    - Parity: use serde_derive for UserDefaults
    - Parity: support deserialization of old UserDefault json format
    - Parity: make UserDefaults serde backwards compatible
    - Parity: tabify indentation in UserDefaults
  - Fix bugfix hard fork logic ([#9138](https://github.com/paritytech/parity-ethereum/pull/9138))
    - Fix bugfix hard fork logic
    - Remove dustProtectionTransition from bugfix category
      - Eip-168 is not enabled by default
    - Remove unnecessary 'static
  - Disable per-sender limit for local transactions. ([#9148](https://github.com/paritytech/parity-ethereum/pull/9148))
    - Disable per-sender limit for local transactions.
    - Add a missing new line.
  - Rpc: fix is_major_importing sync state condition ([#9112](https://github.com/paritytech/parity-ethereum/pull/9112))
    - Rpc: fix is_major_importing sync state condition
    - Rpc: fix informant printout when waiting for peers
  - Fix verification in ethcore-sync collect_blocks ([#9135](https://github.com/paritytech/parity-ethereum/pull/9135))
  - Docker: update hub dockerfile ([#9173](https://github.com/paritytech/parity-ethereum/pull/9173))
    - Update Dockerfile for hub
      - Update to Ubuntu Xenial 16.04
      - Fix cmake version
    - Docker: fix tab indentation in hub dockerfile
  - Rpc: fix broken merge
  - Rpc: remove node_health leftover from merge
  - Rpc: remove dapps leftover from merge

## Parity-Ethereum [v2.0.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.0.0) "Ethereum" (2018-07-18)

This is the Parity-Ethereum//v2.0.0-beta release, code-named "Ethereum", **YOLO!**

Please note, Parity-Ethereum//v2.0.0 comes with some breaking changes that might be interrupting your usual workflows. Please mind them before upgrading:

- The Parity client is now called _Parity-Ethereum_ to distinguish it from other software we provide, such as [_Parity-Bitcoin_](https://github.com/paritytech/parity-bitcoin/) and [_Parity-Polkadot_](https://github.com/paritytech/polkadot) ([#9052](https://github.com/paritytech/parity-ethereum/pull/9052)).
- The public node and the user interface (a.k.a. _"Parity Wallet"_) are completely removed from the Parity-Ethereum//v2.0.0 client ([#8758](https://github.com/paritytech/parity-ethereum/pull/8758), [#8783](https://github.com/paritytech/parity-ethereum/pull/8783), [#8641](https://github.com/paritytech/parity-ethereum/pull/8641)). Users interested running a Parity Wallet, check out [the stand-alone UI application](https://github.com/Parity-JS/shell/releases).
- The DApps subsystem was completely removed from the client ([#9017](https://github.com/paritytech/parity-ethereum/pull/9017), [#9107](https://github.com/paritytech/parity-ethereum/pull/9107)). Again, use the standalone wallet if you wish to continue working with them.
- Windows and MacOS versions are not available as installer anymore and the system trays were removed ([#8778](https://github.com/paritytech/parity-ethereum/pull/8778)). If you desire to run Parity-Ethereum on Windows or MacOS, you still can get the binaries from our mirrors. Furthermore, MacOS users are encouraged [to use our homebrew tap](https://github.com/paritytech/homebrew-paritytech/).
- Linux versions are not available as deb-/rpm-packages anymore ([#8887](https://github.com/paritytech/parity-ethereum/pull/8887)). Communities are encouraged to provide their own packages or maintain their own repositories, such as [Arch Linux does](https://www.archlinux.org/packages/community/x86_64/parity/) for instance.
- MD5-checksums are completely replaced by SHA256-checksums ([#8884](https://github.com/paritytech/parity-ethereum/pull/8884)). This is also reflected on our homepage by now.
- Deprecated, removed, or replaced CLI-options are hidden from client `--help` to further discourage their usage ([#8967](https://github.com/paritytech/parity-ethereum/pull/8967)).

Additional noteworthy changes to the client:

- Tracing of precompiled contracts when the transfer value is not zero ([#8486](https://github.com/paritytech/parity-ethereum/pull/8486))
- _Parity-Ethereum_ as a library now provides APIs for running full and light nodes and a C interface ([#8412](https://github.com/paritytech/parity-ethereum/pull/8412)). Shared crates are now available in [_Parity-Common_](https://github.com/paritytech/parity-common) ([#9083](https://github.com/paritytech/parity-ethereum/pull/9083)).
- The Morden database and keys are now moved to a `./Morden` subdirectory instead of `./test` which is by default used by Ropsten ([#8621](https://github.com/paritytech/parity-ethereum/pull/8621)).
- Adding support for having an on-chain contract calculating the block rewards ([#8419](https://github.com/paritytech/parity-ethereum/pull/8419)).
- Enforcing warp-only synchronization with `--warp-barrier [blocknumber]` flag ([#8228](https://github.com/paritytech/parity-ethereum/pull/8228)).
- Adding a fork-choice and meta-data framework suitable for implementing Casper ([#8401](https://github.com/paritytech/parity-ethereum/pull/8401)).
- Returning an error if RLP-size of a transaction exceeds a 300kB limit ([#8473](https://github.com/paritytech/parity-ethereum/pull/8473)).
- Warp-sync is now resumable by keeping the downloaded chunks between client restarts. Also, it seeds downloaded snapshots for other nodes ([#8544](https://github.com/paritytech/parity-ethereum/pull/8544)).
- The developer chain `--chain dev` now contains Byzantium features, this breaks existing developer chains ([#8717](https://github.com/paritytech/parity-ethereum/pull/8717)).
- The EIP150, EIP160 and EIP161 forks are now to be specified in common params section of a chain-spec file instead of the Ethash params to enable these features on non-proof-of-work chains ([#8614](https://github.com/paritytech/parity-ethereum/pull/8614)). Please update your chain specs.
- Allowing to disable local-by-default for transactions with new configurations ([#8882](https://github.com/paritytech/parity-ethereum/pull/8882)).
- Never drop local transactions from different senders ([#9002](https://github.com/paritytech/parity-ethereum/pull/9002)).
- Optimize pending transactions filter and fix ethstats reporting of pending transactions ([#9026](https://github.com/paritytech/parity-ethereum/pull/9026)).
- Add separate database directory for light client allowing to run full and light nodes at the same time ([#9064](https://github.com/paritytech/parity-ethereum/pull/9064)).

If you are upgrading directly from versions 1.10.9 or earlier, please note important changes to our transaction-queue implementation, namely:

- The pool now limits transactions per-sender (see `--tx-queue-per-sender`), local transactions also have to obey that limit. Consider increasing the limit via CLI-flag when running benchmarks or sending a lot of transactions at once.
- In case the pool is full, transactions received over the network, but originating from accounts that you have private keys for might not get accepted to the pool any more with higher priority. Consider running with larger pool size or submitting the transactions directly on the node via `eth_sendRawTransaction`.

The full list of included changes:

- Backports to 2.0.0-beta ([#9094](https://github.com/paritytech/parity-ethereum/pull/9094))
  - Parity-version: betalize 2.0
  - Multiple improvements to discovery ping handling ([#8771](https://github.com/paritytech/parity-ethereum/pull/8771))
    - Discovery: Only add nodes to routing table after receiving pong.
    - Discovery: Refactor packet creation into its own function.
    - Discovery: Additional testing for new add_node behavior.
    - Discovery: Track expiration of pings to non-yet-in-bucket nodes.
    - Discovery: Verify echo hash on pong packets.
    - Discovery: Track timeouts on FIND_NODE requests.
    - Discovery: Retry failed pings with exponential backoff.
    - !fixup Use slice instead of Vec for request_backoff.
  - Add separate database directory for light client ([#9064](https://github.com/paritytech/parity-ethereum/pull/9064))
    - Add separate default DB path for light client ([#8927](https://github.com/paritytech/parity-ethereum/pull/8927))
    - Improve readability
  - Revert "Replace `std::env::home_dir` with `dirs::home_dir` ([#9077](https://github.com/paritytech/parity-ethereum/pull/9077))" ([#9097](https://github.com/paritytech/parity-ethereum/pull/9097))
    - Revert "Replace `std::env::home_dir` with `dirs::home_dir` ([#9077](https://github.com/paritytech/parity-ethereum/pull/9077))"
      - This reverts commit 7e77932.
    - Restore some of the changes
    - Update parity-common
  - Offload cull to IoWorker. ([#9099](https://github.com/paritytech/parity-ethereum/pull/9099))
  - Fix work-notify. ([#9104](https://github.com/paritytech/parity-ethereum/pull/9104))
  - Update hidapi, fixes [#7542](https://github.com/paritytech/parity-ethereum/issues/7542) ([#9108](https://github.com/paritytech/parity-ethereum/pull/9108))
  - Docker: add cmake dependency ([#9111](https://github.com/paritytech/parity-ethereum/pull/9111))
  - Update light client hardcoded headers ([#9098](https://github.com/paritytech/parity-ethereum/pull/9098))
    - Insert Kovan hardcoded headers until 7690241
    - Insert Kovan hardcoded headers until block 7690241
    - Insert Ropsten hardcoded headers until 3612673
    - Insert Mainnet hardcoded headers until block 5941249
  - Make sure to produce full blocks. ([#9115](https://github.com/paritytech/parity-ethereum/pull/9115))
  - Insert ETC (classic) hardcoded headers until block 6170625 ([#9121](https://github.com/paritytech/parity-ethereum/pull/9121))
  - Fix verification in ethcore-sync collect_blocks ([#9135](https://github.com/paritytech/parity-ethereum/pull/9135))
  - Completely remove all dapps struct from rpc ([#9107](https://github.com/paritytech/parity-ethereum/pull/9107))
    - Completely remove all dapps struct from rpc
    - Remove unused pub use
  - `evm bench` fix broken dependencies ([#9134](https://github.com/paritytech/parity-ethereum/pull/9134))
    - `evm bench` use valid dependencies
    - Benchmarks of the `evm` used stale versions of a couple a crates that this commit fixes!
    - Fix warnings
  - Update snapcraft.yaml ([#9132](https://github.com/paritytech/parity-ethereum/pull/9132))
- Parity Ethereum 2.0.0 ([#9052](https://github.com/paritytech/parity-ethereum/pull/9052))
- Don't fetch snapshot chunks at random ([#9088](https://github.com/paritytech/parity-ethereum/pull/9088))
- Remove the dapps system ([#9017](https://github.com/paritytech/parity-ethereum/pull/9017))
- Fix nightly warnings ([#9080](https://github.com/paritytech/parity-ethereum/pull/9080))
- Db: remove wal disabling / fast-and-loose option. ([#8963](https://github.com/paritytech/parity-ethereum/pull/8963))
- Transactions hashes missing in trace_replayBlockTransactions method result [#8725](https://github.com/paritytech/parity-ethereum/issues/8725) ([#8883](https://github.com/paritytech/parity-ethereum/pull/8883))
- Delete crates from parity-ethereum and fetch them from parity-common instead ([#9083](https://github.com/paritytech/parity-ethereum/pull/9083))
- Updater verification ([#8787](https://github.com/paritytech/parity-ethereum/pull/8787))
- Phrasing, precisions and typos in CLI help ([#9060](https://github.com/paritytech/parity-ethereum/pull/9060))
- Some work towards iOS build ([#9045](https://github.com/paritytech/parity-ethereum/pull/9045))
- Clean up deprecated options and add CHECK macro ([#9036](https://github.com/paritytech/parity-ethereum/pull/9036))
- Replace `std::env::home_dir` with `dirs::home_dir` ([#9077](https://github.com/paritytech/parity-ethereum/pull/9077))
- Fix warning in secret-store test ([#9074](https://github.com/paritytech/parity-ethereum/pull/9074))
- Seedhashcompute remove needless `new` impl ([#9063](https://github.com/paritytech/parity-ethereum/pull/9063))
- Remove trait bounds from several structs ([#9055](https://github.com/paritytech/parity-ethereum/pull/9055))
- Docs: add changelog for 1.10.9 stable and 1.11.6 beta ([#9069](https://github.com/paritytech/parity-ethereum/pull/9069))
- Enable test in `miner/pool/test` ([#9072](https://github.com/paritytech/parity-ethereum/pull/9072))
- Fetch: replace futures-timer with tokio-timer ([#9066](https://github.com/paritytech/parity-ethereum/pull/9066))
- Remove util-error ([#9054](https://github.com/paritytech/parity-ethereum/pull/9054))
- Fixes for misbehavior reporting in AuthorityRound ([#8998](https://github.com/paritytech/parity-ethereum/pull/8998))
- A last bunch of txqueue performance optimizations ([#9024](https://github.com/paritytech/parity-ethereum/pull/9024))
- Reduce number of constraints for triedb types ([#9043](https://github.com/paritytech/parity-ethereum/pull/9043))
- Bump fs-swap to 0.2.3 so it is compatible with osx 10.11 again ([#9050](https://github.com/paritytech/parity-ethereum/pull/9050))
- Recursive test ([#9042](https://github.com/paritytech/parity-ethereum/pull/9042))
- Introduce more optional features in ethcore ([#9020](https://github.com/paritytech/parity-ethereum/pull/9020))
- Update ETSC bootnodes ([#9038](https://github.com/paritytech/parity-ethereum/pull/9038))
- Optimize pending transactions filter ([#9026](https://github.com/paritytech/parity-ethereum/pull/9026))
- Eip160/eip161 spec: u64 -> BlockNumber ([#9044](https://github.com/paritytech/parity-ethereum/pull/9044))
- Move the C/C++ example to another directory ([#9032](https://github.com/paritytech/parity-ethereum/pull/9032))
- Bump parking_lot to 0.6 ([#9013](https://github.com/paritytech/parity-ethereum/pull/9013))
- Never drop local transactions from different senders. ([#9002](https://github.com/paritytech/parity-ethereum/pull/9002))
- Precise HTTP or WebSockets for JSON-RPC options ([#9027](https://github.com/paritytech/parity-ethereum/pull/9027))
- Recently rejected cache for transaction queue ([#9005](https://github.com/paritytech/parity-ethereum/pull/9005))
- Make HashDB generic ([#8739](https://github.com/paritytech/parity-ethereum/pull/8739))
- Only return error log for rustls ([#9025](https://github.com/paritytech/parity-ethereum/pull/9025))
- Update Changelogs for 1.10.8 and 1.11.5 ([#9012](https://github.com/paritytech/parity-ethereum/pull/9012))
- Attempt to graceful shutdown in case of panics ([#8999](https://github.com/paritytech/parity-ethereum/pull/8999))
- Simplify kvdb error types ([#8924](https://github.com/paritytech/parity-ethereum/pull/8924))
- Add option for user to set max size limit for RPC requests ([#9010](https://github.com/paritytech/parity-ethereum/pull/9010))
- Bump ntp to 0.5.0 ([#9009](https://github.com/paritytech/parity-ethereum/pull/9009))
- Removed duplicate dependency ([#9021](https://github.com/paritytech/parity-ethereum/pull/9021))
- Minimal effective gas price in the queue ([#8934](https://github.com/paritytech/parity-ethereum/pull/8934))
- Parity: fix db path when migrating to blooms db ([#8975](https://github.com/paritytech/parity-ethereum/pull/8975))
- Preserve the current abort behavior ([#8995](https://github.com/paritytech/parity-ethereum/pull/8995))
- Improve should_replace on NonceAndGasPrice ([#8980](https://github.com/paritytech/parity-ethereum/pull/8980))
- Tentative fix for missing dependency error ([#8973](https://github.com/paritytech/parity-ethereum/pull/8973))
- Refactor evm Instruction to be a c-like enum ([#8914](https://github.com/paritytech/parity-ethereum/pull/8914))
- Fix deadlock in blockchain. ([#8977](https://github.com/paritytech/parity-ethereum/pull/8977))
- Snap: downgrade rust to revision 1.26.2, ref snapcraft/+bug/1778530 ([#8984](https://github.com/paritytech/parity-ethereum/pull/8984))
- Use local parity-dapps-glue instead of crate published at crates.io ([#8983](https://github.com/paritytech/parity-ethereum/pull/8983))
- Parity: omit redundant last imported block number in light sync informant ([#8962](https://github.com/paritytech/parity-ethereum/pull/8962))
- Disable hardware-wallets on platforms that don't support `libusb` ([#8464](https://github.com/paritytech/parity-ethereum/pull/8464))
- Bump error-chain and quick_error versions ([#8972](https://github.com/paritytech/parity-ethereum/pull/8972))
- Evm benchmark utilities ([#8944](https://github.com/paritytech/parity-ethereum/pull/8944))
- Parity: hide legacy options from cli --help ([#8967](https://github.com/paritytech/parity-ethereum/pull/8967))
- Scripts: fix docker build tag on latest using master ([#8952](https://github.com/paritytech/parity-ethereum/pull/8952))
- Add type for passwords. ([#8920](https://github.com/paritytech/parity-ethereum/pull/8920))
- Deps: bump fs-swap ([#8953](https://github.com/paritytech/parity-ethereum/pull/8953))
- Eliminate some more `transmute()` ([#8879](https://github.com/paritytech/parity-ethereum/pull/8879))
- Restrict vault.json permssion to owner and using random suffix for temp vault.json file ([#8932](https://github.com/paritytech/parity-ethereum/pull/8932))
- Print SS.self_public when starting SS node ([#8949](https://github.com/paritytech/parity-ethereum/pull/8949))
- Scripts: minor improvements ([#8930](https://github.com/paritytech/parity-ethereum/pull/8930))
- Rpc: cap gas limit of local calls ([#8943](https://github.com/paritytech/parity-ethereum/pull/8943))
- Docs: update changelogs ([#8931](https://github.com/paritytech/parity-ethereum/pull/8931))
- Ethcore: fix compilation when using slow-blocks or evm-debug features ([#8936](https://github.com/paritytech/parity-ethereum/pull/8936))
- Fixed blooms dir creation ([#8941](https://github.com/paritytech/parity-ethereum/pull/8941))
- Update hardcoded headers ([#8925](https://github.com/paritytech/parity-ethereum/pull/8925))
- New blooms database ([#8712](https://github.com/paritytech/parity-ethereum/pull/8712))
- Ethstore: retry deduplication of wallet file names until success ([#8910](https://github.com/paritytech/parity-ethereum/pull/8910))
- Update ropsten.json ([#8926](https://github.com/paritytech/parity-ethereum/pull/8926))
- Include node identity in the P2P advertised client version. ([#8830](https://github.com/paritytech/parity-ethereum/pull/8830))
- Allow disabling local-by-default for transactions with new config entry ([#8882](https://github.com/paritytech/parity-ethereum/pull/8882))
- Allow Poll Lifetime to be configured via CLI ([#8885](https://github.com/paritytech/parity-ethereum/pull/8885))
- Cleanup nibbleslice ([#8915](https://github.com/paritytech/parity-ethereum/pull/8915))
- Hardware-wallets `Clean up things I missed in the latest PR` ([#8890](https://github.com/paritytech/parity-ethereum/pull/8890))
- Remove debian/.deb and centos/.rpm packaging scripts ([#8887](https://github.com/paritytech/parity-ethereum/pull/8887))
- Remove a weird emoji in new_social docs ([#8913](https://github.com/paritytech/parity-ethereum/pull/8913))
- Minor fix in chain supplier and light provider ([#8906](https://github.com/paritytech/parity-ethereum/pull/8906))
- Block 0 is valid in queries ([#8891](https://github.com/paritytech/parity-ethereum/pull/8891))
- Fixed osx permissions ([#8901](https://github.com/paritytech/parity-ethereum/pull/8901))
- Atomic create new files with permissions to owner in ethstore ([#8896](https://github.com/paritytech/parity-ethereum/pull/8896))
- Add ETC Cooperative-run load balanced parity node ([#8892](https://github.com/paritytech/parity-ethereum/pull/8892))
- Add support for --chain tobalaba ([#8870](https://github.com/paritytech/parity-ethereum/pull/8870))
- Fix some warns on nightly ([#8889](https://github.com/paritytech/parity-ethereum/pull/8889))
- Add new ovh bootnodes and fix port for foundation bootnode 3.2 ([#8886](https://github.com/paritytech/parity-ethereum/pull/8886))
- Secretstore: service pack 1 ([#8435](https://github.com/paritytech/parity-ethereum/pull/8435))
- Handle removed logs in filter changes and add geth compatibility field ([#8796](https://github.com/paritytech/parity-ethereum/pull/8796))
- Fixed ipc leak, closes [#8774](https://github.com/paritytech/parity-ethereum/issues/8774) ([#8876](https://github.com/paritytech/parity-ethereum/pull/8876))
- Scripts: remove md5 checksums ([#8884](https://github.com/paritytech/parity-ethereum/pull/8884))
- Hardware_wallet/Ledger `Sign messages` + some refactoring ([#8868](https://github.com/paritytech/parity-ethereum/pull/8868))
- Check whether we need resealing in miner and unwrap has_account in account_provider ([#8853](https://github.com/paritytech/parity-ethereum/pull/8853))
- Docker: Fix alpine build ([#8878](https://github.com/paritytech/parity-ethereum/pull/8878))
- Remove mac os installers etc ([#8875](https://github.com/paritytech/parity-ethereum/pull/8875))
- Readme.md: update the list of dependencies ([#8864](https://github.com/paritytech/parity-ethereum/pull/8864))
- Fix concurrent access to signer queue ([#8854](https://github.com/paritytech/parity-ethereum/pull/8854))
- Tx permission contract improvement ([#8400](https://github.com/paritytech/parity-ethereum/pull/8400))
- Limit the number of transactions in pending set ([#8777](https://github.com/paritytech/parity-ethereum/pull/8777))
- Use sealing.enabled to emit eth_mining information ([#8844](https://github.com/paritytech/parity-ethereum/pull/8844))
- Don't allocate in expect_valid_rlp unless necessary ([#8867](https://github.com/paritytech/parity-ethereum/pull/8867))
- Fix Cli Return Code on --help for ethkey, ethstore & whisper ([#8863](https://github.com/paritytech/parity-ethereum/pull/8863))
- Fix subcrate test compile ([#8862](https://github.com/paritytech/parity-ethereum/pull/8862))
- Network-devp2p: downgrade logging to debug, add target ([#8784](https://github.com/paritytech/parity-ethereum/pull/8784))
- Clearing up a comment about the prefix for signing ([#8828](https://github.com/paritytech/parity-ethereum/pull/8828))
- Disable parallel verification and skip verifiying already imported txs. ([#8834](https://github.com/paritytech/parity-ethereum/pull/8834))
- Devp2p: Move UDP socket handling from Discovery to Host. ([#8790](https://github.com/paritytech/parity-ethereum/pull/8790))
- Fixed AuthorityRound deadlock on shutdown, closes [#8088](https://github.com/paritytech/parity-ethereum/issues/8088) ([#8803](https://github.com/paritytech/parity-ethereum/pull/8803))
- Specify critical release flag per network ([#8821](https://github.com/paritytech/parity-ethereum/pull/8821))
- Fix `deadlock_detection` feature branch compilation ([#8824](https://github.com/paritytech/parity-ethereum/pull/8824))
- Use system allocator when profiling memory ([#8831](https://github.com/paritytech/parity-ethereum/pull/8831))
- Added from and to to Receipt ([#8756](https://github.com/paritytech/parity-ethereum/pull/8756))
- Ethcore: fix ancient block error msg handling ([#8832](https://github.com/paritytech/parity-ethereum/pull/8832))
- Ci: Fix docker tags ([#8822](https://github.com/paritytech/parity-ethereum/pull/8822))
- Parity: fix indentation in sync logging ([#8794](https://github.com/paritytech/parity-ethereum/pull/8794))
- Removed obsolete IpcMode enum ([#8819](https://github.com/paritytech/parity-ethereum/pull/8819))
- Remove UI related settings from CLI ([#8783](https://github.com/paritytech/parity-ethereum/pull/8783))
- Remove windows tray and installer ([#8778](https://github.com/paritytech/parity-ethereum/pull/8778))
- Docs: add changelogs for 1.10.6 and 1.11.3 ([#8810](https://github.com/paritytech/parity-ethereum/pull/8810))
- Fix ancient blocks queue deadlock ([#8751](https://github.com/paritytech/parity-ethereum/pull/8751))
- Disallow unsigned transactions in case EIP-86 is disabled ([#8802](https://github.com/paritytech/parity-ethereum/pull/8802))
- Fix evmbin compilation ([#8795](https://github.com/paritytech/parity-ethereum/pull/8795))
- Have space between feature cfg flag ([#8791](https://github.com/paritytech/parity-ethereum/pull/8791))
- Rpc: fix address formatting in TransactionRequest Display ([#8786](https://github.com/paritytech/parity-ethereum/pull/8786))
- Conditionally compile ethcore public test helpers ([#8743](https://github.com/paritytech/parity-ethereum/pull/8743))
- Remove Result wrapper from AccountProvider in RPC impls ([#8763](https://github.com/paritytech/parity-ethereum/pull/8763))
- Update `license header` and `scripts` ([#8666](https://github.com/paritytech/parity-ethereum/pull/8666))
- Remove HostTrait altogether ([#8681](https://github.com/paritytech/parity-ethereum/pull/8681))
- Ethcore-sync: fix connection to peers behind chain fork block ([#8710](https://github.com/paritytech/parity-ethereum/pull/8710))
- Remove public node settings from cli ([#8758](https://github.com/paritytech/parity-ethereum/pull/8758))
- Custom Error Messages on ENFILE and EMFILE IO Errors ([#8744](https://github.com/paritytech/parity-ethereum/pull/8744))
- Ci: Fixes for Android Pipeline ([#8745](https://github.com/paritytech/parity-ethereum/pull/8745))
- Remove NetworkService::config() ([#8653](https://github.com/paritytech/parity-ethereum/pull/8653))
- Fix XOR distance calculation in discovery Kademlia impl ([#8589](https://github.com/paritytech/parity-ethereum/pull/8589))
- Print warnings when fetching pending blocks ([#8711](https://github.com/paritytech/parity-ethereum/pull/8711))
- Fix PoW blockchains sealing notifications in chain_new_blocks ([#8656](https://github.com/paritytech/parity-ethereum/pull/8656))
- Remove -k/--insecure option from curl installer ([#8719](https://github.com/paritytech/parity-ethereum/pull/8719))
- Ease tiny-keccak version requirements (1.4.1 -> 1.4) ([#8726](https://github.com/paritytech/parity-ethereum/pull/8726))
- Bump tinykeccak to 1.4 ([#8728](https://github.com/paritytech/parity-ethereum/pull/8728))
- Remove a couple of unnecessary `transmute()` ([#8736](https://github.com/paritytech/parity-ethereum/pull/8736))
- Fix some nits using clippy ([#8731](https://github.com/paritytech/parity-ethereum/pull/8731))
- Add 'interface' option to cli ([#8699](https://github.com/paritytech/parity-ethereum/pull/8699))
- Remove unused function new_pow_test_spec ([#8735](https://github.com/paritytech/parity-ethereum/pull/8735))
- Add a deadlock detection thread ([#8727](https://github.com/paritytech/parity-ethereum/pull/8727))
- Fix local transactions policy. ([#8691](https://github.com/paritytech/parity-ethereum/pull/8691))
- Shutdown the Snapshot Service early ([#8658](https://github.com/paritytech/parity-ethereum/pull/8658))
- Network-devp2p: handle UselessPeer disconnect ([#8686](https://github.com/paritytech/parity-ethereum/pull/8686))
- Fix compilation error on nightly rust ([#8707](https://github.com/paritytech/parity-ethereum/pull/8707))
- Add a test for decoding corrupt data ([#8713](https://github.com/paritytech/parity-ethereum/pull/8713))
- Update dev chain ([#8717](https://github.com/paritytech/parity-ethereum/pull/8717))
- Remove unused imports ([#8722](https://github.com/paritytech/parity-ethereum/pull/8722))
- Implement recursive Debug for Nodes in patrica_trie::TrieDB ([#8697](https://github.com/paritytech/parity-ethereum/pull/8697))
- Parity: trim whitespace when parsing duration strings ([#8692](https://github.com/paritytech/parity-ethereum/pull/8692))
- Set the request index to that of the current request ([#8683](https://github.com/paritytech/parity-ethereum/pull/8683))
- Remove empty file ([#8705](https://github.com/paritytech/parity-ethereum/pull/8705))
- Update mod.rs ([#8695](https://github.com/paritytech/parity-ethereum/pull/8695))
- Use impl Future in the light client RPC helpers ([#8628](https://github.com/paritytech/parity-ethereum/pull/8628))
- Fix cli signer ([#8682](https://github.com/paritytech/parity-ethereum/pull/8682))
- Allow making direct RPC queries from the C API ([#8588](https://github.com/paritytech/parity-ethereum/pull/8588))
- Remove the error when stopping the network ([#8671](https://github.com/paritytech/parity-ethereum/pull/8671))
- Move connection_filter to the network crate ([#8674](https://github.com/paritytech/parity-ethereum/pull/8674))
- Remove HostInfo::client_version() and secret() ([#8677](https://github.com/paritytech/parity-ethereum/pull/8677))
- Refactor EIP150, EIP160 and EIP161 forks to be specified in CommonParams ([#8614](https://github.com/paritytech/parity-ethereum/pull/8614))
- Parity: improve cli help and logging ([#8665](https://github.com/paritytech/parity-ethereum/pull/8665))
- Updated tiny-keccak to 1.4.2 ([#8669](https://github.com/paritytech/parity-ethereum/pull/8669))
- Remove the Keccak C library and use the pure Rust impl ([#8657](https://github.com/paritytech/parity-ethereum/pull/8657))
- Remove HostInfo::next_nonce ([#8644](https://github.com/paritytech/parity-ethereum/pull/8644))
- Fix not downloading old blocks ([#8642](https://github.com/paritytech/parity-ethereum/pull/8642))
- Resumable warp-sync / Seed downloaded snapshots ([#8544](https://github.com/paritytech/parity-ethereum/pull/8544))
- Don't open Browser post-install on Mac ([#8641](https://github.com/paritytech/parity-ethereum/pull/8641))
- Changelog for 1.10.4-stable and 1.11.1-beta ([#8637](https://github.com/paritytech/parity-ethereum/pull/8637))
- Typo ([#8640](https://github.com/paritytech/parity-ethereum/pull/8640))
- Fork choice and metadata framework for Engine ([#8401](https://github.com/paritytech/parity-ethereum/pull/8401))
- Check that the Android build doesn't dep on c++_shared ([#8538](https://github.com/paritytech/parity-ethereum/pull/8538))
- Remove NetworkContext::io_channel() ([#8625](https://github.com/paritytech/parity-ethereum/pull/8625))
- Fix light sync with initial validator-set contract ([#8528](https://github.com/paritytech/parity-ethereum/pull/8528))
- Store morden db and keys in "path/to/parity/data/Morden" (ropsten uses "test", like before) ([#8621](https://github.com/paritytech/parity-ethereum/pull/8621))
- ´main.rs´ typo ([#8629](https://github.com/paritytech/parity-ethereum/pull/8629))
- Fix BlockReward contract "arithmetic operation overflow" ([#8611](https://github.com/paritytech/parity-ethereum/pull/8611))
- Gitlab test script fixes ([#8573](https://github.com/paritytech/parity-ethereum/pull/8573))
- Remove manually added text to the errors ([#8595](https://github.com/paritytech/parity-ethereum/pull/8595))
- Fix account list double 0x display ([#8596](https://github.com/paritytech/parity-ethereum/pull/8596))
- Typo: wrong indentation in kovan config ([#8610](https://github.com/paritytech/parity-ethereum/pull/8610))
- Fix packet count when talking with PAR2 peers ([#8555](https://github.com/paritytech/parity-ethereum/pull/8555))
- Use full qualified syntax for itertools::Itertools::flatten ([#8606](https://github.com/paritytech/parity-ethereum/pull/8606))
- 2 tiny modification on snapshot ([#8601](https://github.com/paritytech/parity-ethereum/pull/8601))
- Fix the mio test again ([#8602](https://github.com/paritytech/parity-ethereum/pull/8602))
- Remove inject.js server-side injection for dapps ([#8539](https://github.com/paritytech/parity-ethereum/pull/8539))
- Block_header can fail so return Result ([#8581](https://github.com/paritytech/parity-ethereum/pull/8581))
- Block::decode() returns Result ([#8586](https://github.com/paritytech/parity-ethereum/pull/8586))
- Fix compiler warning ([#8590](https://github.com/paritytech/parity-ethereum/pull/8590))
- Fix Parity UI link ([#8600](https://github.com/paritytech/parity-ethereum/pull/8600))
- Make mio optional in ethcore-io ([#8537](https://github.com/paritytech/parity-ethereum/pull/8537))
- Attempt to fix intermittent test failures ([#8584](https://github.com/paritytech/parity-ethereum/pull/8584))
- Changelog and Readme ([#8591](https://github.com/paritytech/parity-ethereum/pull/8591))
- Added Dockerfile for alpine linux by @andresilva, closes [#3565](https://github.com/paritytech/parity-ethereum/issues/3565) ([#8587](https://github.com/paritytech/parity-ethereum/pull/8587))
- Add whisper CLI to the pipelines ([#8578](https://github.com/paritytech/parity-ethereum/pull/8578))
- Rename `whisper-cli binary` to `whisper` ([#8579](https://github.com/paritytech/parity-ethereum/pull/8579))
- Changelog nit ([#8585](https://github.com/paritytech/parity-ethereum/pull/8585))
- Remove unnecessary cloning in overwrite_with ([#8580](https://github.com/paritytech/parity-ethereum/pull/8580))
- Handle socket address parsing errors ([#8545](https://github.com/paritytech/parity-ethereum/pull/8545))
- Update CHANGELOG for 1.9, 1.10, and 1.11 ([#8556](https://github.com/paritytech/parity-ethereum/pull/8556))
- Decoding headers can fail ([#8570](https://github.com/paritytech/parity-ethereum/pull/8570))
- Refactoring `ethcore-sync` - Fixing warp-sync barrier ([#8543](https://github.com/paritytech/parity-ethereum/pull/8543))
- Remove State::replace_backend ([#8569](https://github.com/paritytech/parity-ethereum/pull/8569))
- Make trace-time publishable. ([#8568](https://github.com/paritytech/parity-ethereum/pull/8568))
- Don't block sync when importing old blocks ([#8530](https://github.com/paritytech/parity-ethereum/pull/8530))
- Trace precompiled contracts when the transfer value is not zero ([#8486](https://github.com/paritytech/parity-ethereum/pull/8486))
- Parity as a library ([#8412](https://github.com/paritytech/parity-ethereum/pull/8412))
- Rlp decode returns Result ([#8527](https://github.com/paritytech/parity-ethereum/pull/8527))
- Node table sorting according to last contact data ([#8541](https://github.com/paritytech/parity-ethereum/pull/8541))
- Keep all enacted blocks notify in order ([#8524](https://github.com/paritytech/parity-ethereum/pull/8524))
- Ethcore, rpc, machine: refactor block reward application and tracing ([#8490](https://github.com/paritytech/parity-ethereum/pull/8490))
- Consolidate crypto functionality in `ethcore-crypto`. ([#8432](https://github.com/paritytech/parity-ethereum/pull/8432))
- Eip 145: Bitwise shifting instructions in EVM ([#8451](https://github.com/paritytech/parity-ethereum/pull/8451))
- Remove expect ([#8536](https://github.com/paritytech/parity-ethereum/pull/8536))
- Don't panic in import_block if invalid rlp ([#8522](https://github.com/paritytech/parity-ethereum/pull/8522))
- Pass on storage keys tracing to handle the case when it is not modified ([#8491](https://github.com/paritytech/parity-ethereum/pull/8491))
- Fetching logs by hash in blockchain database ([#8463](https://github.com/paritytech/parity-ethereum/pull/8463))
- Transaction Pool improvements ([#8470](https://github.com/paritytech/parity-ethereum/pull/8470))
- More changes for Android ([#8421](https://github.com/paritytech/parity-ethereum/pull/8421))
- Enable WebAssembly and Byzantium for Ellaism ([#8520](https://github.com/paritytech/parity-ethereum/pull/8520))
- Secretstore: merge two types of errors into single one + Error::is_non_fatal ([#8357](https://github.com/paritytech/parity-ethereum/pull/8357))
- Hardware Wallet trait ([#8071](https://github.com/paritytech/parity-ethereum/pull/8071))
- Directly return None if tracing is disabled ([#8504](https://github.com/paritytech/parity-ethereum/pull/8504))
- Show imported messages for light client ([#8517](https://github.com/paritytech/parity-ethereum/pull/8517))
- Remove unused dependency `bigint` ([#8505](https://github.com/paritytech/parity-ethereum/pull/8505))
- `duration_ns: u64 -> duration: Duration` ([#8457](https://github.com/paritytech/parity-ethereum/pull/8457))
- Return error if RLP size of transaction exceeds the limit ([#8473](https://github.com/paritytech/parity-ethereum/pull/8473))
- Remove three old warp boot nodes. ([#8497](https://github.com/paritytech/parity-ethereum/pull/8497))
- Update wasmi and pwasm-utils ([#8493](https://github.com/paritytech/parity-ethereum/pull/8493))
- Update hardcodedSync for Ethereum, Kovan, and Ropsten ([#8489](https://github.com/paritytech/parity-ethereum/pull/8489))
- Fix snap builds ([#8483](https://github.com/paritytech/parity-ethereum/pull/8483))
- Bump master to 1.12 ([#8477](https://github.com/paritytech/parity-ethereum/pull/8477))
- Don't require write lock when fetching status. ([#8481](https://github.com/paritytech/parity-ethereum/pull/8481))
- Use rename_all for RichBlock and RichHeader serialization ([#8471](https://github.com/paritytech/parity-ethereum/pull/8471))
