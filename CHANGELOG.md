## Parity-Ethereum [v2.2.2](https://github.com/paritytech/parity-ethereum/releases/tag/v2.2.2) (2018-11-29)

Parity-Ethereum 2.2.2-beta is an exciting release. Among others, it improves sync performance, peering stability, block propagation, and transaction propagation times. Also, a warp-sync no longer removes existing blocks from the database, but rather reuses locally available information to decrease sync times and reduces required bandwidth.

Before upgrading to 2.2.2, please also verify the validity of your chain specs. Parity Ethereum now denies unknown fields in the specification. To do this, use the chainspec tool:

```
cargo build --release -p chainspec
./target/release/chainspec /path/to/spec.json
```

Last but not least, JSONRPC APIs which are not yet accepted as an EIP in the `eth`, `personal`, or `web3` namespace, are now considere experimental as their final specification might change in future. These APIs have to be manually enabled by explicitly running `--jsonrpc-experimental`.

The full list of included changes:

- Backports For beta 2.2.2 ([#9976](https://github.com/paritytech/parity-ethereum/pull/9976))
  - Version: bump beta to 2.2.2
  - Add experimental RPCs flag ([#9928](https://github.com/paritytech/parity-ethereum/pull/9928))
  - Keep existing blocks when restoring a Snapshot ([#8643](https://github.com/paritytech/parity-ethereum/pull/8643))
    - Rename db_restore => client
    - First step: make it compile!
    - Second step: working implementation!
    - Refactoring
    - Fix tests
    - Migrate ancient blocks interacting backward
    - Early return in block migration if snapshot is aborted
    - Remove RwLock getter (PR Grumble I)
    - Remove dependency on `Client`: only used Traits
    - Add test for recovering aborted snapshot recovery
    - Add test for migrating old blocks
    - Release RwLock earlier
    - Revert Cargo.lock
    - Update _update ancient block_ logic: set local in `commit`
    - Update typo in ethcore/src/snapshot/service.rs
  - Adjust requests costs for light client ([#9925](https://github.com/paritytech/parity-ethereum/pull/9925))
    - Pip Table Cost relative to average peers instead of max peers
    - Add tracing in PIP new_cost_table
    - Update stat peer_count
    - Use number of leeching peers for Light serve costs
    - Fix test::light_params_load_share_depends_on_max_peers (wrong type)
    - Remove (now) useless test
    - Remove `load_share` from LightParams.Config
    - Add LEECHER_COUNT_FACTOR
    - Pr Grumble: u64 to u32 for f64 casting
    - Prevent u32 overflow for avg_peer_count
    - Add tests for LightSync::Statistics
  - Fix empty steps ([#9939](https://github.com/paritytech/parity-ethereum/pull/9939))
    - Don't send empty step twice or empty step then block.
    - Perform basic validation of locally sealed blocks.
    - Don't include empty step twice.
  - Prevent silent errors in daemon mode, closes [#9367](https://github.com/paritytech/parity-ethereum/issues/9367) ([#9946](https://github.com/paritytech/parity-ethereum/pull/9946))
  - Fix a deadlock ([#9952](https://github.com/paritytech/parity-ethereum/pull/9952))
    - Update informant:
      - Decimal in Mgas/s
      - Print every 5s (not randomly between 5s and 10s)
    - Fix dead-lock in `blockchain.rs`
    - Update locks ordering
  - Fix light client informant while syncing ([#9932](https://github.com/paritytech/parity-ethereum/pull/9932))
    - Add `is_idle` to LightSync to check importing status
    - Use SyncStateWrapper to make sure is_idle gets updates
    - Update is_major_import to use verified queue size as well
    - Add comment for `is_idle`
    - Add Debug to `SyncStateWrapper`
    - `fn get` -> `fn into_inner`
  -  Ci: rearrange pipeline by logic ([#9970](https://github.com/paritytech/parity-ethereum/pull/9970))
    - Ci: rearrange pipeline by logic
    - Ci: rename docs script
  - Fix docker build ([#9971](https://github.com/paritytech/parity-ethereum/pull/9971))
  - Deny unknown fields for chainspec ([#9972](https://github.com/paritytech/parity-ethereum/pull/9972))
    - Add deny_unknown_fields to chainspec
    - Add tests and fix existing one
    - Remove serde_ignored dependency for chainspec
    - Fix rpc test eth chain spec
    - Fix starting_nonce_test spec
  - Improve block and transaction propagation ([#9954](https://github.com/paritytech/parity-ethereum/pull/9954))
    - Refactor sync to add priority tasks.
    - Send priority tasks notifications.
    - Propagate blocks, optimize transactions.
    - Implement transaction propagation. Use sync_channel.
    - Tone down info.
    - Prevent deadlock by not waiting forever for sync lock.
    - Fix lock order.
    - Don't use sync_channel to prevent deadlocks.
    - Fix tests.
  - Fix unstable peers and slowness in sync ([#9967](https://github.com/paritytech/parity-ethereum/pull/9967))
    - Don't sync all peers after each response
    - Update formating
    - Fix tests: add `continue_sync` to `Sync_step`
    - Update ethcore/sync/src/chain/mod.rs
    - Fix rpc middlewares
  - Fix Cargo.lock
  - Json: resolve merge in spec
  - Rpc: fix starting_nonce_test
  - Ci: allow nightl job to fail

## Parity-Ethereum [v2.2.1](https://github.com/paritytech/parity-ethereum/releases/tag/v2.2.1) (2018-11-15)

Parity-Ethereum 2.2.1-beta is the first v2.2 release, and might introduce features that break previous work flows, among others:

- Prevent zero network ID ([#9763](https://github.com/paritytech/parity-ethereum/pull/9763)) and drop support for Olympic testnet ([#9801](https://github.com/paritytech/parity-ethereum/pull/9801)): The Olympic test net is dead for years and never used a chain ID but network ID zero. Parity Ethereum is now preventing the network ID to be zero, thus Olympic support is dropped. Make sure to chose positive non-zero network IDs in future.
- Multithreaded snapshot creation ([#9239](https://github.com/paritytech/parity-ethereum/pull/9239)): adds a CLI argument `--snapshot-threads` which specifies the number of threads. This helps improving the performance of full nodes that wish to provide warp-snapshots for the network. The gain in performance comes with a slight drawback in increased snapshot size.
- Expose config max-round-blocks-to-import ([#9439](https://github.com/paritytech/parity-ethereum/pull/9439)): Parity Ethereum imports blocks in rounds. If at the end of any round, the queue is not empty, we consider it to be _importing_ and won't notify pubsub. On large re-orgs (10+ blocks), this is possible. The default `max_round_blocks_to_import` is increased to 12 and configurable via the `--max-round-blocks-to-import` CLI flag. With unstable network conditions, it is advised to increase the number. This shouldn't have any noticeable performance impact unless the number is set to really large.
- Increase Gas-floor-target and Gas Cap ([#9564](https://github.com/paritytech/parity-ethereum/pull/9564)): the default values for gas floor target are `8_000_000` and gas cap `10_000_000`, similar to Geth 1.8.15+.
- Produce portable binaries ([#9725](https://github.com/paritytech/parity-ethereum/pull/9725)): we now produce portable binaries, but it may incur some performance degradation. For ultimate performance it's now better to compile Parity Ethereum from source with `PORTABLE=OFF` environment variable.
- RPC: `parity_allTransactionHashes` ([#9745](https://github.com/paritytech/parity-ethereum/pull/9745)): Get all pending transactions from the queue with the high performant `parity_allTransactionHashes` RPC method.
- Support `eth_chainId` RPC method ([#9783](https://github.com/paritytech/parity-ethereum/pull/9783)): implements EIP-695 to get the chainID via RPC.
- AuRa: finalize blocks ([#9692](https://github.com/paritytech/parity-ethereum/pull/9692)): The AuRa engine was updated to emit ancestry actions to finalize blocks. The full client stores block finality in the database, the engine builds finality from an ancestry of `ExtendedHeader`; `is_epoch_end` was updated to take a vec of recently finalized headers; `is_epoch_end_light` was added which maintains the previous interface and is used by the light client since the client itself doesn't track finality.

The full list of included changes:

- Backport to parity 2.2.1 beta ([#9905](https://github.com/paritytech/parity-ethereum/pull/9905))
  - Bump version to 2.2.1
  - Fix: Intermittent failing CI due to addr in use ([#9885](https://github.com/paritytech/parity-ethereum/pull/9885))
  - Fix Parity not closing on Ctrl-C ([#9886](https://github.com/paritytech/parity-ethereum/pull/9886))
  - Fix json tracer overflow ([#9873](https://github.com/paritytech/parity-ethereum/pull/9873))
  - Fix docker script ([#9854](https://github.com/paritytech/parity-ethereum/pull/9854))
  - Add hardcoded headers for light client ([#9907](https://github.com/paritytech/parity-ethereum/pull/9907))
  - Gitlab-ci: make android release build succeed ([#9743](https://github.com/paritytech/parity-ethereum/pull/9743))
  - Allow to seal work on latest block ([#9876](https://github.com/paritytech/parity-ethereum/pull/9876))
  - Remove rust-toolchain file ([#9906](https://github.com/paritytech/parity-ethereum/pull/9906))
  - Light-fetch: Differentiate between out-of-gas/manual throw and use required gas from response on failure ([#9824](https://github.com/paritytech/parity-ethereum/pull/9824))
  - Eip-712 implementation ([#9631](https://github.com/paritytech/parity-ethereum/pull/9631))
  - Eip-191 implementation ([#9701](https://github.com/paritytech/parity-ethereum/pull/9701))
  - Simplify cargo audit ([#9918](https://github.com/paritytech/parity-ethereum/pull/9918))
  - Fix performance issue importing Kovan blocks ([#9914](https://github.com/paritytech/parity-ethereum/pull/9914))
  - Ci: nuke the gitlab caches ([#9855](https://github.com/paritytech/parity-ethereum/pull/9855))
- Backports to parity beta 2.2.0 ([#9820](https://github.com/paritytech/parity-ethereum/pull/9820))
  - Ci: remove failing tests for android, windows, and macos ([#9788](https://github.com/paritytech/parity-ethereum/pull/9788))
  - Implement NoProof for json tests and update tests reference ([#9814](https://github.com/paritytech/parity-ethereum/pull/9814))
  - Move state root verification before gas used ([#9841](https://github.com/paritytech/parity-ethereum/pull/9841))
  - Classic.json Bootnode Update ([#9828](https://github.com/paritytech/parity-ethereum/pull/9828))
- Rpc: parity_allTransactionHashes ([#9745](https://github.com/paritytech/parity-ethereum/pull/9745))
- Revert "prevent zero networkID ([#9763](https://github.com/paritytech/parity-ethereum/pull/9763))" ([#9815](https://github.com/paritytech/parity-ethereum/pull/9815))
- Allow zero chain id in EIP155 signing process ([#9792](https://github.com/paritytech/parity-ethereum/pull/9792))
- Add readiness check for docker container ([#9804](https://github.com/paritytech/parity-ethereum/pull/9804))
- Insert dev account before unlocking ([#9813](https://github.com/paritytech/parity-ethereum/pull/9813))
- Removed "rustup" & added new runner tag ([#9731](https://github.com/paritytech/parity-ethereum/pull/9731))
- Expose config max-round-blocks-to-import ([#9439](https://github.com/paritytech/parity-ethereum/pull/9439))
- Aura: finalize blocks ([#9692](https://github.com/paritytech/parity-ethereum/pull/9692))
- Sync: retry different peer after empty subchain heads response ([#9753](https://github.com/paritytech/parity-ethereum/pull/9753))
- Fix(light-rpc/parity) : Remove unused client ([#9802](https://github.com/paritytech/parity-ethereum/pull/9802))
- Drops support for olympic testnet, closes [#9800](https://github.com/paritytech/parity-ethereum/issues/9800) ([#9801](https://github.com/paritytech/parity-ethereum/pull/9801))
- Replace `tokio_core` with `tokio` (`ring` -> 0.13) ([#9657](https://github.com/paritytech/parity-ethereum/pull/9657))
- Support eth_chainId RPC method ([#9783](https://github.com/paritytech/parity-ethereum/pull/9783))
- Ethcore: bump ropsten forkblock checkpoint ([#9775](https://github.com/paritytech/parity-ethereum/pull/9775))
- Docs: changelogs for 2.0.8 and 2.1.3 ([#9758](https://github.com/paritytech/parity-ethereum/pull/9758))
- Prevent zero networkID ([#9763](https://github.com/paritytech/parity-ethereum/pull/9763))
- Skip seal fields count check when --no-seal-check is used ([#9757](https://github.com/paritytech/parity-ethereum/pull/9757))
- Aura: fix panic on extra_info with unsealed block ([#9755](https://github.com/paritytech/parity-ethereum/pull/9755))
- Docs: update changelogs ([#9742](https://github.com/paritytech/parity-ethereum/pull/9742))
- Removed extra assert in generation_session_is_removed_when_succeeded ([#9738](https://github.com/paritytech/parity-ethereum/pull/9738))
- Make checkpoint_storage_at use plain loop instead of recursion ([#9734](https://github.com/paritytech/parity-ethereum/pull/9734))
- Use signed 256-bit integer for sstore gas refund substate ([#9746](https://github.com/paritytech/parity-ethereum/pull/9746))
- Heads ref not present for branches beta and stable ([#9741](https://github.com/paritytech/parity-ethereum/pull/9741))
- Add Callisto support ([#9534](https://github.com/paritytech/parity-ethereum/pull/9534))
- Add --force to cargo audit install script ([#9735](https://github.com/paritytech/parity-ethereum/pull/9735))
- Remove unused expired value from Handshake ([#9732](https://github.com/paritytech/parity-ethereum/pull/9732))
- Add hardcoded headers ([#9730](https://github.com/paritytech/parity-ethereum/pull/9730))
- Produce portable binaries ([#9725](https://github.com/paritytech/parity-ethereum/pull/9725))
- Gitlab ci: releasable_branches: change variables condition to schedule ([#9729](https://github.com/paritytech/parity-ethereum/pull/9729))
- Update a few parity-common dependencies ([#9663](https://github.com/paritytech/parity-ethereum/pull/9663))
- Hf in POA Core (2018-10-22) ([#9724](https://github.com/paritytech/parity-ethereum/pull/9724))
- Schedule nightly builds ([#9717](https://github.com/paritytech/parity-ethereum/pull/9717))
- Fix ancient blocks sync ([#9531](https://github.com/paritytech/parity-ethereum/pull/9531))
- Ci: Skip docs job for nightly ([#9693](https://github.com/paritytech/parity-ethereum/pull/9693))
- Fix (light/provider) : Make `read_only executions` read-only ([#9591](https://github.com/paritytech/parity-ethereum/pull/9591))
- Ethcore: fix detection of major import ([#9552](https://github.com/paritytech/parity-ethereum/pull/9552))
- Return 0 on error ([#9705](https://github.com/paritytech/parity-ethereum/pull/9705))
- Ethcore: delay ropsten hardfork ([#9704](https://github.com/paritytech/parity-ethereum/pull/9704))
- Make instantSeal engine backwards compatible, closes [#9696](https://github.com/paritytech/parity-ethereum/issues/9696) ([#9700](https://github.com/paritytech/parity-ethereum/pull/9700))
- Implement CREATE2 gas changes and fix some potential overflowing ([#9694](https://github.com/paritytech/parity-ethereum/pull/9694))
- Don't hash the init_code of CREATE. ([#9688](https://github.com/paritytech/parity-ethereum/pull/9688))
- Ethcore: minor optimization of modexp by using LR exponentiation ([#9697](https://github.com/paritytech/parity-ethereum/pull/9697))
- Removed redundant clone before each block import ([#9683](https://github.com/paritytech/parity-ethereum/pull/9683))
- Add Foundation Bootnodes ([#9666](https://github.com/paritytech/parity-ethereum/pull/9666))
- Docker: run as parity user ([#9689](https://github.com/paritytech/parity-ethereum/pull/9689))
- Ethcore: mcip3 block reward contract ([#9605](https://github.com/paritytech/parity-ethereum/pull/9605))
- Verify block syncing responses against requests ([#9670](https://github.com/paritytech/parity-ethereum/pull/9670))
- Add a new RPC `parity_submitWorkDetail` similar `eth_submitWork` but return block hash ([#9404](https://github.com/paritytech/parity-ethereum/pull/9404))
- Resumable EVM and heap-allocated callstack ([#9360](https://github.com/paritytech/parity-ethereum/pull/9360))
- Update parity-wordlist library ([#9682](https://github.com/paritytech/parity-ethereum/pull/9682))
- Ci: Remove unnecessary pipes ([#9681](https://github.com/paritytech/parity-ethereum/pull/9681))
- Test.sh: use cargo --target for platforms other than linux, win or mac ([#9650](https://github.com/paritytech/parity-ethereum/pull/9650))
- Ci: fix push script ([#9679](https://github.com/paritytech/parity-ethereum/pull/9679))
- Hardfork the testnets ([#9562](https://github.com/paritytech/parity-ethereum/pull/9562))
- Calculate sha3 instead of sha256 for push-release. ([#9673](https://github.com/paritytech/parity-ethereum/pull/9673))
- Ethcore-io retries failed work steal ([#9651](https://github.com/paritytech/parity-ethereum/pull/9651))
- Fix(light_fetch): avoid race with BlockNumber::Latest ([#9665](https://github.com/paritytech/parity-ethereum/pull/9665))
- Test fix for windows cache name... ([#9658](https://github.com/paritytech/parity-ethereum/pull/9658))
- Refactor(fetch) : light use only one `DNS` thread ([#9647](https://github.com/paritytech/parity-ethereum/pull/9647))
- Ethereum libfuzzer integration small change ([#9547](https://github.com/paritytech/parity-ethereum/pull/9547))
- Cli: remove reference to --no-ui in --unlock flag help ([#9616](https://github.com/paritytech/parity-ethereum/pull/9616))
- Remove master from releasable branches ([#9655](https://github.com/paritytech/parity-ethereum/pull/9655))
- Ethcore/VerificationQueue don't spawn up extra `worker-threads` when explictly specified not to ([#9620](https://github.com/paritytech/parity-ethereum/pull/9620))
- Rpc: parity_getBlockReceipts ([#9527](https://github.com/paritytech/parity-ethereum/pull/9527))
- Remove unused dependencies ([#9589](https://github.com/paritytech/parity-ethereum/pull/9589))
- Ignore key_server_cluster randomly failing tests ([#9639](https://github.com/paritytech/parity-ethereum/pull/9639))
- Ethcore: handle vm exception when estimating gas ([#9615](https://github.com/paritytech/parity-ethereum/pull/9615))
- Fix bad-block reporting no reason ([#9638](https://github.com/paritytech/parity-ethereum/pull/9638))
- Use static call and apparent value transfer for block reward contract code ([#9603](https://github.com/paritytech/parity-ethereum/pull/9603))
- Hf in POA Sokol (2018-09-19) ([#9607](https://github.com/paritytech/parity-ethereum/pull/9607))
- Bump smallvec to 0.6 in ethcore-light, ethstore and whisper ([#9588](https://github.com/paritytech/parity-ethereum/pull/9588))
- Add constantinople conf to EvmTestClient. ([#9570](https://github.com/paritytech/parity-ethereum/pull/9570))
- Fix(network): don't disconnect reserved peers ([#9608](https://github.com/paritytech/parity-ethereum/pull/9608))
- Fix failing node-table tests on mac os, closes [#9632](https://github.com/paritytech/parity-ethereum/issues/9632) ([#9633](https://github.com/paritytech/parity-ethereum/pull/9633))
- Update ropsten.json ([#9602](https://github.com/paritytech/parity-ethereum/pull/9602))
- Simplify ethcore errors by removing BlockImportError ([#9593](https://github.com/paritytech/parity-ethereum/pull/9593))
- Fix windows compilation, replaces [#9561](https://github.com/paritytech/parity-ethereum/issues/9561) ([#9621](https://github.com/paritytech/parity-ethereum/pull/9621))
- Master: rpc-docs set github token ([#9610](https://github.com/paritytech/parity-ethereum/pull/9610))
- Docs: add changelogs for 1.11.10, 1.11.11, 2.0.3, 2.0.4, 2.0.5, 2.0.6, 2.1.0, and 2.1.1 ([#9554](https://github.com/paritytech/parity-ethereum/pull/9554))
- Docs(rpc): annotate tag with the provided message ([#9601](https://github.com/paritytech/parity-ethereum/pull/9601))
- Ci: fix regex roll_eyes ([#9597](https://github.com/paritytech/parity-ethereum/pull/9597))
- Remove snapcraft clean ([#9585](https://github.com/paritytech/parity-ethereum/pull/9585))
- Add snapcraft package image (master) ([#9584](https://github.com/paritytech/parity-ethereum/pull/9584))
- Docs(rpc): push the branch along with tags ([#9578](https://github.com/paritytech/parity-ethereum/pull/9578))
- Fix typo for jsonrpc-threads flag ([#9574](https://github.com/paritytech/parity-ethereum/pull/9574))
- Fix informant compile ([#9571](https://github.com/paritytech/parity-ethereum/pull/9571))
- Added ropsten bootnodes ([#9569](https://github.com/paritytech/parity-ethereum/pull/9569))
- Increase Gas-floor-target and Gas Cap ([#9564](https://github.com/paritytech/parity-ethereum/pull/9564))
- While working on the platform tests make them non-breaking ([#9563](https://github.com/paritytech/parity-ethereum/pull/9563))
- Improve P2P discovery ([#9526](https://github.com/paritytech/parity-ethereum/pull/9526))
- Move dockerfile for android build container to scripts repo ([#9560](https://github.com/paritytech/parity-ethereum/pull/9560))
- Simultaneous platform tests WIP ([#9557](https://github.com/paritytech/parity-ethereum/pull/9557))
- Update ethabi-derive, serde, serde_json, serde_derive, syn && quote ([#9553](https://github.com/paritytech/parity-ethereum/pull/9553))
- Ci: fix rpc docs generation 2 ([#9550](https://github.com/paritytech/parity-ethereum/pull/9550))
- Ci: always run build pipelines for win, mac, linux, and android ([#9537](https://github.com/paritytech/parity-ethereum/pull/9537))
- Multithreaded snapshot creation ([#9239](https://github.com/paritytech/parity-ethereum/pull/9239))
- New ethabi ([#9511](https://github.com/paritytech/parity-ethereum/pull/9511))
- Remove initial token for WS. ([#9545](https://github.com/paritytech/parity-ethereum/pull/9545))
- Net_version caches network_id to avoid redundant aquire of sync readlock ([#9544](https://github.com/paritytech/parity-ethereum/pull/9544))
- Correct before_script for nightly build versions ([#9543](https://github.com/paritytech/parity-ethereum/pull/9543))
- Deps: bump kvdb-rocksdb to 0.1.4 ([#9539](https://github.com/paritytech/parity-ethereum/pull/9539))
- State: test when contract creation fails, old storage values should re-appear ([#9532](https://github.com/paritytech/parity-ethereum/pull/9532))
- Allow dropping light client RPC query with no results ([#9318](https://github.com/paritytech/parity-ethereum/pull/9318))
- Bump master to 2.2.0 ([#9517](https://github.com/paritytech/parity-ethereum/pull/9517))
- Enable all Constantinople hard fork changes in constantinople_test.json ([#9505](https://github.com/paritytech/parity-ethereum/pull/9505))
- [Light] Validate `account balance` before importing transactions ([#9417](https://github.com/paritytech/parity-ethereum/pull/9417))
- In create memory calculation is the same for create2 because the additional parameter was popped before. ([#9522](https://github.com/paritytech/parity-ethereum/pull/9522))
- Update patricia trie to 0.2.2 ([#9525](https://github.com/paritytech/parity-ethereum/pull/9525))
- Replace hardcoded JSON with serde json! macro ([#9489](https://github.com/paritytech/parity-ethereum/pull/9489))
- Fix typo in version string ([#9516](https://github.com/paritytech/parity-ethereum/pull/9516))

## Previous releases

- [CHANGELOG-2.1](docs/CHANGELOG-2.1.md) (_stable_)
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
