## Parity [v1.10.8](https://github.com/paritytech/parity/releases/tag/v1.10.8) (2018-06-29)

Parity 1.10.8 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Backports ([#8986](https://github.com/paritytech/parity/pull/8986))
  - Snap: downgrade rust to revision 1.26.2, ref snapcraft/+bug/1778530 ([#8984](https://github.com/paritytech/parity/pull/8984))
    - Snap: downgrade rust to revision 1.26.2, ref snapcraft/+bug/1778530
    - Snap: use plugin rust
  - Fix deadlock in blockchain. ([#8977](https://github.com/paritytech/parity/pull/8977))
  - Remove js-glue from workspace
- Bump stable to 1.10.8 ([#8951](https://github.com/paritytech/parity/pull/8951))
  - Parity-version: bump stable to 1.10.8
  - Update ropsten.json ([#8926](https://github.com/paritytech/parity/pull/8926))
  - Scripts: minor improvements ([#8930](https://github.com/paritytech/parity/pull/8930))
    - CI: enable 'latest' docker tag on master pipeline
    - CI: mark both beta and stable as stable snap.
    - CI: sign all windows binaries
    - Scripts: remove whisper target not available in stable
  - Scripts: fix gitlab strip binaries
  - Scripts: fix docker build tag on latest using master ([#8952](https://github.com/paritytech/parity/pull/8952))
  - Rpc: cap gas limit of local calls ([#8943](https://github.com/paritytech/parity/pull/8943))

## Parity [v1.10.7](https://github.com/paritytech/parity/releases/tag/v1.10.7) (2018-06-20)

Parity 1.10.7 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Backports ([#8919](https://github.com/paritytech/parity/pull/8919))
  - Fixed AuthorityRound deadlock on shutdown, closes [#8088](https://github.com/paritytech/parity/issues/8088) ([#8803](https://github.com/paritytech/parity/pull/8803))
  - CI: Fix docker tags ([#8822](https://github.com/paritytech/parity/pull/8822))
    - Scripts: enable docker builds for beta and stable
    - Scripts: docker latest should be beta not master
    - Scripts: docker latest is master
  - Fix concurrent access to signer queue ([#8854](https://github.com/paritytech/parity/pull/8854))
    - Fix concurrent access to signer queue
    - Put request back to the queue if confirmation failed
    - Typo: fix docs and rename functions to be more specific
    - Change trace info "Transaction" -> "Request"
  - Add new ovh bootnodes and fix port for foundation bootnode 3.2 ([#8886](https://github.com/paritytech/parity/pull/8886))
    - Add new ovh bootnodes and fix port for foundation bootnode 3.2
    - Remove old bootnodes.
    - Remove duplicate 1118980bf48b0a3640bdba04e0fe78b1add18e1cd99bf22d53daac1fd9972ad650df52176e7c7d89d1114cfef2bc23a2959aa54998a46afcf7d91809f0855082
  - Block 0 is valid in queries ([#8891](https://github.com/paritytech/parity/pull/8891))
  - Update jsonrpc libs, fixed ipc leak, closes [#8774](https://github.com/paritytech/parity/issues/8774) ([#8876](https://github.com/paritytech/parity/pull/8876))
  - Add ETC Cooperative-run load balanced parity node ([#8892](https://github.com/paritytech/parity/pull/8892))
  - Minor fix in chain supplier and light provider ([#8906](https://github.com/paritytech/parity/pull/8906))
    - Fix chain supplier increment
    - Fix light provider block_headers
- Parity-version: stable release 1.10.7 ([#8855](https://github.com/paritytech/parity/pull/8855))
  - Cherry-pick network-specific release flag ([#8821](https://github.com/paritytech/parity/pull/8821))
  - Parity-version: bump stable to 1.10.7

## Parity [v1.10.6](https://github.com/paritytech/parity/releases/tag/v1.10.6) (2018-06-05)

Parity 1.10.6 is a security-relevant release. Please upgrade your nodes as soon as possible.

If you can not upgrade to 1.10+ yet, please use the following branches and build your own binaries from source:

- git checkout [old-stable-1.9](https://github.com/paritytech/parity/tree/old-stable-1.9) # `v1.9.8` (EOL)
- git checkout [old-stable-1.8](https://github.com/paritytech/parity/tree/old-stable-1.8) # `v1.8.12` (EOL)
- git checkout [old-stable-1.7](https://github.com/paritytech/parity/tree/old-stable-1.7) # `v1.7.14` (EOL)

The full list of included changes:

- Parity-version: bump stable to 1.10.6 ([#8805](https://github.com/paritytech/parity/pull/8805))
  - Parity-version: bump stable to 1.10.6
  - Disallow unsigned transactions in case EIP-86 is disabled ([#8802](https://github.com/paritytech/parity/pull/8802))
- Update shell32-sys to fix windows build ([#8793](https://github.com/paritytech/parity/pull/8793))
- Backports ([#8782](https://github.com/paritytech/parity/pull/8782))
  - Fix light sync with initial validator-set contract ([#8528](https://github.com/paritytech/parity/pull/8528))
    - Fix #8468
    - Use U256::max_value() instead
    - Fix again
    - Also change initial transaction gas
  - Don't open Browser post-install on Mac ([#8641](https://github.com/paritytech/parity/pull/8641))
  - Prefix uint fmt with `0x` with alternate flag
  - Set the request index to that of the current request ([#8683](https://github.com/paritytech/parity/pull/8683))
    - Set the request index to that of the current request
  - Node table sorting according to last contact data ([#8541](https://github.com/paritytech/parity/pull/8541))
    - Network-devp2p: sort nodes in node table using last contact data
    - Network-devp2p: rename node contact types in node table json output
    - Network-devp2p: fix node table tests
    - Network-devp2p: note node failure when failed to establish connection
    - Network-devp2p: handle UselessPeer error
    - Network-devp2p: note failure when marking node as useless
  - Network-devp2p: handle UselessPeer disconnect ([#8686](https://github.com/paritytech/parity/pull/8686))
- Parity: bump stable version to 1.10.5 ([#8749](https://github.com/paritytech/parity/pull/8749))
  - Parity: bump stable version to 1.10.5
  - Fix failing doc tests running on non-code

## Parity [v1.10.4](https://github.com/paritytech/parity/releases/tag/v1.10.4) (2018-05-15)

Parity 1.10.4 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Backports ([#8623](https://github.com/paritytech/parity/pull/8623))
  - Fix account list double 0x display ([#8596](https://github.com/paritytech/parity/pull/8596))
    - Remove unused self import
    - Fix account list double 0x display
  - Trace precompiled contracts when the transfer value is not zero ([#8486](https://github.com/paritytech/parity/pull/8486))
    - Trace precompiled contracts when the transfer value is not zero
    - Add tests for precompiled CALL tracing
    - Use byzantium test machine for the new test
    - Add notes in comments on why we don't trace all precompileds
    - Use is_transferred instead of transferred
  - Gitlab test script fixes ([#8573](https://github.com/paritytech/parity/pull/8573))
    - Exclude /docs from modified files.
    - Ensure all references in the working tree are available
    - Remove duplicated line from test script
- Bump stable to 1.10.4 ([#8626](https://github.com/paritytech/parity/pull/8626))
- Allow stable snaps to be stable. ([#8582](https://github.com/paritytech/parity/pull/8582))

## Parity [v1.10.3](https://github.com/paritytech/parity/releases/tag/v1.10.3) (2018-05-08)

Parity 1.10.3 marks the first stable release on the 1.10 track. Among others, it improves performance and stability.

The full list of included changes:

- Backports ([#8557](https://github.com/paritytech/parity/pull/8557))
  - Update wasmi and pwasm-utils ([#8493](https://github.com/paritytech/parity/pull/8493))
    - Update wasmi to 0.2
    - Update pwasm-utils to 0.1.5
  - Fetching logs by hash in blockchain database ([#8463](https://github.com/paritytech/parity/pull/8463))
    - Fetch logs by hash in blockchain database
    - Fix tests
    - Add unit test for branch block logs fetching
    - Add docs that blocks must already be sorted
    - Handle branch block cases properly
    - typo: empty -> is_empty
    - Remove return_empty_if_none by using a closure
    - Use BTreeSet to avoid sorting again
    - Move is_canon to BlockChain
    - typo: pass value by reference
    - Use loop and wrap inside blocks to simplify the code
    - typo: missed a comment
  - Pass on storage keys tracing to handle the case when it is not modified ([#8491](https://github.com/paritytech/parity/pull/8491))
    - Pass on storage keys even if it is not modified
    - typo: account and storage query `to_pod_diff` builds both `touched_addresses` merge and storage keys merge.
    - Fix tests
    - Use state query directly because of suicided accounts
    - Fix a RefCell borrow issue
    - Add tests for unmodified storage trace
    - Address grumbles
    - typo: remove unwanted empty line
    - ensure_cached compiles with the original signature
  - Enable WebAssembly and Byzantium for Ellaism ([#8520](https://github.com/paritytech/parity/pull/8520))
    - Enable WebAssembly and Byzantium for Ellaism
    - Fix indentation
    - Remove empty lines
  - Fix compilation.
- Stabilize 1.10.3 ([#8474](https://github.com/paritytech/parity/pull/8474))
  - Stabelize 1.10
  - Bump stable to 1.10.3
  - Update Gitlab scripts
  - Fix snap builds ([#8483](https://github.com/paritytech/parity/pull/8483))
  - Fix docker build ([#8462](https://github.com/paritytech/parity/pull/8462))
  - Use `master` as Docker's `latest` (`beta-release` is not used anymore)

## Parity [v1.10.2](https://github.com/paritytech/parity/releases/tag/v1.10.2) (2018-04-24)

Parity 1.10.2 is a bug-fix release to improve performance and stability.

The full list of included changes:

- Update Parity beta to 1.10.2 + Backports ([#8455](https://github.com/paritytech/parity/pull/8455))
  - Update Parity beta to 1.10.2
  - Allow 32-bit pipelines to fail ([#8454](https://github.com/paritytech/parity/pull/8454))
    - Disable 32-bit targets for Gitlab
    - Rename Linux pipelines
  - Update wasmi ([#8452](https://github.com/paritytech/parity/pull/8452))
  - Fix Cargo.lock
- Backports ([#8450](https://github.com/paritytech/parity/pull/8450))
  - Use forked app_dirs crate for reverted Windows dir behavior  ([#8438](https://github.com/paritytech/parity/pull/8438))
    - Remove unused app_dirs dependency in CLI
    - Use forked app_dirs crate for reverted Windows dir behavior
  - Remove Tendermint extra_info due to seal inconsistencies ([#8367](https://github.com/paritytech/parity/pull/8367))
  - Handle queue import errors a bit more gracefully ([#8385](https://github.com/paritytech/parity/pull/8385))
  - Improve VM executor stack size estimation rules ([#8439](https://github.com/paritytech/parity/pull/8439))
    - Improve VM executor stack size estimation rules
    - Typo: docs add "(Debug build)" comment
    - Fix an off by one typo and set minimal stack size
    - Use saturating_sub to avoid potential overflow

## Parity [v1.10.1](https://github.com/paritytech/parity/releases/tag/v1.10.1) (2018-04-17)

Parity 1.10.1 is a bug-fix release to improve performance and stability. Among other changes, you can now use `--warp-barrier [BLOCK]` to specify a minimum block number to `--warp` to. This is useful in cases where clients restore to outdated snapshots far behind the latest chain head.

The full list of included changes:

- Bump beta to 1.10.1 ([#8350](https://github.com/paritytech/parity/pull/8350))
  - Bump beta to 1.10.1
  - Unflag critical release
- Backports ([#8346](https://github.com/paritytech/parity/pull/8346))
  - Warp-only sync with warp-barrier [blocknumber] flag. ([#8228](https://github.com/paritytech/parity/pull/8228))
    - Warp-only sync with warp-after [blocknumber] flag.
    - Fix tests.
    - Fix configuration tests.
    - Rename to warp barrier.
  - Allow unsafe js eval on Parity Wallet. ([#8204](https://github.com/paritytech/parity/pull/8204))
  - Update musicoin spec in line with gmc v2.6.2 ([#8242](https://github.com/paritytech/parity/pull/8242))
  - Supress TemporaryInvalid verification failures. ([#8256](https://github.com/paritytech/parity/pull/8256))
  - Include suicided accounts in state diff ([#8297](https://github.com/paritytech/parity/pull/8297))
    - Include suicided accounts in state diff
    - Shorten form match -> if let
    - Test suicide trace diff in State
  - Replace_home for password_files, reserved_peers and log_file ([#8324](https://github.com/paritytech/parity/pull/8324))
    - Replace_home for password_files, reserved_peers and log_file
    - Typo: arg_log_file is Option
    - Enable UI by default, but only display info page.
    - Fix test.
    - Fix naming and remove old todo.
    - Change "wallet" with "browser UI"
- Change name Wallet -> UI ([#8164](https://github.com/paritytech/parity/pull/8164)) ([#8205](https://github.com/paritytech/parity/pull/8205))
  - Change name Wallet -> UI
  - Make warning bold
- Backport [#8099](https://github.com/paritytech/parity/pull/8099) ([#8132](https://github.com/paritytech/parity/pull/8132))
- WASM libs ([#8220](https://github.com/paritytech/parity/pull/8220))
  - Bump wasm libs ([#8171](https://github.com/paritytech/parity/pull/8171))
  - Bump wasmi version ([#8209](https://github.com/paritytech/parity/pull/8209))
- Update hyper to 0.11.24 ([#8203](https://github.com/paritytech/parity/pull/8203))
- Updated jsonrpc to include latest backports (beta) ([#8181](https://github.com/paritytech/parity/pull/8181))
  - Updated jsonrpc to include latest backports
  - Update dependencies.

## Parity [v1.10.0](https://github.com/paritytech/parity/releases/tag/v1.10.0) (2018-03-22)

This is the Parity 1.10.0-beta release! Cool!

### Disabling the Parity Wallet

The **Parity Wallet (a.k.a. "UI") is now disabled by default**. We are preparing to split the wallet from the core client.

To reactivate the parity wallet, you have to run Parity either with `parity --force-ui` (not recommended) or `parity ui` (deprecated) from the command line. Or, if you feel super fancy and want to test our pre-releases of the stand-alone electron wallet, head over to the [Parity-JS repositories and check the releases](https://github.com/Parity-JS/shell/releases).

Further reading:

- [Docs: Parity Wallet](https://wiki.parity.io/Parity-Wallet)
- [Docs: How to customize Parity UI?](https://wiki.parity.io/FAQ-Customize-Parity-UI.html)
- [Github: Parity-JS](https://github.com/parity-js)

### Introducing the Wasm VM

We are excited to announce support for **Wasm Smart Contracts on Kovan network**. The hard-fork to activate the Wasm-VM will take place on block `6_600_000`.

To enable Wasm contracts on your custom network, just schedule a `wasmActivationTransition` at your favorite block number (e.g., `42`, `666`, or `0xbada55`). To hack your first Wasm smart contracts in Rust, have a look at the [Parity Wasm Tutorials](https://github.com/paritytech/pwasm-tutorial).

Further reading:

- [Docs: WebAssembly (wasm)](https://wiki.parity.io/WebAssembly-Home)
- [Docs: Wasm VM Design](https://wiki.parity.io/WebAssembly-Design)
- [Docs: Wasm tutorials and examples](https://wiki.parity.io/WebAssembly-Links)

### Empty step messages in PoA

To **reduce blockchain bloat, proof-of-authority networks can now enable _empty step messages_ which replace empty blocks**. Each step message will be signed and broadcasted by the issuing authorities, and included and rewarded in the next non-empty block.

To enable empty step messages, set the `emptyStepsTransition` to your favorite block number. You can also specify a maximum number of empty steps with `maximumEmptySteps` in your chain spec.

### Other noteworthy changes

We removed the old database migrations from 2016. In case you upgrade Parity from a really, really old version, you will have to reset your database manually first with `parity <options> db kill`.

We fixed  DELEGATECALL `from` and `to` fields, see [#7166](https://github.com/paritytech/parity/issues/7166).

We reduced the default USD per transaction value to 0.0001. Thanks, @MysticRyuujin!

The Musicoin chain is now enabled with Byzantium features starting at block `2_222_222`.

### Overview of all changes included

The full list of included changes:

- Re-enable signer, even with no UI. ([#8167](https://github.com/paritytech/parity/pull/8167)) ([#8168](https://github.com/paritytech/parity/pull/8168))
  - Re-enable signer, even with no UI.
  - Fix message.
- Beta Backports ([#8136](https://github.com/paritytech/parity/pull/8136))
  - Support parity protocol. ([#8035](https://github.com/paritytech/parity/pull/8035))
  - updater: apply exponential backoff after download failure ([#8059](https://github.com/paritytech/parity/pull/8059))
    - updater: apply exponential backoff after download failure
    - updater: reset backoff on new release
  - Max code size on Kovan ([#8067](https://github.com/paritytech/parity/pull/8067))
    - Enable code size limit on kovan
    - Fix formatting.
  - Limit incoming connections.  ([#8060](https://github.com/paritytech/parity/pull/8060))
    - Limit ingress connections
    - Optimized handshakes logging
  - WASM libraries bump ([#7970](https://github.com/paritytech/parity/pull/7970))
    - update wasmi, parity-wasm, wasm-utils to latest version
    - Update to new wasmi & error handling
    - also utilize new stack limiter
    - fix typo
    - replace dependency url
    - Cargo.lock update
  - add some dos protection ([#8084](https://github.com/paritytech/parity/pull/8084))
  - revert removing blooms ([#8066](https://github.com/paritytech/parity/pull/8066))
  - Revert "fix traces, removed bloomchain crate, closes [#7228](https://github.com/paritytech/parity/issues/7228), closes [#7167](https://github.com/paritytech/parity/issues/7167)"
  - Revert "fixed broken logs ([#7934](https://github.com/paritytech/parity/pull/7934))"
    - fixed broken logs
    - bring back old lock order
    - remove migration v13
    - revert CURRENT_VERSION to 12 in migration.rs
  - more dos protection ([#8104](https://github.com/paritytech/parity/pull/8104))
  - Const time comparison ([#8113](https://github.com/paritytech/parity/pull/8113))
    - Use `subtle::slices_equal` for constant time comparison.
    - Also update the existing version of subtle in `ethcrypto` from 0.1 to 0.5
    - Test specifically for InvalidPassword error.
  - fix trace filter returning returning unrelated reward calls, closes #8070 ([#8098](https://github.com/paritytech/parity/pull/8098))
  - network: init discovery using healthy nodes ([#8061](https://github.com/paritytech/parity/pull/8061))
    - network: init discovery using healthy nodes
    - network: fix style grumble
    - network: fix typo
  - Postpone Kovan hard fork ([#8137](https://github.com/paritytech/parity/pull/8137))
    - ethcore: postpone Kovan hard fork
    - util: update version fork metadata
  - Disable UI by default. ([#8105](https://github.com/paritytech/parity/pull/8105))
  - dapps: update parity-ui dependencies ([#8160](https://github.com/paritytech/parity/pull/8160))
- Probe changes one step deeper ([#8134](https://github.com/paritytech/parity/pull/8134)) ([#8135](https://github.com/paritytech/parity/pull/8135))
- Beta backports ([#8053](https://github.com/paritytech/parity/pull/8053))
  - CI: Fix cargo cache ([#7968](https://github.com/paritytech/parity/pull/7968))
    - Fix cache
    - Only clean locked cargo cache on windows
  - fixed ethstore sign ([#8026](https://github.com/paritytech/parity/pull/8026))
  - fixed parsing ethash seals and verify_block_undordered ([#8031](https://github.com/paritytech/parity/pull/8031))
  - fix for verify_block_basic crashing on invalid transaction rlp ([#8032](https://github.com/paritytech/parity/pull/8032))
  - fix cache & snapcraft CI build ([#8052](https://github.com/paritytech/parity/pull/8052))
  - Add MCIP-6 Byzyantium transition to Musicoin spec ([#7841](https://github.com/paritytech/parity/pull/7841))
    - Add test chain spec for musicoin byzantium testnet
    - Add MCIP-6 Byzyantium transition to Musicoin spec
    - Update mcip6_byz.json
    - ethcore: update musicoin byzantium block number
    - ethcore: update musicoin bootnodes
    - Update musicoin.json
    - More bootnodes.
- Make 1.10 beta ([#8022](https://github.com/paritytech/parity/pull/8022))
  - Make 1.10 beta
  - Fix gitlab builds
- SecretStore: secretstore_generateDocumentKey RPC ([#7864](https://github.com/paritytech/parity/pull/7864))
- SecretStore: ECDSA session for cases when 2*t < N ([#7739](https://github.com/paritytech/parity/pull/7739))
- bump tiny-keccak ([#8019](https://github.com/paritytech/parity/pull/8019))
- Remove un-necessary comment ([#8014](https://github.com/paritytech/parity/pull/8014))
- clean up account fmt::Debug ([#7983](https://github.com/paritytech/parity/pull/7983))
- improve quality of vote_collector module ([#7984](https://github.com/paritytech/parity/pull/7984))
- ExecutedBlock cleanup ([#7991](https://github.com/paritytech/parity/pull/7991))
- Hardware-wallet/usb-subscribe-refactor ([#7860](https://github.com/paritytech/parity/pull/7860))
- remove wildcard imports from views, make tests more idiomatic ([#7986](https://github.com/paritytech/parity/pull/7986))
- moved PerfTimer to a separate crate - "trace-time" ([#7985](https://github.com/paritytech/parity/pull/7985))
- clean up ethcore::spec module imports ([#7990](https://github.com/paritytech/parity/pull/7990))
- rpc: don't include current block in new_block_filter ([#7982](https://github.com/paritytech/parity/pull/7982))
- fix traces, removed bloomchain crate ([#7979](https://github.com/paritytech/parity/pull/7979))
- simplify compression and move it out of rlp crate ([#7957](https://github.com/paritytech/parity/pull/7957))
- removed old migrations ([#7974](https://github.com/paritytech/parity/pull/7974))
- Reject too large packets in snapshot sync. ([#7977](https://github.com/paritytech/parity/pull/7977))
- fixed broken logs ([#7934](https://github.com/paritytech/parity/pull/7934))
- Increase max download limit to 128MB ([#7965](https://github.com/paritytech/parity/pull/7965))
- Calculate proper keccak256/sha3 using parity. ([#7953](https://github.com/paritytech/parity/pull/7953))
- Add changelog for 1.8.10 stable and 1.9.3 beta ([#7947](https://github.com/paritytech/parity/pull/7947))
- kvdb-rocksdb: remove buffered operations when committing transaction ([#7950](https://github.com/paritytech/parity/pull/7950))
- Bump WebSockets ([#7952](https://github.com/paritytech/parity/pull/7952))
- removed redundant Bloom conversions ([#7932](https://github.com/paritytech/parity/pull/7932))
- simplify RefInfo fmt ([#7929](https://github.com/paritytech/parity/pull/7929))
- Kovan WASM fork code ([#7849](https://github.com/paritytech/parity/pull/7849))
- bring back trie and triehash benches ([#7926](https://github.com/paritytech/parity/pull/7926))
- removed redundant PodAccount::new method ([#7928](https://github.com/paritytech/parity/pull/7928))
- removed dummy wrapper structure - LogGroupPosition ([#7922](https://github.com/paritytech/parity/pull/7922))
- spec: Validate required divisor fields are not 0 ([#7933](https://github.com/paritytech/parity/pull/7933))
- simplify Client::filter_traces method ([#7936](https://github.com/paritytech/parity/pull/7936))
- gitlab cache ([#7921](https://github.com/paritytech/parity/pull/7921))
- Fix a division by zero in light client RPC handler ([#7917](https://github.com/paritytech/parity/pull/7917))
- triehash optimisations ([#7920](https://github.com/paritytech/parity/pull/7920))
- removed redundant Blockchain::db method ([#7919](https://github.com/paritytech/parity/pull/7919))
- removed redundant Blockchain::rewind method ([#7918](https://github.com/paritytech/parity/pull/7918))
- Pending transactions subscription ([#7906](https://github.com/paritytech/parity/pull/7906))
- removed redundant otry! macro from ethcore ([#7916](https://github.com/paritytech/parity/pull/7916))
- Make block generator easier to use ([#7888](https://github.com/paritytech/parity/pull/7888))
- ECIP 1041 - Remove Difficulty Bomb ([#7905](https://github.com/paritytech/parity/pull/7905))
- Fix CSP for dapps that require eval. ([#7867](https://github.com/paritytech/parity/pull/7867))
- Fix gitlab ([#7901](https://github.com/paritytech/parity/pull/7901))
- Gitlb snap master patch ([#7900](https://github.com/paritytech/parity/pull/7900))
- fix snap build master ([#7896](https://github.com/paritytech/parity/pull/7896))
- Fix wallet import ([#7873](https://github.com/paritytech/parity/pull/7873))
- Fix snapcraft nightly ([#7884](https://github.com/paritytech/parity/pull/7884))
- Add a timeout for light client sync requests ([#7848](https://github.com/paritytech/parity/pull/7848))
- SecretStore: fixed test ([#7878](https://github.com/paritytech/parity/pull/7878))
- Fix checksums and auto-update push ([#7846](https://github.com/paritytech/parity/pull/7846))
- Forward-port snap fixes ([#7831](https://github.com/paritytech/parity/pull/7831))
- Update gitlab-test.sh ([#7883](https://github.com/paritytech/parity/pull/7883))
- Fix installer binary names for macos and windows ([#7881](https://github.com/paritytech/parity/pull/7881))
- Fix string typo: "develoopment" -> "development" ([#7874](https://github.com/paritytech/parity/pull/7874))
- Update the instructions to install the stable snap ([#7876](https://github.com/paritytech/parity/pull/7876))
- SecretStore: 'broadcast' decryption session ([#7843](https://github.com/paritytech/parity/pull/7843))
- Flush keyfiles. Resolves #7632 ([#7868](https://github.com/paritytech/parity/pull/7868))
- Read registry_address from given block ([#7866](https://github.com/paritytech/parity/pull/7866))
- Clean up docs formatting for Wasm runtime ([#7869](https://github.com/paritytech/parity/pull/7869))
- WASM: Disable internal memory ([#7842](https://github.com/paritytech/parity/pull/7842))
- Update gitlab-build.sh ([#7855](https://github.com/paritytech/parity/pull/7855))
- ethabi version 5 ([#7723](https://github.com/paritytech/parity/pull/7723))
- Light client: randomize the peer we dispatch requests to ([#7844](https://github.com/paritytech/parity/pull/7844))
- Store updater metadata in a single place ([#7832](https://github.com/paritytech/parity/pull/7832))
- Add new EF ropstens nodes. ([#7824](https://github.com/paritytech/parity/pull/7824))
- refactor stratum to remove retain cycle ([#7827](https://github.com/paritytech/parity/pull/7827))
- Bump jsonrpc. ([#7828](https://github.com/paritytech/parity/pull/7828))
- Add binary identifiers and sha256sum to builds ([#7830](https://github.com/paritytech/parity/pull/7830))
- Update references to UI shell & wallet ([#7808](https://github.com/paritytech/parity/pull/7808))
- Adjust storage update evm-style ([#7812](https://github.com/paritytech/parity/pull/7812))
- Updated WASM Runtime & new interpreter (wasmi) ([#7796](https://github.com/paritytech/parity/pull/7796))
- SecretStore: ignore removed authorities when running auto-migration ([#7674](https://github.com/paritytech/parity/pull/7674))
- Fix build ([#7807](https://github.com/paritytech/parity/pull/7807))
- Move js & js-old code to github.com/parity-js ([#7685](https://github.com/paritytech/parity/pull/7685))
- More changelogs :) ([#7782](https://github.com/paritytech/parity/pull/7782))
- Actualized API set in help ([#7790](https://github.com/paritytech/parity/pull/7790))
- Removed obsolete file ([#7788](https://github.com/paritytech/parity/pull/7788))
- Update ropsten bootnodes ([#7776](https://github.com/paritytech/parity/pull/7776))
- CHANGELOG for 1.9.1 and 1.8.8 ([#7775](https://github.com/paritytech/parity/pull/7775))
- Enable byzantium features on non-ethash chains ([#7753](https://github.com/paritytech/parity/pull/7753))
- Fix client not being dropped on shutdown ([#7695](https://github.com/paritytech/parity/pull/7695))
- Filter-out nodes.json ([#7716](https://github.com/paritytech/parity/pull/7716))
- Removes redundant parentheses ([#7721](https://github.com/paritytech/parity/pull/7721))
- Transaction-pool fixes ([#7741](https://github.com/paritytech/parity/pull/7741))
- More visible download link in README.md ([#7707](https://github.com/paritytech/parity/pull/7707))
- Changelog for 1.9.0 ([#7664](https://github.com/paritytech/parity/pull/7664))
- Add scroll when too many accounts ([#7677](https://github.com/paritytech/parity/pull/7677))
- SecretStore: return HTTP 403 (access denied) if consensus is unreachable ([#7656](https://github.com/paritytech/parity/pull/7656))
- Moved StopGaurd to it's own crate ([#7635](https://github.com/paritytech/parity/pull/7635))

## Previous releases

- [CHANGELOG-1.9](docs/CHANGELOG-1.9.md) (_stable_)
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
