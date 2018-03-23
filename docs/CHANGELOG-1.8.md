Note: Parity 1.8 reached End-of-Life on 2018-03-22 (EOL).

## Parity [v1.8.11](https://github.com/paritytech/parity/releases/tag/v1.8.11) (2018-03-01)

Parity 1.8.11 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Bump stable to 1.8.11 ([#8010](https://github.com/paritytech/parity/pull/8010))
- Stable Backports ([#8008](https://github.com/paritytech/parity/pull/8008))
  - Reject too large packets in snapshot sync. ([#7977](https://github.com/paritytech/parity/pull/7977))
  - Increase max download limit to 128MB ([#7965](https://github.com/paritytech/parity/pull/7965))
  - Calculate proper keccak256/sha3 using parity. ([#7953](https://github.com/paritytech/parity/pull/7953))
  - Bump WebSockets ([#7952](https://github.com/paritytech/parity/pull/7952))

## Parity [v1.8.10](https://github.com/paritytech/parity/releases/tag/v1.8.10) (2018-02-20)

Parity 1.8.10 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Bump react-qr-reader ([#7941)](https://github.com/paritytech/parity/pull/7941))
  - Bump react-qr-reader
  - Explicit add webrtc-adapter, non-napa qrcode-generator
- Fix feature=final ([#7914)](https://github.com/paritytech/parity/pull/7914))
- Fix snap build stable ([#7897)](https://github.com/paritytech/parity/pull/7897))
- Backport core PRs to stable ([#7892)](https://github.com/paritytech/parity/pull/7892))
  - Update back-references more aggressively after answering from cache ([#7578)](https://github.com/paritytech/parity/pull/7578))
  - Store updater metadata in a single place ([#7832)](https://github.com/paritytech/parity/pull/7832))
  - Flush keyfiles. Resolves [#7632](https://github.com/paritytech/parity/issues/7632) ([#7868)](https://github.com/paritytech/parity/pull/7868))
  - Fix wallet import ([#7873)](https://github.com/paritytech/parity/pull/7873))
- Backport Master CI PRs to Stable ([#7889)](https://github.com/paritytech/parity/pull/7889))
  - Add binary identifiers and sha256sum to builds ([#7830)](https://github.com/paritytech/parity/pull/7830))
  - Fix checksums and auto-update push ([#7846)](https://github.com/paritytech/parity/pull/7846))
  - Update gitlab-build.sh ([#7855)](https://github.com/paritytech/parity/pull/7855))
  - Fix installer binary names for macos and windows ([#7881)](https://github.com/paritytech/parity/pull/7881))
  - Update gitlab-test.sh ([#7883)](https://github.com/paritytech/parity/pull/7883))
  - Fix snapcraft nightly ([#7884)](https://github.com/paritytech/parity/pull/7884))
  - Bump stable to 1.8.10
  - Make track stable

## Parity [v1.8.9](https://github.com/paritytech/parity/releases/tag/v1.8.9) (2018-02-02)

Parity 1.8.9 is a bug-fix release to improve performance and stability. It restores ERC-20 token balances and adds additional bootnodes for the Ropsten test network.

The full list of included changes:

- Update gitlab-build.sh
- Backports stable ([#7779](https://github.com/paritytech/parity/pull/7779))
  - Bump stable to 1.8.9
  - Update ropsten.json ([#7776](https://github.com/paritytech/parity/pull/7776))
- Fix tokenAddress reference ([#7777](https://github.com/paritytech/parity/pull/7777))
- Snapcraft push stable

## Parity [v1.8.8](https://github.com/paritytech/parity/releases/tag/v1.8.8) (2018-02-01)

Parity 1.8.8 is a bug-fix release to improve performance and stability. It restores ERC-20 token balances, improves networking, fixes database corruptions on client shutdown, and fixes issues with the `--password` command-line flag. Happy syncing!

The full list of included changes:

- Gitlab stable ([#7768](https://github.com/paritytech/parity/pull/7768))
  - Fix snapcraft build for stable
  - Initial support push snap packages to snapcraft.io
  - Edge-devel
- Snapcraft push ans fix build ([#7767](https://github.com/paritytech/parity/pull/7767))
  - Fix snapcraft build for stable
  - Initial support push snap packages to snapcraft.io
  - Edge-devel
- Remove snapcraft.yml from scripts
- Resolve conflicts
- Token filter balances (throttle) ([#7755](https://github.com/paritytech/parity/pull/7755))
- Fix snapcraft build (stable) ([#7763](https://github.com/paritytech/parity/pull/7763))
  - Fix snapcraft build for stable
  - Fix snapcraft build (stable)
- [Stable] Backports ([#7758](https://github.com/paritytech/parity/pull/7758))
  - Filter-out nodes.json ([#7716](https://github.com/paritytech/parity/pull/7716))
    - Filter-out nodes.json
    - Network: sort node table nodes by failure ratio
    - Network: fix node table tests
    - Network: fit node failure percentage into buckets of 5%
    - Network: consider number of attempts in sorting of node table
    - Network: fix node table grumbles
  - Fix client not being dropped on shutdown ([#7695](https://github.com/paritytech/parity/pull/7695))
    - Parity: wait for client to drop on shutdown
    - Parity: fix grumbles in shutdown wait
    - Parity: increase shutdown timeouts
  - Wrap --help output to 120 characters ([#7626](https://github.com/paritytech/parity/pull/7626))
    - Update Clap dependency and remove workarounds
    - WIP
    - Remove line breaks in help messages for now
    - Multiple values can only be separated by commas (closes [#7428](https://github.com/paritytech/parity/issues/7428))
    - Grumbles; refactor repeating code; add constant
    - Use a single Wrapper rather than allocate a new one for each call
    - Wrap --help to 120 characters rather than 100 characters
- Use explicit branch name in push ([#7757](https://github.com/paritytech/parity/pull/7757))
- Bump stable to 1.8.8 ([#7752](https://github.com/paritytech/parity/pull/7752))
- Fix js-release in stable ([#7682](https://github.com/paritytech/parity/pull/7682))
- Update Info.plist
- Fix conditions in gitlab-test ([#7675](https://github.com/paritytech/parity/pull/7675))
- Remove cargo cache

## Parity [v1.8.7](https://github.com/paritytech/parity/releases/tag/v1.8.7) (2018-01-24)

Parity 1.8.7 is the first stable release of the 1.8 channel. It includes various bug and stability fixes. Users on Kovan or other Aura-based networks are advised to upgrade as this release fixes an issue introduced with 1.8.6 and 1.7.12 that causes Proof-of-Authority nodes to stop synchronizing the chain.

The full list of included changes:

- Bump stable version ([#7665](https://github.com/paritytech/parity/pull/7665))
  - Bump stable to 1.8.7
- Backports to stable ([#7661](https://github.com/paritytech/parity/pull/7661))
  - Fixed delegatecall's from/to ([#7568](https://github.com/paritytech/parity/pull/7568))
    - Fixed delegatecall's from/to, closes [#7166](https://github.com/paritytech/parity/issues/7166)
    - Added tests for delegatecall traces, [#7167](https://github.com/paritytech/parity/issues/7167)
  - Fix Temporarily Invalid blocks handling ([#7613](https://github.com/paritytech/parity/pull/7613))
    - Handle temporarily invalid blocks in sync.
    - Fix tests.
  - Improve handling of RocksDB corruption ([#7630](https://github.com/paritytech/parity/pull/7630))
    - Kvdb-rocksdb: update rust-rocksdb version
    - Kvdb-rocksdb: mark corruptions and attempt repair on db open
    - Kvdb-rocksdb: better corruption detection on open
    - Kvdb-rocksdb: add corruption_file_name const
    - Kvdb-rocksdb: rename mark_corruption to check_for_corruption
- Add docker build for stable and cargo cache ([#7672](https://github.com/paritytech/parity/pull/7672))
- Fix snapcraft build for stable ([#7669](https://github.com/paritytech/parity/pull/7669))
- Update .gitlab-ci.yml ([#7599](https://github.com/paritytech/parity/pull/7599))
  - Fix cache:key
- Make 1.8 stable ([#7532](https://github.com/paritytech/parity/pull/7532))
  - Bump 1.8 to stable
  - Make js-precompiled stable

## Parity [v1.8.6](https://github.com/paritytech/parity/releases/tag/v1.8.6) (2018-01-10)

Parity 1.8.6 fixes a critical issue with the database eventually filling up user's disks. Upgrading is highly recommended as it will significantly improve your user experience. As a bonus, this release should enable users with slower hard-disk drives to catch up with the latest block again. Also, warp-sync performance was significantly improved. Please note, that the initial database compaction after upgrading might temporarily reduce the node's performance.

In addition to our gas price histogram, this version now allows you to dynamically set the default gas price as percentile from the last 100 blocks (it defaults to median: `50`).

    --gas-price-percentile=[PCT]        Set PCT percentile gas price value from
                                        last 100 blocks as default gas price
                                        when sending transactions.

Last but not least, this release also fixes consensus issues with the Expanse chain enabling Byzantium. If you run Parity configured for Expanse, you might have to resync your chain after the upgrade.

The full list of included changes:

- RocksDB fix ([#7508](https://github.com/paritytech/parity/pull/7508))
  - Kvdb: update rust-rocksdb version
- Backports to beta ([#7434](https://github.com/paritytech/parity/pull/7434))
  - Wait for future blocks in AuRa ([#7368](https://github.com/paritytech/parity/pull/7368))
    - Mark future blocks as temporarily invalid.
    - Don't check max.
  - Fix tracing failed calls. ([#7412](https://github.com/paritytech/parity/pull/7412))
  - Problem: sending any Whisper message fails ([#7421](https://github.com/paritytech/parity/pull/7421))
  - Strict config parsing ([#7433](https://github.com/paritytech/parity/pull/7433))
  - Problem: AuRa's unsafeties around step duration ([#7282](https://github.com/paritytech/parity/pull/7282))
  - Remove expanse chain ([#7437](https://github.com/paritytech/parity/pull/7437))
    - Remove expanse from available chains
    - Remove all EXP references from old wallet
    - Fix tests
  - Remove expanse chain ([#7437](https://github.com/paritytech/parity/pull/7437))
  - Expanse Byzantium update w/ correct metropolis difficulty increment divisor ([#7463](https://github.com/paritytech/parity/pull/7463))
    - Byzantium Update for Expanse
    - Expip2 changes - update duration limit
    - Fix missing EXPIP-2 fields
    - Format numbers as hex
    - Fix compilation errors
    - Group expanse chain spec fields together
    - Set metropolisDifficultyIncrementDivisor for Expanse
    - Revert #7437
    - Add Expanse block 900_000 hash checkpoint
  - Advance AuRa step as far as we can and prevent invalid blocks. ([#7451](https://github.com/paritytech/parity/pull/7451))
    - Advance AuRa step as far as we can.
    - Wait for future blocks.
  - Fixed panic when io is not available for export block, closes [#7486](https://github.com/paritytech/parity/issues/7486) ([#7495](https://github.com/paritytech/parity/pull/7495))
  - Update Parity Mainnet Bootnodes ([#7476](https://github.com/paritytech/parity/pull/7476))
    - Replace the Azure HDD bootnodes with the new ones :)
  - Expose default gas price percentile configuration in CLI ([#7497](https://github.com/paritytech/parity/pull/7497))
    - Expose gas price percentile.
    - Fix light eth_call.
    - Fix gas_price in light client
- Backport nonces reservations ([#7439](https://github.com/paritytech/parity/pull/7439))
  - Reserve nonces for signing ([#6834](https://github.com/paritytech/parity/pull/6834))
    - Nonce future - reserve and dispatch
    - Single thread nonce tests
    - Track status of reserved nonces.
    - Initialization of nonce reservations.
    - Prospective Signer
    - Fix cli tests.
  - Fix nonce reservation ([#7025](https://github.com/paritytech/parity/pull/7025))
    - Use nonce reservation per address
    - Create hashmap in RPC Apis
    - Garbage collect hashmap entries.
    - HashMap::retain
- Bump beta to 1.8.6 ([#7442](https://github.com/paritytech/parity/pull/7442))
- KVDB backports ([#7438](https://github.com/paritytech/parity/pull/7438))
  - Separated kvdb into 3 crates: kvdb, kvdb-memorydb && kvdb-rocksdb ([#6720](https://github.com/paritytech/parity/pull/6720))
    - Separated kvdb into 3 crates: kvdb, kvdb-memorydb && kvdb-rocksdb, ref [#6693](https://github.com/paritytech/parity/issues/6693)
      - Fixed kvdb-memorydb && kvdb-rocksdb authors
      - Fixed wrong kvdb import in json_tests
    - Util tests use kvdb_memorydb instead of kvdb_rocksdb, closes [#6739](https://github.com/paritytech/parity/issues/6739)
      - Renamed kvdb_memorydb::in_memory -> kvdb_memorydb::create
      - Docs
      - Removed redundant mut from kvdb-memorydb
  - Upgrade to RocksDB 5.8.8 and tune settings to reduce space amplification ([#7348](https://github.com/paritytech/parity/pull/7348))
    - kvdb-rocksdb: update to RocksDB 5.8.8
    - kvdb-rocksdb: tune RocksDB options
      - Switch to level-style compaction
      - Increase default block size (16K), and use bigger blocks for HDDs (64K)
      - Increase default file size base (64MB SSDs, 256MB HDDs)
      - Create a single block cache shared across all column families
      - Tune compaction settings using RocksDB helper functions, taking into account
      - Memory budget spread across all columns
      - Configure backgrounds jobs based on the number of CPUs
      - Set some default recommended settings
    - ethcore: remove unused config blockchain.db_cache_size
    - parity: increase default value for db_cache_size
    - kvdb-rocksdb: enable compression on all levels
    - kvdb-rocksdb: set global db_write_bufer_size
    - kvdb-rocksdb: reduce db_write_bufer_size to force earlier flushing
    - kvdb-rocksdb: use master branch for rust-rocksdb dependency

## Parity [v1.8.5](https://github.com/paritytech/parity/releases/tag/v1.8.5) (2017-12-29)

Parity 1.8.5 changes the default behavior of JSON-RPC CORS setting, detects same-key engine signers in Aura networks, and updates bootnodes for the Kovan and Foundation networks.

Note: The default value of `--jsonrpc-cors` option has been altered to disallow (potentially malicious) websites from accessing the low-sensitivity RPCs (viewing exposed accounts, proposing transactions for signing). Currently domains need to be whitelisted manually. To bring back previous behaviour run with `--jsonrpc-cors all` or `--jsonrpc-cors http://example.com`.

The full list of included changes:

- Beta Backports ([#7297](https://github.com/paritytech/parity/pull/7297))
  - New warp enodes ([#7287](https://github.com/paritytech/parity/pull/7287))
    - New warp enodes
    - Added one more warp enode; replaced spaces with tabs
    - Bump beta to 1.8.5
    - Update kovan boot nodes
  - Detect different node, same-key signing in aura ([#7245](https://github.com/paritytech/parity/pull/7245))
    - Detect different node, same-key signing in aura
    - Reduce scope of warning
    - Fix Cargo.lock
    - Updating mainnet bootnodes.
  - Update bootnodes ([#7363](https://github.com/paritytech/parity/pull/7363))
    - Updating mainnet bootnodes.
    - Add additional parity-beta bootnodes.
    - Restore old parity bootnodes and update foudation bootnodes
- Fix default CORS. ([#7388](https://github.com/paritytech/parity/pull/7388))

## Parity [v1.8.4](https://github.com/paritytech/parity/releases/tag/v1.8.4) (2017-12-12)

Parity 1.8.4 applies fixes for Proof-of-Authority networks and schedules the Kovan-Byzantium hard-fork.

- The Kovan testnet will fork on block `5067000` at `Thu Dec 14 2017 05:40:03 UTC`.
  - This enables Byzantium features on Kovan.
  - This disables uncles on Kovan for stability reasons.
- Proof-of-Authority networks are advised to set `maximumUncleCount` to 0 in a future `maximumUncleCountTransition` for stability reasons.
  - See the [Kovan chain spec](https://github.com/paritytech/parity/blob/master/ethcore/res/ethereum/kovan.json) for an example.
  - New PoA networks created with Parity will have this feature enabled by default.

Furthermore, this release includes the ECIP-1039 Monetary policy rounding specification for Ethereum Classic, reduces the maximum Ethash-block timestamp drift to 15 seconds, and fixes various bugs for WASM and the RPC APIs.

The full list of included changes:

- Beta Backports and HF block update ([#7244](https://github.com/paritytech/parity/pull/7244))
  - Reduce max block timestamp drift to 15 seconds ([#7240](https://github.com/paritytech/parity/pull/7240))
    - Add test for block timestamp validation within allowed drift
  - Update kovan HF block number.
- Beta Kovan HF ([#7234](https://github.com/paritytech/parity/pull/7234))
  - Kovan HF.
  - Bump version.
  - Fix aura difficulty race ([#7198](https://github.com/paritytech/parity/pull/7198))
    - Fix test key
    - Extract out score calculation
    - Fix build
  - Update kovan HF block number.
  - Add missing byzantium builtins.
  - Bump installers versions.
  - Increase allowed time drift to 10s. ([#7238](https://github.com/paritytech/parity/pull/7238))
- Beta Backports ([#7197](https://github.com/paritytech/parity/pull/7197))
  - Maximum uncle count transition ([#7196](https://github.com/paritytech/parity/pull/7196))
    - Enable delayed maximum_uncle_count activation.
    - Fix tests.
    - Defer kovan HF.
  - Disable uncles by default ([#7006](https://github.com/paritytech/parity/pull/7006))
  - Escape inifinite loop in estimte_gas ([#7075](https://github.com/paritytech/parity/pull/7075))
  - ECIP-1039: Monetary policy rounding specification ([#7067](https://github.com/paritytech/parity/pull/7067))
  - WASM Remove blockhash error ([#7121](https://github.com/paritytech/parity/pull/7121))
    - Remove blockhash error
    - Update tests.
  - WASM storage_read and storage_write don't return anything ([#7110](https://github.com/paritytech/parity/pull/7110))
  - WASM parse payload from panics ([#7097](https://github.com/paritytech/parity/pull/7097))
  - Fix no-default-features. ([#7096](https://github.com/paritytech/parity/pull/7096))

## Parity [v1.8.3](https://github.com/paritytech/parity/releases/tag/v1.8.3) (2017-11-15)

Parity 1.8.3 contains several bug-fixes and removes the ability to deploy built-in multi-signature wallets.

The full list of included changes:

- Backports to beta ([#7043](https://github.com/paritytech/parity/pull/7043))
  - pwasm-std update ([#7018](https://github.com/paritytech/parity/pull/7018))
  - Version 1.8.3
  - Make CLI arguments parsing more backwards compatible ([#7004](https://github.com/paritytech/parity/pull/7004))
  - Skip nonce check for gas estimation ([#6997](https://github.com/paritytech/parity/pull/6997))
  - Events in WASM runtime ([#6967](https://github.com/paritytech/parity/pull/6967))
  - Return decoded seal fields. ([#6932](https://github.com/paritytech/parity/pull/6932))
  - Fix serialization of status in transaction receipts. ([#6926](https://github.com/paritytech/parity/pull/6926))
  - Windows fixes ([#6921](https://github.com/paritytech/parity/pull/6921))
- Disallow built-in multi-sig deploy (only watch) ([#7014](https://github.com/paritytech/parity/pull/7014))
- Add hint in ActionParams for splitting code/data ([#6968](https://github.com/paritytech/parity/pull/6968))
  - Action params and embedded params handling
  - Fix name-spaces

## Parity [v1.8.2](https://github.com/paritytech/parity/releases/tag/v1.8.2) (2017-10-26)

Parity 1.8.2 fixes an important potential consensus issue and a few additional minor issues:

- `blockNumber` transaction field is now returned correctly in RPC calls.
- Possible crash when `--force-sealing` option is used.

The full list of included changes:

- Beta Backports ([#6891](https://github.com/paritytech/parity/pull/6891))
  - Bump to v1.8.2
  - Refactor static context check in CREATE. ([#6886](https://github.com/paritytech/parity/pull/6886))
    - Refactor static context check in CREATE.
    - Fix wasm.
  - Fix serialization of non-localized transactions ([#6868](https://github.com/paritytech/parity/pull/6868))
    - Fix serialization of non-localized transactions.
    - Return proper SignedTransactions representation.
  - Allow force sealing and reseal=0 for non-dev chains. ([#6878](https://github.com/paritytech/parity/pull/6878))

## Parity [v1.8.1](https://github.com/paritytech/parity/releases/tag/v1.8.1) (2017-10-20)

Parity 1.8.1 fixes several bugs with token balances, tweaks snapshot-sync, improves the performance of nodes with huge amounts of accounts and changes the Trezor account derivation path.

**Important Note**: The **Trezor** account derivation path was changed in this release ([#6815](https://github.com/paritytech/parity/pull/6815)) to always use the first account (`m/44'/60'/0'/0/0` instead of `m/44'/60'/0'/0`). This way we enable compatibility with other Ethereum wallets supporting Trezor hardware-wallets. However, **action is required** before upgrading, if you have funds on your Parity Trezor wallet. If you already upgraded to 1.8.1, please downgrade to 1.8.0 first to recover the funds with the following steps:

1. Make sure you have 1.8.0-beta and your Trezor plugged in.
2. Create a new standard Parity account. Make sure you have backups of the recovery phrase and don't forget the password.
3. Move your funds from the Trezor hardware-wallet account to the freshly generated Parity account.
4. Upgrade to 1.8.1-beta and plug in your Trezor.
5. Move your funds from your Parity account to the new Trezor account.
6. Keep using Parity as normal.

If you don't want to downgrade or move your funds off your Trezor-device, you can also use the official Trezor application or other wallets allowing to select the derivation path to access the funds.

The full list of included changes:

- Add ECIP1017 to Morden config ([#6845](https://github.com/paritytech/parity/pull/6845))
- Ethstore optimizations ([#6844](https://github.com/paritytech/parity/pull/6844))
- Bumb to v1.8.1 ([#6843](https://github.com/paritytech/parity/pull/6843))
- Backport ([#6837](https://github.com/paritytech/parity/pull/6837))
  - Tweaked snapshot sync threshold ([#6829](https://github.com/paritytech/parity/pull/6829))
  - Change keypath derivation logic ([#6815](https://github.com/paritytech/parity/pull/6815))
- Refresh cached tokens based on registry info & random balances ([#6824](https://github.com/paritytech/parity/pull/6824))
  - Refresh cached tokens based on registry info & random balances ([#6818](https://github.com/paritytech/parity/pull/6818))
  - Don't display errored token images

## Parity [v1.8.0](https://github.com/paritytech/parity/releases/tag/v1.8.0) (2017-10-15)

We are happy to announce our newest Parity 1.8 release. Among others, it enables the following features:

- Full Whisper v6 integration
- Trezor hardware-wallet support
- WASM contract support
- PICOPS KYC-certified accounts and vouching for community-dapps
- Light client compatibility for Proof-of-Authority networks
- Transaction permissioning and permissioned p2p-connections
- Full Byzantium-fork compatibility
- Full Musicoin MCIP-3 UBI-fork compatibility

Further, users upgrading from 1.7 should acknowledge the following changes:

- The chain-engine was further abstracted and chain-specs need to be upgraded. [#6134](https://github.com/paritytech/parity/pull/6134) [#6591](https://github.com/paritytech/parity/pull/6591)
- `network_id` was renamed to `chain_id` where applicable. [#6345](https://github.com/paritytech/parity/pull/6345)
- `trace_filter` RPC method now comes with pagination. [#6312](https://github.com/paritytech/parity/pull/6312)
- Added tracing of rewards on closing blocks. [#6194](https://github.com/paritytech/parity/pull/6194)

The full list of included changes:

- Updated ethabi to fix auto-update ([#6771](https://github.com/paritytech/parity/pull/6771))
- Fixed kovan chain validation ([#6760](https://github.com/paritytech/parity/pull/6760))
  - Fixed kovan chain validation
  - Fork detection
  - Fixed typo
- Bumped fork block number for auto-update ([#6755](https://github.com/paritytech/parity/pull/6755))
- CLI: Reject invalid argument values rather than ignore them ([#6747](https://github.com/paritytech/parity/pull/6747))
- Fixed modexp gas calculation overflow ([#6745](https://github.com/paritytech/parity/pull/6745))
- Backport beta - Fixes Badges ([#6732](https://github.com/paritytech/parity/pull/6732))
  - Fix badges not showing up ([#6730](https://github.com/paritytech/parity/pull/6730))
  - Always fetch meta data first [badges]
- Bump to v1.8.0 in beta
- Fix tokens and badges ([#6725](https://github.com/paritytech/parity/pull/6725))
  - Update new token fetching
  - Working Certifications Monitoring
  - Update on Certification / Revoke
  - Fix none-fetched tokens value display
  - Fix tests
- Check vouch status on appId in addition to contentHash ([#6719](https://github.com/paritytech/parity/pull/6719))
  - Check vouch status on appId in addition to contentHash
  - Simplify var expansion
- Prevent going offline when restoring or taking a snapshot [#6694](https://github.com/paritytech/parity/pull/6694)
- Graceful exit when invalid CLI flags are passed (#6485) [#6711](https://github.com/paritytech/parity/pull/6711)
- Fixed RETURNDATA out of bounds check [#6718](https://github.com/paritytech/parity/pull/6718)
- Display vouched overlay on dapps [#6710](https://github.com/paritytech/parity/pull/6710)
- Fix gas estimation if `from` is not provided. [#6714](https://github.com/paritytech/parity/pull/6714)
- Emulate signer pubsub on public node [#6708](https://github.com/paritytech/parity/pull/6708)
- Removes  dependency on rustc_serialize (#5988) [#6705](https://github.com/paritytech/parity/pull/6705)
- Fixed potential modexp exp len overflow [#6686](https://github.com/paritytech/parity/pull/6686)
- Fix asciiToHex for characters < 0x10 [#6702](https://github.com/paritytech/parity/pull/6702)
- Fix address input [#6701](https://github.com/paritytech/parity/pull/6701)
- Allow signer signing display of markdown [#6707](https://github.com/paritytech/parity/pull/6707)
- Fixed build warnings [#6664](https://github.com/paritytech/parity/pull/6664)
- Fix warp sync blockers detection [#6691](https://github.com/paritytech/parity/pull/6691)
- Difficulty tests [#6687](https://github.com/paritytech/parity/pull/6687)
- Separate migrations from util [#6690](https://github.com/paritytech/parity/pull/6690)
- Changelog for 1.7.3 [#6678](https://github.com/paritytech/parity/pull/6678)
- WASM gas schedule [#6638](https://github.com/paritytech/parity/pull/6638)
- Fix wallet view [#6597](https://github.com/paritytech/parity/pull/6597)
- Byzantium fork block number [#6660](https://github.com/paritytech/parity/pull/6660)
- Fixed RETURNDATA size for built-ins [#6652](https://github.com/paritytech/parity/pull/6652)
- Light Client: fetch transactions/receipts by transaction hash [#6641](https://github.com/paritytech/parity/pull/6641)
- Add Musicoin and MCIP-3 UBI hardfork.  [#6621](https://github.com/paritytech/parity/pull/6621)
- fix 1.8 backcompat: revert to manual encoding/decoding of transition proofs [#6665](https://github.com/paritytech/parity/pull/6665)
- Tweaked block download timeouts (#6595) [#6655](https://github.com/paritytech/parity/pull/6655)
- Renamed RPC receipt statusCode field to status [#6650](https://github.com/paritytech/parity/pull/6650)
- SecretStore: session level timeout [#6631](https://github.com/paritytech/parity/pull/6631)
- SecretStore: ShareRemove of 'isolated' nodes [#6630](https://github.com/paritytech/parity/pull/6630)
- SecretStore: exclusive sessions [#6624](https://github.com/paritytech/parity/pull/6624)
- Fixed network protocol version negotiation [#6649](https://github.com/paritytech/parity/pull/6649)
- Updated systemd files for linux (Resolves #6592) [#6598](https://github.com/paritytech/parity/pull/6598)
- move additional_params to machine, fixes registry on non-ethash chains [#6646](https://github.com/paritytech/parity/pull/6646)
- Fix Token Transfer in transaction list [#6589](https://github.com/paritytech/parity/pull/6589)
- Update jsonrpc dependencies and rewrite dapps to futures. [#6522](https://github.com/paritytech/parity/pull/6522)
- Balance queries implemented in WASM runtime [#6639](https://github.com/paritytech/parity/pull/6639)
- Don't expose port 80 for parity anymore [#6633](https://github.com/paritytech/parity/pull/6633)
- WASM Runtime refactoring [#6596](https://github.com/paritytech/parity/pull/6596)
- Fix compilation [#6625](https://github.com/paritytech/parity/pull/6625)
- Downgrade futures to suppress warnings. [#6620](https://github.com/paritytech/parity/pull/6620)
- Add pagination for trace_filter rpc method [#6312](https://github.com/paritytech/parity/pull/6312)
- Disallow pasting recovery phrases on first run [#6602](https://github.com/paritytech/parity/pull/6602)
- fix typo: Unkown => Unknown [#6559](https://github.com/paritytech/parity/pull/6559)
- SecretStore: administrative sessions prototypes [#6605](https://github.com/paritytech/parity/pull/6605)
- fix parity.io link 404 [#6617](https://github.com/paritytech/parity/pull/6617)
- SecretStore: add node to existing session poc + discussion [#6480](https://github.com/paritytech/parity/pull/6480)
- Generalize engine trait [#6591](https://github.com/paritytech/parity/pull/6591)
- Add RPC eth_chainId for querying the current blockchain chain ID [#6329](https://github.com/paritytech/parity/pull/6329)
- Debounce sync status. [#6572](https://github.com/paritytech/parity/pull/6572)
- [Public Node] Disable tx scheduling and hardware wallets [#6588](https://github.com/paritytech/parity/pull/6588)
- Use memmap for dag cache [#6193](https://github.com/paritytech/parity/pull/6193)
- Rename Requests to Batch [#6582](https://github.com/paritytech/parity/pull/6582)
- Use host as ws/dapps url if present. [#6566](https://github.com/paritytech/parity/pull/6566)
- Sync progress and error handling fixes [#6560](https://github.com/paritytech/parity/pull/6560)
- Fixed receipt serialization and RPC [#6555](https://github.com/paritytech/parity/pull/6555)
- Fix number of confirmations for transaction [#6552](https://github.com/paritytech/parity/pull/6552)
- Fix #6540 [#6556](https://github.com/paritytech/parity/pull/6556)
- Fix failing hardware tests [#6553](https://github.com/paritytech/parity/pull/6553)
- Required validators >= num owners in Wallet Creation [#6551](https://github.com/paritytech/parity/pull/6551)
- Random cleanups / improvements to a state [#6472](https://github.com/paritytech/parity/pull/6472)
- Changelog for 1.7.2 [#6363](https://github.com/paritytech/parity/pull/6363)
- Ropsten fork [#6533](https://github.com/paritytech/parity/pull/6533)
- Byzantium updates [#5855](https://github.com/paritytech/parity/pull/5855)
- Fix extension detection [#6452](https://github.com/paritytech/parity/pull/6452)
- Downgrade futures to supress warnings [#6521](https://github.com/paritytech/parity/pull/6521)
- separate trie from util and make its dependencies into libs [#6478](https://github.com/paritytech/parity/pull/6478)
- WASM sha3 test [#6512](https://github.com/paritytech/parity/pull/6512)
- Fix broken JavaScript tests [#6498](https://github.com/paritytech/parity/pull/6498)
- SecretStore: use random key to encrypt channel + session-level nonce [#6470](https://github.com/paritytech/parity/pull/6470)
- Trezor Support [#6403](https://github.com/paritytech/parity/pull/6403)
- Fix compiler warning [#6491](https://github.com/paritytech/parity/pull/6491)
- Fix typo [#6505](https://github.com/paritytech/parity/pull/6505)
- WASM: added math overflow test [#6474](https://github.com/paritytech/parity/pull/6474)
- Fix slow balances [#6471](https://github.com/paritytech/parity/pull/6471)
- WASM runtime update [#6467](https://github.com/paritytech/parity/pull/6467)
- Compatibility with whisper v6 [#6179](https://github.com/paritytech/parity/pull/6179)
- light-poa round 2: allow optional casting of engine client to full client [#6468](https://github.com/paritytech/parity/pull/6468)
- Moved attributes under docs [#6475](https://github.com/paritytech/parity/pull/6475)
- cleanup util dependencies [#6464](https://github.com/paritytech/parity/pull/6464)
- removed redundant earlymergedb trace guards [#6463](https://github.com/paritytech/parity/pull/6463)
- UtilError utilizes error_chain! [#6461](https://github.com/paritytech/parity/pull/6461)
- fixed master [#6465](https://github.com/paritytech/parity/pull/6465)
- Refactor and port CLI from Docopt to Clap (#2066) [#6356](https://github.com/paritytech/parity/pull/6356)
- Add language selector in production [#6317](https://github.com/paritytech/parity/pull/6317)
- eth_call returns output of contract creations [#6420](https://github.com/paritytech/parity/pull/6420)
- Refactor: Don't reexport bigint from util [#6459](https://github.com/paritytech/parity/pull/6459)
- Transaction permissioning [#6441](https://github.com/paritytech/parity/pull/6441)
- Added missing SecretStore tests - signing session [#6411](https://github.com/paritytech/parity/pull/6411)
- Light-client sync for contract-based PoA [#6370](https://github.com/paritytech/parity/pull/6370)
- triehash is separated from util [#6428](https://github.com/paritytech/parity/pull/6428)
- remove re-export of parking_lot in util [#6435](https://github.com/paritytech/parity/pull/6435)
- fix modexp bug: return 0 if base is zero [#6424](https://github.com/paritytech/parity/pull/6424)
- separate semantic_version from util [#6438](https://github.com/paritytech/parity/pull/6438)
- move timer.rs to ethcore [#6437](https://github.com/paritytech/parity/pull/6437)
- remove re-export of ansi_term in util [#6433](https://github.com/paritytech/parity/pull/6433)
- Pub sub blocks [#6139](https://github.com/paritytech/parity/pull/6139)
- replace trait Hashable with fn keccak [#6423](https://github.com/paritytech/parity/pull/6423)
- add more hash backward compatibility test for bloom [#6425](https://github.com/paritytech/parity/pull/6425)
- remove the redundant hasher in Bloom [#6404](https://github.com/paritytech/parity/pull/6404)
- Remove re-export of HeapSizeOf in util (part of #6418) [#6419](https://github.com/paritytech/parity/pull/6419)
- Rewards on closing blocks [#6194](https://github.com/paritytech/parity/pull/6194)
- ensure balances of constructor accounts are kept [#6413](https://github.com/paritytech/parity/pull/6413)
- removed recursion from triedbmut::lookup [#6394](https://github.com/paritytech/parity/pull/6394)
- do not activate genesis epoch in immediate transition validator contract [#6349](https://github.com/paritytech/parity/pull/6349)
- Use git for the snap version [#6271](https://github.com/paritytech/parity/pull/6271)
- Permissioned p2p connections [#6359](https://github.com/paritytech/parity/pull/6359)
- Don't accept transactions above block gas limit. [#6408](https://github.com/paritytech/parity/pull/6408)
- Fix memory tracing. [#6399](https://github.com/paritytech/parity/pull/6399)
- earlydb optimizations [#6393](https://github.com/paritytech/parity/pull/6393)
- Optimized PlainHasher hashing. Trie insertions are >15 faster [#6321](https://github.com/paritytech/parity/pull/6321)
- Trie optimizations [#6389](https://github.com/paritytech/parity/pull/6389)
- small optimizations for triehash [#6392](https://github.com/paritytech/parity/pull/6392)
- Bring back IPFS tests. [#6398](https://github.com/paritytech/parity/pull/6398)
- Running state test using parity-evm [#6355](https://github.com/paritytech/parity/pull/6355)
- Wasm math tests extended [#6354](https://github.com/paritytech/parity/pull/6354)
- Expose health status over RPC [#6274](https://github.com/paritytech/parity/pull/6274)
- fix bloom bitvecjournal storage allocation [#6390](https://github.com/paritytech/parity/pull/6390)
- fixed pending block panic [#6391](https://github.com/paritytech/parity/pull/6391)
- Infoline less opaque for UI/visibility [#6364](https://github.com/paritytech/parity/pull/6364)
- Fix eth_call. [#6365](https://github.com/paritytech/parity/pull/6365)
- updated bigint [#6341](https://github.com/paritytech/parity/pull/6341)
- Optimize trie iter by avoiding redundant copying [#6347](https://github.com/paritytech/parity/pull/6347)
- Only keep a single rocksdb debug log file [#6346](https://github.com/paritytech/parity/pull/6346)
- Tweaked snapshot params [#6344](https://github.com/paritytech/parity/pull/6344)
- Rename network_id to chain_id where applicable. [#6345](https://github.com/paritytech/parity/pull/6345)
- Itertools are no longer reexported from util, optimized triedb iter [#6322](https://github.com/paritytech/parity/pull/6322)
- Better check the created accounts before showing Startup Wizard [#6331](https://github.com/paritytech/parity/pull/6331)
- Better error messages for invalid types in RPC [#6311](https://github.com/paritytech/parity/pull/6311)
- fix panic in parity-evm json tracer [#6338](https://github.com/paritytech/parity/pull/6338)
- WASM math test [#6305](https://github.com/paritytech/parity/pull/6305)
- rlp_derive [#6125](https://github.com/paritytech/parity/pull/6125)
- Fix --chain parsing in parity-evm. [#6314](https://github.com/paritytech/parity/pull/6314)
- Unexpose RPC methods on :8180 [#6295](https://github.com/paritytech/parity/pull/6295)
- Ignore errors from dappsUrl when starting UI. [#6296](https://github.com/paritytech/parity/pull/6296)
- updated bigint with optimized mul and from_big_indian [#6323](https://github.com/paritytech/parity/pull/6323)
- SecretStore: bunch of fixes and improvements [#6168](https://github.com/paritytech/parity/pull/6168)
- Master requires rust 1.19 [#6308](https://github.com/paritytech/parity/pull/6308)
- Add more descriptive error when signing/decrypting using hw wallet. [#6302](https://github.com/paritytech/parity/pull/6302)
- Increase default gas limit for eth_call. [#6299](https://github.com/paritytech/parity/pull/6299)
- rust-toolchain file on master [#6266](https://github.com/paritytech/parity/pull/6266)
- Migrate wasm-tests to updated runtime [#6278](https://github.com/paritytech/parity/pull/6278)
- Extension fixes [#6284](https://github.com/paritytech/parity/pull/6284)
- Fix a hash displayed in tooltip when signing arbitrary data [#6283](https://github.com/paritytech/parity/pull/6283)
- Time should not contribue to overall status. [#6276](https://github.com/paritytech/parity/pull/6276)
- Add --to and --gas-price to evmbin [#6277](https://github.com/paritytech/parity/pull/6277)
- Fix dapps CSP when UI is exposed externally [#6178](https://github.com/paritytech/parity/pull/6178)
- Add warning to web browser and fix links. [#6232](https://github.com/paritytech/parity/pull/6232)
- Update Settings/Proxy view to match entries in proxy.pac [#4771](https://github.com/paritytech/parity/pull/4771)
- Dapp refresh [#5752](https://github.com/paritytech/parity/pull/5752)
- Add support for ConsenSys multisig wallet [#6153](https://github.com/paritytech/parity/pull/6153)
- updated jsonrpc [#6264](https://github.com/paritytech/parity/pull/6264)
- SecretStore: encrypt messages using private key from key store [#6146](https://github.com/paritytech/parity/pull/6146)
- Wasm storage read test [#6255](https://github.com/paritytech/parity/pull/6255)
- propagate stratum submit share error upstream [#6260](https://github.com/paritytech/parity/pull/6260)
- Using multiple NTP servers [#6173](https://github.com/paritytech/parity/pull/6173)
- Add GitHub issue templates. [#6259](https://github.com/paritytech/parity/pull/6259)
- format instant change proofs correctly [#6241](https://github.com/paritytech/parity/pull/6241)
- price-info does not depend on util [#6231](https://github.com/paritytech/parity/pull/6231)
- native-contracts crate does not depend on util any more [#6233](https://github.com/paritytech/parity/pull/6233)
- Bump master to 1.8.0 [#6256](https://github.com/paritytech/parity/pull/6256)
- SecretStore: do not cache ACL contract + on-chain key servers configuration [#6107](https://github.com/paritytech/parity/pull/6107)
- Fix the README badges [#6229](https://github.com/paritytech/parity/pull/6229)
- updated tiny-keccak to 1.3 [#6248](https://github.com/paritytech/parity/pull/6248)
- Small grammatical error [#6244](https://github.com/paritytech/parity/pull/6244)
- Multi-call RPC [#6195](https://github.com/paritytech/parity/pull/6195)
- InstantSeal fix [#6223](https://github.com/paritytech/parity/pull/6223)
- Untrusted RLP length overflow check  [#6227](https://github.com/paritytech/parity/pull/6227)
- Chainspec validation [#6197](https://github.com/paritytech/parity/pull/6197)
- Fix cache path when using --base-path [#6212](https://github.com/paritytech/parity/pull/6212)
- removed std reexports from util && fixed broken tests [#6187](https://github.com/paritytech/parity/pull/6187)
- WASM MVP continued [#6132](https://github.com/paritytech/parity/pull/6132)
- Decouple virtual machines [#6184](https://github.com/paritytech/parity/pull/6184)
- Realloc test added [#6177](https://github.com/paritytech/parity/pull/6177)
- Re-enable wallets, fixed forgetting accounts [#6196](https://github.com/paritytech/parity/pull/6196)
- Move more params to the common section. [#6134](https://github.com/paritytech/parity/pull/6134)
- Whisper js [#6161](https://github.com/paritytech/parity/pull/6161)
- typo in uninstaller [#6185](https://github.com/paritytech/parity/pull/6185)
- fix #6052. honor --no-color for signer command [#6100](https://github.com/paritytech/parity/pull/6100)
- Refactor --allow-ips to handle custom ip-ranges [#6144](https://github.com/paritytech/parity/pull/6144)
- Update Changelog for 1.6.10 and 1.7.0 [#6183](https://github.com/paritytech/parity/pull/6183)
- Fix unsoundness in ethash's unsafe code [#6140](https://github.com/paritytech/parity/pull/6140)
