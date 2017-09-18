## Parity [v1.7.2](https://github.com/paritytech/parity/releases/tag/v1.7.2) (2017-09-18)

Parity 1.7.2 is a bug-fix release to improve performance and stability. Among others, it addresses the following:

- Byzantium fork support for the Ropsten and Foundation networks.
- Added support for the ConsenSys and Gnosis multi-signature wallets.
- Significantly increased token registry and token balance lookup performance.
- Fixed issues with the health status indicator in the wallet.
- Tweaked warp-sync to quickly catch up with chains fallen back more than 10,000 blocks.
- Fixes to the Chrome extension and macOS installer upgrades.

Full list of included changes:

- Fix output from eth_call. ([#6538](https://github.com/paritytech/parity/pull/6538))
- Ropsten fork ([#6532](https://github.com/paritytech/parity/pull/6532))
- Byzantium updates ([#6529](https://github.com/paritytech/parity/pull/6529))
  - Fix modexp bug: return 0 if base=0 ([#6424](https://github.com/paritytech/parity/pull/6424))
  - Running state test using parity-evm ([#6355](https://github.com/paritytech/parity/pull/6355))
    - Initial version of state tests.
    - Refactor state to support tracing.
    - Unify TransactResult.
    - Add test.
  - Byzantium updates ([#5855](https://github.com/paritytech/parity/pull/5855))
    - EIP-211 updates
    - Benchmarks
    - Blockhash instruction gas cost updated
    - More benches
    - EIP-684
    - EIP-649
    - EIP-658
    - Updated some tests
    - Modexp fixes
    - STATICCALL fixes
    - Pairing fixes
    - More STATICALL fixes
    - Use paritytech/bn
    - Fixed REVERTing of contract creation
    - Fixed more tests
    - Fixed more tests
    - Blockchain tests
    - Enable previously broken tests
    - Transition test
    - Updated tests
    - Fixed modexp reading huge numbers
    - Enabled max_code_size test
    - Review fixes
    - Updated pairing pricing
    - Missing commas (style)
    - Update test.rs
    - Small improvements
    - Eip161abc
- Fix extension detection ([#6452](https://github.com/paritytech/parity/pull/6452)) ([#6524](https://github.com/paritytech/parity/pull/6524))
  - Fix extension detection.
  - Fix mobx quirks.
  - Update submodule.
- Fix detecting hardware wallets. ([#6509](https://github.com/paritytech/parity/pull/6509))
- Allow hardware device reads without lock. ([#6517](https://github.com/paritytech/parity/pull/6517))
- Backports [#6497](https://github.com/paritytech/parity/pull/6497)
  - Fix slow balances ([#6471](https://github.com/paritytech/parity/pull/6471))
    - Update token updates
    - Update token info fetching
    - Update logger
    - Minor fixes to updates and notifications for balances
    - Use Pubsub
    - Fix timeout.
    - Use pubsub for status.
    - Fix signer subscription.
    - Process tokens in chunks.
    - Fix tokens loaded by chunks
    - Dispatch tokens asap
    - Fix chunks processing.
    - Better filter options
    - Parallel log fetching.
    - Fix signer polling.
    - Fix initial block query.
    - Token balances updates : the right(er) way
    - Better tokens info fetching
    - Fixes in token data fetching
    - Only fetch what's needed (tokens)
    - Fix linting issues
    - Update wasm-tests.
    - Fixing balances fetching
    - Fix requests tracking in UI
    - Fix request watching
    - Update the Logger
    - PR Grumbles Fixes
  - Eth_call returns output of contract creations ([#6420](https://github.com/paritytech/parity/pull/6420))
    - Eth_call returns output of contract creations
    - Fix parameters order.
    - Save outputs for light client as well.
  - Don't accept transactions above block gas limit.
  - Expose health status over RPC ([#6274](https://github.com/paritytech/parity/pull/6274))
     - Node-health to a separate crate.
     - Initialize node_health outside of dapps.
     - Expose health over RPC.
     - Bring back 412 and fix JS.
     - Add health to workspace and tests.
     - Fix compilation without default features.
     - Fix borked merge.
     - Revert to generics to avoid virtual calls.
     - Fix node-health tests.
     - Add missing trailing comma.
  - Fixing/removing failing JS tests.
  - Do not activate genesis epoch in immediate transition validator contract ([#6349](https://github.com/paritytech/parity/pull/6349))
  - Fix memory tracing.
  - Add test to cover that.
  - Ensure balances of constructor accounts are kept
  - Test balance of spec-constructed account is kept
- Fix warning spam. [#6369](https://github.com/paritytech/parity/pull/6369)
- Bump to 1.7.2
- Fix eth_call [#6366](https://github.com/paritytech/parity/pull/6366)
- Backporting [#6352](https://github.com/paritytech/parity/pull/6352)
  - Better check the created accounts before showing Startup Wizard [#6331](https://github.com/paritytech/parity/pull/6331)
  - Tweaked snapshot params [#6344](https://github.com/paritytech/parity/pull/6344)
- Increase default gas limit for eth_call [#6337](https://github.com/paritytech/parity/pull/6337)
  - Fix balance increase.
  - Cap gas limit for dapp-originating requests.
- Backports [#6333](https://github.com/paritytech/parity/pull/6333)
  - Overflow check in addition
  - Unexpose methods on UI RPC. [#6295](https://github.com/paritytech/parity/pull/6295)
  - Add more descriptive error when signing/decrypting using hw wallet.
  - Format instant change proofs correctly
  - Propagate stratum submit share error upstream [#6260](https://github.com/paritytech/parity/pull/6260)
  - Updated jsonrpc [#6264](https://github.com/paritytech/parity/pull/6264)
  - Using multiple NTP servers [#6173](https://github.com/paritytech/parity/pull/6173)
    - Small improvements to time estimation.
    - Allow multiple NTP servers to be used.
    - Removing boxing.
    - Update list of servers and add reference.
  - Fix dapps CSP when UI is exposed externally [#6178](https://github.com/paritytech/parity/pull/6178)
    - Allow embeding on any page when ui-hosts=all and fix dev_ui
  - Fix cache path when using --base-path [#6212](https://github.com/paritytech/parity/pull/6212)
  - Bump to v1.7.1
- UI backports [#6332](https://github.com/paritytech/parity/pull/6332)
  - Time should not contribue to overall status. [#6276](https://github.com/paritytech/parity/pull/6276)
  - Add warning to web browser and fix links. [#6232](https://github.com/paritytech/parity/pull/6232)
  - Extension fixes [#6284](https://github.com/paritytech/parity/pull/6284)
    - Fix token symbols in extension.
    - Allow connections from firefox extension.
  - Add support for ConsenSys multisig wallet [#6153](https://github.com/paritytech/parity/pull/6153)
    - First draft of ConsenSys wallet
    - Fix transfer store // WIP Consensys Wallet
    - Rename walletABI JSON file
    - Fix wrong daylimit in wallet modal
    - Confirm/Revoke ConsensysWallet txs
    - Change of settings for the Multisig Wallet
- Update README for beta [#6270](https://github.com/paritytech/parity/pull/6270)
- Fixed macOS installer upgrade [#6221](https://github.com/paritytech/parity/pull/6221)

## Parity [v1.7.0](https://github.com/paritytech/parity/releases/tag/v1.7.0) (2017-07-28)

Parity 1.7.0 is a major release introducing several important features:

- **Experimental [Light client](https://github.com/paritytech/parity/wiki/The-Parity-Light-Protocol-(PIP)) support**. Start Parity with `--light` to enable light mode. Please, note: The wallet UI integration for the light client is not included, yet.
- **Experimental web wallet**. A hosted version of Parity that keeps the keys and signs transactions using your browser storage. Try it at https://wallet.parity.io or run your own with `--public-node`.
- **WASM contract support**. Private networks can run contracts compiled into WASM bytecode. _More information and documentation to follow_.
- **DApps and RPC server merge**. DApp and RPC are now available through a single API endpoint. DApp server related settings are deprecated.
- **Export accounts from the wallet**. Backing up your keys can now simply be managed through the wallet interface.
- **PoA/Kovan validator set contract**. The PoA network validator-set management via smart contract is now supported by warp and, in the near future, light sync.
- **PubSub API**. https://github.com/paritytech/parity/wiki/JSONRPC-Parity-Pub-Sub-module
- **Signer apps for IOS and Android**.

Full list of included changes:

- Backports [#6163](https://github.com/paritytech/parity/pull/6163)
  - Light client improvements ([#6156](https://github.com/paritytech/parity/pull/6156))
    - No seal checking
    - Import command and --no-seal-check for light client
    - Fix eth_call
    - Tweak registry dapps lookup
    - Ignore failed requests to non-server peers
  - Fix connecting to wildcard addresses. ([#6167](https://github.com/paritytech/parity/pull/6167))
  - Don't display an overlay in case the time sync check fails. ([#6164](https://github.com/paritytech/parity/pull/6164))
    - Small improvements to time estimation.
    - Temporarily disable NTP time check by default.
- Light client fixes ([#6148](https://github.com/paritytech/parity/pull/6148)) [#6151](https://github.com/paritytech/parity/pull/6151)
  - Light client fixes
  - Fix memory-lru-cache
  - Clear pending reqs on disconnect
- Filter tokens logs from current block, not genesis ([#6128](https://github.com/paritytech/parity/pull/6128)) [#6141](https://github.com/paritytech/parity/pull/6141)
- Fix QR scanner returning null on confirm [#6122](https://github.com/paritytech/parity/pull/6122)
- Check QR before lowercase ([#6119](https://github.com/paritytech/parity/pull/6119)) [#6120](https://github.com/paritytech/parity/pull/6120)
- Remove chunk to restore from pending set only upon successful import [#6117](https://github.com/paritytech/parity/pull/6117)
- Fixed node address detection on incoming connection [#6094](https://github.com/paritytech/parity/pull/6094)
- Place RETURNDATA behind block number gate [#6095](https://github.com/paritytech/parity/pull/6095)
- Update wallet library binaries [#6108](https://github.com/paritytech/parity/pull/6108)
- Backported wallet fix [#6105](https://github.com/paritytech/parity/pull/6105)
  - Fix initialisation bug. ([#6102](https://github.com/paritytech/parity/pull/6102))
  - Update wallet library modifiers ([#6103](https://github.com/paritytech/parity/pull/6103))
- Place RETURNDATA behind block number gate [#6095](https://github.com/paritytech/parity/pull/6095)
- Fixed node address detection on incoming connection [#6094](https://github.com/paritytech/parity/pull/6094)
- Bump snap version and tweak importing detection logic ([#6079](https://github.com/paritytech/parity/pull/6079)) [#6081](https://github.com/paritytech/parity/pull/6081)
  - bump last tick just before printing info and restore sync detection
  - bump kovan snapshot version
  - Fixed sync tests
  - Fixed rpc tests
- Acquire client report under lock in informant [#6071](https://github.com/paritytech/parity/pull/6071)
- Show busy indicator on Address forget [#6069](https://github.com/paritytech/parity/pull/6069)
- Add CSP for worker-src ([#6059](https://github.com/paritytech/parity/pull/6059)) [#6064](https://github.com/paritytech/parity/pull/6064)
  - Specify worker-src seperately, add blob
  - Upgrade react-qr-scan to latest version
- Set release channel to beta
- Limit transaction queue memory & limit future queue [#6038](https://github.com/paritytech/parity/pull/6038)
- Fix CI build issue [#6050](https://github.com/paritytech/parity/pull/6050)
- New contract PoA sync fixes [#5991](https://github.com/paritytech/parity/pull/5991)
- Fixed link to Multisig Contract Wallet on master [#5984](https://github.com/paritytech/parity/pull/5984)
- Ethcore crate split part 1 [#6041](https://github.com/paritytech/parity/pull/6041)
- Fix status icon [#6039](https://github.com/paritytech/parity/pull/6039)
- Errors & warnings for inappropriate RPCs [#6029](https://github.com/paritytech/parity/pull/6029)
- Add missing CSP for web3.site [#5992](https://github.com/paritytech/parity/pull/5992)
- Remove cargo install --git from README.md [#6037](https://github.com/paritytech/parity/pull/6037)
- Node Health warnings [#5951](https://github.com/paritytech/parity/pull/5951)
- RPC cpu pool [#6023](https://github.com/paritytech/parity/pull/6023)
- Use crates.io dependencies for parity-wasm [#6036](https://github.com/paritytech/parity/pull/6036)
- Add test for loading the chain specs [#6028](https://github.com/paritytech/parity/pull/6028)
- Whitelist APIs for generic Pub-Sub [#5840](https://github.com/paritytech/parity/pull/5840)
- WASM contracts MVP [#5679](https://github.com/paritytech/parity/pull/5679)
- Fix valid QR scan not advancing [#6033](https://github.com/paritytech/parity/pull/6033)
- --reseal-on-uncle [#5940](https://github.com/paritytech/parity/pull/5940)
- Support comments in reserved peers file ([#6004](https://github.com/paritytech/parity/pull/6004)) [#6012](https://github.com/paritytech/parity/pull/6012)
- Add new md tnc [#5937](https://github.com/paritytech/parity/pull/5937)
- Fix output of parity-evm in case of bad instruction [#5955](https://github.com/paritytech/parity/pull/5955)
- Don't send notifications to unsubscribed clients of PubSub [#5960](https://github.com/paritytech/parity/pull/5960)
- Proper light client informant and more verification of imported headers [#5897](https://github.com/paritytech/parity/pull/5897)
- New Kovan bootnodes [#6017](https://github.com/paritytech/parity/pull/6017)
- Use standard paths for Ethash cache [#5881](https://github.com/paritytech/parity/pull/5881)
- Defer code hash calculation. [#5959](https://github.com/paritytech/parity/pull/5959)
- Fix first run wizard. [#6000](https://github.com/paritytech/parity/pull/6000)
- migration to serde 1.0 [#5996](https://github.com/paritytech/parity/pull/5996)
- SecretStore: generating signatures [#5764](https://github.com/paritytech/parity/pull/5764)
- bigint upgraded to version 3.0 [#5986](https://github.com/paritytech/parity/pull/5986)
- config: don't allow dev chain with force sealing option [#5965](https://github.com/paritytech/parity/pull/5965)
- Update lockfile for miniz-sys and gcc [#5969](https://github.com/paritytech/parity/pull/5969)
- Clean up function naming in RPC error module [#5995](https://github.com/paritytech/parity/pull/5995)
- Fix underflow in gas calculation [#5975](https://github.com/paritytech/parity/pull/5975)
- PubSub for parity-js [#5830](https://github.com/paritytech/parity/pull/5830)
- Report whether a peer was kept from `Handler::on_connect` [#5958](https://github.com/paritytech/parity/pull/5958)
- Implement skeleton for transaction index and epoch transition proof PIP messages [#5908](https://github.com/paritytech/parity/pull/5908)
- TransactionQueue improvements [#5917](https://github.com/paritytech/parity/pull/5917)
- constant time HMAC comparison and clarify docs in ethkey [#5952](https://github.com/paritytech/parity/pull/5952)
- Avoid pre-computing jump destinations [#5954](https://github.com/paritytech/parity/pull/5954)
- Upgrade elastic array [#5949](https://github.com/paritytech/parity/pull/5949)
- PoA: Wait for transition finality before applying [#5774](https://github.com/paritytech/parity/pull/5774)
- Logs Pub-Sub [#5705](https://github.com/paritytech/parity/pull/5705)
- Add the command to install the parity snap [#5945](https://github.com/paritytech/parity/pull/5945)
- Reduce unnecessary allocations [#5944](https://github.com/paritytech/parity/pull/5944)
- Clarify confusing messages. [#5935](https://github.com/paritytech/parity/pull/5935)
- Content Security Policy [#5790](https://github.com/paritytech/parity/pull/5790)
- CLI: Export error message and less verbose peer counter. [#5870](https://github.com/paritytech/parity/pull/5870)
- network: make it more explicit about StreamToken and TimerToken [#5939](https://github.com/paritytech/parity/pull/5939)
- sync: make it more idiomatic rust [#5938](https://github.com/paritytech/parity/pull/5938)
- Prioritize accounts over address book [#5909](https://github.com/paritytech/parity/pull/5909)
- Fixing failing compilation of RPC test on master. [#5916](https://github.com/paritytech/parity/pull/5916)
- Empty local middleware, until explicitly requested [#5912](https://github.com/paritytech/parity/pull/5912)
- Cancel propagated TX [#5899](https://github.com/paritytech/parity/pull/5899)
- fix minor race condition in aura seal generation [#5910](https://github.com/paritytech/parity/pull/5910)
- Docs for Pub-Sub, optional parameter for parity_subscribe [#5833](https://github.com/paritytech/parity/pull/5833)
- Fix gas editor doubling-up on gas [#5820](https://github.com/paritytech/parity/pull/5820)
- Information about used paths added to general output block [#5904](https://github.com/paritytech/parity/pull/5904)
- Domain-locked web tokens. [#5894](https://github.com/paritytech/parity/pull/5894)
- Removed panic handlers [#5895](https://github.com/paritytech/parity/pull/5895)
- Latest changes from Rust RocksDB binding merged [#5905](https://github.com/paritytech/parity/pull/5905)
- Adjust keyethereum/secp256 aliasses [#5903](https://github.com/paritytech/parity/pull/5903)
- Keyethereum fs dependency [#5902](https://github.com/paritytech/parity/pull/5902)
- Ethereum Classic Monetary Policy [#5741](https://github.com/paritytech/parity/pull/5741)
- Initial token should allow full access. [#5873](https://github.com/paritytech/parity/pull/5873)
- Fixed account selection for Dapps on public node [#5856](https://github.com/paritytech/parity/pull/5856)
- blacklist bad snapshot manifest hashes upon failure [#5874](https://github.com/paritytech/parity/pull/5874)
- Fix wrongly called timeouts [#5838](https://github.com/paritytech/parity/pull/5838)
- ArchiveDB and other small fixes [#5867](https://github.com/paritytech/parity/pull/5867)
- convert try!() to ? [#5866](https://github.com/paritytech/parity/pull/5866)
- Make config file optional in systemd [#5847](https://github.com/paritytech/parity/pull/5847)
- EIP-116 (214), [#4833](https://github.com/paritytech/parity/issues/4833) [#4851](https://github.com/paritytech/parity/pull/4851)
- all executables are workspace members [#5865](https://github.com/paritytech/parity/pull/5865)
- minor optimizations of the modexp builtin [#5860](https://github.com/paritytech/parity/pull/5860)
- three small commits for HashDB and MemoryDB [#5766](https://github.com/paritytech/parity/pull/5766)
- use rust 1.18's retain to boost the purge performance [#5801](https://github.com/paritytech/parity/pull/5801)
- Allow IPFS server to accept POST requests [#5858](https://github.com/paritytech/parity/pull/5858)
- Dutch i18n from [#5802](https://github.com/paritytech/parity/issues/5802) for master [#5836](https://github.com/paritytech/parity/pull/5836)
- Typos in token deploy dapp ui [#5851](https://github.com/paritytech/parity/pull/5851)
- A CLI flag to allow fast transaction signing when account is unlocked. [#5778](https://github.com/paritytech/parity/pull/5778)
- Removing `additional` field from EVM instructions [#5821](https://github.com/paritytech/parity/pull/5821)
- Don't fail on wrong log decoding [#5813](https://github.com/paritytech/parity/pull/5813)
- Use randomized subscription ids for PubSub [#5756](https://github.com/paritytech/parity/pull/5756)
- Fixed mem write for empty slice [#5827](https://github.com/paritytech/parity/pull/5827)
- Fix party technologies [#5810](https://github.com/paritytech/parity/pull/5810)
- Revert "Fixed mem write for empty slice" [#5826](https://github.com/paritytech/parity/pull/5826)
- Fixed mem write for empty slice [#5825](https://github.com/paritytech/parity/pull/5825)
- Fix JS tests [#5822](https://github.com/paritytech/parity/pull/5822)
- Bump native-tls and openssl crates. [#5817](https://github.com/paritytech/parity/pull/5817)
- Public node using WASM [#5734](https://github.com/paritytech/parity/pull/5734)
- enforce block signer == author field in PoA [#5808](https://github.com/paritytech/parity/pull/5808)
- Fix stack display in evmbin. [#5733](https://github.com/paritytech/parity/pull/5733)
- Disable UI if it's not compiled in. [#5773](https://github.com/paritytech/parity/pull/5773)
- Require phrase confirmation. [#5731](https://github.com/paritytech/parity/pull/5731)
- Duration limit made optional for EthashParams [#5777](https://github.com/paritytech/parity/pull/5777)
- Update Changelog for 1.6.8 [#5798](https://github.com/paritytech/parity/pull/5798)
- Replace Ethcore comany name in T&C and some other places [#5796](https://github.com/paritytech/parity/pull/5796)
- PubSub for IPC. [#5800](https://github.com/paritytech/parity/pull/5800)
- Fix terminology distributed -> decentralized applications [#5797](https://github.com/paritytech/parity/pull/5797)
- Disable compression for RLP strings [#5786](https://github.com/paritytech/parity/pull/5786)
- update the source for the snapcraft package [#5781](https://github.com/paritytech/parity/pull/5781)
- Fixed default UI port for mac installer [#5782](https://github.com/paritytech/parity/pull/5782)
- Block invalid account name creation [#5784](https://github.com/paritytech/parity/pull/5784)
- Update Cid/multihash/ring/tinykeccak [#5785](https://github.com/paritytech/parity/pull/5785)
- use NULL_RLP, remove NULL_RLP_STATIC [#5742](https://github.com/paritytech/parity/pull/5742)
- Blacklist empty phrase account. [#5730](https://github.com/paritytech/parity/pull/5730)
- EIP-211 RETURNDATACOPY and RETURNDATASIZE [#5678](https://github.com/paritytech/parity/pull/5678)
- Bump mio [#5763](https://github.com/paritytech/parity/pull/5763)
- Fixing UI issues after UI server refactor [#5710](https://github.com/paritytech/parity/pull/5710)
- Fix WS server expose issue. [#5728](https://github.com/paritytech/parity/pull/5728)
- Fix local transactions without condition. [#5716](https://github.com/paritytech/parity/pull/5716)
- Bump parity-wordlist. [#5748](https://github.com/paritytech/parity/pull/5748)
- two small changes in evm [#5700](https://github.com/paritytech/parity/pull/5700)
- Evmbin: JSON format printing pre-state. [#5712](https://github.com/paritytech/parity/pull/5712)
- Recover from empty phrase in dev mode [#5698](https://github.com/paritytech/parity/pull/5698)
- EIP-210 BLOCKHASH changes [#5505](https://github.com/paritytech/parity/pull/5505)
- fixes typo [#5708](https://github.com/paritytech/parity/pull/5708)
- Bump rocksdb [#5707](https://github.com/paritytech/parity/pull/5707)
- Fixed --datadir option [#5697](https://github.com/paritytech/parity/pull/5697)
- rpc -> weak to arc [#5688](https://github.com/paritytech/parity/pull/5688)
- typo fix [#5699](https://github.com/paritytech/parity/pull/5699)
- Revamping parity-evmbin [#5696](https://github.com/paritytech/parity/pull/5696)
- Update dependencies and bigint api [#5685](https://github.com/paritytech/parity/pull/5685)
- UI server refactoring [#5580](https://github.com/paritytech/parity/pull/5580)
- Fix from/into electrum in ethkey [#5686](https://github.com/paritytech/parity/pull/5686)
- Add unit tests [#5668](https://github.com/paritytech/parity/pull/5668)
- Guanqun add unit tests [#5671](https://github.com/paritytech/parity/pull/5671)
- Parity-PubSub as a separate API. [#5676](https://github.com/paritytech/parity/pull/5676)
- EIP-140 REVERT opcode [#5477](https://github.com/paritytech/parity/pull/5477)
- Update CHANGELOG for 1.6.7 [#5683](https://github.com/paritytech/parity/pull/5683)
- Updated docs slightly. [#5674](https://github.com/paritytech/parity/pull/5674)
- Fix build [#5684](https://github.com/paritytech/parity/pull/5684)
- Back-references for the on-demand service [#5573](https://github.com/paritytech/parity/pull/5573)
- Dynamically adjust PIP request costs based on gathered data [#5603](https://github.com/paritytech/parity/pull/5603)
- use cargo workspace [#5601](https://github.com/paritytech/parity/pull/5601)
- Latest headers Pub-Sub [#5655](https://github.com/paritytech/parity/pull/5655)
- improved dockerfile builds [#5659](https://github.com/paritytech/parity/pull/5659)
- Adding CLI options: port shift and unsafe expose. [#5677](https://github.com/paritytech/parity/pull/5677)
- Report missing author in Aura [#5583](https://github.com/paritytech/parity/pull/5583)
- typo fix [#5669](https://github.com/paritytech/parity/pull/5669)
- Remove public middleware (temporary) [#5665](https://github.com/paritytech/parity/pull/5665)
- Remove additional polyfill [#5663](https://github.com/paritytech/parity/pull/5663)
- Importing accounts from files. [#5644](https://github.com/paritytech/parity/pull/5644)
- remove the deprecated options in rustfmt.toml [#5616](https://github.com/paritytech/parity/pull/5616)
- Update the Console dapp [#5602](https://github.com/paritytech/parity/pull/5602)
- Create an account for chain=dev [#5612](https://github.com/paritytech/parity/pull/5612)
- Use babel-runtime as opposed to babel-polyfill [#5662](https://github.com/paritytech/parity/pull/5662)
- Connection dialog timestamp info [#5554](https://github.com/paritytech/parity/pull/5554)
- use copy_from_slice instead of for loop [#5647](https://github.com/paritytech/parity/pull/5647)
- Light friendly dapps [#5634](https://github.com/paritytech/parity/pull/5634)
- Add Recover button to Accounts and warnings [#5645](https://github.com/paritytech/parity/pull/5645)
- Update eth_sign docs. [#5631](https://github.com/paritytech/parity/pull/5631)
- Proper signer Pub-Sub for pending requests. [#5594](https://github.com/paritytech/parity/pull/5594)
- Bump bigint to 1.0.5 [#5641](https://github.com/paritytech/parity/pull/5641)
- PoA warp implementation [#5488](https://github.com/paritytech/parity/pull/5488)
- Improve on-demand dispatch and add support for batch requests [#5419](https://github.com/paritytech/parity/pull/5419)
- Use default account for sending transactions [#5588](https://github.com/paritytech/parity/pull/5588)
- Add peer management to the Status tab [#5566](https://github.com/paritytech/parity/pull/5566)
- Add monotonic step transition [#5587](https://github.com/paritytech/parity/pull/5587)
- Decrypting for external accounts. [#5581](https://github.com/paritytech/parity/pull/5581)
- only enable warp sync when engine supports it [#5595](https://github.com/paritytech/parity/pull/5595)
- fix the doc of installing rust [#5586](https://github.com/paritytech/parity/pull/5586)
- Small fixes [#5584](https://github.com/paritytech/parity/pull/5584)
- SecretStore: remove session on master node [#5545](https://github.com/paritytech/parity/pull/5545)
- run-clean [#5607](https://github.com/paritytech/parity/pull/5607)
- relicense RLP to MIT/Apache2 [#5591](https://github.com/paritytech/parity/pull/5591)
- Fix eth_sign signature encoding. [#5597](https://github.com/paritytech/parity/pull/5597)
- Check pending request on Node local transactions [#5564](https://github.com/paritytech/parity/pull/5564)
- Add tooltips on ActionBar [#5562](https://github.com/paritytech/parity/pull/5562)
- Can't deploy without compiling Contract [#5593](https://github.com/paritytech/parity/pull/5593)
- Add a warning when node is syncing [#5565](https://github.com/paritytech/parity/pull/5565)
- Update registry middleware [#5585](https://github.com/paritytech/parity/pull/5585)
- Set block condition to BigNumber in MethodDecoding [#5592](https://github.com/paritytech/parity/pull/5592)
- Load the sources immediately in Contract Dev [#5575](https://github.com/paritytech/parity/pull/5575)
- Remove formal verification messages in Dev Contract [#5574](https://github.com/paritytech/parity/pull/5574)
- Fix event params decoding when no names for parameters [#5567](https://github.com/paritytech/parity/pull/5567)
- Do not convert to Dates twice [#5563](https://github.com/paritytech/parity/pull/5563)
- Fix Multisig wallet settings [#5560](https://github.com/paritytech/parity/pull/5560)
- Typo [#5547](https://github.com/paritytech/parity/pull/5547)
- Generic PubSub implementation [#5456](https://github.com/paritytech/parity/pull/5456)
- Fix CI paths. [#5570](https://github.com/paritytech/parity/pull/5570)
- reorg into blocks before minimum history [#5558](https://github.com/paritytech/parity/pull/5558)
- EIP-86 update [#5506](https://github.com/paritytech/parity/pull/5506)
- Secretstore RPCs + integration [#5439](https://github.com/paritytech/parity/pull/5439)
- Fixes Parity Bar position [#5557](https://github.com/paritytech/parity/pull/5557)
- Fixes invalid log in BadgeReg events [#5556](https://github.com/paritytech/parity/pull/5556)
- Fix issues in Contract Development view [#5555](https://github.com/paritytech/parity/pull/5555)
- Added missing methods [#5542](https://github.com/paritytech/parity/pull/5542)
- option to disable persistent txqueue [#5544](https://github.com/paritytech/parity/pull/5544)
- Bump jsonrpc [#5552](https://github.com/paritytech/parity/pull/5552)
- Retrieve block headers only for header-only info [#5480](https://github.com/paritytech/parity/pull/5480)
- add snap to CI [#5519](https://github.com/paritytech/parity/pull/5519)
- Pass additional data when reporting [#5527](https://github.com/paritytech/parity/pull/5527)
- Calculate post-constructors state root in spec at load time [#5523](https://github.com/paritytech/parity/pull/5523)
- Fix utf8 decoding [#5533](https://github.com/paritytech/parity/pull/5533)
- Add CHANGELOG.md [#5513](https://github.com/paritytech/parity/pull/5513)
- Change all occurrences of ethcore.io into parity.io [#5528](https://github.com/paritytech/parity/pull/5528)
- Memory usage optimization [#5526](https://github.com/paritytech/parity/pull/5526)
- Compose transaction RPC. [#5524](https://github.com/paritytech/parity/pull/5524)
- Support external eth_sign  [#5481](https://github.com/paritytech/parity/pull/5481)
- Treat block numbers as strings, not BigNums. [#5449](https://github.com/paritytech/parity/pull/5449)
- npm cleanups [#5512](https://github.com/paritytech/parity/pull/5512)
- Export acc js [#4973](https://github.com/paritytech/parity/pull/4973)
- YARN [#5395](https://github.com/paritytech/parity/pull/5395)
- Fix linting issues [#5511](https://github.com/paritytech/parity/pull/5511)
- Chinese Translation [#5460](https://github.com/paritytech/parity/pull/5460)
- Fixing secretstore TODOs - part 2 [#5416](https://github.com/paritytech/parity/pull/5416)
- fix json format of state snapshot [#5504](https://github.com/paritytech/parity/pull/5504)
- Bump jsonrpc version [#5489](https://github.com/paritytech/parity/pull/5489)
- Groundwork for generalized warp sync [#5454](https://github.com/paritytech/parity/pull/5454)
- Add the packaging metadata to build the parity snap [#5496](https://github.com/paritytech/parity/pull/5496)
- Cancel tx JS [#4958](https://github.com/paritytech/parity/pull/4958)
- EIP-212 (bn128 curve pairing) [#5307](https://github.com/paritytech/parity/pull/5307)
- fix panickers in tree-route [#5479](https://github.com/paritytech/parity/pull/5479)
- Update links to etherscan.io [#5455](https://github.com/paritytech/parity/pull/5455)
- Refresh UI on nodeKind changes, e.g. personal -> public [#5312](https://github.com/paritytech/parity/pull/5312)
- Correct contract address for EIP-86 [#5473](https://github.com/paritytech/parity/pull/5473)
- Force two decimals for USD conversion rate [#5471](https://github.com/paritytech/parity/pull/5471)
- Refactoring of Tokens & Balances [#5372](https://github.com/paritytech/parity/pull/5372)
- Background-repeat round [#5475](https://github.com/paritytech/parity/pull/5475)
- nl i18n updated [#5461](https://github.com/paritytech/parity/pull/5461)
- Show ETH value (even 0) if ETH transfer in transaction list [#5406](https://github.com/paritytech/parity/pull/5406)
- Store the pending requests per network version [#5405](https://github.com/paritytech/parity/pull/5405)
- Use in-memory database for tests [#5451](https://github.com/paritytech/parity/pull/5451)
- WebSockets RPC server [#5425](https://github.com/paritytech/parity/pull/5425)
- Added missing docs [#5452](https://github.com/paritytech/parity/pull/5452)
- Tests and tweaks for public node middleware [#5417](https://github.com/paritytech/parity/pull/5417)
- Fix removal of hash-mismatched files. [#5440](https://github.com/paritytech/parity/pull/5440)
- parity_getBlockHeaderByNumber and LightFetch utility [#5383](https://github.com/paritytech/parity/pull/5383)
- New state tests [#5418](https://github.com/paritytech/parity/pull/5418)
- Fix buffer length for QR code gen. [#5447](https://github.com/paritytech/parity/pull/5447)
- Add raw hash signing [#5423](https://github.com/paritytech/parity/pull/5423)
- Filters and block RPCs for the light client [#5320](https://github.com/paritytech/parity/pull/5320)
- Work around mismatch for QR checksum [#5374](https://github.com/paritytech/parity/pull/5374)
- easy to use conversion from and to string for ethstore::Crypto [#5437](https://github.com/paritytech/parity/pull/5437)
- Tendermint fixes [#5415](https://github.com/paritytech/parity/pull/5415)
- Adrianbrink lightclientcache branch. [#5428](https://github.com/paritytech/parity/pull/5428)
- Add caching to HeaderChain struct [#5403](https://github.com/paritytech/parity/pull/5403)
- Add decryption to the UI (in the Signer) [#5422](https://github.com/paritytech/parity/pull/5422)
- Add CIDv0 RPC [#5414](https://github.com/paritytech/parity/pull/5414)
- Updating documentation for RPCs [#5392](https://github.com/paritytech/parity/pull/5392)
- Fixing secretstore TODOs - part 1 [#5386](https://github.com/paritytech/parity/pull/5386)
- Fixing disappearing content. [#5399](https://github.com/paritytech/parity/pull/5399)
- Snapshot chunks packed by size [#5318](https://github.com/paritytech/parity/pull/5318)
- APIs wildcards and simple arithmetic. [#5402](https://github.com/paritytech/parity/pull/5402)
- Fixing compilation without dapps. [#5410](https://github.com/paritytech/parity/pull/5410)
- Don't use port 8080 anymore [#5397](https://github.com/paritytech/parity/pull/5397)
- Quick'n'dirty CLI for the light client [#5002](https://github.com/paritytech/parity/pull/5002)
- set gas limit before proving transactions [#5401](https://github.com/paritytech/parity/pull/5401)
- Public node: perf and fixes [#5390](https://github.com/paritytech/parity/pull/5390)
- Straight download path in the readme [#5393](https://github.com/paritytech/parity/pull/5393)
- On-chain ACL checker for secretstore [#5015](https://github.com/paritytech/parity/pull/5015)
- Allow empty-encoded values from QR encoding [#5385](https://github.com/paritytech/parity/pull/5385)
- Update npm build for new inclusions [#5381](https://github.com/paritytech/parity/pull/5381)
- Fix for Ubuntu Dockerfile [#5356](https://github.com/paritytech/parity/pull/5356)
- Secretstore over network [#4974](https://github.com/paritytech/parity/pull/4974)
- Dapps and RPC server merge [#5365](https://github.com/paritytech/parity/pull/5365)
- trigger js build release [#5379](https://github.com/paritytech/parity/pull/5379)
- Update expanse json with fork at block 600000 [#5351](https://github.com/paritytech/parity/pull/5351)
- Futures-based native wrappers for contract ABIs [#5341](https://github.com/paritytech/parity/pull/5341)
- Kovan warp sync fixed [#5337](https://github.com/paritytech/parity/pull/5337)
- Aura eip155 validation transition [#5362](https://github.com/paritytech/parity/pull/5362)
- Shared wordlist for brain wallets [#5331](https://github.com/paritytech/parity/pull/5331)
- Allow signing via Qr [#4881](https://github.com/paritytech/parity/pull/4881)
- Allow entry of url or hash for DappReg meta [#5360](https://github.com/paritytech/parity/pull/5360)
- Adjust tx overlay colours [#5353](https://github.com/paritytech/parity/pull/5353)
- Add ability to disallow API subscriptions [#5366](https://github.com/paritytech/parity/pull/5366)
- EIP-213 (bn128 curve operations) [#4999](https://github.com/paritytech/parity/pull/4999)
- Fix analize output file name [#5357](https://github.com/paritytech/parity/pull/5357)
- Add default eip155 validation [#5346](https://github.com/paritytech/parity/pull/5346)
- Add new seed nodes for Classic chain [#5345](https://github.com/paritytech/parity/pull/5345)
- Shared wordlist for frontend [#5336](https://github.com/paritytech/parity/pull/5336)
- fix rpc tests [#5338](https://github.com/paritytech/parity/pull/5338)
- Public node with accounts and signing in Frontend [#5304](https://github.com/paritytech/parity/pull/5304)
- Rename Status/Status -> Status/NodeStatus [#5332](https://github.com/paritytech/parity/pull/5332)
- Updating paths to repos. [#5330](https://github.com/paritytech/parity/pull/5330)
- Separate status for canceled local transactions. [#5319](https://github.com/paritytech/parity/pull/5319)
- Cleanup the Status View [#5317](https://github.com/paritytech/parity/pull/5317)
- Update UI minimised requests [#5324](https://github.com/paritytech/parity/pull/5324)
- Order signer transactions FIFO [#5321](https://github.com/paritytech/parity/pull/5321)
- updating dependencies [#5028](https://github.com/paritytech/parity/pull/5028)
- Minimise transactions progress [#4942](https://github.com/paritytech/parity/pull/4942)
- Fix eth_sign showing as wallet account [#5309](https://github.com/paritytech/parity/pull/5309)
- Ropsten revival [#5302](https://github.com/paritytech/parity/pull/5302)
- Strict validation transitions [#4988](https://github.com/paritytech/parity/pull/4988)
- Fix default list sorting [#5303](https://github.com/paritytech/parity/pull/5303)
- Use unique owners for multisig wallets [#5298](https://github.com/paritytech/parity/pull/5298)
- Copy all existing i18n strings into zh (as-is translation aid) [#5305](https://github.com/paritytech/parity/pull/5305)
- Fix booleans in Typedinput [#5295](https://github.com/paritytech/parity/pull/5295)
- node kind RPC [#5025](https://github.com/paritytech/parity/pull/5025)
- Fix the use of MobX in playground [#5294](https://github.com/paritytech/parity/pull/5294)
- Fine grained snapshot chunking [#5019](https://github.com/paritytech/parity/pull/5019)
- Add lint:i18n to find missing & extra keys [#5290](https://github.com/paritytech/parity/pull/5290)
- Scaffolding for zh translations, including first-round by @btceth [#5289](https://github.com/paritytech/parity/pull/5289)
- JS package bumps [#5287](https://github.com/paritytech/parity/pull/5287)
- Auto-extract new i18n strings (update) [#5288](https://github.com/paritytech/parity/pull/5288)
- eip100b [#5027](https://github.com/paritytech/parity/pull/5027)
- Set earliest era in snapshot restoration [#5021](https://github.com/paritytech/parity/pull/5021)
- Avoid clogging up tmp when updater dir has bad permissions. [#5024](https://github.com/paritytech/parity/pull/5024)
- Resilient warp sync [#5018](https://github.com/paritytech/parity/pull/5018)
- Create webpack analysis files (size) [#5009](https://github.com/paritytech/parity/pull/5009)
- Dispatch an open event on drag of Parity Bar [#4987](https://github.com/paritytech/parity/pull/4987)
- Various installer and tray apps fixes [#4970](https://github.com/paritytech/parity/pull/4970)
- Export account RPC [#4967](https://github.com/paritytech/parity/pull/4967)
- Switching ValidatorSet [#4961](https://github.com/paritytech/parity/pull/4961)
- Implement PIP messages, request builder, and handlers [#4945](https://github.com/paritytech/parity/pull/4945)
- auto lint [#5003](https://github.com/paritytech/parity/pull/5003)
- Fix FireFox overflows [#5000](https://github.com/paritytech/parity/pull/5000)
- Show busy indicator, focus first field in password change [#4997](https://github.com/paritytech/parity/pull/4997)
- Consistent store naming in the Signer components [#4996](https://github.com/paritytech/parity/pull/4996)
- second (and last) part of rlp refactor [#4901](https://github.com/paritytech/parity/pull/4901)
- Double click to select account creation type [#4986](https://github.com/paritytech/parity/pull/4986)
- Fixes to the Registry dapp [#4984](https://github.com/paritytech/parity/pull/4984)
- Extend api.util [#4979](https://github.com/paritytech/parity/pull/4979)
- Updating JSON-RPC crates [#4934](https://github.com/paritytech/parity/pull/4934)
- splitting part of util into smaller crates [#4956](https://github.com/paritytech/parity/pull/4956)
- Updating syntex et al [#4983](https://github.com/paritytech/parity/pull/4983)
- EIP198 and built-in activation [#4926](https://github.com/paritytech/parity/pull/4926)
- Fix MethodDecoding for Arrays [#4977](https://github.com/paritytech/parity/pull/4977)
- Try to fix WS race condition connection [#4976](https://github.com/paritytech/parity/pull/4976)
- eth_sign where account === undefined [#4964](https://github.com/paritytech/parity/pull/4964)
- Fix references to api outside of `parity.js` [#4981](https://github.com/paritytech/parity/pull/4981)
- Fix Password Dialog form overflow [#4968](https://github.com/paritytech/parity/pull/4968)
- Changing Mutex into RwLock for transaction queue [#4951](https://github.com/paritytech/parity/pull/4951)
- Disable max seal period for external sealing [#4927](https://github.com/paritytech/parity/pull/4927)
- Attach hardware wallets already in addressbook [#4912](https://github.com/paritytech/parity/pull/4912)
- rlp serialization refactor [#4873](https://github.com/paritytech/parity/pull/4873)
- Bump nanomsg [#4965](https://github.com/paritytech/parity/pull/4965)
- Fixed multi-chunk ledger transactions on windows [#4960](https://github.com/paritytech/parity/pull/4960)
- Fix outputs in Contract Constant Queries [#4953](https://github.com/paritytech/parity/pull/4953)
- systemd: Start parity after network.target [#4952](https://github.com/paritytech/parity/pull/4952)
- Remove transaction RPC [#4949](https://github.com/paritytech/parity/pull/4949)
- Swap out ethcore.io url for parity.io [#4947](https://github.com/paritytech/parity/pull/4947)
- Don't remove confirmed requests to early. [#4933](https://github.com/paritytech/parity/pull/4933)
- Ensure sealing work enabled in miner once subscribers added [#4930](https://github.com/paritytech/parity/pull/4930)
- Add z-index to small modals as well [#4923](https://github.com/paritytech/parity/pull/4923)
- Bump nanomsg [#4946](https://github.com/paritytech/parity/pull/4946)
- Bumping multihash and libc [#4943](https://github.com/paritytech/parity/pull/4943)
- Edit ETH value, gas and gas price in Contract Deployment [#4919](https://github.com/paritytech/parity/pull/4919)
- Add ability to configure Secure API [#4922](https://github.com/paritytech/parity/pull/4922)
- Add Token image from URL [#4916](https://github.com/paritytech/parity/pull/4916)
- Use the registry fee in Token Deployment dapp [#4915](https://github.com/paritytech/parity/pull/4915)
- Add reseal max period [#4903](https://github.com/paritytech/parity/pull/4903)
- Detect rust compiler version in Parity build script, closes 4742 [#4907](https://github.com/paritytech/parity/pull/4907)
- Add Vaults logic to First Run [#4914](https://github.com/paritytech/parity/pull/4914)
- Updated gcc and rayon crates to remove outdated num_cpus dependency [#4909](https://github.com/paritytech/parity/pull/4909)
- Renaming evm binary to avoid conflicts. [#4899](https://github.com/paritytech/parity/pull/4899)
- Better error handling for traces RPC [#4849](https://github.com/paritytech/parity/pull/4849)
- Safari SectionList fix [#4895](https://github.com/paritytech/parity/pull/4895)
- Safari Dialog scrolling fix [#4893](https://github.com/paritytech/parity/pull/4893)
- Spelling :) [#4900](https://github.com/paritytech/parity/pull/4900)
- Additional kovan params [#4892](https://github.com/paritytech/parity/pull/4892)
- trigger js-precompiled build [#4898](https://github.com/paritytech/parity/pull/4898)
- Recalculate receipt roots in close_and_lock [#4884](https://github.com/paritytech/parity/pull/4884)
- Reload UI on network switch [#4864](https://github.com/paritytech/parity/pull/4864)
- Update parity-ui-precompiled with branch [#4850](https://github.com/paritytech/parity/pull/4850)
- OSX Installer is no longer experimental [#4882](https://github.com/paritytech/parity/pull/4882)
- Chain-selection from UI [#4859](https://github.com/paritytech/parity/pull/4859)
- removed redundant (and unused) FromJson trait [#4871](https://github.com/paritytech/parity/pull/4871)
- fix typos and grammar [#4880](https://github.com/paritytech/parity/pull/4880)
- Remove old experimental remote-db code [#4872](https://github.com/paritytech/parity/pull/4872)
- removed redundant FixedHash trait, fixes [#4029](https://github.com/paritytech/parity/issues/4029) [#4866](https://github.com/paritytech/parity/pull/4866)
- Reference JSON-RPC more changes-friendly [#4870](https://github.com/paritytech/parity/pull/4870)
- Better handling of Solidity compliation [#4860](https://github.com/paritytech/parity/pull/4860)
- Go through contract links in Transaction List display [#4863](https://github.com/paritytech/parity/pull/4863)
- Fix Gas Price Selector Tooltips [#4865](https://github.com/paritytech/parity/pull/4865)
- Fix auto-updater [#4867](https://github.com/paritytech/parity/pull/4867)
- Make the UI work offline [#4861](https://github.com/paritytech/parity/pull/4861)
- Subscribe to accounts info in Signer / ParityBar [#4856](https://github.com/paritytech/parity/pull/4856)
- Don't link libsnappy explicitly [#4841](https://github.com/paritytech/parity/pull/4841)
- Fix paste in Inputs [#4854](https://github.com/paritytech/parity/pull/4854)
- Extract i18n from shared UI components [#4834](https://github.com/paritytech/parity/pull/4834)
- Fix paste in Inputs [#4844](https://github.com/paritytech/parity/pull/4844)
- Pull contract deployment title from available steps [#4848](https://github.com/paritytech/parity/pull/4848)
- Supress USB error message [#4839](https://github.com/paritytech/parity/pull/4839)
- Fix getTransactionCount in --geth mode [#4837](https://github.com/paritytech/parity/pull/4837)
- CI: test coverage (for core and js) [#4832](https://github.com/paritytech/parity/pull/4832)
- Lowering threshold for transactions above gas limit [#4831](https://github.com/paritytech/parity/pull/4831)
- Fix TxViewer when no `to` (contract deployment) [#4847](https://github.com/paritytech/parity/pull/4847)
- Fix method decoding [#4845](https://github.com/paritytech/parity/pull/4845)
- Add React Hot Reload to dapps + TokenDeploy fix [#4846](https://github.com/paritytech/parity/pull/4846)
- Dapps show multiple times in some cases [#4843](https://github.com/paritytech/parity/pull/4843)
- Fixes to the Registry dapp [#4838](https://github.com/paritytech/parity/pull/4838)
- Show token icons on list summary pages [#4826](https://github.com/paritytech/parity/pull/4826)
- Calibrate step before rejection [#4800](https://github.com/paritytech/parity/pull/4800)
- Add replay protection [#4808](https://github.com/paritytech/parity/pull/4808)
- Better icon on windows [#4804](https://github.com/paritytech/parity/pull/4804)
- Better logic for contract deployments detection [#4821](https://github.com/paritytech/parity/pull/4821)
- Fix wrong default values for contract queries inputs [#4819](https://github.com/paritytech/parity/pull/4819)
- Adjust selection colours/display [#4811](https://github.com/paritytech/parity/pull/4811)
- Update the Wallet Library Registry key [#4817](https://github.com/paritytech/parity/pull/4817)
- Update Wallet to new Wallet Code [#4805](https://github.com/paritytech/parity/pull/4805)

## Parity [v1.6.10](https://github.com/paritytech/parity/releases/tag/v1.6.10) (2017-07-25)

This is a hotfix release for the stable channel addressing the recent [multi-signature wallet vulnerability](https://blog.parity.io/security-alert-high-2/). Note, upgrading is not mandatory, and all future multi-sig wallets created by any version of Parity are secure.

All Changes:

- Backports for stable [#6116](https://github.com/paritytech/parity/pull/6116)
  - Remove chunk to restore from pending set only upon successful import [#6112](https://github.com/paritytech/parity/pull/6112)
  - Blacklist bad snapshot manifest hashes upon failure [#5874](https://github.com/paritytech/parity/pull/5874)
  - Bump snap version and tweak importing detection logic [#6079](https://github.com/paritytech/parity/pull/6079) (modified to work)
- Fix docker build for stable [#6118](https://github.com/paritytech/parity/pull/6118)
- Update wallet library binaries [#6108](https://github.com/paritytech/parity/pull/6108)
- Backported wallet fix [#6104](https://github.com/paritytech/parity/pull/6104)
  - Fix initialisation bug. ([#6102](https://github.com/paritytech/parity/pull/6102))
  - Update wallet library modifiers ([#6103](https://github.com/paritytech/parity/pull/6103))
- Bump to v1.6.10

## Parity [v1.6.9](https://github.com/paritytech/parity/releases/tag/v1.6.9) (2017-07-16)

This is a first stable release of 1.6 series. It contains a number of minor fixes and introduces the `--reseal-on-uncles` option for miners.

Full list of changes:

- Backports [#6061](https://github.com/paritytech/parity/pull/6061)
  - Ethereum Classic Monetary Policy [#5741](https://github.com/paritytech/parity/pull/5741)
    - Update rewards for uncle miners for ECIP1017
    - Fix an off-by-one error in ECIP1017 era calculation
    - `ecip1017_era_rounds` missing from EthashParams when run in build bot
    - strip out ecip1017_eras_block_reward function and add unit test
  - JS precompiled set to stable
- Backports [#6060](https://github.com/paritytech/parity/pull/6060)
  - --reseal-on-uncle [#5940](https://github.com/paritytech/parity/pull/5940)
    - Optimized uncle check
    - Additional uncle check
    - Updated comment
  - Bump to v1.6.9
  - CLI: Export error message and less verbose peer counter. [#5870](https://github.com/paritytech/parity/pull/5870)
    - Removed numbed of active connections from informant
    - Print error message when fatdb is required
    - Remove peers from UI

## Parity [v1.6.8](https://github.com/paritytech/parity/releases/tag/v1.6.8) (2017-06-08)

This release addresses:

- a rare condition where quickly creating a new account was generating an account not matching the recovery phrase.
- compressed RLP strings caused wrong/empty transaction receipts on Classic network.
- blacklisting the _empty phrase_ account from UI and RPC on non-development chains. See also [this blog post](https://blog.parity.io/restoring-blank-seed-phrase/).
- canceling transactions that didn't have a condition.
- the updated Expanse fork block and chain ID.

Full changelog:

- Backporting to beta [#5791](https://github.com/paritytech/parity/pull/5791)
  - Bump to v1.6.8
  - Update expanse json with fork at block 600000 [#5351](https://github.com/paritytech/parity/pull/5351)
    - Update expanse json with fork at block 600000
    - Update exp chainID to 2
  - Bumped mio [#5763](https://github.com/paritytech/parity/pull/5763)
  - Fixed default UI port for mac installer [#5782](https://github.com/paritytech/parity/pull/5782)
  - Blacklist empty phrase account. [#5730](https://github.com/paritytech/parity/pull/5730)
  - Update Cid/multihash/ring/tinykeccak [#5785](https://github.com/paritytech/parity/pull/5785)
    - Updating ring,multihash,tiny-keccak
    - Updating CID in ipfs.
  - Disable compression for RLP strings [#5786](https://github.com/paritytech/parity/pull/5786)
- Beta Backports [#5789](https://github.com/paritytech/parity/pull/5789)
  - Fix local transactions without condition. [#5716](https://github.com/paritytech/parity/pull/5716)
  - Block invalid account name creation [#5784](https://github.com/paritytech/parity/pull/5784)
    - Additional non-empty phrase check (fromNew)
    - Explicit canCreate check in create (not only on UI)
    - BN instance check (fixes Geth imports)
    - Fixup tests after better checks
  - Recover from empty phrase in dev mode [#5698](https://github.com/paritytech/parity/pull/5698)
    - Add dev chain to isTest
    - Fix signer
    - Fix no condition transactions
    - Fix case: old parity
    - Fix propTypes.

## Parity [v1.6.7](https://github.com/paritytech/parity/releases/tag/v1.6.7) (2017-05-18)

This release addresses:

- potential usability issues with [import and recovery of existing accounts](https://blog.parity.io/restoring-blank-seed-phrase/).
- canceling scheduled transactions via RPC or UI.
- warp sync issues with the Kovan network.

Full changelog:

- Backporting to beta [#5657](https://github.com/paritytech/parity/pull/5657)
  - Add CHANGELOG.md [#5513](https://github.com/paritytech/parity/pull/5513)
  - Reorg into blocks before minimum history [#5558](https://github.com/paritytech/parity/pull/5558)
  - Bump to v1.6.7
- Cancel Transaction [#5656](https://github.com/paritytech/parity/pull/5656)
  - option to disable persistent txqueue [#5544](https://github.com/paritytech/parity/pull/5544)
  - Remove transaction RPC [#4949](https://github.com/paritytech/parity/pull/4949)
  - Cancel tx JS [#4958](https://github.com/paritytech/parity/pull/4958)
  - Updating documentation for RPCs [#5392](https://github.com/paritytech/parity/pull/5392)
- Backport Recover button [#5654](https://github.com/paritytech/parity/pull/5654)
  - Backport [#5645](https://github.com/paritytech/parity/pull/5645)
- Add monotonic step to Kovan [#5630](https://github.com/paritytech/parity/pull/5630)
  - Add monotonic transition to kovan [#5587](https://github.com/paritytech/parity/pull/5587)
- Fix ethsign [#5600](https://github.com/paritytech/parity/pull/5600)
- Registry backports [#5445](https://github.com/paritytech/parity/pull/5445)
  - Fixes to the Registry dapp [#4984](https://github.com/paritytech/parity/pull/4984)
  - Fix references to api outside of `parity.js` [#4981](https://github.com/paritytech/parity/pull/4981)

## Parity [v1.6.6](https://github.com/paritytech/parity/releases/tag/v1.6.6) (2017-04-11)

This release brings warp sync support for kovan network.

- Beta Backports [#5434](https://github.com/paritytech/parity/pull/5434)
  - Bump to v1.6.6
  - Strict validation transitions [#4988](https://github.com/paritytech/parity/pull/4988)
    - Ability to make validation stricter
    - Fix consensus
    - Remove logger
  - Fix eth_sign showing as wallet account [#5309](https://github.com/paritytech/parity/pull/5309)
    - DefaultProps for account
    - Pass signing account
    - Update tests for Connect(...)
  - Add new seed nodes [#5345](https://github.com/paritytech/parity/pull/5345)
  - Kovan warp sync fixed
- Aura eip155 validation transition [#5363](https://github.com/paritytech/parity/pull/5363)
  - Add eip155 validation
  - Add transition block
- Default eip155 validation [#5350](https://github.com/paritytech/parity/pull/5350)
- Backport syntax libs update [#5316](https://github.com/paritytech/parity/pull/5316)

## Parity [v1.6.5](https://github.com/paritytech/parity/releases/tag/v1.6.5) (2017-03-28)

This release contains the following changes:

- Warp sync snapshot format improvements.
- Fix for Firefox UI issues.
- Fix for restoring from a file snapshot.
- Fix for auto-updater error handling.
- Updated configuration for [Ropsten revival](https://github.com/ethereum/ropsten/blob/master/revival.md). Make sure to delete old Ropsten blockchain first with `parity db kill --chain ropsten`. After that you can sync normally with `parity --chain ropsten`.

Full changes:

- Beta Backports [#5299](https://github.com/paritytech/parity/pull/5299)
  - Fix FireFox overflows [#5000](https://github.com/paritytech/parity/pull/5000)
    - Max width for container
    - Set min-width
  - Switching ValidatorSet [#4961](https://github.com/paritytech/parity/pull/4961)
    - Add multi validator set
    - Nicer comment
    - Validate in constructor
    - Reporting
  - Avoid clogging up tmp when updater dir has bad permissions. [#5024](https://github.com/paritytech/parity/pull/5024)
  - Force earliest era set in snapshot restore [#5021](https://github.com/paritytech/parity/pull/5021)
  - Bumb to v1.6.5
  - Fine grained snapshot chunking
  - Ropsten revival
- Fix validator contract syncing [#4789](https://github.com/paritytech/parity/pull/4789) [#5011](https://github.com/paritytech/parity/pull/5011)
  - Make validator set aware of various states
  - Fix updater build
  - Clean up contract call
  - Failing sync test
  - Adjust tests
  - Nicer indent
  - Revert bound divisor

## Parity [v1.5.12](https://github.com/paritytech/parity/releases/tag/v1.5.12) (2017-03-27)

Stable release that adds support for a new warp sync snapshot format.

- Stable Backports [#5297](https://github.com/paritytech/parity/pull/5297)
  - Bump to v1.5.12
  - Fine grained snapshot chunking

## Parity [v1.6.4](https://github.com/paritytech/parity/releases/tag/v1.6.4) (2017-03-22)

A number of issues fixed in this release:

- Ledger device connectivity issues for some users on Windows.
- Improved vault usability.
- Stratum mining no longer requires `--force-sealing`.
- `evm` binary has been renamed to `parity-evm` to avoid conflict with cpp-ethereum package.

Full Changes:

- Backporting to beta [#4995](https://github.com/paritytech/parity/pull/4995)
  - Bump to v1.6.4
  - Ensure sealing work enabled if notifier registed
  - Fix condition check
  - Always send full chunks [#4960](https://github.com/paritytech/parity/pull/4960)
  - Bump nanomsg [#4965](https://github.com/paritytech/parity/pull/4965)
  - Renaming evm binary to avoid conflicts. [#4899](https://github.com/paritytech/parity/pull/4899)
- Beta UI backports [#4993](https://github.com/paritytech/parity/pull/4993)
  - Update js-precompiled 20170314-121823
  - Attach hardware wallets already in addressbook [#4912](https://github.com/paritytech/parity/pull/4912)
    - Attach hardware wallets already in addressbook
    - Only set values changed
  - Add Vaults logic to First Run [#4894](https://github.com/paritytech/parity/issues/4894) [#4914](https://github.com/paritytech/parity/pull/4914)
  - Add ability to configure Secure API (for [#4885](https://github.com/paritytech/parity/issues/4885)) [#4922](https://github.com/paritytech/parity/pull/4922)
  - Add z-index to small modals as well [#4923](https://github.com/paritytech/parity/pull/4923)
  - Eth_sign where account === undefined [#4964](https://github.com/paritytech/parity/pull/4964)
    - Update for case where account === undefined
    - Update tests to not mask account === undefined
    - Default account = {} where undefined (thanks [@tomusdrw](https://github.com/tomusdrw))
  - Fix Password Dialog forms style issue [#4968](https://github.com/paritytech/parity/pull/4968)


## Parity [v1.6.3](https://github.com/paritytech/parity/releases/tag/v1.6.3) (2017-03-14)

This release fixes issue compatibility with Safari on MacOS.

- Safari fixes [#4902](https://github.com/paritytech/parity/pull/4902)
  - Add intitial max-width to sections
  - Move background z-index to -1

## Parity [v1.5.11](https://github.com/paritytech/parity/releases/tag/v1.5.11) (2017-03-14)

Parity 1.5.11 Includes a patch for a more comprehensive block verification.

- Bump to v1.5.11
- Additional kovan params
- Recalculate receipt roots in close_and_lock
- Bump to v1.5.10

## Parity [v1.6.2](https://github.com/paritytech/parity/releases/tag/v1.6.2) (2017-03-13)

A major release introducing a few new features:

- Revamped UI.
- Account Vaults.
- Support for Ledger hardware wallet devices.
- Stratum protocol for PoW mining.
- A new MacOS installer. Parity for MacOS now includes a Menu Bar icon that allows controlling Parity service.
- Disk backed transaction store. Pending transactions are now saved to disk and won't get lost when Parity is restarted.
- Improved memory management.

See the [blog post](https://blog.parity.io/announcing-parity-1-6/) for more details.

Full Changes:

- Fix auto-updater beta [#4868](https://github.com/paritytech/parity/pull/4868)
- Beta UI backports [#4855](https://github.com/paritytech/parity/pull/4855)
  - Added React Hot Reload to dapps + TokenDeplpoy fix ([#4846](https://github.com/paritytech/parity/pull/4846))
  - Fix method decoding ([#4845](https://github.com/paritytech/parity/pull/4845))
    - Fix contract deployment method decoding in Signer
    - Linting
  - Fix TxViewer when no `to` (contract deployment) ([#4847](https://github.com/paritytech/parity/pull/4847))
    - Added React Hot Reload to dapps + TokenDeplpoy fix
    - Fixes to the LocalTx dapp
    - Don't send the nonce for mined transactions
    - Don't encode empty to values for options
  - Pull steps from actual available steps ([#4848](https://github.com/paritytech/parity/pull/4848))
  - Wait for the value to have changed in the input ([#4844](https://github.com/paritytech/parity/pull/4844))
  - Backport Regsirty changes from [#4589](https://github.com/paritytech/parity/pull/4589)
  - Test fixes for [#4589](https://github.com/paritytech/parity/pull/4589)
- Beta Simple score [#4852](https://github.com/paritytech/parity/pull/4852)
  - Simple score
  - Ignore part of a test
- Backporting to beta [#4840](https://github.com/paritytech/parity/pull/4840)
  - Fixes to the Registry dapp ([#4838](https://github.com/paritytech/parity/pull/4838))
    - Fix wrong ABI methods
    - Fix comparison
  - Bump to v1.6.1
- Show token icons on list summary pages ([#4826](https://github.com/paritytech/parity/pull/4826)) [#4827](https://github.com/paritytech/parity/pull/4827)
  - Adjust balance overlay margins (no jumps)
  - Img only balances, small verifications
  - Invalid tests removed
  - Always wrap display (Thanks [@ngotchac](https://github.com/ngotchac))
  - Update tests to reflect reality
- Beta Engine backports [#4806](https://github.com/paritytech/parity/pull/4806)
  - Calibrate before rejection
  - Change flag name
  - Add eip155
  - Make network_id default
- Beta UI backports [#4823](https://github.com/paritytech/parity/pull/4823)
  - Better logic for contract deployments ([#4821](https://github.com/paritytech/parity/pull/4821))
- Beta UI backports [#4818](https://github.com/paritytech/parity/pull/4818)
  - Update the key ([#4817](https://github.com/paritytech/parity/pull/4817))
  - Adjust selection colours/display ([#4811](https://github.com/paritytech/parity/pull/4811))
    - Adjust selection colours to match with mui
    - allow -> disable (simplify selections)
    - Only use top-border
    - Overlay selection line
    - Slightly more muted unselected
    - Restore address icon
  - Fix default values for contract queries
- Beta UI backports [#4809](https://github.com/paritytech/parity/pull/4809)
  - Update Wallet to new Wallet Code ([#4805](https://github.com/paritytech/parity/pull/4805))
    - Update Wallet Version
    - Update Wallet Library
    - Update Wallets Bytecodes
    - Typo
    - Separate Deploy in Contract API
    - Use the new Wallet ABI // Update wallet code
    - WIP .// Deploy from Wallet
    - Update Wallet contract
    - Contract Deployment for Wallet
    - Working deployments for Single Owned Wallet contracts
    - Linting
    - Create a Wallet from a Wallet
    - Linting
    - Fix Signer transactions // Add Gas Used for transactions
    - Deploy wallet contract fix
    - Fix too high gas estimate for Wallet Contract Deploys
    - Final piece ; deploying from Wallet owned by wallet
    - Update Wallet Code
    - Updated the Wallet Codes
    - Fixing Wallet Deployments
    - Add Support for older wallets
    - Linting
  - SMS Faucet ([#4774](https://github.com/paritytech/parity/pull/4774))
    - Faucet
    - Remove flakey button-index testing
    - Only display faucet when sms verified (mainnet)
    - Simplify availability checks
    - WIP
    - Resuest from verified -> verified
    - Update endpoint, display response text
    - Error icon on errors
    - Parse hash text response
    - Use /api/:address endpoint
    - Hash -> data
    - Adjust sms-certified message
  - Fix SectionList hovering issue ([#4749](https://github.com/paritytech/parity/pull/4749))
    - Fix SectionList Items hover when <3 items
    - Even easier...
  - Lint (new)
- Update ETC bootnodes [#4794](https://github.com/paritytech/parity/pull/4794)
- Update comments and reg ABI [#4787](https://github.com/paritytech/parity/pull/4787)
- Optimize signature for fallback function. [#4780](https://github.com/paritytech/parity/pull/4780)
- Rephrasing token generation screen. [#4777](https://github.com/paritytech/parity/pull/4777)
- Etherscan links based on netVersion identifier [#4772](https://github.com/paritytech/parity/pull/4772)
- Update README.md  [#4762](https://github.com/paritytech/parity/pull/4762)
- Fix invalid props to verification code [#4766](https://github.com/paritytech/parity/pull/4766)
- Extend authority round consensus test [#4756](https://github.com/paritytech/parity/pull/4756)
- Revert last hyper "fix" [#4752](https://github.com/paritytech/parity/pull/4752)
- Vault Management UI (round 3) [#4652](https://github.com/paritytech/parity/pull/4652)
- Update SelectionList indicators [#4736](https://github.com/paritytech/parity/pull/4736)
- Update testnet detection [#4746](https://github.com/paritytech/parity/pull/4746)
- Fix Portal in Portal ESC issue [#4745](https://github.com/paritytech/parity/pull/4745)
- Update wiki [#4743](https://github.com/paritytech/parity/pull/4743)
- Account selector close operations [#4728](https://github.com/paritytech/parity/pull/4728)
- Fix Account Selection in Signer [#4744](https://github.com/paritytech/parity/pull/4744)
- Support both V1 & V2 DataChanged events in registry [#4734](https://github.com/paritytech/parity/pull/4734)
- Add info on forks. [#4733](https://github.com/paritytech/parity/pull/4733)
- Add registry addr [#4732](https://github.com/paritytech/parity/pull/4732)
- UI support for hardware wallets [#4539](https://github.com/paritytech/parity/pull/4539)
- S/delete/forget/ for wallets [#4729](https://github.com/paritytech/parity/pull/4729)
- New chains [#4720](https://github.com/paritytech/parity/pull/4720)
- Enable --warp by default [#4719](https://github.com/paritytech/parity/pull/4719)
- Update Uglify (fix to 2.8.2) to fix binary builds [#4723](https://github.com/paritytech/parity/pull/4723)
- Extract i18n strings in modals/* [#4706](https://github.com/paritytech/parity/pull/4706)
- Provide uncle size where available in RPC [#4713](https://github.com/paritytech/parity/pull/4713)
- EC math functions [#4696](https://github.com/paritytech/parity/pull/4696)
- Add registrar fields [#4716](https://github.com/paritytech/parity/pull/4716)
- Extract i18n strings in views/* [#4695](https://github.com/paritytech/parity/pull/4695)
- Removing network=disable from config files [#4715](https://github.com/paritytech/parity/pull/4715)
- Fast in-place migration for adding and removing column families [#4687](https://github.com/paritytech/parity/pull/4687)
- Display badges on summary view [#4689](https://github.com/paritytech/parity/pull/4689)
- Consistent file uploads [#4699](https://github.com/paritytech/parity/pull/4699)
- Rename https://mkr.market -> https://oasisdex.com [#4701](https://github.com/paritytech/parity/pull/4701)
- Stop copy & clickthrough from list summaries [#4700](https://github.com/paritytech/parity/pull/4700)
- Display ... for address summary overflows [#4691](https://github.com/paritytech/parity/pull/4691)
- Less agressive grayscale/opacity in SelectionList [#4688](https://github.com/paritytech/parity/pull/4688)
- Propagate trie errors upwards from State [#4655](https://github.com/paritytech/parity/pull/4655)
- Generic state backend [#4632](https://github.com/paritytech/parity/pull/4632)
- Enhance dialog layouts (round 1) [#4637](https://github.com/paritytech/parity/pull/4637)
- Vault Management UI (round 2) [#4631](https://github.com/paritytech/parity/pull/4631)
- Fix Portal broad event stopper [#4674](https://github.com/paritytech/parity/pull/4674)
- Custom dev chain presets [#4671](https://github.com/paritytech/parity/pull/4671)
- Max gas limit and min gas price [#4661](https://github.com/paritytech/parity/pull/4661)
- Align list displays with SectionList (UI consistency) [#4621](https://github.com/paritytech/parity/pull/4621)
- Add SelectionList component to DRY up [#4639](https://github.com/paritytech/parity/pull/4639)
- I18n NL linting updates [#4662](https://github.com/paritytech/parity/pull/4662)
- Misc. small UI fixes [#4657](https://github.com/paritytech/parity/pull/4657)
- More CLI settings for IPFS API [#4608](https://github.com/paritytech/parity/pull/4608)
- Fix Tendermint deadlock [#4654](https://github.com/paritytech/parity/pull/4654)
- Nl translations [#4649](https://github.com/paritytech/parity/pull/4649)
- Update transaction condition documentation [#4659](https://github.com/paritytech/parity/pull/4659)
- Bump hyper versions [#4645](https://github.com/paritytech/parity/pull/4645)
- Sane updater [#4658](https://github.com/paritytech/parity/pull/4658)
- Remainder of RPC APIs implemented for the light client [#4594](https://github.com/paritytech/parity/pull/4594)
- Preserve vault meta when changing pwd [#4650](https://github.com/paritytech/parity/pull/4650)
- Fix Geth account import [#4641](https://github.com/paritytech/parity/pull/4641)
- Tweak some checks. [#4633](https://github.com/paritytech/parity/pull/4633)
- Attempt to fix subscribeToEvents test [#4638](https://github.com/paritytech/parity/pull/4638)
- Fix selection value from RadioButtons [#4636](https://github.com/paritytech/parity/pull/4636)
- Convert all remaining Modals to use Portal (UI consistency) [#4625](https://github.com/paritytech/parity/pull/4625)
- Default account selection update [#4609](https://github.com/paritytech/parity/pull/4609)
- Display ETH balance in overlay account selector [#4588](https://github.com/paritytech/parity/pull/4588)
- Fixed minor grammar mistake in readme [#4627](https://github.com/paritytech/parity/pull/4627)
- Extract newly available i18n strings [#4623](https://github.com/paritytech/parity/pull/4623)
- Save pending local transactions in the database [#4566](https://github.com/paritytech/parity/pull/4566)
- Bump CID version to allow compilation on all platforms [#4614](https://github.com/paritytech/parity/pull/4614)
- Vault Management UI (first round) [#4446](https://github.com/paritytech/parity/pull/4446)
- Let Engine decide if it seals internally [#4613](https://github.com/paritytech/parity/pull/4613)
- Show only known accounts/wallets/addresses on Home [#4612](https://github.com/paritytech/parity/pull/4612)
- Proper default accounts RPCs [#4580](https://github.com/paritytech/parity/pull/4580)
- Hash-fetch errors in case upstream returns non-200 [#4599](https://github.com/paritytech/parity/pull/4599)
- Added pending transaction info to eth_getTransactionByHash [#4570](https://github.com/paritytech/parity/pull/4570)
- Secret store - initial version [#4567](https://github.com/paritytech/parity/pull/4567)
- Handle invalid ABI retrieved from address_book gracefully [#4606](https://github.com/paritytech/parity/pull/4606)
- Optimize key directory reloads [#4583](https://github.com/paritytech/parity/pull/4583)
- Revert Double Click on Accounts to close in Signer Bar [#4590](https://github.com/paritytech/parity/pull/4590)
- IPFS MVP [#4545](https://github.com/paritytech/parity/pull/4545)
- Networking fixes [#4563](https://github.com/paritytech/parity/pull/4563)
- Remove eth_compile* RPCs [#4577](https://github.com/paritytech/parity/pull/4577)
- Ledger wallet signing fixed [#4578](https://github.com/paritytech/parity/pull/4578)
- Remove vertx from Webpack config [#4576](https://github.com/paritytech/parity/pull/4576)
- Better display of tags [#4564](https://github.com/paritytech/parity/pull/4564)
- Added vaults support to `ethstore-cli` [#4532](https://github.com/paritytech/parity/pull/4532)
- Fixed font URLs [#4579](https://github.com/paritytech/parity/pull/4579)
- Explicitly set seconds to 0 from selector [#4559](https://github.com/paritytech/parity/pull/4559)
- Fixes evmbin compilation and adding to standard build. [#4561](https://github.com/paritytech/parity/pull/4561)
- Alias for personal_sendTransaction [#4554](https://github.com/paritytech/parity/pull/4554)
- Key derivation in ethstore & rpc [#4515](https://github.com/paritytech/parity/pull/4515)
- Skip OOG check for simple transfers [#4558](https://github.com/paritytech/parity/pull/4558)
- Light Client transaction queue, initial LightDispatcher [#4501](https://github.com/paritytech/parity/pull/4501)
- Fixes BadgeReg Middleware [#4556](https://github.com/paritytech/parity/pull/4556)
- Fix pasting of value in Input fields [#4555](https://github.com/paritytech/parity/pull/4555)
- Tooltips with react-intl [#4549](https://github.com/paritytech/parity/pull/4549)
- Close on double-click for Signer Account selection [#4540](https://github.com/paritytech/parity/pull/4540)
- Signer provenance [#4477](https://github.com/paritytech/parity/pull/4477)
- Fix console dapp [#4544](https://github.com/paritytech/parity/pull/4544)
- Extract i18n string into i18n/_defaults (base of translations) [#4514](https://github.com/paritytech/parity/pull/4514)
- Fix contract queries bug [#4534](https://github.com/paritytech/parity/pull/4534)
- Fixing namespace of couple methods in console. [#4538](https://github.com/paritytech/parity/pull/4538)
- Home landing page [#4178](https://github.com/paritytech/parity/pull/4178)
- Bump JSON RPC crates versions [#4530](https://github.com/paritytech/parity/pull/4530)
- Update rust version in README [#4531](https://github.com/paritytech/parity/pull/4531)
- Lower default pruning history and memory [#4528](https://github.com/paritytech/parity/pull/4528)
- Serde 0.9 [#4508](https://github.com/paritytech/parity/pull/4508)
- Fixes to Token Deploy dapp [#4513](https://github.com/paritytech/parity/pull/4513)
- Fixed receipt decoding [#4521](https://github.com/paritytech/parity/pull/4521)
- Several fixes to the Wallet in general [#4504](https://github.com/paritytech/parity/pull/4504)
- Use the current contract name for Solidity compilation [#4510](https://github.com/paritytech/parity/pull/4510)
- Preparation for Light client RPC [#4485](https://github.com/paritytech/parity/pull/4485)
- Fix Dutch translation [#4509](https://github.com/paritytech/parity/pull/4509)
- Fixed a warning and bumped libusb-sys [#4507](https://github.com/paritytech/parity/pull/4507)
- Fix TnC overflows on small screens [#4505](https://github.com/paritytech/parity/pull/4505)
- Fix no data sent in TxQueue dapp [#4502](https://github.com/paritytech/parity/pull/4502)
- Ledger wallet support [#4486](https://github.com/paritytech/parity/pull/4486)
- Add new Componennt for Token Images [#4498](https://github.com/paritytech/parity/pull/4498)
- Fix address and accounts links [#4491](https://github.com/paritytech/parity/pull/4491)
- Fix Token Reg Dapp issues in Firefox [#4489](https://github.com/paritytech/parity/pull/4489)
- Parity.js interfaces for vaults [#4497](https://github.com/paritytech/parity/pull/4497)
- Initial Dutch translations [#4484](https://github.com/paritytech/parity/pull/4484)
- Fix key.meta.vault for root dir keys && read vault.meta without vault key [#4482](https://github.com/paritytech/parity/pull/4482)
- Arbitrary labels for extended keys (u32, H256 built-in) [#4438](https://github.com/paritytech/parity/pull/4438)
- Fix ethstore build [#4492](https://github.com/paritytech/parity/pull/4492)
- Fixed compilation of ethstore-cli [#4493](https://github.com/paritytech/parity/pull/4493)
- Build embedded Parity JS properly and separatly  [#4426](https://github.com/paritytech/parity/pull/4426)
- Static link for snappy [#4487](https://github.com/paritytech/parity/pull/4487)
- Work with string numbers in contract (Fixes #4472) [#4478](https://github.com/paritytech/parity/pull/4478)
- Metadata support for vaults [#4475](https://github.com/paritytech/parity/pull/4475)
- Sort gas price corpus when hitting genesis [#4470](https://github.com/paritytech/parity/pull/4470)
- Fixing CORS headers for parity.web3.site [#4461](https://github.com/paritytech/parity/pull/4461)
- Make signing compatible with geth. [#4468](https://github.com/paritytech/parity/pull/4468)
- Handle registry not found errors [#4465](https://github.com/paritytech/parity/pull/4465)
- Fix Portal scrolling getting stuck [#4455](https://github.com/paritytech/parity/pull/4455)
- Fix AccountCard stretch to 100% [#4450](https://github.com/paritytech/parity/pull/4450)
- Include total difficulty in CHTs and hide implementation details from consumers [#4428](https://github.com/paritytech/parity/pull/4428)
- Fix RLP encoding for types recursively calling `RlpStream::append` [#4362](https://github.com/paritytech/parity/pull/4362)
- Open popup without attempting inline [#4440](https://github.com/paritytech/parity/pull/4440)
- Fixing histogram again ([#4464](https://github.com/paritytech/parity/issues/4464)) port from beta [#4467](https://github.com/paritytech/parity/pull/4467)
- Vaults RPCs [#4366](https://github.com/paritytech/parity/pull/4366)
- Ethkey - extended keys [#4377](https://github.com/paritytech/parity/pull/4377)
- Use secure websocket from HTTPS clients [#4436](https://github.com/paritytech/parity/pull/4436)
- RPC middleware: Informant & Client.keep_alive [#4384](https://github.com/paritytech/parity/pull/4384)
- Fix eth_sign/parity_postSign [#4432](https://github.com/paritytech/parity/pull/4432)
- Web view with web3.site support [#4313](https://github.com/paritytech/parity/pull/4313)
- Extend Portal component with title, buttons & steps (as per Modal) [#4392](https://github.com/paritytech/parity/pull/4392)
- Extension installation overlay [#4423](https://github.com/paritytech/parity/pull/4423)
- Add block & timestamp conditions to Signer [#4411](https://github.com/paritytech/parity/pull/4411)
- Transaction timestamp condition [#4419](https://github.com/paritytech/parity/pull/4419)
- Poll for defaultAccount to update dapp & overlay subscriptions [#4417](https://github.com/paritytech/parity/pull/4417)
- Validate dapps accounts with address book [#4407](https://github.com/paritytech/parity/pull/4407)
- Dapps use defaultAccount instead of own selectors [#4386](https://github.com/paritytech/parity/pull/4386)
- Fix lock and rename tracing [#4403](https://github.com/paritytech/parity/pull/4403)
- Restarting fetch client every now and then [#4399](https://github.com/paritytech/parity/pull/4399)
- Perform a sync between Rust and JS when generating markdown instead of in spec tests [#4408](https://github.com/paritytech/parity/pull/4408)
- Registry dapp: make lookup use lower case [#4409](https://github.com/paritytech/parity/pull/4409)
- Available Dapp selection alignment with Permissions (Portal) [#4374](https://github.com/paritytech/parity/pull/4374)
- More permissive verification process [#4317](https://github.com/paritytech/parity/pull/4317)
- Fix ParityBar account selection overflows [#4405](https://github.com/paritytech/parity/pull/4405)
- Mac binaries signing [#4397](https://github.com/paritytech/parity/pull/4397)
- Revert "remove [ci skip]" [#4398](https://github.com/paritytech/parity/pull/4398)
- Registry, s/a the owner/the owner/ [#4391](https://github.com/paritytech/parity/pull/4391)
- Fixing invalid address in docs [#4388](https://github.com/paritytech/parity/pull/4388)
- Remove [ci skip] [#4381](https://github.com/paritytech/parity/pull/4381)
- Fixing estimate gas in case histogram is not available [#4387](https://github.com/paritytech/parity/pull/4387)
- Default Account selector in Signer overlay [#4375](https://github.com/paritytech/parity/pull/4375)
- Fixing web3 in console [#4382](https://github.com/paritytech/parity/pull/4382)
- Add parity_defaultAccount RPC (with subscription) [#4383](https://github.com/paritytech/parity/pull/4383)
- Full JSON-RPC docs + sync tests. [#4335](https://github.com/paritytech/parity/pull/4335)
- Expose util as Api.util [#4372](https://github.com/paritytech/parity/pull/4372)
- Dapp Account Selection & Defaults [#4355](https://github.com/paritytech/parity/pull/4355)
- Publish @parity/jsonrpc [#4365](https://github.com/paritytech/parity/pull/4365)
- Fix signing [#4363](https://github.com/paritytech/parity/pull/4363)
- Fixing embedded bar not closing in chrome extension [#4367](https://github.com/paritytech/parity/pull/4367)
- Update AccountCard for re-use [#4350](https://github.com/paritytech/parity/pull/4350)
- Add proper event listener to Portal [#4359](https://github.com/paritytech/parity/pull/4359)
- Optional from field in Transaction Requests [#4332](https://github.com/paritytech/parity/pull/4332)
- Rust 1.14 in README [ci-skip] [#4361](https://github.com/paritytech/parity/pull/4361)
- Fix JournalDB::earliest_era on empty database [#4316](https://github.com/paritytech/parity/pull/4316)
- Fixed race condition deadlock on fetching enode URL [#4354](https://github.com/paritytech/parity/pull/4354)
- Allow Portal to be used as top-level modal [#4338](https://github.com/paritytech/parity/pull/4338)
- Fix postsign [#4347](https://github.com/paritytech/parity/pull/4347)
- Renaming signAndSendTransaction to sendTransaction [#4351](https://github.com/paritytech/parity/pull/4351)
- Add api.util.encodeMethodCall to parity.js [#4330](https://github.com/paritytech/parity/pull/4330)
- Initial commit for vaults [#4312](https://github.com/paritytech/parity/pull/4312)
- Returning default account as coinbase + allow altering sender in signer [#4323](https://github.com/paritytech/parity/pull/4323)
- Persistent tracking of dapps [#4302](https://github.com/paritytech/parity/pull/4302)
- Exposing all RPCs over dapps port as CLI option [#4346](https://github.com/paritytech/parity/pull/4346)
- New macOS App [#4345](https://github.com/paritytech/parity/pull/4345)
- Display QrCode for accounts, addresses & contracts [#4329](https://github.com/paritytech/parity/pull/4329)
- Add QrCode & Copy to ShapeShift [#4322](https://github.com/paritytech/parity/pull/4322)
- Parity.js api.parity.chainStatus should handle { blockGap: null } [#4327](https://github.com/paritytech/parity/pull/4327)
- DeleteAccount & LoadContract modal updates [#4320](https://github.com/paritytech/parity/pull/4320)
- Split Tab from TabBar [#4318](https://github.com/paritytech/parity/pull/4318)
- Contracts interface expansion [#4307](https://github.com/paritytech/parity/pull/4307)
- HistoryStore for tracking relevant routes [#4305](https://github.com/paritytech/parity/pull/4305)
- Split Dapp icon into ui/DappIcon (re-use) [#4308](https://github.com/paritytech/parity/pull/4308)
- Add a Playground for the UI Components [#4301](https://github.com/paritytech/parity/pull/4301)
- Update CreateWallet with FormattedMessage [#4298](https://github.com/paritytech/parity/pull/4298)
- Update dates for new PRs missed [#4306](https://github.com/paritytech/parity/pull/4306)
- EIP-98: Optional transaction state root [#4296](https://github.com/paritytech/parity/pull/4296)
- Fix whitespace [#4299](https://github.com/paritytech/parity/pull/4299)
- Attempt to fix console. [#4294](https://github.com/paritytech/parity/pull/4294)
- Ui/SectionList component [#4292](https://github.com/paritytech/parity/pull/4292)
- Stratum up [#4233](https://github.com/paritytech/parity/pull/4233)
- Logging transaction duration [#4297](https://github.com/paritytech/parity/pull/4297)
- Generic engine utilities [#4258](https://github.com/paritytech/parity/pull/4258)
- JSON-RPC interfaces with documentation [#4276](https://github.com/paritytech/parity/pull/4276)
- Dont decode seal fields [#4263](https://github.com/paritytech/parity/pull/4263)
- Skip misbehaving test until properly fixed [#4283](https://github.com/paritytech/parity/pull/4283)
- Additional logs for own transactions [#4278](https://github.com/paritytech/parity/pull/4278)
- Ensure write lock isn't held when calling handlers [#4285](https://github.com/paritytech/parity/pull/4285)
- Feature selector [#4074](https://github.com/paritytech/parity/pull/4074)
- AccountCreate updates [#3988](https://github.com/paritytech/parity/pull/3988)
- Extended JS interface -> Markdown generator [#4275](https://github.com/paritytech/parity/pull/4275)
- Added 3 warpnodes for ropsten [#4289](https://github.com/paritytech/parity/pull/4289)
- Ledger Communication JS toolkit [#4268](https://github.com/paritytech/parity/pull/4268)
- ValidatorSet reporting [#4208](https://github.com/paritytech/parity/pull/4208)
- Add support for api.subscribe('parity_accountsInfo') [#4273](https://github.com/paritytech/parity/pull/4273)
- Display AccountCard name via IdentityName [#4235](https://github.com/paritytech/parity/pull/4235)
- Dapp visibility save/load tests [#4150](https://github.com/paritytech/parity/pull/4150)
- Fix wrong output format of peers [#4270](https://github.com/paritytech/parity/pull/4270)
- Chain scoring [#4218](https://github.com/paritytech/parity/pull/4218)
- Rust 1.14 for windows builds [#4269](https://github.com/paritytech/parity/pull/4269)
- Eslint formatting updates [#4234](https://github.com/paritytech/parity/pull/4234)
- Embeddable ParityBar [#4222](https://github.com/paritytech/parity/pull/4222)
- Update deb-build.sh to fix libssl dependency [#4260](https://github.com/paritytech/parity/pull/4260)
- Integration with zgp whitelist contract [#4215](https://github.com/paritytech/parity/pull/4215)
- Adjust the location of the signer snippet [#4155](https://github.com/paritytech/parity/pull/4155)
- Fix wrong token handling [#4254](https://github.com/paritytech/parity/pull/4254)
- Additional building-block UI components [#4239](https://github.com/paritytech/parity/pull/4239)
- Bump package.json to 0.3.0 (1.6 track) [#4244](https://github.com/paritytech/parity/pull/4244)
- Disable incoming ETH notifications [#4243](https://github.com/paritytech/parity/pull/4243)
- Memory-based pruning history size [#4114](https://github.com/paritytech/parity/pull/4114)
- Common EngineSigner [#4189](https://github.com/paritytech/parity/pull/4189)
- Verification: don't request a code twice [#4221](https://github.com/paritytech/parity/pull/4221)
- S/Delete Contract/Forget Contract/ [#4237](https://github.com/paritytech/parity/pull/4237)
- Light protocol syncing improvements [#4212](https://github.com/paritytech/parity/pull/4212)
- LES Peer Info [#4195](https://github.com/paritytech/parity/pull/4195)
- Don't panic on uknown git commit hash [#4231](https://github.com/paritytech/parity/pull/4231)
- Cache registry reverses in local storage [#4182](https://github.com/paritytech/parity/pull/4182)
- Update version numbers in README [#4223](https://github.com/paritytech/parity/pull/4223)
- CHT calculations for full nodes [#4181](https://github.com/paritytech/parity/pull/4181)
- Use single source of info for dapp meta (build & display) [#4217](https://github.com/paritytech/parity/pull/4217)
- Non-secure API for DappReg [#4216](https://github.com/paritytech/parity/pull/4216)
- Console now has admin [#4220](https://github.com/paritytech/parity/pull/4220)
- Verification: add mainnet BadgeReg ids [#4190](https://github.com/paritytech/parity/pull/4190)
- Fixing minimal transaction queue price [#4204](https://github.com/paritytech/parity/pull/4204)
- Remove unnecessary Engine method [#4184](https://github.com/paritytech/parity/pull/4184)
- Fixed --base-path on windows [#4193](https://github.com/paritytech/parity/pull/4193)
- Fixing etherscan price parsing [#4202](https://github.com/paritytech/parity/pull/4202)
- LES: Better timeouts + Track failed requests [#4093](https://github.com/paritytech/parity/pull/4093)
- ESLint additional rules [#4186](https://github.com/paritytech/parity/pull/4186)
- JsonRPC bump for IPC fix [#4200](https://github.com/paritytech/parity/pull/4200)
- Poll for upgrades as part of global status (long) [#4197](https://github.com/paritytech/parity/pull/4197)
- Updater fixes [#4196](https://github.com/paritytech/parity/pull/4196)
- Prevent duplicate incoming connections [#4180](https://github.com/paritytech/parity/pull/4180)
- Minor typo to ensure it updates only when synced. [#4188](https://github.com/paritytech/parity/pull/4188)
- Minor refactor for clarity [#4174](https://github.com/paritytech/parity/pull/4174)
- Secret - from hash function, also validate data [#4159](https://github.com/paritytech/parity/pull/4159)
- Gas_limit for blocks, mined by Parity will be divisible by 37 [#4154](https://github.com/paritytech/parity/pull/4154)
- Support HTML5-routed dapps [#4173](https://github.com/paritytech/parity/pull/4173)
- Fix subscribeToEvents test [#4166](https://github.com/paritytech/parity/pull/4166)
- Fix dapps not loading [#4170](https://github.com/paritytech/parity/pull/4170)
- Fix broken token images [#4169](https://github.com/paritytech/parity/pull/4169)
- Bumping hyper [#4167](https://github.com/paritytech/parity/pull/4167)
- Icarus -> update, increase web timeout. [#4165](https://github.com/paritytech/parity/pull/4165)
- Add a password strength component [#4153](https://github.com/paritytech/parity/pull/4153)
- Stop flickering + added loader in AddressSelector [#4149](https://github.com/paritytech/parity/pull/4149)
- On demand LES request [#4036](https://github.com/paritytech/parity/pull/4036)
- Ropsten fork detection [#4163](https://github.com/paritytech/parity/pull/4163)
- Pull in console dapp as builtin [#4145](https://github.com/paritytech/parity/pull/4145)
- Optimized hash lookups [#4144](https://github.com/paritytech/parity/pull/4144)
- UnverifiedTransaction type [#4134](https://github.com/paritytech/parity/pull/4134)
- Verification: check if server is running [#4140](https://github.com/paritytech/parity/pull/4140)
- Remove onSubmit of current (no auto-change on password edit) [#4151](https://github.com/paritytech/parity/pull/4151)
- Trim spaces from InputAddress [#4126](https://github.com/paritytech/parity/pull/4126)
- Don't pop-up notifications after network switch [#4076](https://github.com/paritytech/parity/pull/4076)
- Use estimateGas error (as per updated implementation) [#4131](https://github.com/paritytech/parity/pull/4131)
- Improvements and optimisations to estimate_gas [#4142](https://github.com/paritytech/parity/pull/4142)
- New jsonrpc-core with futures and metadata support [#3859](https://github.com/paritytech/parity/pull/3859)
- Reenable mainnet update server. [#4137](https://github.com/paritytech/parity/pull/4137)
- Temporarily skip failing test [#4138](https://github.com/paritytech/parity/pull/4138)
- Refactor VoteCollector [#4101](https://github.com/paritytech/parity/pull/4101)
- Another minor estimation fix [#4133](https://github.com/paritytech/parity/pull/4133)
- Add proper label to method decoding inputs [#4136](https://github.com/paritytech/parity/pull/4136)
- Remove bindActionCreators({}, dispatch) (empty, unneeded) [#4135](https://github.com/paritytech/parity/pull/4135)
- Better contract error log reporting & handling [#4128](https://github.com/paritytech/parity/pull/4128)
- Fix broken Transfer : total account balance [#4127](https://github.com/paritytech/parity/pull/4127)
- Test harness for lightsync [#4109](https://github.com/paritytech/parity/pull/4109)
- Fix call/estimate_gas [#4121](https://github.com/paritytech/parity/pull/4121)
- Fixing decoding ABI with signatures in names [#4125](https://github.com/paritytech/parity/pull/4125)
- Get rid of unsafe code in ethkey, propagate incorrect Secret errors. [#4119](https://github.com/paritytech/parity/pull/4119)
- Basic tests for subscribeToEvents [#4115](https://github.com/paritytech/parity/pull/4115)
- Auto-detect hex encoded bytes in sha3 [#4108](https://github.com/paritytech/parity/pull/4108)
- Use binary chop to estimate gas accurately [#4100](https://github.com/paritytech/parity/pull/4100)
- V1.6 in master [#4113](https://github.com/paritytech/parity/pull/4113)
- Ignore get_price_info test by default. [#4112](https://github.com/paritytech/parity/pull/4112)
- Fix wrong information logging [#4106](https://github.com/paritytech/parity/pull/4106)
- Avoid comms with not-yet-active release update server. [#4111](https://github.com/paritytech/parity/pull/4111)
- Update Transfer logic + Better logging [#4098](https://github.com/paritytech/parity/pull/4098)
- Fix Signer : wrong account on reload [#4104](https://github.com/paritytech/parity/pull/4104)
- Cache registry reverses, completion in address selector [#4066](https://github.com/paritytech/parity/pull/4066)
- Validator/authority contract [#3937](https://github.com/paritytech/parity/pull/3937)
- No reorg limit for ancient blocks [#4099](https://github.com/paritytech/parity/pull/4099)
- Update registration after every write [#4102](https://github.com/paritytech/parity/pull/4102)
- Default to no auto-update. [#4092](https://github.com/paritytech/parity/pull/4092)
- Don't remove out of date local transactions [#4094](https://github.com/paritytech/parity/pull/4094)

## Parity [v1.5.9](https://github.com/paritytech/parity/releases/tag/v1.5.9) (2017-03-11)

First stable release of 1.5.x series. This release enables EIP-161 transaction replay protection for PoA networks.

- Bump to v1.5.9
- Fix auto-updater stable [#4869](https://github.com/paritytech/parity/pull/4869)
- Fixing windows build script
- Bump js-precompiled 20170308-152339
- Force js update
- Stable Engine backports [#4807](https://github.com/paritytech/parity/pull/4807)
  - Calibrate before rejection
  - Change flag name
  - Add eip155
  - Fix build
  - Make network_id default
- Switch js branch to stable
- Bump to v1.5.8

## Parity [v1.5.7](https://github.com/paritytech/parity/releases/tag/v1.5.7) (2017-03-07)

This release resolves a single issue with failing auto-updates.

- Update ETC bootnodes [#4794](https://github.com/paritytech/parity/pull/4794)
- Bump to v1.5.7
- Sane updater [#4658](https://github.com/paritytech/parity/pull/4658)
  - Disable if files can't be moved.
  - Make updater avoid downloading earlier versions.

## Parity [v1.5.6](https://github.com/paritytech/parity/releases/tag/v1.5.6) (2017-03-06)

This release among various stability fixes adds support for a new [Kovan](https://github.com/kovan-testnet/proposal) testnet.

See [full list of changes.](https://github.com/paritytech/parity/compare/v1.5.4...v1.5.6):

- Beta Update comments and reg ABI [#4788](https://github.com/paritytech/parity/pull/4788)
  - Update comments.
  - Fix up new ABI.
- Bump to v1.5.6 in beta [#4786](https://github.com/paritytech/parity/pull/4786)
- Beta Optimize signature for fallback function. ([#4780](https://github.com/paritytech/parity/pull/4780)) [#4784](https://github.com/paritytech/parity/pull/4784)
- Beta Add registrar fields ([#4716](https://github.com/paritytech/parity/pull/4716)) [#4781](https://github.com/paritytech/parity/pull/4781)
- Beta Etherscan links ([#4772](https://github.com/paritytech/parity/pull/4772)) [#4778](https://github.com/paritytech/parity/pull/4778)
  - Etherscan links [#4772](https://github.com/paritytech/parity/pull/4772)
    - Use netVersion to determine external links
    - Update additional isTest references
  - Port tests
  - Update address links
  - Signer accountlink isTest
- Beta Fix invalid props [#4767](https://github.com/paritytech/parity/pull/4767)
- Backporting to beta [#4741](https://github.com/paritytech/parity/pull/4741)
  - New chains [#4720](https://github.com/paritytech/parity/pull/4720)
    - Add Kovan chain.
    - Fix up --testnet.
    - Fix tests.
  - Fix to UglifyJS 2.8.2 to fix app build issues [#4723](https://github.com/paritytech/parity/pull/4723)
  - Update classic bootnodes, ref #4717 [#4735](https://github.com/paritytech/parity/pull/4735)
  - Allow failure docker beta
  - Adjust pruning history default to 64 [#4709](https://github.com/paritytech/parity/pull/4709)
  - Backporting from master
    - Update docker-build.sh
    - Update gitlab.ci
    - Fix docker hub build
    - Update gitlab
    - Docker beta-release->latest
    - Add registry.
    - Add info on forks.
    - Fixed spec file
  - Support both V1 & V2 DataChanged events in registry [#4734](https://github.com/paritytech/parity/pull/4734)
    - Add info on forks.
    - Add new registry ABI
    - Import registry2 & fix exports
    - Select ABI based on code hash
    - Render new event types (owner not available)
    - New registry.
    - Rename old chain.
    - Fix test.
    - Another fix.
    - Finish rename.
  - Fixed fonts URLs [#4579](https://github.com/paritytech/parity/pull/4579)
  - Fix Token Reg Dapp issues in Firefox [#4489](https://github.com/paritytech/parity/pull/4489)
    - Fix overflow issues in Firefox [#4348](https://github.com/paritytech/parity/issues/4348)
    - Fix wrong Promise inferance
    - Revert "Add new Componennt for Token Images [#4496](https://github.com/paritytech/parity/issues/4496)"
    - Add new Componennt for Token Images [#4496](https://github.com/paritytech/parity/issues/4496)
  - Add StackEventListener [#4745](https://github.com/paritytech/parity/pull/4745)
  - Update testnet detection [#4746](https://github.com/paritytech/parity/pull/4746)
  - Fix Account Selection in Signer [#4744](https://github.com/paritytech/parity/pull/4744)
    - Can pass FormattedMessage to Input (eg. Status // RPC Enabled)
    - Simple fixed-width fix for Accoutn Selection in Parity Signer
- Beta backports [#4763](https://github.com/paritytech/parity/pull/4763)
  - Https://mkr-market -> https://oasisdex.com [#4701](https://github.com/paritytech/parity/pull/4701)
  - Wallet s/delete/forget/ [#4741](https://github.com/paritytech/parity/pull/4741)
- Update classic bootnodes [#4735](https://github.com/paritytech/parity/pull/4735)
- Engine backports [#4718](https://github.com/paritytech/parity/pull/4718)
  - Custom dev presets
  - Add registrar field
  - Use constructor for dev registrar
  - Fix test
- Beta Adjust pruning history default to 64 [#4709](https://github.com/paritytech/parity/pull/4709)
- Bump to v1.5.5

## Parity [v1.5.4](https://github.com/paritytech/parity/releases/tag/v1.5.4) (2017-02-23)

A couple of issue fixed in this release:

- Parity now allows uncles headers to have timestamp set to arbitrary future value.
- Importing keys from geth is now working again.

Changes:

- Beta Fix Geth account import [#4643](https://github.com/paritytech/parity/pull/4643)
  - Fix Geth import - actually pass addresses through
  - Fix geth accounts not displayed
  - Port saving of returned addresses (master MobX, beta state)
  - Log result -> importGethAccounts
- Beta Backporting ([#4633](https://github.com/paritytech/parity/pull/4633)) [#4640](https://github.com/paritytech/parity/pull/4640)
  - Tweak some checks.
  - Fixed build and added a difficulty test
  - Bump to v1.5.4

## Parity [v1.4.12](https://github.com/paritytech/parity/releases/tag/v1.4.12) (2017-02-22)

This stable release fixes an issue with block uncle validation. Parity now allows uncle headers to have timestamp set to arbitrary future value.

- Stable Backporting ([#4633](https://github.com/paritytech/parity/pull/4633)) [#4642](https://github.com/paritytech/parity/pull/4642)
  - Tweak some checks.
  - Fixed build and added a difficulty test
  - Bump to v1.4.12
- Add missing maxCodeSize [#4585](https://github.com/paritytech/parity/pull/4585)

## Parity [v1.5.3](https://github.com/paritytech/parity/releases/tag/v1.5.3) (2017-02-20)

This is a maintenance release that fixes a number of stability issues. Notably this resolves an issue where Parity would allow a pre EIP-155 transaction into the sealed block.

See [full list of changes](https://github.com/paritytech/parity/compare/v1.5.2...v1.5.3):

- Bump to v1.5.3 [#4611](https://github.com/paritytech/parity/pull/4611)
- Handle invalid ABI retrieved from address_book gracefully ([#4606](https://github.com/paritytech/parity/pull/4606)) [#4610](https://github.com/paritytech/parity/pull/4610)
  - Handle invalid ABI gracefully
  - Also include failed abi in log
- Backporting to beta [#4602](https://github.com/paritytech/parity/pull/4602)
  - Static link for snappy
  - added 3 warpnodes for ropsten ([#4289](https://github.com/paritytech/parity/pull/4289))
  - Fixed indentation
- Validate transaction before adding to the queue [#4600](https://github.com/paritytech/parity/pull/4600)
- Beta backports [#4569](https://github.com/paritytech/parity/pull/4569)
  - Fixing evmbin compilation and added standard build. ([#4561](https://github.com/paritytech/parity/pull/4561))
  - Alias for personal_sendTransaction ([#4554](https://github.com/paritytech/parity/pull/4554))
  - Fix console dapp ([#4544](https://github.com/paritytech/parity/pull/4544))
  - Fixing linting issues. Better support for console as secure app
  - Fixing linting issues
  - Fix no data sent in TxQueue dapp ([#4502](https://github.com/paritytech/parity/pull/4502))
  - Fix wrong PropType req for Embedded Signer
  - Fix wrong data for tx #4499
- Explicitly set seconds to 0 from selector ([#4559](https://github.com/paritytech/parity/pull/4559)) [#4571](https://github.com/paritytech/parity/pull/4571)
  - Explicitly set seconds/milli to 0
  - Use condition time & block setters consistently
  - Fix failing test
  - test for 0 ms & sec
  - It cannot hurt, clone date before setting
  - Prettier date test constants (OCD)
- Remove invalid expectation [#4542](https://github.com/paritytech/parity/pull/4542)
- Skip OOG check for simple transfers [#4558](https://github.com/paritytech/parity/pull/4558) [#4560](https://github.com/paritytech/parity/pull/4560)
 - Skip OOG check for simple transfers [#4558](https://github.com/paritytech/parity/pull/4558)
 - Fix failing test

## Parity [v1.4.11](https://github.com/paritytech/parity/releases/tag/v1.4.11) (2017-02-17)

This release corrects the Ropsten chain specification file.

- Bump to v1.4.11 [#4587](https://github.com/paritytech/parity/pull/4587)
- Fixing etherscan price parsing ([#4202](https://github.com/paritytech/parity/pull/4202)) [#4209](https://github.com/paritytech/parity/pull/4209)
 - Fixing etherscan price parsing
 - Handling all errors
- Removed pdbs
- Add missing maxCodeSize [#4585](https://github.com/paritytech/parity/pull/4585)

## Parity [v1.5.2](https://github.com/paritytech/parity/releases/tag/v1.5.2) (2017-02-08)

This release brings a few stability fixes along with a feature that allows queuing transactions that are activated and send out on selected date or block number.
- Debian packages have been updated to require `libssl1.0.0` for better compatibility.
- eth_sign (and parity_postSign) used to return concatenated r ++ s ++ v with v being 0 or 1. it now agrees with geth as v ++ r ++ s with v being 27 or 28.

Parity Wallet
- Accounts & ShapeShift integration now displays QR code for scanning from mobile wallets
- Dapp integration now allows for the selection of available accounts and the setting of the default account
- Transaction creation now allows for the selection of future blocks or timestamps after which the transaction is released

Parity Extension
- First release of the Parity Extension, allowing for Parity integration from web-based dapps

See [full list of changes](https://github.com/paritytech/parity/compare/v1.5.0...v1.5.2):
- Work with string numbers in contract (Fixes #4472) ([#4478](https://github.com/paritytech/parity/pull/4478)) [#4480](https://github.com/paritytech/parity/pull/4480)
- Eth_sign improvements backport [#4473](https://github.com/paritytech/parity/pull/4473)
  - Fix postsign ([#4347](https://github.com/paritytech/parity/pull/4347))
  - Fix whitespace.
  - Fix post sign.
  - Fix message.
  - Fix tests.
  - Rest of the problems.
  - All hail the linter and its omniscience.
  - ...and its divine omniscience.
  - Grumbles and wording.
  - Make signing compatible with geth. ([#4468](https://github.com/paritytech/parity/pull/4468))
- Sort gas price corpus when hitting genesis [#4471](https://github.com/paritytech/parity/pull/4471)
- Wallet dev chain fix [#4466](https://github.com/paritytech/parity/pull/4466)
- Fixing histogram again [#4464](https://github.com/paritytech/parity/pull/4464)
- Beta backports [#4462](https://github.com/paritytech/parity/pull/4462)
  - Support HTML5-routed dapps ([#4173](https://github.com/paritytech/parity/pull/4173))
  - Fix compilation on latest nightly
  - Updating precompiled
- Fix Portal scrolling getting stuck [#4456](https://github.com/paritytech/parity/pull/4456)
  - Fix Portal scrolling getting stuck
  - DappCard container flex
  - Container height to 100%
- Fix AccountCard stretch to 100% [#4451](https://github.com/paritytech/parity/pull/4451)
- Fix wrong output format of peers ([#4270](https://github.com/paritytech/parity/pull/4270)) [#4442](https://github.com/paritytech/parity/pull/4442)
  - Fix wrong output format of peers
  - Add outPeer tests
- Opening extension page without inline installation [#4441](https://github.com/paritytech/parity/pull/4441)
  - Open popup without attempting inline
  - Cater for all .web3.site addresses
- Fix svg extension image webpack inlining [#4437](https://github.com/paritytech/parity/pull/4437)
- Backporting to beta [#4434](https://github.com/paritytech/parity/pull/4434)
  - Bump to v1.5.2
  - Fix eth_sign/parity_postSign ([#4432](https://github.com/paritytech/parity/pull/4432))
  - Fix dispatch for signing.
  - Remove console log
  - Fix signing & tests.
- Returning default account as coinbase [#4431](https://github.com/paritytech/parity/pull/4431)
  - Returning first address as coinbase
  - Allowing sender alteration in signer
  - Adding default account RPC
- UI updates for 1.5.1 [#4429](https://github.com/paritytech/parity/pull/4429)
  - S/Delete Contract/Forget Contract/ ([#4237](https://github.com/paritytech/parity/pull/4237))
  - Adjust the location of the signer snippet ([#4155](https://github.com/paritytech/parity/pull/4155))
  - Additional building-block UI components ([#4239](https://github.com/paritytech/parity/pull/4239))
  - Currency WIP
  - Expand tests
  - Pass className
  - Add QrCode
  - Export new components in ~/ui
  - S/this.props.netSymbol/netSymbol/
  - Fix import case
  - Ui/SectionList component ([#4292](https://github.com/paritytech/parity/pull/4292))
  - Array chunking utility
  - Add SectionList component
  - Add TODOs to indicate possible future work
  - Add missing overlay style (as used in dapps at present)
  - Add a Playground for the UI Components ([#4301](https://github.com/paritytech/parity/pull/4301))
  - Playground // WIP
  - Linting
  - Add Examples with code
  - CSS Linting
  - Linting
  - Add Connected Currency Symbol
  - 2015-2017
  - Added `renderSymbol` tests
  - PR grumbles
  - Add Eth and Btc QRCode examples
  - 2015-2017
  - Add tests for playground
  - Fixing tests
  - Split Dapp icon into ui/DappIcon ([#4308](https://github.com/paritytech/parity/pull/4308))
  - Add QrCode & Copy to ShapeShift ([#4322](https://github.com/paritytech/parity/pull/4322))
  - Extract CopyIcon to ~/ui/Icons
  - Add copy & QrCode address
  - Default size 4
  - Add bitcoin: link
  - Use protocol links applicable to coin exchanged
  - Remove .only
  - Display QrCode for accounts, addresses & contracts ([#4329](https://github.com/paritytech/parity/pull/4329))
  - Allow Portal to be used as top-level modal ([#4338](https://github.com/paritytech/parity/pull/4338))
  - Portal
  - Allow Portal to be used in as both top-level and popover
  - Modal/popover variable naming
  - Export Portal in ~/ui
  - Properly handle optional onKeyDown
  - Add simple Playground Example
  - Add proper event listener to Portal ([#4359](https://github.com/paritytech/parity/pull/4359))
  - Display AccountCard name via IdentityName ([#4235](https://github.com/paritytech/parity/pull/4235))
  - Fix signing ([#4363](https://github.com/paritytech/parity/pull/4363))
  - Dapp Account Selection & Defaults ([#4355](https://github.com/paritytech/parity/pull/4355))
  - Add parity_defaultAccount RPC (with subscription) ([#4383](https://github.com/paritytech/parity/pull/4383))
  - Default Account selector in Signer overlay ([#4375](https://github.com/paritytech/parity/pull/4375))
  - Typo, fixes #4271 ([#4391](https://github.com/paritytech/parity/pull/4391))
  - Fix ParityBar account selection overflows ([#4405](https://github.com/paritytech/parity/pull/4405))
  - Available Dapp selection alignment with Permissions (Portal) ([#4374](https://github.com/paritytech/parity/pull/4374))
  - Registry dapp: make lookup use lower case ([#4409](https://github.com/paritytech/parity/pull/4409))
  - Dapps use defaultAccount instead of own selectors ([#4386](https://github.com/paritytech/parity/pull/4386))
  - Poll for defaultAccount to update dapp & overlay subscriptions ([#4417](https://github.com/paritytech/parity/pull/4417))
  - Poll for defaultAccount (Fixes #4413)
  - Fix nextTimeout on catch
  - Store timers
  - Re-enable default updates on change detection
  - Add block & timestamp conditions to Signer ([#4411](https://github.com/paritytech/parity/pull/4411))
  - Extension installation overlay ([#4423](https://github.com/paritytech/parity/pull/4423))
  - Extension installation overlay
  - Pr gumbles
  - Spelling
  - Update Chrome URL
  - Fix for non-included jsonrpc
  - Extend Portal component (as per Modal) [#4392](https://github.com/paritytech/parity/pull/4392)
- Transaction timestamp condition [#4427](https://github.com/paritytech/parity/pull/4427)
- Fixing embedded bar not closing in chrome extension [#4421](https://github.com/paritytech/parity/pull/4421)
- Backporting to beta [#4418](https://github.com/paritytech/parity/pull/4418)
  - Bump to 1.5.1
  - Disable notifications ([#4243](https://github.com/paritytech/parity/pull/4243))
  - Fix wrong token handling ([#4254](https://github.com/paritytech/parity/pull/4254))
  - Fixing wrong token displayed
  - Linting
  - Revert filtering out
  - Revert the revert
  - Don't panic on uknown git commit hash ([#4231](https://github.com/paritytech/parity/pull/4231))
  - Additional logs for own transactions ([#4278](https://github.com/paritytech/parity/pull/4278))
  - Integration with zgp whitelist contract ([#4215](https://github.com/paritytech/parity/pull/4215))
  - Zgp-transactions checker
  - Polishing
  - Rename + refactor
  - Refuse-service-transactions cl option
  - Fixed tests compilation
  - Renaming signAndSendTransaction to sendTransaction ([#4351](https://github.com/paritytech/parity/pull/4351))
  - Fixed deadlock in external_url ([#4354](https://github.com/paritytech/parity/pull/4354))
  - Fixing web3 in console ([#4382](https://github.com/paritytech/parity/pull/4382))
  - Fixing estimate gas in case histogram is not available ([#4387](https://github.com/paritytech/parity/pull/4387))
  - Restarting fetch client every now and then ([#4399](https://github.com/paritytech/parity/pull/4399))
- Embeddable ParityBar ([#4222](https://github.com/paritytech/parity/pull/4222)) [#4287](https://github.com/paritytech/parity/pull/4287)
  - Embeddable ParityBar
  - Replacing storage with store
  - Fixing  references.
  - Addressing style issues
  - Supporting parity background

## Parity [v1.5.0: "Nativity"](https://github.com/paritytech/parity/releases/tag/v1.5.0) (2017-01-19)

Major feature release including _Tendermint_ consensus engine, _Multisig wallet_ support, _badge/certification_ UI integration and _automatic updates_.

Directories:

- New XDG-informed Parity data directory structure. Base dir (`--base-path` or `-d`) that defaulted to `$HOME/.parity` is changed to:
  - `/Users/You/AppData/Roaming/Parity/Ethereum` on Windows
  - `/Users/you/Library/Application Support/io.parity.ethereum` on MacOS
  - `/home/you/.local/share/parity` on Linux/Unix
- Keys are now stored in chain-specific directories . On first run of 1.5, all keys will be moved into the key's directory of the chain you run. You'll need to move the wallet files between directories manually if you wish to split them between testnet/mainnet.
- `--db-path` option now controls the path just for the databases, not for keys (`--keys-path`) or dapps (`--dapps-path`).

Basics:

- Version tracking, consensus-protection, hypervised auto-updating:
  - Parity will ensure syncing is paused if its version cannot support an upcoming hard-fork (disable with `--no-consensus`).
  - Parity will automatically download the latest version and may be updated through Parity Wallet (disable with `--no-download`)
  - Parity can automatically update and seamlessly restart to later versions in the same release track (enable with `--auto-update=all` or `--auto-update=critical`).
  - Parity hypervisor will automatically run the latest version (disable with `--force-direct`).
- Fat database; to enable, sync the chain with the option `--fat-db`.
  - Accounts and storage entries can be enumerated.
  - Chain state can be exported to JSON for analysis with `parity export state`.
- CLI and config options renamed: all variants of `--signer` are renamed to `--ui`.
- Log files are appended by default rather than truncated (useful for daemon deployments).

Parity Wallet:

- Multisig wallet support: "New Wallet" button in the "Accounts" section allows you to create a new multisig wallet or import an existing one.
- Solidity compiler: "Develop Contract" button in the "Contracts" section allows you to write, edit, compile and deploy contracts.
- SMS & e-mail verification: Accounts can now be certified as verified using Parity's SMS and e-Mail verification/registration oracle.
- Badge/certification integration: The `BadgeReg` contract can be used to deploy additional certifications.
- Local transaction propagation tracking: "TxQueue Viewer" in the "Applications" section allows you to track and resubmit previously sent transactions.
- Contract executions can now have gas and gas-price configured.
- Signer can now alter the gas and gas-price of transactions at password-entry.
- The deprecated Chrome "Signer" extension is now incompatible.

[Proof of Authority](https://github.com/paritytech/parity/wiki/Proof-of-Authority-Chains):

- Authority Round consensus engine: `engine: authorityRound {...}`; this is a high-performance Proof-of-Authority consensus engine. It is not BFT under normal circumstances (however the `--force-sealing` flag can be used to ensure consensus even with Byzantine nodes).
- Tendermint Engine: `engine: tendermint {...}`; this is an experimental Proof-of-Authority consensus engine. BFT up to one third of the authorities and falling back to delayed finalization chain ordering (50% fault tolerant).
- Generic seal JSON spec includes engine-specific types (`seal: { generic: { rlp: "0x..." } }` becomes `seal: { authority_round { step: 0, signature: "0x..." } }`.
- To set a node as authority either `--engine-signer ADDRESS` should be used with `--password` or `parity_setEngineSigner(address, password)` RPC should be called. Unlocking the account permanently or using `--author` is now unnecessary.
- Set of authorities can now be specified using a [list or a contract](https://github.com/paritytech/parity/wiki/Consensus-Engines#validator-engines).

Chains:

- Dev chain: `--chain=dev`; instant seal engine (no mining needed). Great for development work.
- Ropsten chain (`--chain=ropsten` or `--chain=testnet`) configures for Ropsten, the new test net.
- Morden chain (`--chain=morden`) changed to "Classic" rules and stays as the Ethereum Classic test net.

RPCs/APIs:

- All JSON-RPC interfaces have strict JSON deserialization - no extra fields are allowed.
- `eth_sign` RPC now hashes given data instead of getting the hash.
- `signer_confirmRequestWithToken`: additional RPC for signing transactions with a rotating token, alleviating the need for keeping an account password in memory.
- `eth_signTransaction` now conforms to the specification, `eth_submitTransaction` is introduced.

Full changes:

- Backporting to beta [#4211](https://github.com/paritytech/parity/pull/4211)
  - JsonRPC bump for IPC fix
  - Fixing etherscan price parsing ([#4202](https://github.com/paritytech/parity/pull/4202))
  - Handling all errors
  - Fixed --base-path on windows ([#4193](https://github.com/paritytech/parity/pull/4193))
  - Add support for optional args with default text
  - Fixing minimal transaction queue price ([#4204](https://github.com/paritytech/parity/pull/4204))
  - Fixing tests
  - verification: add mainnet BadgeReg ids ([#4190](https://github.com/paritytech/parity/pull/4190))
  - verification: fetch contracts by name
  - verification: better wording
  - typo
  - reregistered badges
  - Console now has admin ([#4220](https://github.com/paritytech/parity/pull/4220))
  - Fixes [#4210](https://github.com/paritytech/parity/pull/4210)
  - Non-secure for DappReg ([#4216](https://github.com/paritytech/parity/pull/4216))
- Backporting to beta [#4203](https://github.com/paritytech/parity/pull/4203)
  - Minor typo to ensure it updates only when synced. ([#4188](https://github.com/paritytech/parity/pull/4188))
  - Updater fixes ([#4196](https://github.com/paritytech/parity/pull/4196))
  - Minor typo to ensure it updates only when synced.
  - Fix deadlock.
  - Skip unneeded arg in making list.
  - Allow auto-restart even when not running an update.
  - Fix trace.
  - Update update info on each loop.
  - Fix build.
  - Shutdown all sockets
  - Remove superfluous use.
  - Poll for upgrades as part of global status (long) ([#4197](https://github.com/paritytech/parity/pull/4197))
  - Fix path
  - Prevent duplicate incoming connections ([#4180](https://github.com/paritytech/parity/pull/4180))
- Gas_limit for blocks, mined by Parity will be divisible by 37 ([#4154](https://github.com/paritytech/parity/pull/4154)) [#4176](https://github.com/paritytech/parity/pull/4176)
  - gas_limit for new blocks will divide evenly by 13
  - increased PARITY_GAS_LIMIT_DETERMINANT to 37
  - separate method for marking mined block
  - debug_asserts(gas_limit within protocol range)
  - round_block_gas_limit method is now static
  - made round_block_gas_limit free-function
  - multiplier->multiple
- Backporting to beta [#4175](https://github.com/paritytech/parity/pull/4175)
  - verification: check if server is running ([#4140](https://github.com/paritytech/parity/pull/4140))
  - verification: check if server is running
  - See also ethcore/email-verification#67c6466 and ethcore/sms-verification#a585e42.
  - verification: show in the UI if server is running
  - verification: code style ✨, more i18n
  - fix i18n key
  - Optimized hash lookups ([#4144](https://github.com/paritytech/parity/pull/4144))
  - Optimize hash comparison
  - Use libc
  - Ropsten fork detection ([#4163](https://github.com/paritytech/parity/pull/4163))
  - Stop flickering + added loader in AddressSelector ([#4149](https://github.com/paritytech/parity/pull/4149))
  - Stop UI flickering + added loader to AddressSelector [#4103](https://github.com/paritytech/parity/pull/4103)
  - PR Grumbles
  - Add a password strength component ([#4153](https://github.com/paritytech/parity/pull/4153))
  - Added new PasswordStrength Component
  - Added tests
  - PR Grumbles
  - icarus -> update, increase web timeout. ([#4165](https://github.com/paritytech/parity/pull/4165))
  - Fix estimate gas
  - Fix token images // Error in Contract Queries ([#4169](https://github.com/paritytech/parity/pull/4169))
  - Fix dapps not loading ([#4170](https://github.com/paritytech/parity/pull/4170))
  - Add secure to dappsreg
  - Remove trailing slash // fix dapps
- Bumping hyper [#4168](https://github.com/paritytech/parity/pull/4168)
  - Bumping hyper
  - Bumping again
- Backporting to beta [#4158](https://github.com/paritytech/parity/pull/4158)
  - Remove onSubmit of current (no auto-change on password edit) ([#4151](https://github.com/paritytech/parity/pull/4151))
  - Remove onSubmit from current password
  - Remove onSubmit from hint
  - Pull in console dapp as builtin ([#4145](https://github.com/paritytech/parity/pull/4145))
  - Copy static dapps from static (no build)
  - Console sources
  - Add console to builtins
  - Remove console assets
  - Disable eslint on console.js
  - Enable eslint after disable
  - Webpack copy
- Backporting to beta [#4152](https://github.com/paritytech/parity/pull/4152)
  - Fix broken transfer total balance ([#4127](https://github.com/paritytech/parity/pull/4127))
  - Add proper label to method decoding inputs ([#4136](https://github.com/paritytech/parity/pull/4136))
  - Another minor estimation fix ([#4133](https://github.com/paritytech/parity/pull/4133))
  - Return 0 instead of error with out of gas on estimate_gas
  - Fix stuff up.
  - Another estimate gas fix.
  - Alter balance to maximum possible rather than GP=0.
  - Only increase to amount strictly necessary.
  - Get rid of unsafe code in ethkey, propagate incorrect Secret errors. ([#4119](https://github.com/paritytech/parity/pull/4119))
  - Implementing secret
  - Fixing tests
  - Refactor VoteCollector ([#4101](https://github.com/paritytech/parity/pull/4101))
  - dir
  - simple validator list
  - stub validator contract
  - make the engine hold Weak<Client> instead of IoChannel
  - validator set factory
  - register weak client with ValidatorContract
  - check chain security
  - add address array to generator
  - register provider contract
  - update validator set on notify
  - add validator contract spec
  - simple list test
  - split update and contract test
  - contract change
  - use client in tendermint
  - fix deadlock
  - step duration in params
  - adapt tendermint tests
  - add storage fields to test spec
  - constructor spec
  - execute under wrong address
  - create under correct address
  - revert
  - validator contract constructor
  - move genesis block lookup
  - add removal ability to contract
  - validator contract adding validators
  - fix basic authority
  - validator changing test
  - more docs
  - update sync tests
  - remove env_logger
  - another env_logger
  - cameltoe
  - hold EngineClient instead of Client
  - return error on misbehaviour
  - nicer return
  - sprinkle docs
  - Reenable mainnet update server. ([#4137](https://github.com/paritytech/parity/pull/4137))
  - basic tests for subscribeToEvents ([#4115](https://github.com/paritytech/parity/pull/4115))
  - subscribeToEvent fixtures ✅
  - subscribeToEvent tests ✅
  - temporarily skip failing test ([#4138](https://github.com/paritytech/parity/pull/4138))
  - Improvements and optimisations to estimate_gas ([#4142](https://github.com/paritytech/parity/pull/4142))
  - Return 0 instead of error with out of gas on estimate_gas
  - Fix stuff up.
  - Another estimate gas fix.
  - Alter balance to maximum possible rather than GP=0.
  - Only increase to amount strictly necessary.
  - Improvements and optimisations to estimate_gas.
  - Introduce proper error type
  - Avoid building costly traces
  - Fix tests.
  - Actually fix testsActually fix tests
  - Use estimateGas error (as per updated implementation) ([#4131](https://github.com/paritytech/parity/pull/4131))
  - EXCEPTION_ERROR as per #4142
  - Better error log reporting & handling ([#4128](https://github.com/paritytech/parity/pull/4128))
  - Don't pop-up notifications after network switch ([#4076](https://github.com/paritytech/parity/pull/4076))
  - Better notifications
  - Don't pollute with notifs if switched networks
  - Better connection close/open events / No more notifs on change network
  - PR Grumbles
  - Add close and open events to HTTP // Add tests
  - Fix tests
  - WIP Signer Fix
  - Fix Signer // Better reconnection handling
  - PR Grumbles
  - PR Grumbles
  - Fixes wrong fetching of balances + Notifications
  - Secure API WIP
  - Updated Secure API Connection + Status
  - Linting
  - Updated Secure API Logic
  - Proper handling of token updates // Fixing poping notifications
  - PR Grumbles
  - Fixing tests
  - Trim spaces from InputAddress ([#4126](https://github.com/paritytech/parity/pull/4126))
  - Trim spaces for addresses
  - onSubmit has only value, not event
  - onSubmit (again)
  - Length check on trimmed value
  - Remove bindActionCreators({}, dispatch) (empty) ([#4135](https://github.com/paritytech/parity/pull/4135))
- Backporting to beta [#4118](https://github.com/paritytech/parity/pull/4118)
  - Ignore get_price_info test by default. ([#4112](https://github.com/paritytech/parity/pull/4112))
  - Auto-detect hex encoded bytes in sha3 ([#4108](https://github.com/paritytech/parity/pull/4108))
  - Using types/isHex
  - Removing unused imports
  - Use binary chop to estimate gas accurately ([#4100](https://github.com/paritytech/parity/pull/4100))
  - Initial sketch.
  - Building.
  - Fix a few things.
  - Fix issue, add tracing.
  - Address grumbles
  - Raise upper limit if needed
  - Fix test.
  - Fixing decoding API with signatures in names ([#4125](https://github.com/paritytech/parity/pull/4125))
  - Fix call/estimate_gas ([#4121](https://github.com/paritytech/parity/pull/4121))
  - Return 0 instead of error with out of gas on estimate_gas
  - Fix stuff up.
- Current release: 1.3 -> 1.4 [#4183](https://github.com/paritytech/parity/pull/4183)
- Fix rebroadcast panic [#4084](https://github.com/paritytech/parity/pull/4084)
- Use shallow-only rendering in all tests [#4087](https://github.com/paritytech/parity/pull/4087)
- Sending transactions in chunks [#4089](https://github.com/paritytech/parity/pull/4089)
- Move to new auto-update server. [#4091](https://github.com/paritytech/parity/pull/4091)
- Fixing compilation without dapps. [#4088](https://github.com/paritytech/parity/pull/4088)
- Fix balances update [#4077](https://github.com/paritytech/parity/pull/4077)
- Key derivation in Worker [#4071](https://github.com/paritytech/parity/pull/4071)
- Display contract block creation [#4069](https://github.com/paritytech/parity/pull/4069)
- Improving logs for transactions sync and disable re-broadcasting while syncing [#4065](https://github.com/paritytech/parity/pull/4065)
- Passwords are valid by default [#4075](https://github.com/paritytech/parity/pull/4075)
- Show Origin label to events table [#4073](https://github.com/paritytech/parity/pull/4073)
- Fix tags not working [#4070](https://github.com/paritytech/parity/pull/4070)
- Zero-alloc trie lookups [#3998](https://github.com/paritytech/parity/pull/3998)
- Opening local dapp [#4041](https://github.com/paritytech/parity/pull/4041)
- Bringing back `js-sha3` to fix in-browser signing [#4063](https://github.com/paritytech/parity/pull/4063)
- Fix wrong transaction input for contract deployments [#4052](https://github.com/paritytech/parity/pull/4052)
- Re-broadcast transactions to few random peers on each new block. [#4054](https://github.com/paritytech/parity/pull/4054)
- Removing old transactions from the queue [#4046](https://github.com/paritytech/parity/pull/4046)
- Add block rewards to more Engines [#4055](https://github.com/paritytech/parity/pull/4055)
- Return old trie values on insert and remove [#4053](https://github.com/paritytech/parity/pull/4053)
- Let users open urls from dapps view [#4042](https://github.com/paritytech/parity/pull/4042)
- Util/validation update [#4051](https://github.com/paritytech/parity/pull/4051)
- Convert ShapeShift modal to store [#4035](https://github.com/paritytech/parity/pull/4035)
- Using local path on Windows [#4017](https://github.com/paritytech/parity/pull/4017)
- Fixing minGasLimit > ceil limit mining issue [#4018](https://github.com/paritytech/parity/pull/4018)
- Naive light client synchronization [#3892](https://github.com/paritytech/parity/pull/3892)
- Starting on homestead shows reload snackbar [#4043](https://github.com/paritytech/parity/pull/4043)
- Show contract parameters in MethodDecoding [#4024](https://github.com/paritytech/parity/pull/4024)
- UI component updates [#4010](https://github.com/paritytech/parity/pull/4010)
- Account view updates [#4008](https://github.com/paritytech/parity/pull/4008)
- Better error messages for PoA chains [#4034](https://github.com/paritytech/parity/pull/4034)
- Make some spec fields optional [#4019](https://github.com/paritytech/parity/pull/4019)
- Basic account type [#4021](https://github.com/paritytech/parity/pull/4021)
- Fix wallet in main net [#4038](https://github.com/paritytech/parity/pull/4038)
- Removing orphaned Cargo.toml [#4032](https://github.com/paritytech/parity/pull/4032)
- Address selector: support reverse lookup [#4033](https://github.com/paritytech/parity/pull/4033)
- Only fetch App when necessary [#4023](https://github.com/paritytech/parity/pull/4023)
- Connection UI cleanups & tests for prior PR [#4020](https://github.com/paritytech/parity/pull/4020)
- Unsubscribe error on ShapeShift modal close [#4005](https://github.com/paritytech/parity/pull/4005)
- Add ownership checks the Registry dApp [#4001](https://github.com/paritytech/parity/pull/4001)
- Refresh balances of contacts & contracts when syncing [#4022](https://github.com/paritytech/parity/pull/4022)
- Show message on new chain [#4016](https://github.com/paritytech/parity/pull/4016)
- Use TypedInputs in Contracts view [#4015](https://github.com/paritytech/parity/pull/4015)
- Fix focus on Modal [#4014](https://github.com/paritytech/parity/pull/4014)
- Fix newError noops when not bound to dispacher [#4013](https://github.com/paritytech/parity/pull/4013)
- Parse testnet chain as ropsten [#4004](https://github.com/paritytech/parity/pull/4004)
- Work on Portal Style [#4003](https://github.com/paritytech/parity/pull/4003)
- Make Wallet first-class citizens [#3990](https://github.com/paritytech/parity/pull/3990)
- Don't slice non-existent tags [#4000](https://github.com/paritytech/parity/pull/4000)
- Update dev dependencies and make Webpack less verbose [#3997](https://github.com/paritytech/parity/pull/3997)
- Correct log index in transaction receipt [#3995](https://github.com/paritytech/parity/pull/3995)
- Add Email and Registry lookups to Address Selector [#3992](https://github.com/paritytech/parity/pull/3992)
- Remove node journal: dead code [#3994](https://github.com/paritytech/parity/pull/3994)
- Cleanup AddContract with store [#3981](https://github.com/paritytech/parity/pull/3981)
- Store for EditPassword Modal [#3979](https://github.com/paritytech/parity/pull/3979)
- Additional fetch tests [#3983](https://github.com/paritytech/parity/pull/3983)
- Owning views of blockchain data [#3982](https://github.com/paritytech/parity/pull/3982)
- Make test network generic over peer type [#3974](https://github.com/paritytech/parity/pull/3974)
- Fetch tests (first batch) [#3977](https://github.com/paritytech/parity/pull/3977)
- Fetch certifiers only when needed [#3978](https://github.com/paritytech/parity/pull/3978)
- Visible accounts for dapps (default whitelist) [#3898](https://github.com/paritytech/parity/pull/3898)
- Remove some old (unused/duplicate) files [#3975](https://github.com/paritytech/parity/pull/3975)
- Port `try` macro to new `?` operator. [#3962](https://github.com/paritytech/parity/pull/3962)
- Small UI fixes [#3966](https://github.com/paritytech/parity/pull/3966)
- Fix wrong use of Icons [#3973](https://github.com/paritytech/parity/pull/3973)
- Updating dependencies [#3968](https://github.com/paritytech/parity/pull/3968)
- Web Based Dapps [#3956](https://github.com/paritytech/parity/pull/3956)
- Contract query: render false as false [#3971](https://github.com/paritytech/parity/pull/3971)
- Email verification: add Terms of Service [#3970](https://github.com/paritytech/parity/pull/3970)
- Fix method decoding [#3967](https://github.com/paritytech/parity/pull/3967)
- Store for EditMeta modal [#3959](https://github.com/paritytech/parity/pull/3959)
- Registry dapp: cleanup, support reverse entries [#3933](https://github.com/paritytech/parity/pull/3933)
- New Address Selector Component [#3829](https://github.com/paritytech/parity/pull/3829)
- Limiting accounts returned by parity_accountInfo [#3931](https://github.com/paritytech/parity/pull/3931)
- Unknown block error for RPC [#3965](https://github.com/paritytech/parity/pull/3965)
- Remove unused fields in informant [#3963](https://github.com/paritytech/parity/pull/3963)
- Allow contract constructors in chain spec [#3932](https://github.com/paritytech/parity/pull/3932)
- Sync reorg up to history size [#3874](https://github.com/paritytech/parity/pull/3874)
- Rising the limit for fetch [#3964](https://github.com/paritytech/parity/pull/3964)
- Bring integer arithmetic up to crates.io [#3943](https://github.com/paritytech/parity/pull/3943)
- Eslint rule for block curlies [#3955](https://github.com/paritytech/parity/pull/3955)
- Gas exception warnings on deployment [#3938](https://github.com/paritytech/parity/pull/3938)
- Move verification store into modal [#3951](https://github.com/paritytech/parity/pull/3951)
- Allow setting of minBlock on sending [#3921](https://github.com/paritytech/parity/pull/3921)
- Allow empty address [#3961](https://github.com/paritytech/parity/pull/3961)
- Fix default import [#3960](https://github.com/paritytech/parity/pull/3960)
- Display 0x00..00 as null [#3950](https://github.com/paritytech/parity/pull/3950)
- Global Fetch Service [#3915](https://github.com/paritytech/parity/pull/3915)
- Update babel-loader for WebPack 2.2-rc [#3953](https://github.com/paritytech/parity/pull/3953)
- Fix Webpack build [#3946](https://github.com/paritytech/parity/pull/3946)
- Fix manual input token [#3945](https://github.com/paritytech/parity/pull/3945)
- Update Webpack [#3952](https://github.com/paritytech/parity/pull/3952)
- Add missing Ethcore -> Parity headers [#3948](https://github.com/paritytech/parity/pull/3948)
- Code example: do start before register_protocol [#3947](https://github.com/paritytech/parity/pull/3947)
- Set CHAIN_ID for Classic [#3934](https://github.com/paritytech/parity/pull/3934)
- Fixed compile error. [#3940](https://github.com/paritytech/parity/pull/3940)
- Fix dapps not loading [#3935](https://github.com/paritytech/parity/pull/3935)
- Fix Secure API hangs [#3927](https://github.com/paritytech/parity/pull/3927)
- Parity_chainStatus RPC for block gap info [#3899](https://github.com/paritytech/parity/pull/3899)
- Custom attribute for binary serialization  [#3922](https://github.com/paritytech/parity/pull/3922)
- Split intermediate stage into two. [#3926](https://github.com/paritytech/parity/pull/3926)
- Move release-registering to intermediate stage. [#3920](https://github.com/paritytech/parity/pull/3920)
- Blocktime format rounding [#3894](https://github.com/paritytech/parity/pull/3894)
- Ignore dapps_policy.json [#3919](https://github.com/paritytech/parity/pull/3919)
- Fixing Contract Development [#3912](https://github.com/paritytech/parity/pull/3912)
- Use rhash for non-native CI platforms and submit build. [#3911](https://github.com/paritytech/parity/pull/3911)
- Remove -Zorbit=off from rustflags on windows [#3907](https://github.com/paritytech/parity/pull/3907)
- Fixed upgrading keys on the first run [#3904](https://github.com/paritytech/parity/pull/3904)
- Fix deadlock in queue drop [#3903](https://github.com/paritytech/parity/pull/3903)
- Require only simpler methods on Provider [#3897](https://github.com/paritytech/parity/pull/3897)
- Fix grammar ("you try" -> "you tried" + article) [#3902](https://github.com/paritytech/parity/pull/3902)
- Remove light server capability temporarily [#3872](https://github.com/paritytech/parity/pull/3872)
- Allow retry for future blocks [#3896](https://github.com/paritytech/parity/pull/3896)
- Consistent engine and seal names [#3895](https://github.com/paritytech/parity/pull/3895)
- Update email certification ABI [#3893](https://github.com/paritytech/parity/pull/3893)
- Remove existence & length checks on passwords & phrases [#3854](https://github.com/paritytech/parity/pull/3854)
- Refresh certifications automatically [#3878](https://github.com/paritytech/parity/pull/3878)
- Fix Wallet Settings Modal [#3856](https://github.com/paritytech/parity/pull/3856)
- Fix difficulty adjustment. [#3884](https://github.com/paritytech/parity/pull/3884)
- Final fixups for updater [#3883](https://github.com/paritytech/parity/pull/3883)
- Attempt to fix windows CI. [#3882](https://github.com/paritytech/parity/pull/3882)
- Fixing racy test [#3881](https://github.com/paritytech/parity/pull/3881)
- Fix updater permissions [#3880](https://github.com/paritytech/parity/pull/3880)
- Delayed transactions [#3865](https://github.com/paritytech/parity/pull/3865)
- Don't log auth token [#3853](https://github.com/paritytech/parity/pull/3853)
- Loading default config from default path [#3875](https://github.com/paritytech/parity/pull/3875)
- New paths [#3877](https://github.com/paritytech/parity/pull/3877)
- Update tests, gitlabci [#3876](https://github.com/paritytech/parity/pull/3876)
- Base directory option [#3868](https://github.com/paritytech/parity/pull/3868)
- Auto-updating [#3505](https://github.com/paritytech/parity/pull/3505)
- Fix naming collision [#3873](https://github.com/paritytech/parity/pull/3873)
- Get rid of unecessary redirection while fetching content [#3858](https://github.com/paritytech/parity/pull/3858)
- Fix verification stores [#3864](https://github.com/paritytech/parity/pull/3864)
- Store subscriptionId, align with main subscription model [#3863](https://github.com/paritytech/parity/pull/3863)
- Additional RPCs for dapps accounts management [#3792](https://github.com/paritytech/parity/pull/3792)
- Add Ws Json rpc client and command line utils (take 2) [#3830](https://github.com/paritytech/parity/pull/3830)
- Fix typo in method call (broken contract interface) [#3862](https://github.com/paritytech/parity/pull/3862)
- Fix flaky test [#3860](https://github.com/paritytech/parity/pull/3860)
- Converting traces API to AutoArgs [#3844](https://github.com/paritytech/parity/pull/3844)
- Get certifications from BadgeReg, show them in accounts overview [#3768](https://github.com/paritytech/parity/pull/3768)
- New directory structure [#3828](https://github.com/paritytech/parity/pull/3828)
- First run: skip account creation if they already have accounts [#3827](https://github.com/paritytech/parity/pull/3827)
- Tendermint seal [#3857](https://github.com/paritytech/parity/pull/3857)
- Tendermint Engine [#3759](https://github.com/paritytech/parity/pull/3759)
- Expand lint to catch css issues [#3852](https://github.com/paritytech/parity/pull/3852)
- Inject exports both partiy & web3 [#3851](https://github.com/paritytech/parity/pull/3851)
- Let Webpack talk again [#3848](https://github.com/paritytech/parity/pull/3848)
- AuthorityRound seal and simplify Generic seal Spec [#3843](https://github.com/paritytech/parity/pull/3843)
- Signing transactions with rotating token [#3691](https://github.com/paritytech/parity/pull/3691)
- Bump dev chain [#3835](https://github.com/paritytech/parity/pull/3835)
- Spelling [#3839](https://github.com/paritytech/parity/pull/3839)
- Email verification [#3766](https://github.com/paritytech/parity/pull/3766)
- Network configuration for Ethereum Classic [#3812](https://github.com/paritytech/parity/pull/3812)
- Using jsonrpc-macros [#3831](https://github.com/paritytech/parity/pull/3831)
- Fixed bool dropdown in contract execution [#3823](https://github.com/paritytech/parity/pull/3823)
- Avoid broadcasting transactions to peers that send them [#3796](https://github.com/paritytech/parity/pull/3796)
- Eth_sign RPC now hashes given data instead of getting the hash [#3800](https://github.com/paritytech/parity/pull/3800)
- Add store for MethodDecoding [#3821](https://github.com/paritytech/parity/pull/3821)
- Add store for AddAddress [#3819](https://github.com/paritytech/parity/pull/3819)
- Fix React-Router in i18n locale change [#3815](https://github.com/paritytech/parity/pull/3815)
- Cache fetched Dapps [#3804](https://github.com/paritytech/parity/pull/3804)
- Authors & homepage => Parity [#3818](https://github.com/paritytech/parity/pull/3818)
- Rename Ethcore -> Parity Technologies [#3817](https://github.com/paritytech/parity/pull/3817)
- Allow editing of gasPrice & gas in Signer [#3777](https://github.com/paritytech/parity/pull/3777)
- I18n string dictionaries [#3532](https://github.com/paritytech/parity/pull/3532)
- Fix padding in App [#3813](https://github.com/paritytech/parity/pull/3813)
- Light server improvements and protocol adjustments [#3801](https://github.com/paritytech/parity/pull/3801)
- Tolerate errors in user_defaults [#3810](https://github.com/paritytech/parity/pull/3810)
- Block: enforce gas limit falls within engine bounds [#3809](https://github.com/paritytech/parity/pull/3809)
- Target Babel to latest Chrome Versions in dev [#3806](https://github.com/paritytech/parity/pull/3806)
- Lowercase npm packages [#3807](https://github.com/paritytech/parity/pull/3807)
- Extended publishing of libraries to npm [#3786](https://github.com/paritytech/parity/pull/3786)
- Several Fixes to the UI [#3799](https://github.com/paritytech/parity/pull/3799)
- Remove "s [#3805](https://github.com/paritytech/parity/pull/3805)
- Extract CSS to file in production builds [#3783](https://github.com/paritytech/parity/pull/3783)
- Notify user on transaction received [#3782](https://github.com/paritytech/parity/pull/3782)
- Removing all old entries from transaction queue [#3772](https://github.com/paritytech/parity/pull/3772)
- Status page updates [#3774](https://github.com/paritytech/parity/pull/3774)
- Allow modifications of gas when confirming in signer [#3798](https://github.com/paritytech/parity/pull/3798)
- Network connectivity fixes [#3794](https://github.com/paritytech/parity/pull/3794)
- Make *ID names consistent with std Rust (Id) [#3781](https://github.com/paritytech/parity/pull/3781)
- Update CI builds [#3780](https://github.com/paritytech/parity/pull/3780)
- Update AuthorityRound tests to new spec [#3790](https://github.com/paritytech/parity/pull/3790)
- Fixes to the Wallet UI [#3787](https://github.com/paritytech/parity/pull/3787)
- Add support for wallets without getOwner() interface [#3779](https://github.com/paritytech/parity/pull/3779)
- Update Material-UI [#3785](https://github.com/paritytech/parity/pull/3785)
- Fixes error in Transfer modal [#3788](https://github.com/paritytech/parity/pull/3788)
- LES Part 3: Event handlers and handling responses [#3755](https://github.com/paritytech/parity/pull/3755)
- Basic UI rendering tests [#3743](https://github.com/paritytech/parity/pull/3743)
- Network: process packets only after connection handler finishes [#3776](https://github.com/paritytech/parity/pull/3776)
- AuthorityRound network simulation test [#3778](https://github.com/paritytech/parity/pull/3778)
- GasPrice selection for contract execution [#3770](https://github.com/paritytech/parity/pull/3770)
- Reject existing transactions [#3762](https://github.com/paritytech/parity/pull/3762)
- Allow autoRemove from api.subscribe based on callback return values [#3752](https://github.com/paritytech/parity/pull/3752)
- Replace misplaced & with && in gitlab-ci.yml [#3753](https://github.com/paritytech/parity/pull/3753)
- Lower gas usage for creating a Multisig Wallet [#3773](https://github.com/paritytech/parity/pull/3773)
- Added IO service explicit stop [#3761](https://github.com/paritytech/parity/pull/3761)
- Be lenient around invalid owners map [#3764](https://github.com/paritytech/parity/pull/3764)
- GasEditor component [#3750](https://github.com/paritytech/parity/pull/3750)
- Cleanups [#3742](https://github.com/paritytech/parity/pull/3742)
- Update babel, fix CI build due to breaking changes [#3754](https://github.com/paritytech/parity/pull/3754)
- Small fixes to contract [#3751](https://github.com/paritytech/parity/pull/3751)
- Make engine hold AccountProvider [#3725](https://github.com/paritytech/parity/pull/3725)
- Properly delete addresses/contracts in addressbook [#3739](https://github.com/paritytech/parity/pull/3739)
- Display Wallet Owners Icons in Accounts list [#3741](https://github.com/paritytech/parity/pull/3741)
- Edit Multisig Wallet settings [#3740](https://github.com/paritytech/parity/pull/3740)
- Replace build directory completely [#3748](https://github.com/paritytech/parity/pull/3748)
- Add existing release files before merge [#3747](https://github.com/paritytech/parity/pull/3747)
- Release script back to using fetch/merge [#3746](https://github.com/paritytech/parity/pull/3746)
- Update with -X only for merge [#3745](https://github.com/paritytech/parity/pull/3745)
- Give accounts precedence over address_book entries [#3732](https://github.com/paritytech/parity/pull/3732)
- Enable Panic=abort [#3423](https://github.com/paritytech/parity/pull/3423)
- Cleanups on js-precompiled [#3738](https://github.com/paritytech/parity/pull/3738)
- Add parity_removeAddress RPC [#3735](https://github.com/paritytech/parity/pull/3735)
- Fix up the transaction JSON serialisation for RPC. [#3633](https://github.com/paritytech/parity/pull/3633)
- Queue: CLI for auto-scaling and num verifiers [#3709](https://github.com/paritytech/parity/pull/3709)
- Add functionalities to multi-sig wallet [#3729](https://github.com/paritytech/parity/pull/3729)
- PropTypes as function call [#3731](https://github.com/paritytech/parity/pull/3731)
- Unify proptypes in util/proptypes.js [#3728](https://github.com/paritytech/parity/pull/3728)
- Bump jsonrpc-ipc-server to fix windows build [#3730](https://github.com/paritytech/parity/pull/3730)
- LES Part 2 [#3527](https://github.com/paritytech/parity/pull/3527)
- First draft of the MultiSig Wallet [#3700](https://github.com/paritytech/parity/pull/3700)
- Engine block ordering [#3719](https://github.com/paritytech/parity/pull/3719)
- Use fdlimit utility crate from crates.io [#3716](https://github.com/paritytech/parity/pull/3716)
- Move decoding for contract deployment logic earlier [#3714](https://github.com/paritytech/parity/pull/3714)
- Possible fix for queue drop deadlock [#3702](https://github.com/paritytech/parity/pull/3702)
- Encode networkid as a u64. [#3713](https://github.com/paritytech/parity/pull/3713)
- Use valid RLP in generic genesis seal spec [#3717](https://github.com/paritytech/parity/pull/3717)
- Update JS dependencies [#3710](https://github.com/paritytech/parity/pull/3710)
- Use Webpack Aliases [#3711](https://github.com/paritytech/parity/pull/3711)
- Dapps-specific accounts [#3627](https://github.com/paritytech/parity/pull/3627)
- Signer method parameter decoding & destination info [#3671](https://github.com/paritytech/parity/pull/3671)
- Remove invalid slice test [#3712](https://github.com/paritytech/parity/pull/3712)
- React library update [#3704](https://github.com/paritytech/parity/pull/3704)
- New Loading Component for the UI [#3707](https://github.com/paritytech/parity/pull/3707)
- Refactoring Transfer Modal [#3705](https://github.com/paritytech/parity/pull/3705)
- Fix extra scrollbars in dapps [#3706](https://github.com/paritytech/parity/pull/3706)
- Indent state tests [#3431](https://github.com/paritytech/parity/pull/3431)
- Filter null transactions for display (not available on node) [#3698](https://github.com/paritytech/parity/pull/3698)
- Move recovery phrase print button [#3697](https://github.com/paritytech/parity/pull/3697)
- Fix padding bottom needed after fixed status [#3701](https://github.com/paritytech/parity/pull/3701)
- Don't share the snapshot while downloading old blocks [#3695](https://github.com/paritytech/parity/pull/3695)
- Button to print recovery phrase [#3694](https://github.com/paritytech/parity/pull/3694)
- Fix status bar to bottom of the screen [#3692](https://github.com/paritytech/parity/pull/3692)
- Splitting serialization of signTransaction and sendTransaction confirmation requests [#3642](https://github.com/paritytech/parity/pull/3642)
- Implement basic badges/certifications/flair [#3665](https://github.com/paritytech/parity/pull/3665)
- Simplify Container title rendering [#3680](https://github.com/paritytech/parity/pull/3680)
- Update loading splash to fit in with l&f [#3685](https://github.com/paritytech/parity/pull/3685)
- Safari UI fixes [#3678](https://github.com/paritytech/parity/pull/3678)
- Remove strict mode for DappReg (work-around for package upgrade) [#3681](https://github.com/paritytech/parity/pull/3681)
- Bumping clippy [#3654](https://github.com/paritytech/parity/pull/3654)
- Return of the Fat DB [#3636](https://github.com/paritytech/parity/pull/3636)
- Invalidate blocks from future [#3652](https://github.com/paritytech/parity/pull/3652)
- Make Modal always scrollable [#3667](https://github.com/paritytech/parity/pull/3667)
- Display local/completed transactions [#3630](https://github.com/paritytech/parity/pull/3630)
- Added build-essential dep to dockerfiles [#3666](https://github.com/paritytech/parity/pull/3666)
- Strict config parsing (uknown keys are rejected) [#3663](https://github.com/paritytech/parity/pull/3663)
- Strict deserialization [#3662](https://github.com/paritytech/parity/pull/3662)
- Disable peer if no common block found [#3655](https://github.com/paritytech/parity/pull/3655)
- Show snackbar on password change [#3661](https://github.com/paritytech/parity/pull/3661)
- Bring back PV62 support [#3660](https://github.com/paritytech/parity/pull/3660)
- Unlock expecting quantity [#3659](https://github.com/paritytech/parity/pull/3659)
- Update Webpack => v2 [#3643](https://github.com/paritytech/parity/pull/3643)
- Update SMS verification [#3579](https://github.com/paritytech/parity/pull/3579)
- Simplify tx confirmations display [#3559](https://github.com/paritytech/parity/pull/3559)
- Fixes overflow in Signer tx data [#3657](https://github.com/paritytech/parity/pull/3657)
- Fixed tab bar not updating [#3653](https://github.com/paritytech/parity/pull/3653)
- Set default min tx price to $0.0025 [#3617](https://github.com/paritytech/parity/pull/3617)
- Use accountsInfo instead of eth_accounts for first check [#3618](https://github.com/paritytech/parity/pull/3618)
- Fix Copy to Clipboard Snackbar [#3619](https://github.com/paritytech/parity/pull/3619)
- Manually add \r to Windows phrases pre 1.4.5 [#3615](https://github.com/paritytech/parity/pull/3615)
- Signer layouts to flexbox [#3600](https://github.com/paritytech/parity/pull/3600)
- Fixing wrong tokens type in Redux store [#3621](https://github.com/paritytech/parity/pull/3621)
- Add dappreg link to apps list [#3568](https://github.com/paritytech/parity/pull/3568)
- Smarter balance fetching [#3605](https://github.com/paritytech/parity/pull/3605)
- Dapp iframe allow forms, allow target=_blank [#3597](https://github.com/paritytech/parity/pull/3597)
- Align copy button to input field [#3604](https://github.com/paritytech/parity/pull/3604)
- Appending logs by default [#3609](https://github.com/paritytech/parity/pull/3609)
- Update test, fix number. [#3612](https://github.com/paritytech/parity/pull/3612)
- Fixing phrases generated on windows [#3614](https://github.com/paritytech/parity/pull/3614)
- Check for network ID for live/test matching [#3602](https://github.com/paritytech/parity/pull/3602)
- Always insert traces for genesis. [#3603](https://github.com/paritytech/parity/pull/3603)
- Real deleting accounts [#3540](https://github.com/paritytech/parity/pull/3540)
- Trim whitespace from input recovery phrase [#3599](https://github.com/paritytech/parity/pull/3599)
- Fix local tx requests  [#3589](https://github.com/paritytech/parity/pull/3589)
- Fix CPU usage when idle [#3592](https://github.com/paritytech/parity/pull/3592)
- Don't fetch balances on every new block if syncing [#3591](https://github.com/paritytech/parity/pull/3591)
- Work around WS in UI [#3587](https://github.com/paritytech/parity/pull/3587)
- CLI option to disable ancient block downloading [#3573](https://github.com/paritytech/parity/pull/3573)
- Move Signer balance queries to store for component-wide re-use [#3531](https://github.com/paritytech/parity/pull/3531)
- Fix wrong method name in `contract.js` [#3580](https://github.com/paritytech/parity/pull/3580)
- Smarter Tokens fetching [#3546](https://github.com/paritytech/parity/pull/3546)
- Fix panic on importing own invalid transaction [#3550](https://github.com/paritytech/parity/pull/3550)
- Use an adaptive number of threads in the verification queue [#2445](https://github.com/paritytech/parity/pull/2445)
- Faster UI - React Tweaks [#3555](https://github.com/paritytech/parity/pull/3555)
- Send value & contract execute gas limit warnings [#3512](https://github.com/paritytech/parity/pull/3512)
- Add TxQueue visibility specifier (not added between merges) [#3566](https://github.com/paritytech/parity/pull/3566)
- DappRegistry [#3405](https://github.com/paritytech/parity/pull/3405)
- Import account message [#3552](https://github.com/paritytech/parity/pull/3552)
- --testnet set to ropsten [#3551](https://github.com/paritytech/parity/pull/3551)
- Fix flaky test [#3547](https://github.com/paritytech/parity/pull/3547)
- Sms verification code style [#3564](https://github.com/paritytech/parity/pull/3564)
- [Registry] Clear input and working buttons [#3563](https://github.com/paritytech/parity/pull/3563)
- Fix peers not displaying [#3561](https://github.com/paritytech/parity/pull/3561)
- New registry contract address for ropsten [#3549](https://github.com/paritytech/parity/pull/3549)
- Use contract Registry fee, not a hard-coded value [#3554](https://github.com/paritytech/parity/pull/3554)
- Don't query chain in Signer, use Redux isTest [#3524](https://github.com/paritytech/parity/pull/3524)
- Moving fetching of hash-addressed dapps/content to separate crate. [#3543](https://github.com/paritytech/parity/pull/3543)
- Ropsten network [#3539](https://github.com/paritytech/parity/pull/3539)
- Add simple one-line installer to README.md [#3534](https://github.com/paritytech/parity/pull/3534)
- Propagations & local transactions tracking [#3491](https://github.com/paritytech/parity/pull/3491)
- Correct format of eth_signTransaction [#3503](https://github.com/paritytech/parity/pull/3503)
- ABI can be empty and auto-fill contract name [#3518](https://github.com/paritytech/parity/pull/3518)
- Fix versions for NPM [#3516](https://github.com/paritytech/parity/pull/3516)
- Better GHH event display & tracking [#3498](https://github.com/paritytech/parity/pull/3498)
- Dapp section & visibility changes [#3438](https://github.com/paritytech/parity/pull/3438)
- Fix parity.js badly built [#3526](https://github.com/paritytech/parity/pull/3526)
- Updated the european warp bootnode addresses [#3528](https://github.com/paritytech/parity/pull/3528)
- Limit sync reorg to 20 blocks [#3519](https://github.com/paritytech/parity/pull/3519)
- Revert "Limit sync reorganization to 20 blocks" [#3517](https://github.com/paritytech/parity/pull/3517)
- Check transaction signature when adding to the queue [#3508](https://github.com/paritytech/parity/pull/3508)
- Limit sync reorganization to 20 blocks [#3509](https://github.com/paritytech/parity/pull/3509)
- Keep track of block gasLimit [#3506](https://github.com/paritytech/parity/pull/3506)
- Smarter Status Polling [#3504](https://github.com/paritytech/parity/pull/3504)
- Handle solc combined output [#3496](https://github.com/paritytech/parity/pull/3496)
- Wallet names shouldn't use UUID [#3481](https://github.com/paritytech/parity/pull/3481)
- Make parity.js usable by Node and Browser [#3475](https://github.com/paritytech/parity/pull/3475)
- Sms verification modal [#3336](https://github.com/paritytech/parity/pull/3336)
- Sudo -c is not supported on Mac [#3488](https://github.com/paritytech/parity/pull/3488)
- Add trace_{call, rawTransaction, replayTransaction} [#3492](https://github.com/paritytech/parity/pull/3492)
- Check for possible panics in scrypt key derivation [#3490](https://github.com/paritytech/parity/pull/3490)
- Sync traffic optimization [#3477](https://github.com/paritytech/parity/pull/3477)
- Wallet files shouldn't give away the address [#3378](https://github.com/paritytech/parity/pull/3378)
- Fixing tests, fixing refreshing precompiled [#3483](https://github.com/paritytech/parity/pull/3483)
- Better Errors Snackbar in UI [#3478](https://github.com/paritytech/parity/pull/3478)
- Handle Signer Rejection [#3476](https://github.com/paritytech/parity/pull/3476)
- Enhanced MethodDecoding in Transactions list [#3454](https://github.com/paritytech/parity/pull/3454)
- Signer new-token generates a link and opens browser [#3379](https://github.com/paritytech/parity/pull/3379)
- Make tokenreg dapp fast again [#3474](https://github.com/paritytech/parity/pull/3474)
- Build fix [#3470](https://github.com/paritytech/parity/pull/3470)
- Display deployed Basic token addresses [#3447](https://github.com/paritytech/parity/pull/3447)
- Export accounts as JSON or CSV [#2866](https://github.com/paritytech/parity/pull/2866)
- Set HF2 block number [#3466](https://github.com/paritytech/parity/pull/3466)
- Better word list for secret phrase generation [#3461](https://github.com/paritytech/parity/pull/3461)
- Drop spec when no longer useful [#3460](https://github.com/paritytech/parity/pull/3460)
- Add fallback check in ABI validation [#3459](https://github.com/paritytech/parity/pull/3459)
- Save sort order in LocalStorage [#3457](https://github.com/paritytech/parity/pull/3457)
- Adds onPaste event to Inputs [#3456](https://github.com/paritytech/parity/pull/3456)
- Update signer to take care of text overflows [#3450](https://github.com/paritytech/parity/pull/3450)
- Authority round consensus engine [#3426](https://github.com/paritytech/parity/pull/3426)
- Fix transfer token decimal calculation [#3445](https://github.com/paritytech/parity/pull/3445)
- Restrict max code size for EIP-150 and after. [#3363](https://github.com/paritytech/parity/pull/3363)
- Contract queries should display IdentityIcons [#3453](https://github.com/paritytech/parity/pull/3453)
- Use Babel in vendor when needed [#3451](https://github.com/paritytech/parity/pull/3451)
- Use signature of functions instead of names [#3448](https://github.com/paritytech/parity/pull/3448)
- Handle contract constructor inputs [#3430](https://github.com/paritytech/parity/pull/3430)
- Use Contract owner for unregistering Token [#3446](https://github.com/paritytech/parity/pull/3446)
- Create directories only if feature is enabled [#3442](https://github.com/paritytech/parity/pull/3442)
- Import AddresBook from exported JSON [#3433](https://github.com/paritytech/parity/pull/3433)
- Scrollable accounts in autocomplete [#3427](https://github.com/paritytech/parity/pull/3427)
- Bump ws-rs [#3428](https://github.com/paritytech/parity/pull/3428)
- Swap TokenReg dapp from base to decimals [#3425](https://github.com/paritytech/parity/pull/3425)
- Change beta builds to stable on Travis [#3421](https://github.com/paritytech/parity/pull/3421)
- Refactor copy to clipboard functionality [#3420](https://github.com/paritytech/parity/pull/3420)
- Dev chain [#3385](https://github.com/paritytech/parity/pull/3385)
- Fetch known code from the database during restoration [#3377](https://github.com/paritytech/parity/pull/3377)
- Fixing benches [#3422](https://github.com/paritytech/parity/pull/3422)
- Fix chainspec storage field. [#3406](https://github.com/paritytech/parity/pull/3406)
- Abort snapshot restoration faster [#3356](https://github.com/paritytech/parity/pull/3356)
- Remove addresses, display non-refundable warning on registries [#3403](https://github.com/paritytech/parity/pull/3403)
- Don't auto-unsubscribe when subscriber callback throws [#3401](https://github.com/paritytech/parity/pull/3401)
- Fix dapp account selection [#3399](https://github.com/paritytech/parity/pull/3399)
- Fix travis build: remove unused import [#3381](https://github.com/paritytech/parity/pull/3381)
- Optimize memory footprint [#3376](https://github.com/paritytech/parity/pull/3376)
- Fixing parsing passwords from file [#3367](https://github.com/paritytech/parity/pull/3367)
- Remove some unwraps from parity/helpers [#3364](https://github.com/paritytech/parity/pull/3364)
- Load external, builtin & local apps in parallel [#3340](https://github.com/paritytech/parity/pull/3340)
- Solidity Compiler in UI [#3279](https://github.com/paritytech/parity/pull/3279)
- Determine real-time HTTP connected status [#3335](https://github.com/paritytech/parity/pull/3335)
- Clarify error message about disabled Signer [#3359](https://github.com/paritytech/parity/pull/3359)
- Cater for home.parity hostname in dappsUrl [#3341](https://github.com/paritytech/parity/pull/3341)
- Make sure Token is ECR20 [#3347](https://github.com/paritytech/parity/pull/3347)
- [TokenReg dApp] Fixed Unregister for Contract Owner only [#3346](https://github.com/paritytech/parity/pull/3346)
- LES Part 1 [#3322](https://github.com/paritytech/parity/pull/3322)
- Make transactions load [#3348](https://github.com/paritytech/parity/pull/3348)
- Manual bump package.json [#3345](https://github.com/paritytech/parity/pull/3345)
- Windows app and installer fixes [#3338](https://github.com/paritytech/parity/pull/3338)
- Fix JS API test [#3342](https://github.com/paritytech/parity/pull/3342)
- Git pre-push checks for UI [#3072](https://github.com/paritytech/parity/pull/3072)
- Disarm the HF and add more bootnodes [#3323](https://github.com/paritytech/parity/pull/3323)
- Default contract type on UI [#3310](https://github.com/paritytech/parity/pull/3310)
- In-browser signing support [#3231](https://github.com/paritytech/parity/pull/3231)
- Handle redirects from /api/content on manifest.json gracefully [#3315](https://github.com/paritytech/parity/pull/3315)
- Dapps interface RPC [#3311](https://github.com/paritytech/parity/pull/3311)
- Additional snapshot sync checks [#3318](https://github.com/paritytech/parity/pull/3318)
- Fix spurious signer tests failures [#3312](https://github.com/paritytech/parity/pull/3312)
- Fix signer token updates [#3302](https://github.com/paritytech/parity/pull/3302)
- Update account recovery phrase hint [#3316](https://github.com/paritytech/parity/pull/3316)
- New transaction tests [#3313](https://github.com/paritytech/parity/pull/3313)
- Remove 127.0.0.1 references [#3303](https://github.com/paritytech/parity/pull/3303)
- Fix for opening UI after installation on mac [#3300](https://github.com/paritytech/parity/pull/3300)
- Fixed uncle query [#3299](https://github.com/paritytech/parity/pull/3299)
- Updated blance display with max decimals [#3266](https://github.com/paritytech/parity/pull/3266)
- Refactoring Signer to auto_args + eth_signTransaction [#3261](https://github.com/paritytech/parity/pull/3261)
- Fix typo [#3298](https://github.com/paritytech/parity/pull/3298)
- Change to more common focused spelling [#3264](https://github.com/paritytech/parity/pull/3264)
- Manual bump of package.json (recovery) [#3295](https://github.com/paritytech/parity/pull/3295)
- Fix initial token generation [#3289](https://github.com/paritytech/parity/pull/3289)
- Fixed IO service shutdown [#3286](https://github.com/paritytech/parity/pull/3286)
- Autostart setting for windows tray app [#3269](https://github.com/paritytech/parity/pull/3269)
- Fixes for 1.4 [#3260](https://github.com/paritytech/parity/pull/3260)
- Build tray app for x64 [#3255](https://github.com/paritytech/parity/pull/3255)
- Add secure flag back [#3244](https://github.com/paritytech/parity/pull/3244)
- Verify chunk hashes in cli restore [#3241](https://github.com/paritytech/parity/pull/3241)
- Load network apps manifests as contentHash (no coding) [#3235](https://github.com/paritytech/parity/pull/3235)
- Fixed some typos [#3236](https://github.com/paritytech/parity/pull/3236)
- Rename cli and config options signer->ui [#3232](https://github.com/paritytech/parity/pull/3232)
- Add store for dapps state [#3211](https://github.com/paritytech/parity/pull/3211)
- Fix first-time tagging of contracts [#3222](https://github.com/paritytech/parity/pull/3222)
- Fix /parity-utils/{web3,parity}.js webpack errors [#3221](https://github.com/paritytech/parity/pull/3221)
- Improve 'invalid raw key' error msg [#3219](https://github.com/paritytech/parity/pull/3219)
- Cleaning up polluted namespaces [#3143](https://github.com/paritytech/parity/pull/3143)
- Set passive mode for first run only [#3214](https://github.com/paritytech/parity/pull/3214)
- Parity configuration settings, i.e. mode [#3212](https://github.com/paritytech/parity/pull/3212)
- Ethash unsafety cleanup [#3210](https://github.com/paritytech/parity/pull/3210)
- Mode improvements for UI [#3109](https://github.com/paritytech/parity/pull/3109)
- Delay bomb for Classic (ECIP-1010) [#3179](https://github.com/paritytech/parity/pull/3179)
- Use ethcore_dappsPort when constructing URLs [#3139](https://github.com/paritytech/parity/pull/3139)
- Add copy address button to Contract deploy [#3199](https://github.com/paritytech/parity/pull/3199)
- Expose Parity api as window.secureApi [#3207](https://github.com/paritytech/parity/pull/3207)
- Add error for sendRawTransaction and estimateGas [#3194](https://github.com/paritytech/parity/pull/3194)
- Exposing engine extra info in block RPC [#3169](https://github.com/paritytech/parity/pull/3169)
- V1.5 [#3195](https://github.com/paritytech/parity/pull/3195)
- Remove dapp logos (GHH points to dapp-assets) [#3192](https://github.com/paritytech/parity/pull/3192)
- Fixing possible race in ethcore_hashContent [#3191](https://github.com/paritytech/parity/pull/3191)
- Bump package.json version (1.5 is master) [#3193](https://github.com/paritytech/parity/pull/3193)

## Parity [v1.4.10](https://github.com/paritytech/parity/releases/tag/v1.4.10) (2017-01-18)

Parity 1.4.10 is a first stable release of 1.4.x series. It includes a few minor networking fixes.

- Gas_limit for blocks, mined by Parity will be divisible by 37 (#4154) [#4179](https://github.com/paritytech/parity/pull/4179)
  - gas_limit for new blocks will divide evenly by 13
  - increased PARITY_GAS_LIMIT_DETERMINANT to 37
  - separate method for marking mined block
  - debug_asserts(gas_limit within protocol range)
  - round_block_gas_limit method is now static
  - made round_block_gas_limit free-function
  - multiplier->multiple
- Backporing to 1.4.10-stable [#4110](https://github.com/paritytech/parity/pull/4110)
  - Bump to v1.4.10
  - No reorg limit for ancient blocks
  - Update registration after every write

## Parity [v1.4.9](https://github.com/paritytech/parity/releases/tag/v1.4.9) (2017-01-09)

This fixes an issue introduced in 1.4.8 that causes Parity to panic on propagating transactions in some cases.

- v1.4.9 in beta [#4097](https://github.com/paritytech/parity/pull/4097)
  - Bump to v1.4.9
  - Disable armv6 build
- beta Fix queue deadlock [#4095](https://github.com/paritytech/parity/pull/4095)
- Fix rebroadcast panic beta [#4085](https://github.com/paritytech/parity/pull/4085)
  - fix compile
  - fix backport
  - clean up old method
  - remove unnecessary reference
  - simplify
  - Fixing 'simplify'

## Parity [v1.4.8](https://github.com/paritytech/parity/releases/tag/v1.4.8) (2017-01-06)

Ethereum Classic Hard Fork ready release containing various bugfixes:

- Fix for excessive transactions propagation
- Fix for inconsistent `logIndex` in transaction receipts

See [full list of changes](https://github.com/paritytech/parity/compare/v1.4.7...v1.4.8):

- Beta backports [#4067](https://github.com/paritytech/parity/pull/4067)
- Re-broadcast transactions to few random peers on each new block. (#4054) [#4061](https://github.com/paritytech/parity/pull/4061)
- Tolerate errors in user_defaults [#4060](https://github.com/paritytech/parity/pull/4060)
- ETC Config change backport [#4056](https://github.com/paritytech/parity/pull/4056)
- [beta] Avoid re-broadcasting transactions on each block [#4047](https://github.com/paritytech/parity/pull/4047)
- Beta Backports [#4012](https://github.com/paritytech/parity/pull/4012)

## Parity [v1.4.7](https://github.com/paritytech/parity/releases/tag/v1.4.7) (2016-12-27)

This maintenance release fixes an issue with sync falling behind occasionally.

- Backporting to beta [#3980](https://github.com/paritytech/parity/pull/3980)
- [beta] enforce gas limit falls within engine bounds [#3816](https://github.com/paritytech/parity/pull/3816)


## Parity [v1.3.15](https://github.com/paritytech/parity/releases/tag/v1.3.15) (2016-12-10)

This patch release fixes an issue with syncing on the Ropsten test network.

- Backporting to stable [#3793](https://github.com/paritytech/parity/pull/3793)

## Parity [v1.4.6](https://github.com/paritytech/parity/releases/tag/v1.4.6) (2016-12-05)

This patch release fixes an issue with syncing on the Ropsten test network.

- Backporting to beta [#3718](https://github.com/paritytech/parity/pull/3718)
- [beta] scrollable contract deploy & execute modals [#3656](https://github.com/paritytech/parity/pull/3656)

## Parity [v1.4.5](https://github.com/paritytech/parity/releases/tag/v1.4.5) (2016-11-26)

1.4.5 release fixes a number of issues, notably:
- High CPU usage when idle.
- Key recovery phrases generated on windows now can be imported.

#### Configuration changes
- `--usd-per-tx` is now set to 0.0025 by default.

#### New features
- Support for Ropsten test network is introduced with `--chain=ropsten` or `--testnet`. Morden network is still available via `--chain=morden`

#### Full changes
- [beta] Pin package versions for React [#3628](https://github.com/paritytech/parity/pull/3628)
- Backporting to beta [#3623](https://github.com/paritytech/parity/pull/3623)
- [beta] Ropsten chain for UI [#3622](https://github.com/paritytech/parity/pull/3622)

## Parity [v1.3.14](https://github.com/paritytech/parity/releases/tag/v1.3.14) (2016-11-25)

Parity 1.3.14 fixes a few stability issues and adds support for the Ropsten testnet.

- Backporting to stable [#3616](https://github.com/paritytech/parity/pull/3616)

## Parity [v1.4.4](https://github.com/paritytech/parity/releases/tag/v1.4.4) (2016-11-18)

This is a maintenance release that fixes an issue with EIP-155 transactions being added to the transaction pool. It also improves syncing stability and resolved a number of UI issues.
Full changelog is available [here.](https://github.com/paritytech/parity/commit/3e0d033eaf789cfdf517f4a97effc500f1f9263b)

- [beta] apps typo fix [#3533](https://github.com/paritytech/parity/pull/3533)
- Backporting to beta [#3525](https://github.com/paritytech/parity/pull/3525)

## Parity [v1.3.13](https://github.com/paritytech/parity/releases/tag/v1.3.13) (2016-11-18)

This release fixes an issue with EIP-155 transactions being allowed into the transaction pool.

- [stable] Check tx signatures before adding to the queue. [#3521](https://github.com/paritytech/parity/pull/3521)
- Fix Stable Docker Build [#3479](https://github.com/paritytech/parity/pull/3479)

## Parity [v1.4.3](https://github.com/paritytech/parity/releases/tag/v1.4.3) (2016-11-16)

This release includes memory footprint optimization as well as a few fixes in the UI.
EIP-155/160/161/170 hardfork is enabled at block 2675000 (1885000 for test network).
Full changelog is available [here.](https://github.com/paritytech/parity/compare/v1.4.2...v1.4.3)

- [beta] EIP-170 [#3464](https://github.com/paritytech/parity/pull/3464)
- Backports to beta [#3465](https://github.com/paritytech/parity/pull/3465)
- Backport: additional fields on transaction and receipt [#3463](https://github.com/paritytech/parity/pull/3463)
- v1.4.3 in beta [#3424](https://github.com/paritytech/parity/pull/3424)


## Parity [v1.3.12](https://github.com/paritytech/parity/releases/tag/v1.3.12) (2016-11-16)

This stable release enables EIP-155/160/161/170 hardfork at block 2675000 (1885000 for test network).

- [stable] EIP-170 [#3462](https://github.com/paritytech/parity/pull/3462)
- #3035 Backport to stable [#3441](https://github.com/paritytech/parity/pull/3441)

## Parity [v1.3.11](https://github.com/paritytech/parity/releases/tag/v1.3.11) (2016-11-11)

This is a maintenance release for the stable series to delay the EIP-155/160/161 hard fork transition. **Update from 1.3.10 is mandatory**. It also deprecates and disables the old Parity UI.

- [stable] Disable HF and UI  [#3372](https://github.com/paritytech/parity/pull/3372)
- [stable] EIP-155 update with Vitalik's new test vectors (#3166) [#3190](https://github.com/paritytech/parity/pull/3190)
- Backport EIP-150 to stable [#2672](https://github.com/paritytech/parity/pull/2672)
- Create gitlab-ci.yml for stable [#2517](https://github.com/paritytech/parity/pull/2517)

## Parity [v1.4.2](https://github.com/paritytech/parity/releases/tag/v1.4.2) (2016-11-10)

This release fixes a few additional issues:
- Parity now correctly handles external `--dapps-interface` and  `--ui-interface` in the UI.
- Crash in `eth_getUncle*` has been fixed.
- macOS installer now includes an uninstall script.
- Security token input UI has been fixed.
- Correct display for tokens with minimum decimals.

And some additional minor changes. Full changelog is [available](https://github.com/paritytech/parity/compare/v1.4.1...v1.4.2)
- Backporting to beta [#3344](https://github.com/paritytech/parity/pull/3344)
- Backporting to beta [#3324](https://github.com/paritytech/parity/pull/3324)

## Parity [v1.4.1](https://github.com/paritytech/parity/releases/tag/v1.4.1) (2016-11-09)

This is a hotfix release to address a couple of issues with 1.4.0:

- UI token is requested instead of being supplied automatically.
- Running with `--geth` results in an error.

- Backporting to beta [#3293](https://github.com/paritytech/parity/pull/3293)

## Parity [v1.4.0](https://github.com/paritytech/parity/releases/tag/v1.4.0) (2016-11-07)

First beta release of the 1.4 series.

This includes the new Parity Wallet and Warp-Sync synchronisation as well as several optimisations and fixes.

- Add secure flag back [#3246](https://github.com/paritytech/parity/pull/3246)
- [BETA] verify chunk hashes in cli restore [#3242](https://github.com/paritytech/parity/pull/3242)
- Backporting to beta [#3239](https://github.com/paritytech/parity/pull/3239)
- UI fixes backporting [#3234](https://github.com/paritytech/parity/pull/3234)
- Backporting to beta [#3229](https://github.com/paritytech/parity/pull/3229)
- Beta branch cleanup [#3226](https://github.com/paritytech/parity/pull/3226)
- [beta] Set passive mode for first run only (#3214) [#3216](https://github.com/paritytech/parity/pull/3216)
- Mode configuration backported to beta [#3213](https://github.com/paritytech/parity/pull/3213)
- Backporting [#3198](https://github.com/paritytech/parity/pull/3198)
- [beta] EIP-155 update with Vitalik's new test vectors (#3166) [#3189](https://github.com/paritytech/parity/pull/3189)
- Backporting to beta [#3176](https://github.com/paritytech/parity/pull/3176)
- parity-ui-precompiled pinned to beta [#3168](https://github.com/paritytech/parity/pull/3168)
- EIP-155 update with Vitalik's new test vectors [#3166](https://github.com/paritytech/parity/pull/3166)
- Push precompiled for beta/stable, npm only master [#3163](https://github.com/paritytech/parity/pull/3163)
- Back to real root after npm publish [#3178](https://github.com/paritytech/parity/pull/3178)
- Remove extra cd js [#3177](https://github.com/paritytech/parity/pull/3177)
- Fixes Gas price selection bug [#3175](https://github.com/paritytech/parity/pull/3175)
- Exposing state root and logsBloom in RPC receipts [#3174](https://github.com/paritytech/parity/pull/3174)
- Exposing v,r,s from transaction signature in RPC [#3172](https://github.com/paritytech/parity/pull/3172)
- Enabling personal RPC over IPC by default [#3165](https://github.com/paritytech/parity/pull/3165)
- Gitlab CI badge [#3164](https://github.com/paritytech/parity/pull/3164)
- Dependencies in README [#3162](https://github.com/paritytech/parity/pull/3162)
- Make the footer a bit less ugly. [#3160](https://github.com/paritytech/parity/pull/3160)
- Linux build case sensitivity fix [#3161](https://github.com/paritytech/parity/pull/3161)
- abbreviated enode, `CopyToClipboard` component [#3131](https://github.com/paritytech/parity/pull/3131)
- EIPs 155, 160, 161 [#2976](https://github.com/paritytech/parity/pull/2976)
- beta reset to 1.4.0 [#3157](https://github.com/paritytech/parity/pull/3157)
- Fix histogram [#3150](https://github.com/paritytech/parity/pull/3150)
- Remove network label from TabBar [#3142](https://github.com/paritytech/parity/pull/3142)
- Speed up unresponsive Contract events & Account transactions [#3145](https://github.com/paritytech/parity/pull/3145)
- Better windows shortcut [#3147](https://github.com/paritytech/parity/pull/3147)
- Redirect content to the same address as requested [#3133](https://github.com/paritytech/parity/pull/3133)
- Fixed peer ping timeout [#3137](https://github.com/paritytech/parity/pull/3137)
- Fix for windows build [#3125](https://github.com/paritytech/parity/pull/3125)
- Fix AddessInput icon position [#3132](https://github.com/paritytech/parity/pull/3132)
- Fixed not scrollable accounts in tokenreg dapp [#3128](https://github.com/paritytech/parity/pull/3128)
- Returning cache headers for network content [#3123](https://github.com/paritytech/parity/pull/3123)
- Optimise contract events display [#3120](https://github.com/paritytech/parity/pull/3120)
- Add basic validation for contract execute values [#3118](https://github.com/paritytech/parity/pull/3118)
- Dapps errors embeddable on signer [#3115](https://github.com/paritytech/parity/pull/3115)
- Use enode RPC in UI [#3108](https://github.com/paritytech/parity/pull/3108)
- Windows tray app [#3103](https://github.com/paritytech/parity/pull/3103)
- Displaying CLI errors on stderr [#3116](https://github.com/paritytech/parity/pull/3116)
- new InputAddressSelect component [#3071](https://github.com/paritytech/parity/pull/3071)
- Bump mio [#3117](https://github.com/paritytech/parity/pull/3117)
- Minor typo fixed. [#3110](https://github.com/paritytech/parity/pull/3110)
- Sort by ETH balance and contract by date [#3107](https://github.com/paritytech/parity/pull/3107)
- Add RPC enode lookup [#3096](https://github.com/paritytech/parity/pull/3096)
- Initializing logger for each command [#3090](https://github.com/paritytech/parity/pull/3090)
- Allow registration of content bundles in GitHubHint [#3094](https://github.com/paritytech/parity/pull/3094)
- Add read-only inputs to UI plus Copy to Clipboard buttons [#3095](https://github.com/paritytech/parity/pull/3095)
- Allow boolean dropdowns for contract deploy [#3077](https://github.com/paritytech/parity/pull/3077)
- Add mac installer files [#2995](https://github.com/paritytech/parity/pull/2995)
- Fixing dapps sorting [#3086](https://github.com/paritytech/parity/pull/3086)
- Add a Gitter chat badge to README.md [#3092](https://github.com/paritytech/parity/pull/3092)
- Fixes webpack HTML loader [#3089](https://github.com/paritytech/parity/pull/3089)
- Redirecting /home to new UI [#3084](https://github.com/paritytech/parity/pull/3084)
- Allow GitHubHint content owner to update url [#3083](https://github.com/paritytech/parity/pull/3083)
- Remove token assets (moved to ethcore/dapps-assets) [#3082](https://github.com/paritytech/parity/pull/3082)
- Goodbye Gavcoin, Hello Gavcoin [#3080](https://github.com/paritytech/parity/pull/3080)
- Load network dapps [#3078](https://github.com/paritytech/parity/pull/3078)
- Swap account phrase input to normal (non-multiline) [#3060](https://github.com/paritytech/parity/pull/3060)
- Fix minor typo in informant [#3056](https://github.com/paritytech/parity/pull/3056)
- Warp sync status display [#3045](https://github.com/paritytech/parity/pull/3045)
- Enhance address input [#3065](https://github.com/paritytech/parity/pull/3065)
- Go to Accounts Page if Tooltips are displayed [#3063](https://github.com/paritytech/parity/pull/3063)
- Change contract Execute bool values & query bool value display [#3024](https://github.com/paritytech/parity/pull/3024)
- Update Parity logo [#3036](https://github.com/paritytech/parity/pull/3036)
- settings: replace background patterns (inline) [#3047](https://github.com/paritytech/parity/pull/3047)
- Multiple line description for dapps [#3058](https://github.com/paritytech/parity/pull/3058)
- Fix status log order [#3062](https://github.com/paritytech/parity/pull/3062)
- Graphical gas price selection [#2898](https://github.com/paritytech/parity/pull/2898)
- [Registry dApp] Actions not available before selecting accounts [#3032](https://github.com/paritytech/parity/pull/3032)
- apply post-consolidation migrations after consolidating [#3020](https://github.com/paritytech/parity/pull/3020)
- fix chain badge padding [#3046](https://github.com/paritytech/parity/pull/3046)
- Don't delete Tags input on blur (eg. tab) [#3044](https://github.com/paritytech/parity/pull/3044)
- Fixing last hashes for ethcall [#3043](https://github.com/paritytech/parity/pull/3043)
- Remove signer icons [#3039](https://github.com/paritytech/parity/pull/3039)
- execute periodic snapshot in new thread [#3029](https://github.com/paritytech/parity/pull/3029)
- fix background of embedded signer [#3026](https://github.com/paritytech/parity/pull/3026)
- registry dapp: fix reducer [#3028](https://github.com/paritytech/parity/pull/3028)
- Replace Execute by Query in contract button [#3031](https://github.com/paritytech/parity/pull/3031)
- Fixing GavCoin dApp overflow issues [#3030](https://github.com/paritytech/parity/pull/3030)
- execute contract function: validate address [#3013](https://github.com/paritytech/parity/pull/3013)
- Align tag inputs with other input boxes [#2965](https://github.com/paritytech/parity/pull/2965)
- Sweep panickers from IO and network [#3018](https://github.com/paritytech/parity/pull/3018)
- Terms & Conditions [#3019](https://github.com/paritytech/parity/pull/3019)
- open column families after reparing db corruption [#3017](https://github.com/paritytech/parity/pull/3017)
- Snapshot sync and block gap info in `eth_syncing` [#2948](https://github.com/paritytech/parity/pull/2948)
- personal_ RPCs to AutoArgs [#3000](https://github.com/paritytech/parity/pull/3000)
- RPCs for mode change [#3002](https://github.com/paritytech/parity/pull/3002)
- Fix a test sensitive to slow execution. [#3014](https://github.com/paritytech/parity/pull/3014)
- Fixes search filtering issues [#3011](https://github.com/paritytech/parity/pull/3011)
- Restart sync if no more peers with snapshots [#3007](https://github.com/paritytech/parity/pull/3007)
- Allow empty/non-existant input arrays for ABIs in contract view [#3001](https://github.com/paritytech/parity/pull/3001)
- Allow operation when no registry is available [#2980](https://github.com/paritytech/parity/pull/2980)
- Make JS lint & test run on Travis [#2894](https://github.com/paritytech/parity/pull/2894)
- Update account dropdowns [#2959](https://github.com/paritytech/parity/pull/2959)
- Modify gas price statistics [#2947](https://github.com/paritytech/parity/pull/2947)
- Fixes pending/mined transactions in registry dApp [#3004](https://github.com/paritytech/parity/pull/3004)
- Prevent connecting to self [#2997](https://github.com/paritytech/parity/pull/2997)
- Disable verbose in gitlab CI [#2999](https://github.com/paritytech/parity/pull/2999)
- Allow warnings in gitlab [#2998](https://github.com/paritytech/parity/pull/2998)
- Fix the brainwallet functionality. [#2994](https://github.com/paritytech/parity/pull/2994)
- Provided gas description update [#2993](https://github.com/paritytech/parity/pull/2993)
- Print messages to stderr [#2991](https://github.com/paritytech/parity/pull/2991)
- Networking and syncing tweaks [#2990](https://github.com/paritytech/parity/pull/2990)
- Allow build warnings [#2985](https://github.com/paritytech/parity/pull/2985)
- Display network status for finished Signer requests [#2983](https://github.com/paritytech/parity/pull/2983)
- Fixed rejecting transactions [#2984](https://github.com/paritytech/parity/pull/2984)
- mio version bump [#2982](https://github.com/paritytech/parity/pull/2982)
- Publish parity.js to npmjs registry [#2978](https://github.com/paritytech/parity/pull/2978)
- Import raw private key [#2945](https://github.com/paritytech/parity/pull/2945)
- refactor etherscan.io links [#2896](https://github.com/paritytech/parity/pull/2896)
- Use separate lock for code cache [#2977](https://github.com/paritytech/parity/pull/2977)
- Add favicon [#2974](https://github.com/paritytech/parity/pull/2974)
- Align password change dialog with create dialog ordering [#2970](https://github.com/paritytech/parity/pull/2970)
- WS bump [#2973](https://github.com/paritytech/parity/pull/2973)
- Discovery performance optimization [#2972](https://github.com/paritytech/parity/pull/2972)
- Pass gas & gasPrice to token transfers [#2964](https://github.com/paritytech/parity/pull/2964)
- Updating ws-rs [#2962](https://github.com/paritytech/parity/pull/2962)
- Run cargo with verbose flag when testing [#2943](https://github.com/paritytech/parity/pull/2943)
- Fixing clippy warnings take two [#2961](https://github.com/paritytech/parity/pull/2961)
- Snapshot sync improvements [#2960](https://github.com/paritytech/parity/pull/2960)
- Gavcoin event display updates [#2956](https://github.com/paritytech/parity/pull/2956)
- Eslint fixes [#2957](https://github.com/paritytech/parity/pull/2957)
- Add import of raw private key RPCs [#2942](https://github.com/paritytech/parity/pull/2942)
- Bring in styling queues from original Gavcoin [#2936](https://github.com/paritytech/parity/pull/2936)
- Validating minimal required gas for a transaction [#2937](https://github.com/paritytech/parity/pull/2937)
- Even more snapshot validity checks [#2935](https://github.com/paritytech/parity/pull/2935)
- Shared code cache [#2921](https://github.com/paritytech/parity/pull/2921)
- Updating bootnodes for ETC [#2938](https://github.com/paritytech/parity/pull/2938)
- More bootnodes [#2926](https://github.com/paritytech/parity/pull/2926)
- Revert hash updates until testable [#2925](https://github.com/paritytech/parity/pull/2925)
- Release.sh verbose output [#2924](https://github.com/paritytech/parity/pull/2924)
- additional release.sh debugging info [#2922](https://github.com/paritytech/parity/pull/2922)
- Pass the js-precompiled commit hash to cargo update [#2920](https://github.com/paritytech/parity/pull/2920)
- Next nonce RPC [#2917](https://github.com/paritytech/parity/pull/2917)
- Get rid of duplicated code in EVM [#2915](https://github.com/paritytech/parity/pull/2915)
- Transaction Queue banning [#2524](https://github.com/paritytech/parity/pull/2524)
- Revert to gas price ordering [#2919](https://github.com/paritytech/parity/pull/2919)
- Personal split [#2879](https://github.com/paritytech/parity/pull/2879)
- Fixing config values for pruning_history [#2918](https://github.com/paritytech/parity/pull/2918)
- Apply pending block details on commit [#2254](https://github.com/paritytech/parity/pull/2254)
- Fixed GetNodeData output [#2892](https://github.com/paritytech/parity/pull/2892)
- New sync protocol ID [#2912](https://github.com/paritytech/parity/pull/2912)
- Clippy bump [#2877](https://github.com/paritytech/parity/pull/2877)
- iconomi token images [#2906](https://github.com/paritytech/parity/pull/2906)
- Fixes too long description and Token balance value in Dapps/Accounts [#2902](https://github.com/paritytech/parity/pull/2902)
- Add missing images for local dapps [#2890](https://github.com/paritytech/parity/pull/2890)
- Fix Webpack, again [#2895](https://github.com/paritytech/parity/pull/2895)
- Enable suicide json test [#2893](https://github.com/paritytech/parity/pull/2893)
- More snapshot fixes and optimizations [#2883](https://github.com/paritytech/parity/pull/2883)
- Fixes CI JS precompiled build [#2886](https://github.com/paritytech/parity/pull/2886)
- Fix empty tags modification [#2884](https://github.com/paritytech/parity/pull/2884)
- Fix up informant. [#2865](https://github.com/paritytech/parity/pull/2865)
- Get rid of MemoryDB denote [#2881](https://github.com/paritytech/parity/pull/2881)
- Add inject to "bundle everything" list [#2871](https://github.com/paritytech/parity/pull/2871)
- Fixes signer and MUI errors throwing [#2876](https://github.com/paritytech/parity/pull/2876)
- Fix failing tests after log parsing updates [#2878](https://github.com/paritytech/parity/pull/2878)
- Sweep some more panics [#2848](https://github.com/paritytech/parity/pull/2848)
- Make GitLab js-precompiled really update Cargo.toml in main repo [#2869](https://github.com/paritytech/parity/pull/2869)
- IPC version bump [#2870](https://github.com/paritytech/parity/pull/2870)
- Snapshot sync fixes and optimizations [#2863](https://github.com/paritytech/parity/pull/2863)
- Add Check and Change Password for an Account [#2861](https://github.com/paritytech/parity/pull/2861)
- Output git fetch/push to log files [#2862](https://github.com/paritytech/parity/pull/2862)
- Align contract event log l&f with transactions [#2812](https://github.com/paritytech/parity/pull/2812)
- Nicer port in use errors [#2859](https://github.com/paritytech/parity/pull/2859)
- Remove personal_* calls from dapps [#2860](https://github.com/paritytech/parity/pull/2860)
- Token sorting, zero-ETH transfer & token decimals [#2805](https://github.com/paritytech/parity/pull/2805)
- Don't fail badly when no transactions in last 100 blocks. [#2856](https://github.com/paritytech/parity/pull/2856)
- Fixing home.parity address for new signer [#2851](https://github.com/paritytech/parity/pull/2851)
- Enabling UI build back [#2853](https://github.com/paritytech/parity/pull/2853)
- Remove eventName in unsubscribe API arguments [#2844](https://github.com/paritytech/parity/pull/2844)
- Don't return empty names as clickable titles [#2809](https://github.com/paritytech/parity/pull/2809)
- Auto-bump js-precompiled on release [#2828](https://github.com/paritytech/parity/pull/2828)
- Remove ethcore::common re-export module [#2792](https://github.com/paritytech/parity/pull/2792)
- Prevent database corruption on OOM [#2832](https://github.com/paritytech/parity/pull/2832)
- Download/Export Addressbook [#2847](https://github.com/paritytech/parity/pull/2847)
- Snapshot and blockchain stability improvements [#2843](https://github.com/paritytech/parity/pull/2843)
- Extended network options [#2845](https://github.com/paritytech/parity/pull/2845)
- fix failing master test build [#2846](https://github.com/paritytech/parity/pull/2846)
- Local dapps embeddable on signer port [#2815](https://github.com/paritytech/parity/pull/2815)
- Trigger accounts/contracts search on search input change [#2838](https://github.com/paritytech/parity/pull/2838)
- Move snapshot sync to a subprotocol [#2820](https://github.com/paritytech/parity/pull/2820)
- fix node log being reversed [#2839](https://github.com/paritytech/parity/pull/2839)
- Fixes currency symbol font size in Shapeshift modal [#2840](https://github.com/paritytech/parity/pull/2840)
- Disable personal APIs by default for security reasons [#2834](https://github.com/paritytech/parity/pull/2834)
- Clear cached content [#2833](https://github.com/paritytech/parity/pull/2833)
- Add ethcore_[dapps|signer]Port APIs [#2821](https://github.com/paritytech/parity/pull/2821)
- CLI option to skip seal check when importing [#2842](https://github.com/paritytech/parity/pull/2842)
- Fix case error in Dapps import [#2837](https://github.com/paritytech/parity/pull/2837)
- Double click on address in account detail view should select it [#2841](https://github.com/paritytech/parity/pull/2841)
- Bump js-precompiled to 20161022-223915 UTC [#2826](https://github.com/paritytech/parity/pull/2826)
- Adjust paths to handle CORS changes [#2816](https://github.com/paritytech/parity/pull/2816)
- RPC for dapps port and signer port [#2819](https://github.com/paritytech/parity/pull/2819)
- Update build to working version on pre-compiled repo [#2825](https://github.com/paritytech/parity/pull/2825)
- Adjust network name badge colours (darker) [#2823](https://github.com/paritytech/parity/pull/2823)
- Removing submodule in favour of rust crate [#2756](https://github.com/paritytech/parity/pull/2756)
- Return old-ish content even when syncing [#2757](https://github.com/paritytech/parity/pull/2757)
- fix Signer UI [#2750](https://github.com/paritytech/parity/pull/2750)
- USG, GBP, Euro & Yuan updates [#2818](https://github.com/paritytech/parity/pull/2818)
- Make locally installed apps available again (Fixes #2771) [#2808](https://github.com/paritytech/parity/pull/2808)
- Additional RPCs for password management [#2779](https://github.com/paritytech/parity/pull/2779)
- flush DB changes on drop [#2795](https://github.com/paritytech/parity/pull/2795)
- rename State::snapshot to checkpoint to avoid confusion [#2796](https://github.com/paritytech/parity/pull/2796)
- Missing changes required to make new UI work [#2793](https://github.com/paritytech/parity/pull/2793)
- Cleanup method decoding (Fixes #2811) [#2810](https://github.com/paritytech/parity/pull/2810)
- Use trace API for decentralized transaction list [#2784](https://github.com/paritytech/parity/pull/2784)
- Automatic compaction selection on Linux [#2785](https://github.com/paritytech/parity/pull/2785)
- Update token images [#2804](https://github.com/paritytech/parity/pull/2804)
- Hackergold token images [#2801](https://github.com/paritytech/parity/pull/2801)
- Additional token images [#2800](https://github.com/paritytech/parity/pull/2800)
- Additional token images [#2798](https://github.com/paritytech/parity/pull/2798)
- Resolve morden fork [#2773](https://github.com/paritytech/parity/pull/2773)
- Using SipHashes from crates.io [#2778](https://github.com/paritytech/parity/pull/2778)
- Fixed issues on Searchable Addresses [#2790](https://github.com/paritytech/parity/pull/2790)
- Currency icons [#2788](https://github.com/paritytech/parity/pull/2788)
- Update token images [#2783](https://github.com/paritytech/parity/pull/2783)
- Fix warning in master [#2775](https://github.com/paritytech/parity/pull/2775)
- Add empty account existence test from beta. [#2769](https://github.com/paritytech/parity/pull/2769)
- Update name of basiccoin manager [#2768](https://github.com/paritytech/parity/pull/2768)
- sweep most unwraps from ethcore crate, dapps crate [#2762](https://github.com/paritytech/parity/pull/2762)
- Check queue to determine major importing [#2763](https://github.com/paritytech/parity/pull/2763)
- Trace filtering fix [#2760](https://github.com/paritytech/parity/pull/2760)
- Update js precompiled to 20161020-141636 [#2761](https://github.com/paritytech/parity/pull/2761)
- Incrementally calculate verification queue heap size [#2749](https://github.com/paritytech/parity/pull/2749)
- Don't add empty accounts to bloom [#2753](https://github.com/paritytech/parity/pull/2753)
- fix contract deployments not showing up [#2759](https://github.com/paritytech/parity/pull/2759)
- Fixes a positioning issue in Address Selection component [#2754](https://github.com/paritytech/parity/pull/2754)
- fix linting issues [#2758](https://github.com/paritytech/parity/pull/2758)
- Making Trie.iter non-recursive [#2733](https://github.com/paritytech/parity/pull/2733)
- Block import optimization [#2748](https://github.com/paritytech/parity/pull/2748)
- Update js-precompiled to 20161020-110858 [#2752](https://github.com/paritytech/parity/pull/2752)
- Fixing small files fetching [#2742](https://github.com/paritytech/parity/pull/2742)
- Fixing stalled sync [#2747](https://github.com/paritytech/parity/pull/2747)
- refactor signer components [#2691](https://github.com/paritytech/parity/pull/2691)
- Png images with backgrounds (original svg) [#2740](https://github.com/paritytech/parity/pull/2740)
- Make address selection searchable [#2739](https://github.com/paritytech/parity/pull/2739)
- very basic dapp add/remove interface [#2721](https://github.com/paritytech/parity/pull/2721)
- Frontport commits from beta to master [#2743](https://github.com/paritytech/parity/pull/2743)
- Implements Trace API Formatter [#2732](https://github.com/paritytech/parity/pull/2732)
- bump parking_lot to 0.3.x series [#2702](https://github.com/paritytech/parity/pull/2702)
- Unify major syncing detection [#2699](https://github.com/paritytech/parity/pull/2699)
- Fixes gas/gasPrice change not reflected in transaction modal [#2735](https://github.com/paritytech/parity/pull/2735)
- Fixing build UI stuff along with Rust [#2726](https://github.com/paritytech/parity/pull/2726)
- Fixed Snackbar not showing and/or behind transactions (#2730) [#2731](https://github.com/paritytech/parity/pull/2731)
- Updating json tests to latest develop commit [#2728](https://github.com/paritytech/parity/pull/2728)
- dapps: show errors [#2727](https://github.com/paritytech/parity/pull/2727)
- node logs: break lines [#2722](https://github.com/paritytech/parity/pull/2722)
- Bumping JSON-RPC http server [#2714](https://github.com/paritytech/parity/pull/2714)
- Add ability to copy address to the clipboard [#2716](https://github.com/paritytech/parity/pull/2716)
- Sort tags when displaying ; use AND for search results [#2720](https://github.com/paritytech/parity/pull/2720)
- allow-same-origin for iframe [#2711](https://github.com/paritytech/parity/pull/2711)
- Update Registry address (mainnet) [#2713](https://github.com/paritytech/parity/pull/2713)
- Allow tags for Accounts, Addresses and Contracts [#2712](https://github.com/paritytech/parity/pull/2712)
- Correct parameters for eth_sign [#2703](https://github.com/paritytech/parity/pull/2703)
- Bump js-precompiled to 20161018-161705 [#2698](https://github.com/paritytech/parity/pull/2698)
- Add inject.js (for web3 exposed) [#2692](https://github.com/paritytech/parity/pull/2692)
- Remove obsolete dapps and update security headers [#2694](https://github.com/paritytech/parity/pull/2694)
- Snapshot sync part 2 [#2098](https://github.com/paritytech/parity/pull/2098)
- Fix issues with no ethereum test dir present (2382) [#2659](https://github.com/paritytech/parity/pull/2659)
- Apply UI PRs after master merge [#2690](https://github.com/paritytech/parity/pull/2690)
- Fix importing traces for non-canon blocks [#2683](https://github.com/paritytech/parity/pull/2683)
- Fixing random test failures [#2577](https://github.com/paritytech/parity/pull/2577)
- Disable IPC in default build for 1.4 [#2657](https://github.com/paritytech/parity/pull/2657)
- use pruning history in CLI snapshots [#2658](https://github.com/paritytech/parity/pull/2658)
- Fixing --no-default-features again and evmbin [#2670](https://github.com/paritytech/parity/pull/2670)
- Settings > Proxy for proxy.pac setup instructions [#2678](https://github.com/paritytech/parity/pull/2678)
- Re-instate transaitions to allow updating busy indicator [#2682](https://github.com/paritytech/parity/pull/2682)
- signer: remove reject counter [#2685](https://github.com/paritytech/parity/pull/2685)
- Initial new UI source code import [#2607](https://github.com/paritytech/parity/pull/2607)
- Additional dapp logo images [#2677](https://github.com/paritytech/parity/pull/2677)
- Redirect from :8080 to :8180 [#2676](https://github.com/paritytech/parity/pull/2676)
- script to update js-precompiled [#2673](https://github.com/paritytech/parity/pull/2673)
- Styling in FF is not 100% [#2669](https://github.com/paritytech/parity/pull/2669)
- Don't allow gavcoin transfer with no balances [#2667](https://github.com/paritytech/parity/pull/2667)
- fix signer rejections [#2666](https://github.com/paritytech/parity/pull/2666)
- better text on unique background pattern [#2664](https://github.com/paritytech/parity/pull/2664)
- Adjust z-index for error overlay [#2662](https://github.com/paritytech/parity/pull/2662)
- Fix address selection for contract deployment [#2660](https://github.com/paritytech/parity/pull/2660)
- Add additional contract images [#2655](https://github.com/paritytech/parity/pull/2655)
- Update /api/* to point to :8080/api/* (first generation interface) [#2612](https://github.com/paritytech/parity/pull/2612)
- Initial import of new UI (compiled JS code) [#2220](https://github.com/paritytech/parity/pull/2220)
- Fixing evmbin compilation [#2652](https://github.com/paritytech/parity/pull/2652)
- Fix up ETC EIP-150 transition to 2,500,000. [#2636](https://github.com/paritytech/parity/pull/2636)
- Fixing compilation without default features [#2638](https://github.com/paritytech/parity/pull/2638)
- [frontport] CLI to specify queue ordering strategy (#2494) [#2623](https://github.com/paritytech/parity/pull/2623)
- Support for decryption in Signer [#2421](https://github.com/paritytech/parity/pull/2421)
- EIP150.1c [#2591](https://github.com/paritytech/parity/pull/2591)
- Release merge with origin with ours strategy [#2631](https://github.com/paritytech/parity/pull/2631)
- Adjust build output directories [#2630](https://github.com/paritytech/parity/pull/2630)
- cater for txhash returning null/empty object [#2629](https://github.com/paritytech/parity/pull/2629)
- snapshot: single byte for empty accounts [#2625](https://github.com/paritytech/parity/pull/2625)
- Configurable history size in master [#2606](https://github.com/paritytech/parity/pull/2606)
- Database performance tweaks [#2619](https://github.com/paritytech/parity/pull/2619)
- Enable suicide json test [#2626](https://github.com/paritytech/parity/pull/2626)
- Split journaldb commit into two functions: journal_under and mark_canonical [#2329](https://github.com/paritytech/parity/pull/2329)
- Fixed tx queue limit for local transactions [#2616](https://github.com/paritytech/parity/pull/2616)
- Additional logs when transactions is removed from queue [#2617](https://github.com/paritytech/parity/pull/2617)
- mitigate refcell conflict in state diffing [#2601](https://github.com/paritytech/parity/pull/2601)
- Fix tests [#2611](https://github.com/paritytech/parity/pull/2611)
- small styling updates [#2610](https://github.com/paritytech/parity/pull/2610)
- Remove web3 from Signer, bring in parity.js API [#2604](https://github.com/paritytech/parity/pull/2604)
- Mostly configurable canonical cache size [#2516](https://github.com/paritytech/parity/pull/2516)
- Added peers details to ethcore_netPeers RPC [#2580](https://github.com/paritytech/parity/pull/2580)
- Display account password hint when available [#2596](https://github.com/paritytech/parity/pull/2596)
- Fix gas estimation on transfer when data supplied [#2593](https://github.com/paritytech/parity/pull/2593)
- remove unused npm packages [#2590](https://github.com/paritytech/parity/pull/2590)
- Bundle fonts as part of the build process [#2588](https://github.com/paritytech/parity/pull/2588)
- Contract constructor params [#2586](https://github.com/paritytech/parity/pull/2586)
- Update json test suite [#2574](https://github.com/paritytech/parity/pull/2574)
- Filter apps that has been replaced for the local list [#2583](https://github.com/paritytech/parity/pull/2583)
- Display local apps listed by Parity [#2581](https://github.com/paritytech/parity/pull/2581)
- Network-specific nodes file [#2569](https://github.com/paritytech/parity/pull/2569)
- Dont close when block is known to be invalid [#2572](https://github.com/paritytech/parity/pull/2572)
- deny compiler warnings in CI [#2570](https://github.com/paritytech/parity/pull/2570)
- adjust alignment of queries [#2573](https://github.com/paritytech/parity/pull/2573)
- update ethcore-bigint crate to 0.1.1 [#2562](https://github.com/paritytech/parity/pull/2562)
- Registry dapp uses setAddress to actually set addresses now [#2568](https://github.com/paritytech/parity/pull/2568)
- Add the new EIP150 test. [#2563](https://github.com/paritytech/parity/pull/2563)
- fix failing tests [#2567](https://github.com/paritytech/parity/pull/2567)
- ΞTH -> ETH [#2566](https://github.com/paritytech/parity/pull/2566)
- Ensure polling is only done when connected [#2565](https://github.com/paritytech/parity/pull/2565)
- Fixed race condition in trace import [#2555](https://github.com/paritytech/parity/pull/2555)
- Disable misbehaving peers while seeking for best block [#2537](https://github.com/paritytech/parity/pull/2537)
- TX queue gas limit config and allow local transactions over the gas limit [#2553](https://github.com/paritytech/parity/pull/2553)
- standard component for address -> name mappings (consistent use everywhere) [#2557](https://github.com/paritytech/parity/pull/2557)
- Remove unwrap from client module [#2554](https://github.com/paritytech/parity/pull/2554)
- Removing panickers from sync module [#2551](https://github.com/paritytech/parity/pull/2551)
- Address images (tokens, dapps) as registered via contentHash (when available) [#2526](https://github.com/paritytech/parity/pull/2526)
- TokenReg set & get images working [#2540](https://github.com/paritytech/parity/pull/2540)
- adjust app_id where /api/content/<hash> is called, fixes #2541 [#2543](https://github.com/paritytech/parity/pull/2543)
- connection dialog now shows up in dapps as well, closes #2538 [#2550](https://github.com/paritytech/parity/pull/2550)
- display account uuid (where available), closes #2546 [#2549](https://github.com/paritytech/parity/pull/2549)
- create accounts via recovery phrase [#2545](https://github.com/paritytech/parity/pull/2545)
- Build ethcore/js-precompiled on GitLab [#2522](https://github.com/paritytech/parity/pull/2522)
- Return errors from eth_call RPC [#2498](https://github.com/paritytech/parity/pull/2498)
- registry dapp: manage records [#2323](https://github.com/paritytech/parity/pull/2323)
- Print backtrace on panic [#2535](https://github.com/paritytech/parity/pull/2535)
- GitHubHint dapp [#2531](https://github.com/paritytech/parity/pull/2531)
- Backports to master [#2530](https://github.com/paritytech/parity/pull/2530)
- Handle reorganizations in the state cache [#2490](https://github.com/paritytech/parity/pull/2490)
- Hypervisor: terminate hanging modules [#2513](https://github.com/paritytech/parity/pull/2513)
- signer & node connection prompts/indicators [#2504](https://github.com/paritytech/parity/pull/2504)
- Using pending block only if is not old [#2514](https://github.com/paritytech/parity/pull/2514)
- More caching optimizations [#2505](https://github.com/paritytech/parity/pull/2505)
- Fixed possible panic in the networking [#2495](https://github.com/paritytech/parity/pull/2495)
- Trim password from file [#2503](https://github.com/paritytech/parity/pull/2503)
- Fixing RPC Filter conversion to EthFilter [#2500](https://github.com/paritytech/parity/pull/2500)
- Fixing error message for transactions [#2496](https://github.com/paritytech/parity/pull/2496)
- Adjustable stack size for EVM [#2483](https://github.com/paritytech/parity/pull/2483)
- [master] Fixing penalization in future [#2499](https://github.com/paritytech/parity/pull/2499)
- Preserve cache on reverting the snapshot [#2488](https://github.com/paritytech/parity/pull/2488)
- RocksDB version bump [#2492](https://github.com/paritytech/parity/pull/2492)
- Increase default size of transaction queue [#2489](https://github.com/paritytech/parity/pull/2489)
- basiccoin v1 available [#2491](https://github.com/paritytech/parity/pull/2491)
- Small EVM optimization [#2487](https://github.com/paritytech/parity/pull/2487)
- Track dirty accounts in the state [#2461](https://github.com/paritytech/parity/pull/2461)
- fix signature lookup address [#2480](https://github.com/paritytech/parity/pull/2480)
- update registrar test with generic non-empty test [#2476](https://github.com/paritytech/parity/pull/2476)
- Derive IPC interface only when ipc feature is on [#2463](https://github.com/paritytech/parity/pull/2463)
- Fix ethstore opening all key files in the directory at once [#2471](https://github.com/paritytech/parity/pull/2471)
- contract api event log fixes [#2469](https://github.com/paritytech/parity/pull/2469)
- basiccoin base functionality in-place [#2468](https://github.com/paritytech/parity/pull/2468)
- Merge IPC codegen attributes into one [#2460](https://github.com/paritytech/parity/pull/2460)
- Close after importing keys from geth [#2464](https://github.com/paritytech/parity/pull/2464)
- Port a couple more RPC APIs to the new auto args [#2325](https://github.com/paritytech/parity/pull/2325)
- update rustc for appveyor to 1.12.0 [#2423](https://github.com/paritytech/parity/pull/2423)
- dapp basiccoin send operations [#2456](https://github.com/paritytech/parity/pull/2456)
- Better EVM informant & Slow transactions warning [#2436](https://github.com/paritytech/parity/pull/2436)
- Fixing Signer token RPC API [#2437](https://github.com/paritytech/parity/pull/2437)
- Fixed FatDB check [#2443](https://github.com/paritytech/parity/pull/2443)
- dapp basiccoin structure [#2444](https://github.com/paritytech/parity/pull/2444)
- Accounts bloom in master [#2426](https://github.com/paritytech/parity/pull/2426)
- Polishing Actually enable fat db pr (#1974) [#2048](https://github.com/paritytech/parity/pull/2048)
- Jumptable cache [#2427](https://github.com/paritytech/parity/pull/2427)
- signaturereg registered, remove hardcoding [#2431](https://github.com/paritytech/parity/pull/2431)
- tokenreg dapp fixes for non-null returns [#2430](https://github.com/paritytech/parity/pull/2430)
- update ABI json to latest deployed versions [#2428](https://github.com/paritytech/parity/pull/2428)
- update Morden registry address [#2417](https://github.com/paritytech/parity/pull/2417)
- Make migration api more friendly [#2420](https://github.com/paritytech/parity/pull/2420)
- Journaling bloom filter crate in util [#2395](https://github.com/paritytech/parity/pull/2395)
- move abis from js/json to contracts/abi [#2418](https://github.com/paritytech/parity/pull/2418)
- Fixing logs-receipt matching [#2403](https://github.com/paritytech/parity/pull/2403)
- fix broken beta compilation [#2405](https://github.com/paritytech/parity/pull/2405)
- registry dapp: transfer names [#2335](https://github.com/paritytech/parity/pull/2335)
- manage firstRun better [#2398](https://github.com/paritytech/parity/pull/2398)
- render contract deployment address [#2397](https://github.com/paritytech/parity/pull/2397)
- Transaction Queue fix [#2392](https://github.com/paritytech/parity/pull/2392)
- contracts abi types & execute value [#2394](https://github.com/paritytech/parity/pull/2394)
- update styling with ParityBar overlay [#2390](https://github.com/paritytech/parity/pull/2390)
- application Signer popup window [#2388](https://github.com/paritytech/parity/pull/2388)
- Fixing Delegate Call in JIT [#2378](https://github.com/paritytech/parity/pull/2378)
- Prioritizing re-imported transactions [#2372](https://github.com/paritytech/parity/pull/2372)
- Revert #2172, pretty much. [#2387](https://github.com/paritytech/parity/pull/2387)
- correct sync memory usage calculation [#2385](https://github.com/paritytech/parity/pull/2385)
- fix migration system for post-consolidation migrations, better errors [#2334](https://github.com/paritytech/parity/pull/2334)
- Fix the traceAddress field in transaction traces. [#2373](https://github.com/paritytech/parity/pull/2373)
- Gavcoin utilises the popup box [#2381](https://github.com/paritytech/parity/pull/2381)
- registry dapp: support dropping names [#2328](https://github.com/paritytech/parity/pull/2328)
- settings view, set background & store views [#2380](https://github.com/paritytech/parity/pull/2380)
- Removing extras data from retracted blocks. [#2375](https://github.com/paritytech/parity/pull/2375)
- fixed #2263, geth keys with ciphertext shorter than 32 bytes [#2318](https://github.com/paritytech/parity/pull/2318)
- Expanse compatibility [#2369](https://github.com/paritytech/parity/pull/2369)
- Allow queries of constant functions on contracts [#2360](https://github.com/paritytech/parity/pull/2360)
- Auto Open/Close the Signer window on new transaction request [#2362](https://github.com/paritytech/parity/pull/2362)
- Specify column cache sizes explicitly; default fallback of 2MB [#2358](https://github.com/paritytech/parity/pull/2358)
- Canonical state cache (master) [#2311](https://github.com/paritytech/parity/pull/2311)
- method signature lookups, parameter decoding & management [#2313](https://github.com/paritytech/parity/pull/2313)
- make block queue into a more generic verification queue and fix block heap size calculation [#2095](https://github.com/paritytech/parity/pull/2095)
- Hash Content RPC method [#2355](https://github.com/paritytech/parity/pull/2355)
- registry dapp: show reserved events by default [#2359](https://github.com/paritytech/parity/pull/2359)
- Display timestamp in Signer requests details [#2324](https://github.com/paritytech/parity/pull/2324)
- Reorder transaction_by_hash to favour canon search [#2332](https://github.com/paritytech/parity/pull/2332)
- Optimize DIV for some common divisors [#2327](https://github.com/paritytech/parity/pull/2327)
- Return error when deserializing invalid hex [#2339](https://github.com/paritytech/parity/pull/2339)
- Changed http:// to https:// on some links [#2349](https://github.com/paritytech/parity/pull/2349)
- user defaults [#2014](https://github.com/paritytech/parity/pull/2014)
- Fixing jit feature compilation [#2310](https://github.com/paritytech/parity/pull/2310)
- Tx Queue improvements  [#2292](https://github.com/paritytech/parity/pull/2292)
- Removing PropTypes on build [#2322](https://github.com/paritytech/parity/pull/2322)
- Lenient bytes deserialization [#2036](https://github.com/paritytech/parity/pull/2036)
- reverse call data decoding given transaction data & method [#2312](https://github.com/paritytech/parity/pull/2312)
- add missing gpl headers to gavcoin dapp [#2317](https://github.com/paritytech/parity/pull/2317)
- contract Events, Functions & Queries sub-components as well as Event log visual updates [#2306](https://github.com/paritytech/parity/pull/2306)
- webpack config updates (really include babel-polyfill, rename npm steps) [#2305](https://github.com/paritytech/parity/pull/2305)
- remove unneeded Form from Account header [#2302](https://github.com/paritytech/parity/pull/2302)
- edit of metadata across accounts, addresses & contracts [#2300](https://github.com/paritytech/parity/pull/2300)
- Adjust all modals for consistency & css DRY-ness [#2301](https://github.com/paritytech/parity/pull/2301)
- update container spacing [#2296](https://github.com/paritytech/parity/pull/2296)
- local cache of generated background (no allocation on each re-render) [#2298](https://github.com/paritytech/parity/pull/2298)
- fix failing tests [#2290](https://github.com/paritytech/parity/pull/2290)
- Respecting standards for tokenreg dapp [#2287](https://github.com/paritytech/parity/pull/2287)
- Separate RPC serialization from implementation [#2072](https://github.com/paritytech/parity/pull/2072)
- Webpack optimisations - Using DLL [#2264](https://github.com/paritytech/parity/pull/2264)
- header background, theme adjustments (not that harsh) [#2273](https://github.com/paritytech/parity/pull/2273)
- contract view (developer-centric) [#2259](https://github.com/paritytech/parity/pull/2259)
- Add hash as CLI function [#1995](https://github.com/paritytech/parity/pull/1995)
- registry: fix mined events showing as pending [#2267](https://github.com/paritytech/parity/pull/2267)
- Dapp - Tokereg ; Query Tokens from TLA or Address [#2266](https://github.com/paritytech/parity/pull/2266)
- Fixes to the Token Registration dApp [#2250](https://github.com/paritytech/parity/pull/2250)
- remove abi *.json duplication, provide a single version of the truth [#2253](https://github.com/paritytech/parity/pull/2253)
- Separate path for ext code size [#2251](https://github.com/paritytech/parity/pull/2251)
- Snapshot format changes [#2234](https://github.com/paritytech/parity/pull/2234)
- Serving content at /api/content/<hash> [#2248](https://github.com/paritytech/parity/pull/2248)
- Fails when deserializing non-hex uints [#2247](https://github.com/paritytech/parity/pull/2247)
- registry dapp: add GPL headers [#2252](https://github.com/paritytech/parity/pull/2252)
- registry dapp: user-friendly lookup [#2229](https://github.com/paritytech/parity/pull/2229)
- registry dapp: show DataChanged events [#2242](https://github.com/paritytech/parity/pull/2242)
- fixups for deploys [#2249](https://github.com/paritytech/parity/pull/2249)
- Fixing output of eth_call and Bytes deserialization [#2230](https://github.com/paritytech/parity/pull/2230)
- Encryption, decryption and public key RPCs. [#1946](https://github.com/paritytech/parity/pull/1946)
- limit number of event logs returned [#2231](https://github.com/paritytech/parity/pull/2231)
- babel-polyfill [#2239](https://github.com/paritytech/parity/pull/2239)
- procedurally generate background based on signer key [#2233](https://github.com/paritytech/parity/pull/2233)
- UI fixes [#2238](https://github.com/paritytech/parity/pull/2238)
- expose isConnected() from transport [#2225](https://github.com/paritytech/parity/pull/2225)
- registry dapp: rename event log [#2227](https://github.com/paritytech/parity/pull/2227)
- registry dapp: show pending events [#2223](https://github.com/paritytech/parity/pull/2223)
- Handle RLP to string UTF-8 decoding errors [#2217](https://github.com/paritytech/parity/pull/2217)
- Use WebSocket transport for all built-in calls [#2216](https://github.com/paritytech/parity/pull/2216)
- Remove panickers from trie iterators [#2209](https://github.com/paritytech/parity/pull/2209)
- Limit for logs filter. [#2180](https://github.com/paritytech/parity/pull/2180)
- Various state copy optimizations [#2172](https://github.com/paritytech/parity/pull/2172)
- New signer token RPC & Initial signer connection without token. [#2096](https://github.com/paritytech/parity/pull/2096)
- signer ui fixes [#2219](https://github.com/paritytech/parity/pull/2219)
- contract deploy ui [#2212](https://github.com/paritytech/parity/pull/2212)
- registry dapp: fix propTypes [#2218](https://github.com/paritytech/parity/pull/2218)
- registry: fix IdentityIcon in events log [#2206](https://github.com/paritytech/parity/pull/2206)
- Fixing evm-debug [#2161](https://github.com/paritytech/parity/pull/2161)
- Fix syncing with pv63 peers [#2204](https://github.com/paritytech/parity/pull/2204)
- registry: show shortened hashes [#2205](https://github.com/paritytech/parity/pull/2205)
- registry dapp: remove owner [#2203](https://github.com/paritytech/parity/pull/2203)
- webpack proxy updates for /api* [#2175](https://github.com/paritytech/parity/pull/2175)
- simplify personal event publishing, fix delete refresh issues [#2183](https://github.com/paritytech/parity/pull/2183)
- fix global & initial states [#2160](https://github.com/paritytech/parity/pull/2160)
- Allow selection & saving of available views [#2131](https://github.com/paritytech/parity/pull/2131)
- global/contract events with promisy subscribe/unsubscribe [#2136](https://github.com/paritytech/parity/pull/2136)
- Token Registry dApp [#2178](https://github.com/paritytech/parity/pull/2178)
- re-usable bytesToHex exposed in api.util [#2174](https://github.com/paritytech/parity/pull/2174)
- Webpack optimisations [#2179](https://github.com/paritytech/parity/pull/2179)
- cleanup on contract event subscriptions [#2104](https://github.com/paritytech/parity/pull/2104)
- move utility functions to api.util [#2105](https://github.com/paritytech/parity/pull/2105)
- registry dapp [#2077](https://github.com/paritytech/parity/pull/2077)
- mui/FlatButton to ui/Button [#2129](https://github.com/paritytech/parity/pull/2129)
- address delete functionality [#2128](https://github.com/paritytech/parity/pull/2128)
- contract deployment updates [#2106](https://github.com/paritytech/parity/pull/2106)
- contract events, indexed string fix [#2108](https://github.com/paritytech/parity/pull/2108)
- Bumping jsonrpc-core & jsonrpc-http-server [#2162](https://github.com/paritytech/parity/pull/2162)
- gitlab testing & build processes [#2090](https://github.com/paritytech/parity/pull/2090)
- Misc small UI fixes (recently broken) [#2101](https://github.com/paritytech/parity/pull/2101)
- Bump clippy & Fix warnings [#2109](https://github.com/paritytech/parity/pull/2109)
- Import command summary [#2102](https://github.com/paritytech/parity/pull/2102)
- check for existence of deprecated ethash file before attempting delete [#2103](https://github.com/paritytech/parity/pull/2103)
- shapeshift Promise API library [#2088](https://github.com/paritytech/parity/pull/2088)
- fund account via ShapeShift [#2099](https://github.com/paritytech/parity/pull/2099)
- Get bigint on crates.io [#2078](https://github.com/paritytech/parity/pull/2078)
- Enable sealing if Engine provides internal sealing given author [#2084](https://github.com/paritytech/parity/pull/2084)
- Config files [#2070](https://github.com/paritytech/parity/pull/2070)
- re-add lodash plugin to babel config [#2092](https://github.com/paritytech/parity/pull/2092)
- Remove old cache data [#2081](https://github.com/paritytech/parity/pull/2081)
- Logs limit & log_index bug [#2073](https://github.com/paritytech/parity/pull/2073)
- flatten store, muiTheme & api providers [#2087](https://github.com/paritytech/parity/pull/2087)
- add babel es2016 & es2017 presets [#2083](https://github.com/paritytech/parity/pull/2083)
- remove all '<name>/index' imports in API [#2089](https://github.com/paritytech/parity/pull/2089)
- add missing GPL headers to all files [#2086](https://github.com/paritytech/parity/pull/2086)
- readme cleanups [#2085](https://github.com/paritytech/parity/pull/2085)
- gavcoin global import of parity api  [#2082](https://github.com/paritytech/parity/pull/2082)
- Fixing removal from gas price when moving future->current [#2076](https://github.com/paritytech/parity/pull/2076)
- Split internal sealing from work preparation [#2071](https://github.com/paritytech/parity/pull/2071)
- ensure the target folder doesn't exist before renaming [#2074](https://github.com/paritytech/parity/pull/2074)
- Get rid of 'Dapp is being downloaded' page [#2055](https://github.com/paritytech/parity/pull/2055)
- fix failing master build: update tests to new init_restore signature. [#2069](https://github.com/paritytech/parity/pull/2069)
- Local snapshot restore [#2058](https://github.com/paritytech/parity/pull/2058)
- import: keep informant going until finished [#2065](https://github.com/paritytech/parity/pull/2065)
- Add a few tests for the snapshot service [#2059](https://github.com/paritytech/parity/pull/2059)
- IPC tweaks [#2046](https://github.com/paritytech/parity/pull/2046)
- Update arm* Docker [#2064](https://github.com/paritytech/parity/pull/2064)
- Fetching any content-addressed content [#2050](https://github.com/paritytech/parity/pull/2050)
- Use proper database configuration in snapshots. [#2052](https://github.com/paritytech/parity/pull/2052)
- periodic snapshot tweaks [#2054](https://github.com/paritytech/parity/pull/2054)
- ethkey-cli [#2057](https://github.com/paritytech/parity/pull/2057)
- Forward ethstore-cli feature [#2056](https://github.com/paritytech/parity/pull/2056)
- handling invalid spec jsons properly, additional tests, closes #1840 [#2049](https://github.com/paritytech/parity/pull/2049)
- Periodic snapshots [#2044](https://github.com/paritytech/parity/pull/2044)
- Snapshot sync [#2047](https://github.com/paritytech/parity/pull/2047)
- Nice error pages for Dapps & Signer [#2033](https://github.com/paritytech/parity/pull/2033)
- Add a few small snapshot tests [#2038](https://github.com/paritytech/parity/pull/2038)
- facelift for traces, added errors [#2042](https://github.com/paritytech/parity/pull/2042)
- Fetching content from HTTPS using `rustls` [#2024](https://github.com/paritytech/parity/pull/2024)
- Skipping log when there are no transactions were sent [#2045](https://github.com/paritytech/parity/pull/2045)
- rlp as separate crate [#2034](https://github.com/paritytech/parity/pull/2034)
- Fixing uint serialization [#2037](https://github.com/paritytech/parity/pull/2037)
- Fixing new transactions propagation [#2039](https://github.com/paritytech/parity/pull/2039)
- Propagating transactions to peers on timer. [#2035](https://github.com/paritytech/parity/pull/2035)
- Remove Populatable and BytesConvertable traits [#2019](https://github.com/paritytech/parity/pull/2019)
- fixed #1933 [#1979](https://github.com/paritytech/parity/pull/1979)
- Synchronization tweaks for IPC services [#2028](https://github.com/paritytech/parity/pull/2028)
- Asynchronous RPC support [#2017](https://github.com/paritytech/parity/pull/2017)
- Disable ArchiveDB counter check [#2016](https://github.com/paritytech/parity/pull/2016)
- always process trie death row on commit, add more tracing [#2025](https://github.com/paritytech/parity/pull/2025)
- fixed transaction addresses mapping, fixes #1971 [#2026](https://github.com/paritytech/parity/pull/2026)
- Adding tests for dapps server. [#2021](https://github.com/paritytech/parity/pull/2021)
- builtin trait refactoring [#2018](https://github.com/paritytech/parity/pull/2018)
- Start parity with systemd [#1967](https://github.com/paritytech/parity/pull/1967)
- Control service for IPC [#2013](https://github.com/paritytech/parity/pull/2013)
- LRU cache for dapps [#2006](https://github.com/paritytech/parity/pull/2006)
- CLI for valid hosts for dapps server [#2005](https://github.com/paritytech/parity/pull/2005)
- Make the block header struct's internals private [#2000](https://github.com/paritytech/parity/pull/2000)
- Take control of recovered snapshots, start restoration asynchronously [#2010](https://github.com/paritytech/parity/pull/2010)
- remove internal locking from DBTransaction [#2003](https://github.com/paritytech/parity/pull/2003)
- Snapshot optimizations [#1991](https://github.com/paritytech/parity/pull/1991)
- Revert removing ecies [#2009](https://github.com/paritytech/parity/pull/2009)
- small blooms optimization [#1998](https://github.com/paritytech/parity/pull/1998)
- protection from adding empty traces && assertion in traces db [#1994](https://github.com/paritytech/parity/pull/1994)
- Stratum IPC service [#1959](https://github.com/paritytech/parity/pull/1959)
- Signature cleanup [#1921](https://github.com/paritytech/parity/pull/1921)
- Fixed discovery skipping some nodes [#1996](https://github.com/paritytech/parity/pull/1996)
- Trie query recording and AccountDB factory for no mangling [#1944](https://github.com/paritytech/parity/pull/1944)
- Validating sha3 of a dapp bundle [#1993](https://github.com/paritytech/parity/pull/1993)
- Improve eth_getWork timeout test rpc_get_work_should_timeout [#1992](https://github.com/paritytech/parity/pull/1992)
- Resolving URLs from contract [#1964](https://github.com/paritytech/parity/pull/1964)
- Add timeout for eth_getWork call [#1975](https://github.com/paritytech/parity/pull/1975)
- CLI for Signer interface [#1980](https://github.com/paritytech/parity/pull/1980)
- IPC timeout multiplied [#1990](https://github.com/paritytech/parity/pull/1990)
- Use relative path for IPC sockets [#1983](https://github.com/paritytech/parity/pull/1983)
- Market-orientated transaction pricing [#1963](https://github.com/paritytech/parity/pull/1963)
- Bump clippy [#1982](https://github.com/paritytech/parity/pull/1982)
- Fixing mutual recursive types serialization [#1977](https://github.com/paritytech/parity/pull/1977)
- Fix open on FreeBSD [#1984](https://github.com/paritytech/parity/pull/1984)
- Upgrade hyper dependency to 0.9 [#1973](https://github.com/paritytech/parity/pull/1973)
- Create network-specific nodes files [#1970](https://github.com/paritytech/parity/pull/1970)
- Getting rid of syntex [#1965](https://github.com/paritytech/parity/pull/1965)
- Remove binary specification from hypervisor [#1960](https://github.com/paritytech/parity/pull/1960)
- Stratum protocol general [#1954](https://github.com/paritytech/parity/pull/1954)
- keep track of first block in blockchain [#1937](https://github.com/paritytech/parity/pull/1937)
- introduce ethcore/state module [#1953](https://github.com/paritytech/parity/pull/1953)
- Apply settings to column families [#1956](https://github.com/paritytech/parity/pull/1956)
- move column family constants into db module [#1955](https://github.com/paritytech/parity/pull/1955)
- ECIES without MAC [#1948](https://github.com/paritytech/parity/pull/1948)
- Fix canny warnings [#1951](https://github.com/paritytech/parity/pull/1951)
- Fetchable dapps [#1949](https://github.com/paritytech/parity/pull/1949)
- remove impossible panickers related to infallible db transaction [#1947](https://github.com/paritytech/parity/pull/1947)
- Minor optimizations [#1943](https://github.com/paritytech/parity/pull/1943)
- remove randomness from bigint benches, fix warnings [#1945](https://github.com/paritytech/parity/pull/1945)
- Fix several RPCs [#1926](https://github.com/paritytech/parity/pull/1926)
- Bump clippy, fix warnings [#1939](https://github.com/paritytech/parity/pull/1939)
- DB WAL size limit [#1935](https://github.com/paritytech/parity/pull/1935)
- Use explicit global namespaces in codegen [#1928](https://github.com/paritytech/parity/pull/1928)
- Fix build on master [#1934](https://github.com/paritytech/parity/pull/1934)
- IPC on by default [#1927](https://github.com/paritytech/parity/pull/1927)
- fix util benches compilation [#1931](https://github.com/paritytech/parity/pull/1931)
- Update gitlab-ci [#1929](https://github.com/paritytech/parity/pull/1929)
- ethkey and ethstore use hash structures from bigint [#1851](https://github.com/paritytech/parity/pull/1851)

## Parity [v1.3.10](https://github.com/paritytech/parity/releases/tag/v1.3.10) (2016-11-04)

The latest 1.3 series release, now considered stable.

This includes several additional optimisations and fixes together with provisional support for the upcoming hard fork for EIP155/160/161.

- Stable branch reset to 1.3.10 [#3156](https://github.com/paritytech/parity/pull/3156)
- Backporting to beta [#3149](https://github.com/paritytech/parity/pull/3149)
- apply post-consolidation migrations after consolidating (BETA) [#3048](https://github.com/paritytech/parity/pull/3048)
- [beta] Fix the brainwallet functionality. (#2994) [#3005](https://github.com/paritytech/parity/pull/3005)
- Bumping json-ipc-server [#2989](https://github.com/paritytech/parity/pull/2989)
- Backports for 1.3.10 [#2987](https://github.com/paritytech/parity/pull/2987)

## Parity [v1.3.9](https://github.com/paritytech/parity/releases/tag/v1.3.9) (2016-10-21)

This release enables EIP-150 hard fork for Ethereum Classic chain and resolves a few stability and performance issues, such as:
- Interrupted syncing on the test network.
- Block import delays caused by a large number of incoming transactions. A full re-sync is recommended for performance improvement to take effect.

Full changes:
- [beta] Resolve morden fork [#2776](https://github.com/paritytech/parity/pull/2776)
- Fixing botched merge [#2767](https://github.com/paritytech/parity/pull/2767)
- Backports for beta [#2764](https://github.com/paritytech/parity/pull/2764)
- Introduce EIP150 hardfork block for ETC [#2736](https://github.com/paritytech/parity/pull/2736)
- [beta] fix issues with no test dir present (#2659) [#2724](https://github.com/paritytech/parity/pull/2724)
- [beta] Bumping jsonrpc-http-server [#2715](https://github.com/paritytech/parity/pull/2715)
- [beta] Fix migration system, better errors [#2661](https://github.com/paritytech/parity/pull/2661)

## Parity [v1.3.8](https://github.com/paritytech/parity/releases/tag/v1.3.8) (2016-10-15)

Parity 1.3.8 is our EIP150 hard-fork compliant release.

Running this will enact a mild change of the protocol at block number 2,463,000 which should occur on Tuesday 18th October 2016 at approximately 12:20 London time (BST). This change alters the gas prices for a number of operations, mainly centring around i/o intensive Merkle trie lookups (`BALANCE`, `EXTCODESIZE` &c.) and state-trie polluters (`SUICIDE`, `CREATE` and `CALL`). These operations were heavily underpriced, an oversight which lead to the recent degradation of network service. The full details of the alteration are specified in [EIP-150](https://github.com/ethereum/EIPs/issues/150).

Additionally several issues have been fixed including:
- a transaction queue limitation leading to dropped transactions;
- a synchronisation issue leading to stalls when syncing;

And some small features including database performance improvements and additional logging.

#### Upgrading private chain specification files.

All the chain specification files now have EIP-150 rules enabled by default. To continue using the chain add the `eip150Transition` key under `Engine/ethash/params` and set it to a future transition block as shown in [this example](https://github.com/paritytech/parity/blob/85eeb3ea6e5e21ad8e5644241edf82eb8069f536/ethcore/res/ethereum/morden.json#L13).

The key related to homestead transition has been renamed from `frontierCompatibilityModeLimit` to `homesteadTransition`.

#### Full changes

- [beta] EIP150.1c [#2599](https://github.com/paritytech/parity/pull/2599)
- Remove count limit for local transactions [#2634](https://github.com/paritytech/parity/pull/2634)
- Tweak DB and mining defaults [#2598](https://github.com/paritytech/parity/pull/2598)
- Revert "Bloom upgrade in beta" [#2635](https://github.com/paritytech/parity/pull/2635)
- Bloom upgrade in beta [#2609](https://github.com/paritytech/parity/pull/2609)
- Backports to beta [#2628](https://github.com/paritytech/parity/pull/2628)

## Parity [v1.3.7](https://github.com/paritytech/parity/releases/tag/v1.3.7) (2016-10-12)

This release contains fixes to reduce memory usage under the DoS attack and improve transaction relay.

- Configurable history size in beta [#2587](https://github.com/paritytech/parity/pull/2587)
- Backports to beta [#2592](https://github.com/paritytech/parity/pull/2592)


## Parity [v1.3.6](https://github.com/paritytech/parity/releases/tag/v1.3.6) (2016-10-11)

Parity 1.3.6 is another hotfix release to address transaction spam and deal with stability issues. With this release transaction pool gas limit no longer applies to local transactions. Full list of changes is available here:

- Backports to beta v1.3.6 [#2571](https://github.com/paritytech/parity/pull/2571)
- Use global state cache when mining [#2529](https://github.com/paritytech/parity/pull/2529)
- Transaction queue limited by gas [#2528](https://github.com/paritytech/parity/pull/2528)

## Parity [v1.3.5](https://github.com/paritytech/parity/releases/tag/v1.3.5) (2016-10-08)

1.3.5 is a hotfix release for the transaction propagation issue. Transaction pool limit is now calculated based on the block gas limit.

- Update appveyor rustc [beta] [#2521](https://github.com/paritytech/parity/pull/2521)
- Increase size of transaction queue by default [#2519](https://github.com/paritytech/parity/pull/2519)

## Parity [v1.3.4](https://github.com/paritytech/parity/releases/tag/v1.3.4) (2016-10-07)

Parity 1.3.4 release contains more optimizations to internal caching as well as stability improvements.

It also introduces an ability for miners to choose a transaction ordering strategy:

    --tx-queue-strategy S    Prioritization strategy used to order transactions
                             in the queue. S may be:
                             gas - Prioritize txs with low gas limit;
                             gas_price - Prioritize txs with high gas price;
                             gas_factor - Prioritize txs using gas price
                             and gas limit ratio [default: gas_factor].

- Backport to beta [#2518](https://github.com/paritytech/parity/pull/2518)
- [beta] Fixing RPC Filter conversion to EthFilter [#2501](https://github.com/paritytech/parity/pull/2501)
- [beta] Using pending block only if is not old [#2515](https://github.com/paritytech/parity/pull/2515)
- Backports into beta [#2512](https://github.com/paritytech/parity/pull/2512)
- CLI to specify queue ordering strategy [#2494](https://github.com/paritytech/parity/pull/2494)
- Fix ethstore opening all key files in the directory at once (BETA) [#2472](https://github.com/paritytech/parity/pull/2472)
- Beta backports [#2465](https://github.com/paritytech/parity/pull/2465)
- IPC-library dependency fork & bump for beta [#2455](https://github.com/paritytech/parity/pull/2455)

## Parity [v1.3.3](https://github.com/paritytech/parity/releases/tag/v1.3.3) (2016-10-04)

1.3.3 is another hotfix release for the DoS attack

- Jumptable cache [#2435](https://github.com/paritytech/parity/pull/2435)
- fix broken beta compilation (backport to beta) [#2414](https://github.com/paritytech/parity/pull/2414)
- Run inplace upgrades after version update [#2411](https://github.com/paritytech/parity/pull/2411)

## Parity [v1.3.2](https://github.com/paritytech/parity/releases/tag/v1.3.2) (2016-09-29)

This is a hotfix release to address stability and performance issues uncovered during the network DoS attack. Full list of changes is available [here](https://github.com/paritytech/parity/compare/v1.3.1...v1.3.2)

- Beta Backports [#2396](https://github.com/paritytech/parity/pull/2396)
- Fixing penalization in future [#2493](https://github.com/paritytech/parity/pull/2493)
- A quick fix for missing tree route blocks [#2400](https://github.com/paritytech/parity/pull/2400)
- Cache the fork block header after snapshot restoration [#2391](https://github.com/paritytech/parity/pull/2391)
- correct sync memory usage calculation (BETA) [#2386](https://github.com/paritytech/parity/pull/2386)
- Accounts bloom [#2357](https://github.com/paritytech/parity/pull/2357)
- Disable colors when generating signer token. [#2379](https://github.com/paritytech/parity/pull/2379)
- Fixing jit feature compilation [#2376](https://github.com/paritytech/parity/pull/2376)
- Clear state cache on sealed block import [#2377](https://github.com/paritytech/parity/pull/2377)
- DIV optimization (beta) [#2353](https://github.com/paritytech/parity/pull/2353)
- Canonical state cache [#2308](https://github.com/paritytech/parity/pull/2308)
- Reorder transaction_by_hash to favour canon search [#2331](https://github.com/paritytech/parity/pull/2331)
- Lenient bytes deserialization [#2340](https://github.com/paritytech/parity/pull/2340)
- Penalize transactions with gas above gas limit [#2271](https://github.com/paritytech/parity/pull/2271)
- Peek transaction queue via RPC [#2270](https://github.com/paritytech/parity/pull/2270)
- Handle RLP to string UTF-8 decoding errors (#2217) [#2226](https://github.com/paritytech/parity/pull/2226)
- Fixing compilation without default features [beta] [#2207](https://github.com/paritytech/parity/pull/2207)
- Avoid cloning clean stuff [beta backport] [#2173](https://github.com/paritytech/parity/pull/2173)
- v1.3.2 in beta [#2200](https://github.com/paritytech/parity/pull/2200)

## Parity [v1.3.1](https://github.com/paritytech/parity/releases/tag/v1.3.1) (2016-09-11)

1.3.1 includes many [bugfixes](https://github.com/paritytech/parity/commit/2a82fa0a47b00bedfec520a2fdd3cc31aa4ccd8c). Critical ones:
- **Chain reorganisation fix** Transaction receipts / traces were sometimes linked with incorrect block hash. Fixed in https://github.com/paritytech/parity/commit/a9587f8965a32c84973c35ce1c8d51d07044143f
- **Trace overflow fix** Overflow which occurred during tracing. Fixed in https://github.com/paritytech/parity/pull/1979

- Backports to beta [#2068](https://github.com/paritytech/parity/pull/2068)
- Fixing serde overflow error (#1977) [#2030](https://github.com/paritytech/parity/pull/2030)
- Simplified db pruning detection in beta [#1924](https://github.com/paritytech/parity/pull/1924)
- Backports to beta [#1919](https://github.com/paritytech/parity/pull/1919)

## Parity [v1.3.0: "Acuity"](https://github.com/paritytech/parity/releases/tag/v1.3.0) (2016-08-12)

As well as many bug fixes, 1.3.0 includes a number of important improvements including:
- **Optimisations** Heavily optimised block/transaction processing core - up to 2x faster than 1.2 series.
- **Database compression** Databases take as much as 30% less storage than before.
- **State snapshotting** An installation synchronised from scratch in 1-2 minutes can be made after downloading the 140MB state snapshot. See [the wiki](https://github.com/paritytech/parity/wiki/Getting-Synced) for more information.
- **Process isolation** The networking/chain-synchronisation is now a fully independent process.

Incremental improvements include:
- Additional [RPCs](https://github.com/paritytech/parity/wiki/JSONRPC) for transaction tracing, state diffing, VM tracing, asynchronous transaction posting, accounts metadata and message signing.
- Improved logging, including for chain reorganisations.
- Added a `--fast-and-loose` option for additional speed-ups which can compromise integrity on a dirty shutdown.
- Column families to ensure maximal inter-database integrity.
- Key naming includes date/time of creation.
- Various improvements to networking robustness and performance.
- Solidity compilation supported through RPC if `solc` is available.
- Various improvements to the miner including [HTTP push work notification](https://github.com/ethcoreparitytech/parity/wiki/Mining#starting-it).

Full changes:
- Bumping Parity UI [#1920](https://github.com/paritytech/parity/pull/1920)
- Adding entrypoints to docker images [#1909](https://github.com/paritytech/parity/pull/1909)
- Save nodes removed from backing_overlay until commit [#1917](https://github.com/paritytech/parity/pull/1917)
- RPC for importing geth keys [#1916](https://github.com/paritytech/parity/pull/1916)
- Peers RPC + UI displaying active/connected/max peers [#1915](https://github.com/paritytech/parity/pull/1915)
- RPC for deriving address from phrase. [#1912](https://github.com/paritytech/parity/pull/1912)
- adjust polling & connection timeouts for ipc [#1910](https://github.com/paritytech/parity/pull/1910)
- Don't return deleted nodes that are not yet flushed [#1908](https://github.com/paritytech/parity/pull/1908)
- Wallet rpcs [#1898](https://github.com/paritytech/parity/pull/1898)
- Fix binary serialization bug [#1907](https://github.com/paritytech/parity/pull/1907)
- fixed #1889, .DS_Store is no longer treated as key file [#1892](https://github.com/paritytech/parity/pull/1892)
- Purging .derefs, fixing clippy warnings. [#1890](https://github.com/paritytech/parity/pull/1890)
- RocksDB version bump [#1904](https://github.com/paritytech/parity/pull/1904)
- Fix ipc compilation and add ipc feature to test targets [#1902](https://github.com/paritytech/parity/pull/1902)
- Autocreating geth dir if none and geth mode on [#1896](https://github.com/paritytech/parity/pull/1896)
- v1.4.0 in master [#1886](https://github.com/paritytech/parity/pull/1886)
- Adding more details to miner log [#1891](https://github.com/paritytech/parity/pull/1891)
- moved hash.rs to bigint library [#1827](https://github.com/paritytech/parity/pull/1827)
- fixed cache_manager lock order [#1877](https://github.com/paritytech/parity/pull/1877)
- Fixing miner deadlock [#1885](https://github.com/paritytech/parity/pull/1885)
- Updating WS + Increasing token validity [#1882](https://github.com/paritytech/parity/pull/1882)
- take snapshot at specified block and slightly better informants [#1873](https://github.com/paritytech/parity/pull/1873)
- RPC errors & logs [#1845](https://github.com/paritytech/parity/pull/1845)
- Reduce max open files [#1876](https://github.com/paritytech/parity/pull/1876)
- Send new block hashes to all peers [#1875](https://github.com/paritytech/parity/pull/1875)
- Use UntrustedRlp for block verification [#1872](https://github.com/paritytech/parity/pull/1872)
- Update cache usage on commiting block info [#1871](https://github.com/paritytech/parity/pull/1871)
- Validating conversion U256->usize when doing gas calculation (for 32bits) [#1870](https://github.com/paritytech/parity/pull/1870)
- Sync to peers with confirmed fork block only [#1863](https://github.com/paritytech/parity/pull/1863)
- miner and client take spec reference [#1853](https://github.com/paritytech/parity/pull/1853)
- Unlock account with timeout for geth compatibility [#1854](https://github.com/paritytech/parity/pull/1854)
- Fixed reported max height and transaction propagation [#1852](https://github.com/paritytech/parity/pull/1852)
- Snapshot creation and restoration [#1679](https://github.com/paritytech/parity/pull/1679)
- fix deprecated typo [#1850](https://github.com/paritytech/parity/pull/1850)
- Split IO and network crates [#1828](https://github.com/paritytech/parity/pull/1828)
- updated classic JSON spec with classic bootnodes, fixes #1842 [#1847](https://github.com/paritytech/parity/pull/1847)
- protect unsafety in plainhasher; get more unique hashes [#1841](https://github.com/paritytech/parity/pull/1841)
- use mutex in dbtransaction [#1843](https://github.com/paritytech/parity/pull/1843)
- Fix state not using "account_starting_nonce" [#1830](https://github.com/paritytech/parity/pull/1830)
- Supporting blockid in eth_call and trace_call/trace_raw [#1837](https://github.com/paritytech/parity/pull/1837)
- eth_checkTransaction renamed to eth_checkRequest [#1817](https://github.com/paritytech/parity/pull/1817)
- Bump json-ipc-server again [#1839](https://github.com/paritytech/parity/pull/1839)
- Fixing another deadlock in trace db [#1833](https://github.com/paritytech/parity/pull/1833)
- Fix up the VM trace. [#1829](https://github.com/paritytech/parity/pull/1829)
- fixed parsing export params, fixes #1826 [#1834](https://github.com/paritytech/parity/pull/1834)
- More performance optimizations [#1814](https://github.com/paritytech/parity/pull/1814)
- Bumping clippy & fixing warnings [#1823](https://github.com/paritytech/parity/pull/1823)
- removed unused code from util and unnecessary dependency of FixedHash [#1824](https://github.com/paritytech/parity/pull/1824)
- Remove (almost all) panickers from trie module [#1776](https://github.com/paritytech/parity/pull/1776)
- Fixing account naming [#1810](https://github.com/paritytech/parity/pull/1810)
- JournalDB inject [#1806](https://github.com/paritytech/parity/pull/1806)
- No block number in get work while in geth-compat mode. [#1821](https://github.com/paritytech/parity/pull/1821)
- Import wallet fix [#1820](https://github.com/paritytech/parity/pull/1820)
- Supporting eth_sign in Signer [#1787](https://github.com/paritytech/parity/pull/1787)
- Fixing cache update after chain reorg [#1816](https://github.com/paritytech/parity/pull/1816)
- Development mode for Signer UI [#1788](https://github.com/paritytech/parity/pull/1788)
- Miner tweaks [#1797](https://github.com/paritytech/parity/pull/1797)
- Util & ipc clenup [#1807](https://github.com/paritytech/parity/pull/1807)
- Fixing unlock parsing [#1802](https://github.com/paritytech/parity/pull/1802)
- fixed importing presale wallet with encseed longer than 96 bytes [#1801](https://github.com/paritytech/parity/pull/1801)
- DRYing build scripts [#1795](https://github.com/paritytech/parity/pull/1795)
- Allow code from spec json [#1790](https://github.com/paritytech/parity/pull/1790)
- nano-tests (ipc transport) to the CI [#1793](https://github.com/paritytech/parity/pull/1793)
- Commit best block after closing transaction [#1791](https://github.com/paritytech/parity/pull/1791)
- Place thread name in the log output [#1792](https://github.com/paritytech/parity/pull/1792)
- Fix ipc tests and bring to CI [#1789](https://github.com/paritytech/parity/pull/1789)
- dynamic keys pickup [#1779](https://github.com/paritytech/parity/pull/1779)
- ipc version bump [#1783](https://github.com/paritytech/parity/pull/1783)
- Prevent deadlock on trace GC [#1780](https://github.com/paritytech/parity/pull/1780)
- fixed trace_transaction crash when block contained suicide [#1781](https://github.com/paritytech/parity/pull/1781)
- Fix block body migration [#1777](https://github.com/paritytech/parity/pull/1777)
- cache manager and clearing tracing cache [#1769](https://github.com/paritytech/parity/pull/1769)
- Return storage as H256 from RPC. [#1774](https://github.com/paritytech/parity/pull/1774)
- Instant sealing engine [#1767](https://github.com/paritytech/parity/pull/1767)
- fix state unsafety with a mostly-guaranteed handle [#1755](https://github.com/paritytech/parity/pull/1755)
- Gas for mem optimization [#1768](https://github.com/paritytech/parity/pull/1768)
- Min and Max peers setting [#1771](https://github.com/paritytech/parity/pull/1771)
- Disable WAL [#1765](https://github.com/paritytech/parity/pull/1765)
- Add new line when printing start strings [#1766](https://github.com/paritytech/parity/pull/1766)
- Log tweak [#1764](https://github.com/paritytech/parity/pull/1764)
- Remove update_sealing call on importing own block [#1762](https://github.com/paritytech/parity/pull/1762)
- Single DB [#1741](https://github.com/paritytech/parity/pull/1741)
- Tweak format of log so it's not so verbose. [#1758](https://github.com/paritytech/parity/pull/1758)
- Combine mining queue and enabled into single locked datum [#1749](https://github.com/paritytech/parity/pull/1749)
- Collect consensus/null engines into a single module [#1754](https://github.com/paritytech/parity/pull/1754)
- Fix failing deserialization test [#1756](https://github.com/paritytech/parity/pull/1756)
- Stackoverflow fix [#1742](https://github.com/paritytech/parity/pull/1742)
- compaction profile used during migration, fixes #1750 [#1751](https://github.com/paritytech/parity/pull/1751)
- Splitting documentation into separate build job [#1752](https://github.com/paritytech/parity/pull/1752)
- handle keys deserialization errors, fixes #1592 [#1701](https://github.com/paritytech/parity/pull/1701)
- add gitlab-ci yaml [#1753](https://github.com/paritytech/parity/pull/1753)
- Better handling of multiple migrations [#1747](https://github.com/paritytech/parity/pull/1747)
- Disconnect peers on a fork [#1738](https://github.com/paritytech/parity/pull/1738)
- Add RPC & client call to replay a transaction. [#1734](https://github.com/paritytech/parity/pull/1734)
- another version bump for jsonrpc-ipc [#1744](https://github.com/paritytech/parity/pull/1744)
- Trace other types of calls [#1727](https://github.com/paritytech/parity/pull/1727)
- Fixing compilation on latest nightly [#1736](https://github.com/paritytech/parity/pull/1736)
- Blocks and snapshot compression [#1687](https://github.com/paritytech/parity/pull/1687)
- bump json-ipc-server version [#1739](https://github.com/paritytech/parity/pull/1739)
- Use std::sync::Condvar [#1732](https://github.com/paritytech/parity/pull/1732)
- Bump json-ipc-server version [#1733](https://github.com/paritytech/parity/pull/1733)
- bump json-ipc-server version [#1731](https://github.com/paritytech/parity/pull/1731)
- Fixing some clippy warnings [#1728](https://github.com/paritytech/parity/pull/1728)
- Bumping Parity UI [#1682](https://github.com/paritytech/parity/pull/1682)
- Various improvements to tracing & diagnostics. [#1707](https://github.com/paritytech/parity/pull/1707)
- Fixed reading chunked EIP8 handshake [#1712](https://github.com/paritytech/parity/pull/1712)
- Fix for importing blocks from a pipe file [#1724](https://github.com/paritytech/parity/pull/1724)
- Proper errors for binary serializer [#1714](https://github.com/paritytech/parity/pull/1714)
- Use a transaction for writing blocks [#1718](https://github.com/paritytech/parity/pull/1718)
- Exclude generated code from coverage [#1720](https://github.com/paritytech/parity/pull/1720)
- Use single binary for ipc modules [#1710](https://github.com/paritytech/parity/pull/1710)
- Log a chain-reorg. [#1715](https://github.com/paritytech/parity/pull/1715)
- Restore new block informant message [#1716](https://github.com/paritytech/parity/pull/1716)
- Parallel block body download [#1659](https://github.com/paritytech/parity/pull/1659)
- Rotate blockchain cache [#1709](https://github.com/paritytech/parity/pull/1709)
- Fix broken internal names. [#1711](https://github.com/paritytech/parity/pull/1711)
- cli overhaul [#1600](https://github.com/paritytech/parity/pull/1600)
- Key files include timestamp in name. [#1700](https://github.com/paritytech/parity/pull/1700)
- Fixing warnings [#1705](https://github.com/paritytech/parity/pull/1705)
- Ethereum classic [#1706](https://github.com/paritytech/parity/pull/1706)
- Docker Arguments [#1703](https://github.com/paritytech/parity/pull/1703)
- Informant tidyup. [#1699](https://github.com/paritytech/parity/pull/1699)
- Name and meta in accounts [#1695](https://github.com/paritytech/parity/pull/1695)
- Stackoverflow #1686 [#1698](https://github.com/paritytech/parity/pull/1698)
- filtering transactions toAddress includes contract creation [#1697](https://github.com/paritytech/parity/pull/1697)
- Prevent syncing to ancient blocks [#1693](https://github.com/paritytech/parity/pull/1693)
- Enable WAL and disable DB repair [#1696](https://github.com/paritytech/parity/pull/1696)
- Returning error when transaction is rejected (for consistency) [#1667](https://github.com/paritytech/parity/pull/1667)
- Disabling signer when in geth-compatibility mode [#1676](https://github.com/paritytech/parity/pull/1676)
- Suicides tracing [#1688](https://github.com/paritytech/parity/pull/1688)
- small cleanup of substate.rs [#1685](https://github.com/paritytech/parity/pull/1685)
- resolve #411: remove install scripts [#1684](https://github.com/paritytech/parity/pull/1684)
- IPC (feature-gated) [#1654](https://github.com/paritytech/parity/pull/1654)
- Bumping JSONRPC-http-server [#1678](https://github.com/paritytech/parity/pull/1678)
- Fixing hash deserialisation [#1674](https://github.com/paritytech/parity/pull/1674)
- Ping discovery nodes gradually [#1671](https://github.com/paritytech/parity/pull/1671)
- Fixing the deadlock on incoming connection [#1672](https://github.com/paritytech/parity/pull/1672)
- Fixing errors returned by sendTransaction* method family [#1665](https://github.com/paritytech/parity/pull/1665)
- Moved syncing log out of the client [#1670](https://github.com/paritytech/parity/pull/1670)
- Host validation (again) [#1666](https://github.com/paritytech/parity/pull/1666)
- Update install-deps.sh [ci skip] [#1664](https://github.com/paritytech/parity/pull/1664)
- fix typos [#1644](https://github.com/paritytech/parity/pull/1644)
- Size for blocks [#1668](https://github.com/paritytech/parity/pull/1668)
- Revert "Validating Host headers in RPC requests" [#1663](https://github.com/paritytech/parity/pull/1663)
- Validating Host headers in RPC requests [#1658](https://github.com/paritytech/parity/pull/1658)
- fixed failing master [#1662](https://github.com/paritytech/parity/pull/1662)
- Fixing clippy warnings [#1660](https://github.com/paritytech/parity/pull/1660)
- Don't ping all nodes on start [#1656](https://github.com/paritytech/parity/pull/1656)
- More performance optimizations [#1649](https://github.com/paritytech/parity/pull/1649)
- Removing unused client code [#1645](https://github.com/paritytech/parity/pull/1645)
- Asynchronous transactions (polling based for now). [#1652](https://github.com/paritytech/parity/pull/1652)
- Sync stand-alone binary and feature-gated dependencies refactoring  [#1637](https://github.com/paritytech/parity/pull/1637)
- Re-enabling Parity UI [#1627](https://github.com/paritytech/parity/pull/1627)
- Blockchain repair on missing state root [#1646](https://github.com/paritytech/parity/pull/1646)
- Multi-mode logging. [#1643](https://github.com/paritytech/parity/pull/1643)
- Pro paths [#1650](https://github.com/paritytech/parity/pull/1650)
- Performance optimizations [#1642](https://github.com/paritytech/parity/pull/1642)
- Removed DAO soft fork traces [#1639](https://github.com/paritytech/parity/pull/1639)
- Compiler version update for windows [#1638](https://github.com/paritytech/parity/pull/1638)
- Delete values immediately from DB overlay [#1631](https://github.com/paritytech/parity/pull/1631)
- DAO hard-fork [#1483](https://github.com/paritytech/parity/pull/1483)
- fix network_start regression [#1629](https://github.com/paritytech/parity/pull/1629)
- Die if the DB is newer than the one supported. [#1630](https://github.com/paritytech/parity/pull/1630)
- Cleanup of colour code. Use is_a_tty. [#1621](https://github.com/paritytech/parity/pull/1621)
- don't batch best block for branches [#1623](https://github.com/paritytech/parity/pull/1623)
- In-memory trie operations [#1408](https://github.com/paritytech/parity/pull/1408)
- Fix "pending" parameter on RPC block requests [#1602](https://github.com/paritytech/parity/pull/1602)
- Allow RPC to use solc to compile solidity [#1607](https://github.com/paritytech/parity/pull/1607)
- IPC RPC deriving for traits [#1599](https://github.com/paritytech/parity/pull/1599)
- Utilize cached kcov if exists [#1619](https://github.com/paritytech/parity/pull/1619)
- Fixing no-ui feature [#1618](https://github.com/paritytech/parity/pull/1618)
- Couple of rocksdb optimizations [#1614](https://github.com/paritytech/parity/pull/1614)
- Miner tests [#1597](https://github.com/paritytech/parity/pull/1597)
- Sync IPC interface [#1584](https://github.com/paritytech/parity/pull/1584)
- Make sure reserved peers are in the node table [#1616](https://github.com/paritytech/parity/pull/1616)
- Fix bloomchain on blockchain repair [#1610](https://github.com/paritytech/parity/pull/1610)
- fixed broken tracing [#1615](https://github.com/paritytech/parity/pull/1615)
- fix benchmark compilation [#1612](https://github.com/paritytech/parity/pull/1612)
- Updating jsonrpc-http-server [#1611](https://github.com/paritytech/parity/pull/1611)
- replace synchronization primitives with those from parking_lot [#1593](https://github.com/paritytech/parity/pull/1593)
- ui compilation feature [#1604](https://github.com/paritytech/parity/pull/1604)
- is_zero() and pow() optimisations for uint [#1608](https://github.com/paritytech/parity/pull/1608)
- Optimizing & Cleaning the build [#1591](https://github.com/paritytech/parity/pull/1591)
- Fix logging [#1590](https://github.com/paritytech/parity/pull/1590)
- remove unnecessary mutex in logging [#1601](https://github.com/paritytech/parity/pull/1601)
- Using streamlined parity-ui repository [#1566](https://github.com/paritytech/parity/pull/1566)
- Optimizing InstructionInfo access. [#1595](https://github.com/paritytech/parity/pull/1595)
- V7 Migration progress indicator [#1594](https://github.com/paritytech/parity/pull/1594)
- bring snapshotting work into master [#1577](https://github.com/paritytech/parity/pull/1577)
- Bump clippy [#1587](https://github.com/paritytech/parity/pull/1587)
- refactoring of handshake messages serialization in ipc [#1586](https://github.com/paritytech/parity/pull/1586)
- expunge &Vec<T> pattern [#1579](https://github.com/paritytech/parity/pull/1579)
- EVM gas for memory tiny optimization [#1578](https://github.com/paritytech/parity/pull/1578)
- cleaned up parity/signer [#1551](https://github.com/paritytech/parity/pull/1551)
- Major sync <-> client interactions refactoring [#1572](https://github.com/paritytech/parity/pull/1572)
- failing test with overlayrecent pruning [#1567](https://github.com/paritytech/parity/pull/1567)
- Enable state queries for OverlayRecent DB [#1575](https://github.com/paritytech/parity/pull/1575)
- have AccountDB use address hash for uniqueness [#1533](https://github.com/paritytech/parity/pull/1533)
- Very basic EVM binary. [#1574](https://github.com/paritytech/parity/pull/1574)
- Some obvious evm & uint optimizations  [#1576](https://github.com/paritytech/parity/pull/1576)
- Fixing clippy warnings [#1568](https://github.com/paritytech/parity/pull/1568)
- Miner's gas price gets updated dynamically [#1570](https://github.com/paritytech/parity/pull/1570)
- bringing hypervisor as a crate in ipc dir [#1565](https://github.com/paritytech/parity/pull/1565)
- Init public interface with IO message [#1573](https://github.com/paritytech/parity/pull/1573)
- Uncommenting simple Miner tests [#1571](https://github.com/paritytech/parity/pull/1571)
- Kill lock unwraps [#1558](https://github.com/paritytech/parity/pull/1558)
- Fixing deadlock in miner [#1569](https://github.com/paritytech/parity/pull/1569)
- Idealpeers in log [#1563](https://github.com/paritytech/parity/pull/1563)
- Simple style fix. [#1561](https://github.com/paritytech/parity/pull/1561)
- Enum variants serialisation test&fix [#1559](https://github.com/paritytech/parity/pull/1559)
- Supporting /api/ping for dapps server [#1543](https://github.com/paritytech/parity/pull/1543)
- Client IPC Interface [#1493](https://github.com/paritytech/parity/pull/1493)
- Kill timers when removing IO handler [#1554](https://github.com/paritytech/parity/pull/1554)
- Fix and add info messages [#1552](https://github.com/paritytech/parity/pull/1552)
- Fix indent of #1541 [#1555](https://github.com/paritytech/parity/pull/1555)
- Update sealing just once when  externally importing many blocks [#1541](https://github.com/paritytech/parity/pull/1541)
- Remove soft-fork stuff. [#1548](https://github.com/paritytech/parity/pull/1548)
- fix codegen warning [#1550](https://github.com/paritytech/parity/pull/1550)
- Extend migration framework [#1546](https://github.com/paritytech/parity/pull/1546)
- Refactoring dapps to support API endpoints. [#1542](https://github.com/paritytech/parity/pull/1542)
- serde is no longer util dependency [#1534](https://github.com/paritytech/parity/pull/1534)
- mention wiki in README [#1549](https://github.com/paritytech/parity/pull/1549)
- Skipping transactions with invalid nonces when pushing to block. [#1545](https://github.com/paritytech/parity/pull/1545)
- Silent running operating modes [#1477](https://github.com/paritytech/parity/pull/1477)
- util cleanup [#1474](https://github.com/paritytech/parity/pull/1474)
- Calculating gas using usize (if supplied gaslimit fits in usize) [#1518](https://github.com/paritytech/parity/pull/1518)
- add owning NibbleVec [#1536](https://github.com/paritytech/parity/pull/1536)
- Attempt to fix blochchain/extras DBs sync [#1538](https://github.com/paritytech/parity/pull/1538)
- Client API refactoring - limiting errors to crate-level error types [#1525](https://github.com/paritytech/parity/pull/1525)
- IPC codegen enhancement - allow void methods [#1540](https://github.com/paritytech/parity/pull/1540)
- Fixing serving nested files for dapps. [#1539](https://github.com/paritytech/parity/pull/1539)
- Fixed public address config [#1537](https://github.com/paritytech/parity/pull/1537)
- Fixing compilation&clippy warnings [#1531](https://github.com/paritytech/parity/pull/1531)
- creating ethereum dir while in geth mode [#1530](https://github.com/paritytech/parity/pull/1530)
- Bumping clippy [#1532](https://github.com/paritytech/parity/pull/1532)
- Make signer default as long as --unlock isn't used. [#1524](https://github.com/paritytech/parity/pull/1524)
- add client timeout when requesting usd price for gas [#1526](https://github.com/paritytech/parity/pull/1526)
- Fix gitter-url link in README.md [#1528](https://github.com/paritytech/parity/pull/1528)
- Fix error message. [#1527](https://github.com/paritytech/parity/pull/1527)
- BTreeMap binary serialization [#1489](https://github.com/paritytech/parity/pull/1489)
- Save block reference in the queue on notification [#1501](https://github.com/paritytech/parity/pull/1501)
- bigint tests to run on CI [#1522](https://github.com/paritytech/parity/pull/1522)
- Client api cleaning - uncles are returned as rlp [#1516](https://github.com/paritytech/parity/pull/1516)
- Fatdb integration with CLI [#1464](https://github.com/paritytech/parity/pull/1464)
- Optimizing/simplifying shr [#1517](https://github.com/paritytech/parity/pull/1517)
- change IPC codegen to allow attributes [#1500](https://github.com/paritytech/parity/pull/1500)
- Fix warnings [#1514](https://github.com/paritytech/parity/pull/1514)
- FatDB [#1452](https://github.com/paritytech/parity/pull/1452)
- Fix the reseal mechanism. [#1513](https://github.com/paritytech/parity/pull/1513)
- Update Dockerfile ubuntu-aarch64 [#1509](https://github.com/paritytech/parity/pull/1509)
- Update Ubuntu-arm Dockerfile [#1510](https://github.com/paritytech/parity/pull/1510)
- Update Ubuntu-jit Dockerfile [#1511](https://github.com/paritytech/parity/pull/1511)
- Update Ubuntu Dockerfile [#1512](https://github.com/paritytech/parity/pull/1512)
- Update CentOS Dockerfile [#1508](https://github.com/paritytech/parity/pull/1508)
- bump status page v0.5.1 [#1502](https://github.com/paritytech/parity/pull/1502)
- Update CentOS Dockerfile [#1507](https://github.com/paritytech/parity/pull/1507)
- Update Dockerfile ubuntu-aarch64 [#1506](https://github.com/paritytech/parity/pull/1506)
- Update Ubuntu-arm Dockerfile [#1505](https://github.com/paritytech/parity/pull/1505)
- Update Ubuntu-jit Dockerfile [#1504](https://github.com/paritytech/parity/pull/1504)
- Update Ubuntu Dockerfile [#1503](https://github.com/paritytech/parity/pull/1503)
- Optionally clone block behind work-package [#1497](https://github.com/paritytech/parity/pull/1497)
- Fix no colour on windows. [#1498](https://github.com/paritytech/parity/pull/1498)
- Workaround for hyper panic [#1495](https://github.com/paritytech/parity/pull/1495)
- Colourful notification on mine [#1488](https://github.com/paritytech/parity/pull/1488)
- Quick fix for max open files error [#1494](https://github.com/paritytech/parity/pull/1494)
- Work notification over HTTP [#1491](https://github.com/paritytech/parity/pull/1491)
- Sealed block importing and propagation optimization [#1478](https://github.com/paritytech/parity/pull/1478)
- vm factory to mining client [#1487](https://github.com/paritytech/parity/pull/1487)
- topbar dialog fix [#1479](https://github.com/paritytech/parity/pull/1479)
- Minor additions to allow resetting of code. [#1482](https://github.com/paritytech/parity/pull/1482)
- Introduce options for fine-grained management of work queue. [#1484](https://github.com/paritytech/parity/pull/1484)
- Snapshot state restoration [#1308](https://github.com/paritytech/parity/pull/1308)
- Merge master into pv64 branch [#1486](https://github.com/paritytech/parity/pull/1486)
- Ensure we don't reject our own transactions for gasprice. [#1485](https://github.com/paritytech/parity/pull/1485)
- Signing parity executable & windows installer in appveyor [#1481](https://github.com/paritytech/parity/pull/1481)
- Rearrange fork CLI options. [#1476](https://github.com/paritytech/parity/pull/1476)
- give appveyor some breath [#1475](https://github.com/paritytech/parity/pull/1475)
- Ensure we always get the latest work when mining on submitted. [#1469](https://github.com/paritytech/parity/pull/1469)
- Tests for views [#1471](https://github.com/paritytech/parity/pull/1471)
- json ipc version bump [#1470](https://github.com/paritytech/parity/pull/1470)
- verifier is no longer a template type of client [#1467](https://github.com/paritytech/parity/pull/1467)
- Allow configuration of when to reseal blocks. [#1460](https://github.com/paritytech/parity/pull/1460)
- removed unsafe code [#1466](https://github.com/paritytech/parity/pull/1466)
- WS bump + Adding default for value [#1465](https://github.com/paritytech/parity/pull/1465)
- Attempt DB repair if corrupted [#1461](https://github.com/paritytech/parity/pull/1461)
- Database configuration extended [#1454](https://github.com/paritytech/parity/pull/1454)
- Updating WS-RS server [#1459](https://github.com/paritytech/parity/pull/1459)
- Reduced IO messages; removed panics on IO notifications [#1457](https://github.com/paritytech/parity/pull/1457)
- Handle errors when starting parity --signer [#1451](https://github.com/paritytech/parity/pull/1451)
- Fixed losing queued blocks on error [#1453](https://github.com/paritytech/parity/pull/1453)
- Updated to latest hyper with patched mio [#1450](https://github.com/paritytech/parity/pull/1450)
- Retweak BASE and MULTIPLIER in rocksdb config. [#1445](https://github.com/paritytech/parity/pull/1445)
- Removing Miner::default. [#1410](https://github.com/paritytech/parity/pull/1410)
- Don't mine without --author [#1436](https://github.com/paritytech/parity/pull/1436)
- Revert the rescuedao extradata. [#1437](https://github.com/paritytech/parity/pull/1437)
- More conservative settings for rocksdb. [#1440](https://github.com/paritytech/parity/pull/1440)
- v1.3.0 in master [#1421](https://github.com/paritytech/parity/pull/1421)
- Update Ubuntu-arm Dockerfile [#1429](https://github.com/paritytech/parity/pull/1429)
- Create Dockerfile ubuntu-aarch64 [#1430](https://github.com/paritytech/parity/pull/1430)
- Update CentOS Dockerfile [#1424](https://github.com/paritytech/parity/pull/1424)
- Update Ubuntu Dockerfile [#1426](https://github.com/paritytech/parity/pull/1426)
- Update Ubuntu-jit Dockerfile [#1427](https://github.com/paritytech/parity/pull/1427)
- Update SF blocknumber to 1800000. [#1418](https://github.com/paritytech/parity/pull/1418)

## Parity [v1.2.4](https://github.com/paritytech/parity/releases/tag/v1.2.4) (2016-08-09)

Parity 1.2.4 Is a maintenance release that fixes a [few](https://github.com/paritytech/parity/pull/1888/commits) issues related to mining and peer synchronization.
This release is marked as stable.

- Backports for beta [#1888](https://github.com/paritytech/parity/pull/1888)
- BETA: fixed trace_transaction crash when block contained suicide [#1782](https://github.com/paritytech/parity/pull/1782)

## Parity [v1.2.3](https://github.com/paritytech/parity/releases/tag/v1.2.3) (2016-07-31)

Parity 1.2.3 is a patch release that addresses network stability issues for both Ethereum HF and Ethereum classic chains and brings a few changes to the transaction tracing API.

#### Tracing API changes
- Added tracing for `CALLCODE`, `DELEGATECALL` and `SUICIDE`
- `trace_call` returns traces in flat format
- Added 2 new methods: `trace_rawTransaction` and `trace_replayTransaction`

Note that to continue using tracing features in this version you need to re-sync the blockchain. This can be done by using `parity export $HOME/ethereum-chain-backup.rlp` , deleting the database usually located at `~/.parity/906a34e69aec8c0d` followed by `parity import $HOME/ethereum-chain-backup.rlp`.

- [beta] Updating UI [#1778](https://github.com/paritytech/parity/pull/1778)
- tracing backport [#1770](https://github.com/paritytech/parity/pull/1770)
- Backport commits to beta [#1763](https://github.com/paritytech/parity/pull/1763)
- Deadlock on incoming connection (#1672) [#1675](https://github.com/paritytech/parity/pull/1675)
- [BETA] Removed DAO soft fork traces [#1640](https://github.com/paritytech/parity/pull/1640)


## Parity [v1.2.2](https://github.com/paritytech/parity/releases/tag/v1.2.2) (2016-07-16)

#### New
- DAO hard-fork.

DAO hard-fork implementation conforms to the [specification](https://blog.slock.it/hard-fork-specification-24b889e70703) and is enabled by default.

#### Changed
- `--reseal-on-txs` defaults to `own`.
- DAO soft-fork support has been removed along with related command line options.

#### Resolved issues
- `--db-cache-size` consuming too much memory.
  `eth_getWork` RPC response additionally includes the block number.
- Skipping transactions with invalid nonces when pushing to block.
- Update sealing just once when externally importing many blocks (#1541).
- Transaction tracing skipping simple transactions (#1606).
- Other small fixes and improvements.

Full changelog

- DAO hard-fork (#1483) [#1636](https://github.com/paritytech/parity/pull/1636)
- Backports for beta [#1628](https://github.com/paritytech/parity/pull/1628)
- don't batch best block for branches (#1623) [#1626](https://github.com/paritytech/parity/pull/1626)
- Merge bugfixes from master to beta [#1605](https://github.com/paritytech/parity/pull/1605)
- (BETA) using block options cache instead of general cache for rocksdb [#1613](https://github.com/paritytech/parity/pull/1613)
- Backport sealing fixes to beta [#1583](https://github.com/paritytech/parity/pull/1583)
- v1.2.2 in beta [#1581](https://github.com/paritytech/parity/pull/1581)
- Skipping transactions with invalid nonces when pushing to block. (#1545) [#1547](https://github.com/paritytech/parity/pull/1547)


## Parity [v1.2.1](https://github.com/paritytech/parity/releases/tag/v1.2.1) (2016-07-01)

#### New
- Options for more precise mining tuning (see below).
- Informative notification when block mined.
- HTTP signal on new work-package.
- Optimised database insertion for self-mined blocks.
- Short-circuit for local transaction gas-price approval.
- A number of issues related to mining have been fixed.

##### Mining options
- `--author` is now required for mining.
- `--reseal-on-txs` Specify which transactions should force the node to reseal a block. By default parity updates the seal on incoming transactions to reduce transaction latency. Set this option to `none` to force updates on new blocks only.
- `--reseal-min-period` Can be used to control how often a new pending block is generated if `none` is not selected on prior option.
- `--work-queue-size` Controls how many pending blocks to keep in memory.
- `--relay-set`  Can be used to enable more strict transaction verification.
- `--remove-solved` Move solved blocks from the work package queue instead of cloning them. This gives a slightly faster import speed, but means that extra solutions submitted for the same work package will go unused.
- `--notify-work` Accepts a list of URLs that will receive a POST request when new work package is available. The body of the POST message is JSON encoded and has the same format as `eth_getWork` RPC response.

##### RPC

`eth_getWork` RPC response additionally includes the block number.

##### DAO soft-fork

DAO soft-fork control options have been replaced by the single `--fork` option which disables the soft-fork by default.

#### Changes

- v1.2.1 in beta [#1492](https://github.com/paritytech/parity/pull/1492)
- (BETA) add artifacts [#1420](https://github.com/paritytech/parity/pull/1420)

## Parity [v1.2.0: "Security"](https://github.com/paritytech/parity/releases/tag/v1.2.0) (2016-06-24)

[Blog post](https://blog.parity.io/announcing-parity-1-2/)

#### New

- Transaction signing UI.
- IPC/RPC module.
- Optimised mining support.
- Windows build.
- DAO soft-fork support.

##### Transaction signing UI

This is a new framework for signing transactions. It fulfills three requirements:
- You should never have to type your passwords into a Dapp.
- No Javascript code should ever hold a secret.
- No transaction should ever be signed without the consent of the user.

The feature is enabled through the `--signer` flag. When enabled, the user must ensure at least one "Signer UI" is set-up for managing transaction confirmation. There are two such UIs available; one through a Google Chrome Extension, separately installable and the second through a special web page hosted locally. Set-up must be done once for each such UI, through copying and pasting a token from the output console of Parity into the UI. Specific instructions are given in the UI.

From this point on, no transaction may ever be signed by Parity except through one of these allowed Signer UIs, and no password should ever be entered anywhere else.

##### IPC/RPC module and Mist/Geth compatibility

Should be started with `--geth` to ensure Mist compatibility.

##### Optimised mining support

Numerous improvements and optimisations have been added to our mining implementation. A large "active queue" ensures that late-included transactions are included in the mined block without sacrificing older results from latent-reported `ethminer` results.

##### Windows build

We're happy to announce full Windows support with 1.2!

##### Soft-fork

This release includes support for the proposed [DAO soft-fork](https://docs.google.com/document/d/10RktunzjKNfp6Y8Cu4EhR5V9IqxEZq42LU126EYhWY4/pub). Upon upgrade, all mining nodes can vote for or against the soft fork (this is done through altering the block gas limit; a gas limit of at most 4M results in the soft-fork being triggered).

By default, nodes vote "for" the DAO soft-fork (and try to reduce the gas limit to 3.1M). To vote against the soft-fork (keeping it at 4.7M), run with `--dont-help-rescue-dao`. Not upgrading is not recommended; if the majority votes with a soft-fork, an upgrade will be necessary to mine on the correct chain.

#### Changed
- Fast pruning method is now default for a fresh sync.
- Web UI renamed to Dapps UI.
- JSONRPC and Dapps UI enabled by default.
- CLI options ending `-off` renamed to GNU-consistent prefix `--no-`.
- Dynamic gas-pricing (data feed and statistical techniques used to determine optimum gas prices).

Full changes:

- Signer enabled by default for UI [#1417](https://github.com/paritytech/parity/pull/1417)
- Remove experimental pruning options. [#1415](https://github.com/paritytech/parity/pull/1415)
- Fixing interface and port for parity ui [#1414](https://github.com/paritytech/parity/pull/1414)
- Configurable gas limit cap. [#1405](https://github.com/paritytech/parity/pull/1405)
- Bumping TopBar, Minimal SignerUI and wallet [#1413](https://github.com/paritytech/parity/pull/1413)
- Sync: Update highest block for progress reporting [#1411](https://github.com/paritytech/parity/pull/1411)
- Tweaked CLI options for the release [#1407](https://github.com/paritytech/parity/pull/1407)
- Further rocksdb tuning [#1409](https://github.com/paritytech/parity/pull/1409)
- Fixing jit compilation [#1406](https://github.com/paritytech/parity/pull/1406)
- Bump clippy [#1403](https://github.com/paritytech/parity/pull/1403)
- Shortcut SF condition when canon known [#1401](https://github.com/paritytech/parity/pull/1401)
- Additional assertions for internal state of queue [#1402](https://github.com/paritytech/parity/pull/1402)
- Replace deprecated hashdb trait names [#1394](https://github.com/paritytech/parity/pull/1394)
- rpc api by default for ipc [#1400](https://github.com/paritytech/parity/pull/1400)
- Ensure judging the SF trigger by relative branch [#1399](https://github.com/paritytech/parity/pull/1399)
- Signer with unlocked account working as expected. [#1398](https://github.com/paritytech/parity/pull/1398)
- Make --signer default. [#1392](https://github.com/paritytech/parity/pull/1392)
- Presale wallet [#1376](https://github.com/paritytech/parity/pull/1376)
- Removing signer connection limit [#1396](https://github.com/paritytech/parity/pull/1396)
- Optional gas price in transactions come from statistics [#1388](https://github.com/paritytech/parity/pull/1388)
- Update README.md with cargo install [ci-skip] [#1389](https://github.com/paritytech/parity/pull/1389)
- Fixing possible overflow during multiplication [#1381](https://github.com/paritytech/parity/pull/1381)
- Update SF to latest spec [#1386](https://github.com/paritytech/parity/pull/1386)
- Sync optimization [#1385](https://github.com/paritytech/parity/pull/1385)
- Fixing order of if statements to avoid overflows. [#1384](https://github.com/paritytech/parity/pull/1384)
- New topbar & signer UI [#1383](https://github.com/paritytech/parity/pull/1383)
- Install trigger for DAO-rescue soft-fork. [#1329](https://github.com/paritytech/parity/pull/1329)
- Rocksdb flush/compact limit [#1375](https://github.com/paritytech/parity/pull/1375)
- CentOS Dockerfile [#1377](https://github.com/paritytech/parity/pull/1377)
- RPC method to return number of unconfirmed transactions... [#1371](https://github.com/paritytech/parity/pull/1371)
- bump jsonrpc-http-server [#1369](https://github.com/paritytech/parity/pull/1369)
- Fix lock order when updating sealing [#1364](https://github.com/paritytech/parity/pull/1364)
- Update sealing on new transactions [#1365](https://github.com/paritytech/parity/pull/1365)
- Fixed panic on aborted connection [#1370](https://github.com/paritytech/parity/pull/1370)
- importing presale wallet [#1368](https://github.com/paritytech/parity/pull/1368)
- Set default database file size large enough [#1363](https://github.com/paritytech/parity/pull/1363)
- Reserved peers rpc API [#1360](https://github.com/paritytech/parity/pull/1360)
- Fixing replacing transaction with lower gas_price result. [#1343](https://github.com/paritytech/parity/pull/1343)
- fixed migration of empty pruning dir [#1362](https://github.com/paritytech/parity/pull/1362)
- Transaction processing queue [#1335](https://github.com/paritytech/parity/pull/1335)
- Fixing last nonce values in case transaction is replaced [#1359](https://github.com/paritytech/parity/pull/1359)
- docopt is an optional dependency of ethkey and ethstore [#1358](https://github.com/paritytech/parity/pull/1358)
- Fixing clippy warnings [#1354](https://github.com/paritytech/parity/pull/1354)
- Reduce locking when syncing [#1357](https://github.com/paritytech/parity/pull/1357)
- removed unnecessary logs [#1356](https://github.com/paritytech/parity/pull/1356)
- Updating parity-dapps [#1353](https://github.com/paritytech/parity/pull/1353)
- moved keystore tests files from util to ethstore [#1352](https://github.com/paritytech/parity/pull/1352)
- removed redundant bigint deps [#1351](https://github.com/paritytech/parity/pull/1351)
- Reopen "reserved peers and reserved-only flag" [#1350](https://github.com/paritytech/parity/pull/1350)
- Configurable rocksdb cache size [#1348](https://github.com/paritytech/parity/pull/1348)
- Fixing future order and errors when reaching limit. [#1346](https://github.com/paritytech/parity/pull/1346)
- Removing priority on local transactions [#1342](https://github.com/paritytech/parity/pull/1342)
- Revert "Reserved peers, reserved-only flag" [#1349](https://github.com/paritytech/parity/pull/1349)
- Sync attack defense: Deactivate peers on invalid block bodies [#1345](https://github.com/paritytech/parity/pull/1345)
- Reserved peers, reserved-only flag [#1347](https://github.com/paritytech/parity/pull/1347)
- CI for ethkey and ethstore [#1341](https://github.com/paritytech/parity/pull/1341)
- Fixed empty block body composition [#1340](https://github.com/paritytech/parity/pull/1340)
- Provide a signer UI token by default. [#1334](https://github.com/paritytech/parity/pull/1334)
- docker uses rustup, fixes #1337 [#1344](https://github.com/paritytech/parity/pull/1344)
- Fixed network service dispose [#1339](https://github.com/paritytech/parity/pull/1339)
- Sync: Cache last sync round block parents [#1331](https://github.com/paritytech/parity/pull/1331)
- secret store separated from util [#1304](https://github.com/paritytech/parity/pull/1304)
- --geth prevent getTransactionReceipt from using pending. [#1325](https://github.com/paritytech/parity/pull/1325)
- Fixing locks order in miner. [#1328](https://github.com/paritytech/parity/pull/1328)
- Update default gas limit, rename field [#1324](https://github.com/paritytech/parity/pull/1324)
- Use constants for DatabaseConfig [#1318](https://github.com/paritytech/parity/pull/1318)
- Fixing clippy warnings [#1321](https://github.com/paritytech/parity/pull/1321)
- Bumping topbar. Fixing ws server closing when suspending [#1312](https://github.com/paritytech/parity/pull/1312)
- Syncing fix [#1320](https://github.com/paritytech/parity/pull/1320)
- Filling-in optional fields of TransactionRequest... [#1305](https://github.com/paritytech/parity/pull/1305)
- Removing MakerOTC and DAO dapps  [#1319](https://github.com/paritytech/parity/pull/1319)
- Disabling ethcore_set* APIs by default (+ Status page update) [#1315](https://github.com/paritytech/parity/pull/1315)
- fixed #1180 [#1282](https://github.com/paritytech/parity/pull/1282)
- Network start/stop [#1313](https://github.com/paritytech/parity/pull/1313)
- Additional logging for own transactions in queue [#1311](https://github.com/paritytech/parity/pull/1311)
- DAO Rescue soft fork [#1309](https://github.com/paritytech/parity/pull/1309)
- Appveyor config for windows build+installer [#1302](https://github.com/paritytech/parity/pull/1302)
- Key load avoid warning [#1303](https://github.com/paritytech/parity/pull/1303)
- More meaningful errors when sending transaction [#1290](https://github.com/paritytech/parity/pull/1290)
- Gas price statistics. [#1291](https://github.com/paritytech/parity/pull/1291)
- Fix read-ahead bug. [#1298](https://github.com/paritytech/parity/pull/1298)
- firewall rules for windows installer [#1297](https://github.com/paritytech/parity/pull/1297)
- x64 program files path for installer [#1296](https://github.com/paritytech/parity/pull/1296)
- Fixed loosing peers on incoming connections. [#1293](https://github.com/paritytech/parity/pull/1293)
- fixed #1261, overflow when calculating work [#1283](https://github.com/paritytech/parity/pull/1283)
- snappy and minor block compression [#1286](https://github.com/paritytech/parity/pull/1286)
- clarify build instructions [#1287](https://github.com/paritytech/parity/pull/1287)
- fixed #1255 [#1280](https://github.com/paritytech/parity/pull/1280)
- bump rust-crypto [#1289](https://github.com/paritytech/parity/pull/1289)
- Security audit issues fixed [#1279](https://github.com/paritytech/parity/pull/1279)
- Fixing origin/host validation [#1273](https://github.com/paritytech/parity/pull/1273)
- windows installer + parity start ui cli option [#1284](https://github.com/paritytech/parity/pull/1284)
- ipc lib version bump [#1285](https://github.com/paritytech/parity/pull/1285)
- Syncing improvements [#1274](https://github.com/paritytech/parity/pull/1274)
- removed redundant if condition [#1270](https://github.com/paritytech/parity/pull/1270)
- Naive chunk creation, snapshotting [#1263](https://github.com/paritytech/parity/pull/1263)
- Fixing generating new token while another parity instance is running. [#1272](https://github.com/paritytech/parity/pull/1272)
- README: rustup and windows instructions [#1266](https://github.com/paritytech/parity/pull/1266)
- Windows build [#1253](https://github.com/paritytech/parity/pull/1253)
- removed try_seal from MiningBlockChainClient [#1262](https://github.com/paritytech/parity/pull/1262)
- simplified block opening [#1232](https://github.com/paritytech/parity/pull/1232)
- Clippy bump [#1259](https://github.com/paritytech/parity/pull/1259)
- Fixing uint ASM macros compilation [#1258](https://github.com/paritytech/parity/pull/1258)
- Signer port returned from RPC + Topbar showing count of unconfirmed transactions. [#1252](https://github.com/paritytech/parity/pull/1252)
- codegen - avoid unwraps leading to compilation crash [#1250](https://github.com/paritytech/parity/pull/1250)
- Dapps bump [#1257](https://github.com/paritytech/parity/pull/1257)
- Windows named pipes [#1254](https://github.com/paritytech/parity/pull/1254)
- remove unsafety from util/hash.rs and util/bigint/uint.rs [#1236](https://github.com/paritytech/parity/pull/1236)
- Fixing CORS settings for special values: * & null. [#1247](https://github.com/paritytech/parity/pull/1247)
- JSONRPC test strings avoid using \ char [#1246](https://github.com/paritytech/parity/pull/1246)
- Tests for JSON serialisation of statediff/vmtrace [#1241](https://github.com/paritytech/parity/pull/1241)
- Bumping Dapps & TopBar to newest version. [#1245](https://github.com/paritytech/parity/pull/1245)
- keys import [#1240](https://github.com/paritytech/parity/pull/1240)
- Splitting RPC Apis into more fine-grained sets [#1234](https://github.com/paritytech/parity/pull/1234)
- Refactor triedb constructors to error on invalid state root  [#1230](https://github.com/paritytech/parity/pull/1230)
- Signer RPC method to check if signer is enabled [#1238](https://github.com/paritytech/parity/pull/1238)
- Fixing signer behaviour when confirming transaction with wrong password. [#1237](https://github.com/paritytech/parity/pull/1237)
- SystemUIs authorization [#1233](https://github.com/paritytech/parity/pull/1233)
- IPC path for tesetnet with --geth compatibility [#1231](https://github.com/paritytech/parity/pull/1231)
- Transaction tracing for eth_call [#1210](https://github.com/paritytech/parity/pull/1210)
- Removing compilation warnings [#1227](https://github.com/paritytech/parity/pull/1227)
- Allowing connections only from chrome-extension and self-hosted client [#1226](https://github.com/paritytech/parity/pull/1226)
- Clippy bump & fixing warnings [#1219](https://github.com/paritytech/parity/pull/1219)
- Bumping serde & syntex [#1216](https://github.com/paritytech/parity/pull/1216)
- Minimal Signer UI (System UI) exposed over websockets. [#1211](https://github.com/paritytech/parity/pull/1211)
- Switch RPC namespace form ethcore_ to trace_ [#1208](https://github.com/paritytech/parity/pull/1208)
- Verify the state root exists before creating a State [#1217](https://github.com/paritytech/parity/pull/1217)
- Integrate state diffing into the ethcore JSONRPC [#1206](https://github.com/paritytech/parity/pull/1206)
- Updating topbar to latest version [#1220](https://github.com/paritytech/parity/pull/1220)
- Loading local Dapps from FS. [#1214](https://github.com/paritytech/parity/pull/1214)
- Ipc serialization & protocol fixes [#1188](https://github.com/paritytech/parity/pull/1188)
- Have Ext::ret take self by value [#1187](https://github.com/paritytech/parity/pull/1187)
- Simple WebSockets notification about new request [#1202](https://github.com/paritytech/parity/pull/1202)
- Removing leftovers of ethminer [#1207](https://github.com/paritytech/parity/pull/1207)
- fixed #1204 [#1205](https://github.com/paritytech/parity/pull/1205)
- VM tracing and JSON RPC endpoint for it. [#1169](https://github.com/paritytech/parity/pull/1169)
- devtools helpers extended [#1186](https://github.com/paritytech/parity/pull/1186)
- Networking refactoring [#1172](https://github.com/paritytech/parity/pull/1172)
- Client & Miner refactoring [#1195](https://github.com/paritytech/parity/pull/1195)
- update readme [#1201](https://github.com/paritytech/parity/pull/1201)
- Simple signing queue, confirmation APIs exposed in signer WebSockets. [#1182](https://github.com/paritytech/parity/pull/1182)
- Using ordered hashmap to keep the order of dapps on home screen [#1199](https://github.com/paritytech/parity/pull/1199)
- Disabling `ethcore` by default, adding x-frame-options header to dapps. [#1197](https://github.com/paritytech/parity/pull/1197)
- transaction count verifier tests [#1196](https://github.com/paritytech/parity/pull/1196)
- expunge x! and xx! from the codebase [#1192](https://github.com/paritytech/parity/pull/1192)
- Database service upgrade (from the ipc branch) [#1185](https://github.com/paritytech/parity/pull/1185)
- stop eth_syncing from returning true forever [#1181](https://github.com/paritytech/parity/pull/1181)
- Sync fixes and tweaks [#1164](https://github.com/paritytech/parity/pull/1164)
- Exposing RPC over Signer WebSockets [#1167](https://github.com/paritytech/parity/pull/1167)
- implement missing rpc methods and tests [#1171](https://github.com/paritytech/parity/pull/1171)
- json ipc server version bump [#1170](https://github.com/paritytech/parity/pull/1170)
- Updated dependencies for windows build [#1173](https://github.com/paritytech/parity/pull/1173)
- Framework for improved RPC unit tests [#1141](https://github.com/paritytech/parity/pull/1141)
- remove all possible unsafe code in crypto [#1168](https://github.com/paritytech/parity/pull/1168)
- Base for Signer Websockets server [#1158](https://github.com/paritytech/parity/pull/1158)
- Write queue to speed-up db ipc [#1160](https://github.com/paritytech/parity/pull/1160)
- Fixing few clippy warnings [#1163](https://github.com/paritytech/parity/pull/1163)
- Change eth_signAndSendTransaction to personal_SignAndSendTransaction [#1154](https://github.com/paritytech/parity/pull/1154)
- Support "earliest" and specific block parameters in RPC where possible [#1149](https://github.com/paritytech/parity/pull/1149)
- migration fixes [#1155](https://github.com/paritytech/parity/pull/1155)
- Empty trusted signer crate with it's general purpose described. [#1150](https://github.com/paritytech/parity/pull/1150)
- More bootnodes for morden. [#1153](https://github.com/paritytech/parity/pull/1153)
- move existing rpc tests into mocked module [#1151](https://github.com/paritytech/parity/pull/1151)
- Bloomchain [#1014](https://github.com/paritytech/parity/pull/1014)
- Renaming dapps repos. Updating dapps [#1142](https://github.com/paritytech/parity/pull/1142)
- fixed pending transactions [#1147](https://github.com/paritytech/parity/pull/1147)
- Basic benches to provide metrics for ipc optimizations [#1145](https://github.com/paritytech/parity/pull/1145)
- Fixing clippy warnings [#1148](https://github.com/paritytech/parity/pull/1148)
- correct signature of SecTrieDB::raw_mut [#1143](https://github.com/paritytech/parity/pull/1143)
- Merge to master and start hypervisor for import/export [#1138](https://github.com/paritytech/parity/pull/1138)
- Bumping clippy. Fixing warnings [#1139](https://github.com/paritytech/parity/pull/1139)
- Display progress when importing [#1136](https://github.com/paritytech/parity/pull/1136)
- foundation of simple db migration [#1128](https://github.com/paritytech/parity/pull/1128)
- Fixpending [#1074](https://github.com/paritytech/parity/pull/1074)
- Sync: Propagate uncles and fix status reporting [#1134](https://github.com/paritytech/parity/pull/1134)
- Coloured, padding logging. [#1133](https://github.com/paritytech/parity/pull/1133)
- Importing [#1132](https://github.com/paritytech/parity/pull/1132)
- Have `die_with_error` use `fmt::Display` rather than Debug [#1116](https://github.com/paritytech/parity/pull/1116)
- Exporting [#1129](https://github.com/paritytech/parity/pull/1129)
- Sign and send transaction [#1124](https://github.com/paritytech/parity/pull/1124)
- Fixing unused imports warnings [#1125](https://github.com/paritytech/parity/pull/1125)
- Adding info messages on mined blocks [#1127](https://github.com/paritytech/parity/pull/1127)
- Fix styling - don't mix spaces with tabs!!! [#1123](https://github.com/paritytech/parity/pull/1123)
- Fix is_syncing so it's false as long as the update is trivial. [#1122](https://github.com/paritytech/parity/pull/1122)
- Relock unlocked accounts after first use [#1120](https://github.com/paritytech/parity/pull/1120)
- Avoid importing keys into wrong place. [#1119](https://github.com/paritytech/parity/pull/1119)
- Implement receipt's gasUsed field [#1118](https://github.com/paritytech/parity/pull/1118)
- New dapps & query parameter handling [#1113](https://github.com/paritytech/parity/pull/1113)
- pretty print trace error [#1098](https://github.com/paritytech/parity/pull/1098)
- New syncing strategy [#1095](https://github.com/paritytech/parity/pull/1095)
- ethcore-db crate [#1097](https://github.com/paritytech/parity/pull/1097)
- Fix the default for pruning. [#1107](https://github.com/paritytech/parity/pull/1107)
- Make Id/ID and db/Db/DB usage consistent [#1105](https://github.com/paritytech/parity/pull/1105)
- Miner holds it's own copy of spec/engine [#1091](https://github.com/paritytech/parity/pull/1091)
- Apps listing API & Home webapp. [#1101](https://github.com/paritytech/parity/pull/1101)
- CLI option for using JITEVM [#1103](https://github.com/paritytech/parity/pull/1103)
- Fix up the seal fields in RPC output [#1096](https://github.com/paritytech/parity/pull/1096)
- Fixing some warnings [#1102](https://github.com/paritytech/parity/pull/1102)
- fixed incorrect decoding of header seal_fields. added tests. #1090 [#1094](https://github.com/paritytech/parity/pull/1094)
- Bumping Clippy [#1093](https://github.com/paritytech/parity/pull/1093)
- Injectable topbar support. [#1092](https://github.com/paritytech/parity/pull/1092)
- New syncing part 1: Block collection [#1088](https://github.com/paritytech/parity/pull/1088)
- Moving all Client public API types to separate mod & binary serialization codegen for that mod [#1051](https://github.com/paritytech/parity/pull/1051)
- Subdomains support in content server (webapps server). [#1082](https://github.com/paritytech/parity/pull/1082)
- Fix uncle getter [#1087](https://github.com/paritytech/parity/pull/1087)
- Provide fallback for usd-per-eth option when offline. [#1085](https://github.com/paritytech/parity/pull/1085)
- path centralized [#1083](https://github.com/paritytech/parity/pull/1083)
- Limiting result of the execution to execution-specific errors [#1071](https://github.com/paritytech/parity/pull/1071)
- Configurable keys security [#1080](https://github.com/paritytech/parity/pull/1080)
- comma delimeting multiple cors headers [#1078](https://github.com/paritytech/parity/pull/1078)
- Update error message [#1081](https://github.com/paritytech/parity/pull/1081)
- Updating dapp-wallet [#1076](https://github.com/paritytech/parity/pull/1076)
- Fixed connecting to local nodes on startup [#1070](https://github.com/paritytech/parity/pull/1070)
- Validate signature in Tx queue [#1068](https://github.com/paritytech/parity/pull/1068)
- moving deps to ethcore/hyper and bumping jsonrpc-http-server version [#1067](https://github.com/paritytech/parity/pull/1067)
- Updating status page. Bringing back wallet [#1064](https://github.com/paritytech/parity/pull/1064)
- Fix --geth IPC for MacOS. [#1062](https://github.com/paritytech/parity/pull/1062)
- Fixing formatter for defaultExtraData [#1060](https://github.com/paritytech/parity/pull/1060)
- --geth IPC compatibility [#1059](https://github.com/paritytech/parity/pull/1059)
- Moving dependencies to ethcore & uniforming syntax libs through all crates [#1050](https://github.com/paritytech/parity/pull/1050)
- update hyper branch mio [#1054](https://github.com/paritytech/parity/pull/1054)
- IPC lib update [#1047](https://github.com/paritytech/parity/pull/1047)
- Updating hyper-mio revision [#1048](https://github.com/paritytech/parity/pull/1048)
- Bump ipc-lib version [#1046](https://github.com/paritytech/parity/pull/1046)
- Tidy up CLI options and make JSONRPC & webapps on by default. [#1045](https://github.com/paritytech/parity/pull/1045)
- Fixing clippy warnings [#1044](https://github.com/paritytech/parity/pull/1044)
- Fixing RPC modules compatibility [#1041](https://github.com/paritytech/parity/pull/1041)
- Fixing hyper-mio revision [#1043](https://github.com/paritytech/parity/pull/1043)
- Updating locations of webapp stuff [#1040](https://github.com/paritytech/parity/pull/1040)
- JSON-RPC over IPC [#1039](https://github.com/paritytech/parity/pull/1039)
- Update nix/mio for ARM [#1036](https://github.com/paritytech/parity/pull/1036)
- Basic Authority [#991](https://github.com/paritytech/parity/pull/991)
- Prioritizing of local transaction [#1023](https://github.com/paritytech/parity/pull/1023)
- Version 1.2 [#1030](https://github.com/paritytech/parity/pull/1030)
- Bumping status page [#1033](https://github.com/paritytech/parity/pull/1033)

## Parity [v1.1.0](https://github.com/paritytech/parity/releases/tag/v1.1.0) (2016-05-02)

Parity 1.1.0 introduces:

- Transaction tracing. Parity now optionally indexes & stores message-call/"internal transaction" information and provides additional RPC for querying.
- Web interface for logs, status & JSON RPC.
- Improved JSON RPC compatibility.
- Reduced memory footprint.
- Optimized EVM interpreter performance.

Full Changes:

- Exposing default extra data via ethcore RPC [#1032](https://github.com/paritytech/parity/pull/1032)
- Net etiquette [#1028](https://github.com/paritytech/parity/pull/1028)
- Bumping clippy & fixing warnings [#1024](https://github.com/paritytech/parity/pull/1024)
- Tracedb interface && cli [#997](https://github.com/paritytech/parity/pull/997)
- Switching to geth-attach supporting version of rpc core and server [#1022](https://github.com/paritytech/parity/pull/1022)
- Fixing status page displaying homestead  [#1020](https://github.com/paritytech/parity/pull/1020)
- Core tracedb functionality. [#996](https://github.com/paritytech/parity/pull/996)
- RPC method for supported modules [#1019](https://github.com/paritytech/parity/pull/1019)
- Updating status page [#1015](https://github.com/paritytech/parity/pull/1015)
- Disabling wallet [#1017](https://github.com/paritytech/parity/pull/1017)
- More detailed fatal error reporting [#1016](https://github.com/paritytech/parity/pull/1016)
- Support 'pending' block in RPC [#1007](https://github.com/paritytech/parity/pull/1007)
- Enable pending block when there is local transaction pending. [#1005](https://github.com/paritytech/parity/pull/1005)
- updating key files permissions on save [#1010](https://github.com/paritytech/parity/pull/1010)
- IPC JSON RPC (for external interface) [#1009](https://github.com/paritytech/parity/pull/1009)
- Fixing Firefox authorization issues [#1013](https://github.com/paritytech/parity/pull/1013)
- cargo update [#1012](https://github.com/paritytech/parity/pull/1012)
- Switching to rust-url@1.0.0 [#1011](https://github.com/paritytech/parity/pull/1011)
- Exception handling in RPC & WebApps [#988](https://github.com/paritytech/parity/pull/988)
- Fixed uint deserialization from hex [#1008](https://github.com/paritytech/parity/pull/1008)
- Tweak timeout and packet size to handle slow networks better [#1004](https://github.com/paritytech/parity/pull/1004)
- db key is generic and can be made smaller [#1006](https://github.com/paritytech/parity/pull/1006)
- IPC with new serialization [#998](https://github.com/paritytech/parity/pull/998)
- make jsonrpc api engine agnostic [#1001](https://github.com/paritytech/parity/pull/1001)
- updated cargo.lock [#1002](https://github.com/paritytech/parity/pull/1002)
- updated parity dependencies [#993](https://github.com/paritytech/parity/pull/993)
- Auto (with codegen) binary serializer  [#980](https://github.com/paritytech/parity/pull/980)
- Fixing transaction queue last_nonces update [#995](https://github.com/paritytech/parity/pull/995)
- import route contains ommited blocks [#994](https://github.com/paritytech/parity/pull/994)
- fixed encoding 0u8 [#992](https://github.com/paritytech/parity/pull/992)
- Use latest netstats [#989](https://github.com/paritytech/parity/pull/989)
- RPC shared external miner [#984](https://github.com/paritytech/parity/pull/984)
- Additional RPC methods for settings [#983](https://github.com/paritytech/parity/pull/983)
- Fixing transaction_queue deadlock [#985](https://github.com/paritytech/parity/pull/985)
- Refactoring of `parity/main.rs` [#981](https://github.com/paritytech/parity/pull/981)
- Fixing clippy warnings. [#982](https://github.com/paritytech/parity/pull/982)
- Bumping status page [#977](https://github.com/paritytech/parity/pull/977)
- querying extras separated to its own module [#972](https://github.com/paritytech/parity/pull/972)
- Exposing application logs via RPC. [#976](https://github.com/paritytech/parity/pull/976)
- Addressing binary serialization for db types [#966](https://github.com/paritytech/parity/pull/966)
- removed redundant unwraps [#935](https://github.com/paritytech/parity/pull/935)
- fixed transaction queue merge conflict [#975](https://github.com/paritytech/parity/pull/975)
- Configurable limit for transaction queue (CLI & Ethcore-RPC) [#974](https://github.com/paritytech/parity/pull/974)
- Enforce limit caused `last_nonce` to return incorrect values. [#973](https://github.com/paritytech/parity/pull/973)
- Even more detailed errors for transaction queue [#969](https://github.com/paritytech/parity/pull/969)
- temporary fix of panic in blockchain garbage collection [#970](https://github.com/paritytech/parity/pull/970)
- IPC codegen - some minor fixes & enhancements [#967](https://github.com/paritytech/parity/pull/967)
- Additional logging for transactions [#968](https://github.com/paritytech/parity/pull/968)
- refactored blockchain extras keys building [#963](https://github.com/paritytech/parity/pull/963)
- Using hyper-mio branch in webapps. [#957](https://github.com/paritytech/parity/pull/957)
- Remove nanomsg from build-dependencies [#965](https://github.com/paritytech/parity/pull/965)
- Fix build for --target=armv7-unknown-linux-gnueabihf [#964](https://github.com/paritytech/parity/pull/964)
- IPC RPC codegen extra feature [#962](https://github.com/paritytech/parity/pull/962)
- IPC RPC codegen for generic implementation [#961](https://github.com/paritytech/parity/pull/961)
- using db_path directory when upgrading [#960](https://github.com/paritytech/parity/pull/960)
- IPC hypervisor [#958](https://github.com/paritytech/parity/pull/958)
- Removing a transaction from queue now removes all from this sender with lower nonces. [#950](https://github.com/paritytech/parity/pull/950)
- bump status page version 0.1.7 [#955](https://github.com/paritytech/parity/pull/955)
- Changing cors header to be optional [#956](https://github.com/paritytech/parity/pull/956)
- Update ARM Dockerfile [#959](https://github.com/paritytech/parity/pull/959)
- Sensible gas limits for eth_sendTransaction [#953](https://github.com/paritytech/parity/pull/953)
- Fix upgrade script and make parity run when no .parity dir. [#954](https://github.com/paritytech/parity/pull/954)
- Tracing and docs for --pruning=auto. [#952](https://github.com/paritytech/parity/pull/952)
- IPC serialization for custom parameters [#946](https://github.com/paritytech/parity/pull/946)
- default filter from block should be Latest, not Earliest [#948](https://github.com/paritytech/parity/pull/948)
- README.md: removes sudo from multirust installation [#943](https://github.com/paritytech/parity/pull/943)
- Disable long lines formatting + ethash example. [#939](https://github.com/paritytech/parity/pull/939)
- Ethcore-specific RPC methods for altering miner parameters. [#934](https://github.com/paritytech/parity/pull/934)
- Use ethcore nanomsg bindings [#941](https://github.com/paritytech/parity/pull/941)
- Update IPC codegen to latest syntax libs [#938](https://github.com/paritytech/parity/pull/938)
- IPC documentation [#937](https://github.com/paritytech/parity/pull/937)
- Bumping clippy and fixing warnings. [#936](https://github.com/paritytech/parity/pull/936)
- Pruning auto [#927](https://github.com/paritytech/parity/pull/927)
- IPC persistent client link [#933](https://github.com/paritytech/parity/pull/933)
- IPC persistent client link [#930](https://github.com/paritytech/parity/pull/930)
- IPC handshake (negotiating protocol/api version) [#928](https://github.com/paritytech/parity/pull/928)
- Upgrade logic between versions [#914](https://github.com/paritytech/parity/pull/914)
- executive tracing cleanup [#903](https://github.com/paritytech/parity/pull/903)
- Ethcore-specific RPC methods [#923](https://github.com/paritytech/parity/pull/923)
- Parameter to allow user to force the sealing mechanism [#918](https://github.com/paritytech/parity/pull/918)
- updated dependencies [#921](https://github.com/paritytech/parity/pull/921)
- Fixed send transaction deadlock [#920](https://github.com/paritytech/parity/pull/920)
- --unlock is comma-delimited. [#916](https://github.com/paritytech/parity/pull/916)
- fixed eth_getLogs [#915](https://github.com/paritytech/parity/pull/915)
- create provided custom dir for keys if none [#912](https://github.com/paritytech/parity/pull/912)
- spec loading cleanup [#858](https://github.com/paritytech/parity/pull/858)
- WebApps HTTP Basic Auth Support [#906](https://github.com/paritytech/parity/pull/906)
- Removing match on constant [#888](https://github.com/paritytech/parity/pull/888)
- Update auth.rs [#907](https://github.com/paritytech/parity/pull/907)
- Enabling webapps compilation by default [#904](https://github.com/paritytech/parity/pull/904)
- fixed #895 [#898](https://github.com/paritytech/parity/pull/898)
- Support for compile-time included WebApplications. [#899](https://github.com/paritytech/parity/pull/899)
- Propagate transaction queue [#894](https://github.com/paritytech/parity/pull/894)
- Use new json RPC server [#901](https://github.com/paritytech/parity/pull/901)
- Gracefully dying when trying to enable RPC and app is compiled without it. [#900](https://github.com/paritytech/parity/pull/900)
- Additional logging and friendlier error messages [#893](https://github.com/paritytech/parity/pull/893)
- Avoid signalling readiness when app is about to be closed. [#897](https://github.com/paritytech/parity/pull/897)
- fixed #875 and added tests for eth_sendTransaction [#890](https://github.com/paritytech/parity/pull/890)
- passing key path to all invocations [#891](https://github.com/paritytech/parity/pull/891)
- Fixed eth_call nonce and gas handling [#892](https://github.com/paritytech/parity/pull/892)
- ipc rpc with nano transport (simple duplex) [#886](https://github.com/paritytech/parity/pull/886)
- Bumping clippy and fixing warnings [#889](https://github.com/paritytech/parity/pull/889)
- More descriptive expectations to transaction queue consistency. [#878](https://github.com/paritytech/parity/pull/878)
- uint bug - replace add with or [#879](https://github.com/paritytech/parity/pull/879)
- Fixing typo in bigint [#877](https://github.com/paritytech/parity/pull/877)
- update misleading cli help msg for author [#874](https://github.com/paritytech/parity/pull/874)
- Find geth data store cross-platform. [#871](https://github.com/paritytech/parity/pull/871)
- Import geth 1.4.0 keys [#872](https://github.com/paritytech/parity/pull/872)
- Syntax helpers for IPC RPC (part 2) [#854](https://github.com/paritytech/parity/pull/854)
- Fixed bootnode URL and error message [#870](https://github.com/paritytech/parity/pull/870)
- replace popcnt with mov (861) [#867](https://github.com/paritytech/parity/pull/867)
- weekly dependencies update [#865](https://github.com/paritytech/parity/pull/865)
- Remove unused mut [#866](https://github.com/paritytech/parity/pull/866)
- fixed #855 [#864](https://github.com/paritytech/parity/pull/864)
- simplified trace from functions, removed clippy warnings [#862](https://github.com/paritytech/parity/pull/862)
- Update deprecated HashDB methods in docs. [#857](https://github.com/paritytech/parity/pull/857)
- refactored loading transaction json tests [#853](https://github.com/paritytech/parity/pull/853)
- reorganised price info lookup [#852](https://github.com/paritytech/parity/pull/852)
- Publish locally-made transactions to peers. [#850](https://github.com/paritytech/parity/pull/850)
- Add generalbeck's token [#847](https://github.com/paritytech/parity/pull/847)
- Fix response for mining. [#846](https://github.com/paritytech/parity/pull/846)
- USD-based pricing of gas. [#843](https://github.com/paritytech/parity/pull/843)
- Parity can accept older work packages [#811](https://github.com/paritytech/parity/pull/811)
- Caching for computing seed hashes (#541) [#841](https://github.com/paritytech/parity/pull/841)
- checking transaction queue for pending transaction [#838](https://github.com/paritytech/parity/pull/838)
- refactored loading of state tests [#817](https://github.com/paritytech/parity/pull/817)
- tests for deserialization of transaction from issue #835 [#837](https://github.com/paritytech/parity/pull/837)
- unlocks with no expiration [on top of 833] [#834](https://github.com/paritytech/parity/pull/834)
- Unlock accounts on CLI. [#833](https://github.com/paritytech/parity/pull/833)
- Make BlockNumber optional, fix eth_call [#829](https://github.com/paritytech/parity/pull/829)
- Test socket to common test code (ethcore-devtools) [#831](https://github.com/paritytech/parity/pull/831)
- Use network id for the web3_net_version return. [#822](https://github.com/paritytech/parity/pull/822)
- json-rpc web3_sha3 [#824](https://github.com/paritytech/parity/pull/824)
- remove some unused files [#819](https://github.com/paritytech/parity/pull/819)
- debug symbols for master/beta [#818](https://github.com/paritytech/parity/pull/818)
- Syntax helpers for IPC RPC [#809](https://github.com/paritytech/parity/pull/809)
- refactored loading of execution tests [#803](https://github.com/paritytech/parity/pull/803)
- Rustfmt.toml [#805](https://github.com/paritytech/parity/pull/805)
- install-partiy runs brew reinstall parity on osx [#810](https://github.com/paritytech/parity/pull/810)
- Fix mining from spinning [#807](https://github.com/paritytech/parity/pull/807)

## Parity [v1.0.2](https://github.com/paritytech/parity/releases/tag/v1.0.2) (2016-04-11)

Parity 1.0.2 release improves Json RPC compatibility and fixes a number of stability issues.

- Flush password prompt [#1031](https://github.com/paritytech/parity/pull/1031)
- [beta] dependencies update [#949](https://github.com/paritytech/parity/pull/949)
- Master to beta v1.0.2 [#922](https://github.com/paritytech/parity/pull/922)
- Master to beta 1.0.2 [#908](https://github.com/paritytech/parity/pull/908)

## Parity [v1.0.1](https://github.com/paritytech/parity/releases/tag/v1.0.1) (2016-03-28)

Parity 1.0.1 update fixes a number of issues with Json RPC, transaction propagation and syncing.

- Imporved sync error handling [#905](https://github.com/paritytech/parity/pull/905)
- Publish locally-made transactions to peers. [#851](https://github.com/paritytech/parity/pull/851)
- Merge fixes from master to beta [#845](https://github.com/paritytech/parity/pull/845)
- Full sync restart on bad block [#844](https://github.com/paritytech/parity/pull/844)
- Make BlockNumber optional, fix eth_call [#828](https://github.com/paritytech/parity/pull/828)
- Web3sha3 beta [#826](https://github.com/paritytech/parity/pull/826)
- Use network id for the web3_net_version return. [#821](https://github.com/paritytech/parity/pull/821)
- Fix mining from spinning [#806](https://github.com/paritytech/parity/pull/806)
- Merge master to beta [#796](https://github.com/paritytech/parity/pull/796)

## Parity [v1.0.0](https://github.com/paritytech/parity/releases/tag/v1.0.0) (2016-03-24)

Parity 1.0.0 release adds the following features:

- Standard JsonRPC interface.
- Full Homestead compatibility.
- Transaction management.
- Mining with external miner.
- Account management.
- Geth key chain compatibility.
- Additional command line options.
- State trie pruning.
- Cache and queue footprint.
- Network discovery & NAT traversal.
- Custom chain specification files.

Note that in this release the state database is in archive (full) mode by default. Run with one of the `--pruning` options to enable pruning.

- First part of multi-mining support [#804](https://github.com/paritytech/parity/pull/804)
- Fixing future-current transactions clash [#802](https://github.com/paritytech/parity/pull/802)
- Increase threads to num_cpus & fix author reporting [#800](https://github.com/paritytech/parity/pull/800)
- another batch of rpc improvements [#798](https://github.com/paritytech/parity/pull/798)
- Avoid tracing DELEGATECALL and CALLCODE. Plus tests for it. [#794](https://github.com/paritytech/parity/pull/794)
- complete getting started steps for OS X [#793](https://github.com/paritytech/parity/pull/793)
- Auto detect available port (with fixed test) [#788](https://github.com/paritytech/parity/pull/788)
- eth_getTransactionReceipt [#792](https://github.com/paritytech/parity/pull/792)
- Comprehensive tests for tracing transactions [#791](https://github.com/paritytech/parity/pull/791)
- Disable preparing work package if miners don't ask for it. [#771](https://github.com/paritytech/parity/pull/771)
- Listen on all interfaces for JSONRPC by default. [#786](https://github.com/paritytech/parity/pull/786)
- eth_call [#783](https://github.com/paritytech/parity/pull/783)
- Revert "Auto detect available port" [#789](https://github.com/paritytech/parity/pull/789)
- added output to execution result [#777](https://github.com/paritytech/parity/pull/777)
- Auto detect available port [#782](https://github.com/paritytech/parity/pull/782)
- Allow 0x prefix for --author. [#785](https://github.com/paritytech/parity/pull/785)
- updated dependencies, moved rpctest to its own submodule [#784](https://github.com/paritytech/parity/pull/784)
- use ethjson module to load chain json tests [#778](https://github.com/paritytech/parity/pull/778)
- Tracing implemented. [#772](https://github.com/paritytech/parity/pull/772)
- test ethjson module on travis [#780](https://github.com/paritytech/parity/pull/780)
- batch of rpc fixes [#775](https://github.com/paritytech/parity/pull/775)
- rpctest executable [#757](https://github.com/paritytech/parity/pull/757)
- Refactoring error transaction_queue error handling and `update_sealing` method. [#753](https://github.com/paritytech/parity/pull/753)
- Avoid importing transactions with gas above 1.1*block_gas_limit to transaction queue [#760](https://github.com/paritytech/parity/pull/760)
- Removing transactions that failed to be pushed to block. [#752](https://github.com/paritytech/parity/pull/752)
- Updating clippy [#766](https://github.com/paritytech/parity/pull/766)
- Attempting to add all transactions to mined block [#754](https://github.com/paritytech/parity/pull/754)
- Prettier version w/o git dir; Use rustc compile time version [#761](https://github.com/paritytech/parity/pull/761)
- Stop adding transactions to queue while not fully synced [#751](https://github.com/paritytech/parity/pull/751)
- Verify sender's balance before importing transaction to queue [#746](https://github.com/paritytech/parity/pull/746)
- Returning number of transactions pending in block not queue [#750](https://github.com/paritytech/parity/pull/750)
- Speeding up build [#733](https://github.com/paritytech/parity/pull/733)
- adding check for a sync when giving work to miner [#742](https://github.com/paritytech/parity/pull/742)
- json deserialization module [#745](https://github.com/paritytech/parity/pull/745)
- Update install-parity.sh [#749](https://github.com/paritytech/parity/pull/749)
- Restart sync on getting old unknown header [#747](https://github.com/paritytech/parity/pull/747)
- Missing return for #737 [#744](https://github.com/paritytech/parity/pull/744)
- Enact block with uncles test [#741](https://github.com/paritytech/parity/pull/741)
- Fix outdated libc version on dependency [#740](https://github.com/paritytech/parity/pull/740)
- Fixing possible race in transaction queue [#735](https://github.com/paritytech/parity/pull/735)
- Sync fixed again [#737](https://github.com/paritytech/parity/pull/737)
- Don't change best block until extras is committed. [#734](https://github.com/paritytech/parity/pull/734)
- stable only until travis speedup [#736](https://github.com/paritytech/parity/pull/736)
- Optimizing uint operations (architecture independent) [#629](https://github.com/paritytech/parity/pull/629)
- Add RLP, not a data item. [#725](https://github.com/paritytech/parity/pull/725)
- PV63 receipts response [#687](https://github.com/paritytech/parity/pull/687)
- another batch of rpc tests [#723](https://github.com/paritytech/parity/pull/723)
- dockerfiles update [#726](https://github.com/paritytech/parity/pull/726)
- Lock reports to avoid out of order badness. [#721](https://github.com/paritytech/parity/pull/721)
- Fixed handshake leak [#722](https://github.com/paritytech/parity/pull/722)
- Allow configuration of target gas limit. [#719](https://github.com/paritytech/parity/pull/719)
- Version 1.1 in master [#714](https://github.com/paritytech/parity/pull/714)
- Silence UDP warnings [#720](https://github.com/paritytech/parity/pull/720)
- Rpc personal tests [#715](https://github.com/paritytech/parity/pull/715)
- Fixing warnings [#704](https://github.com/paritytech/parity/pull/704)
- docopts cleanups [#713](https://github.com/paritytech/parity/pull/713)
- Removed rocksdb build dependency [#717](https://github.com/paritytech/parity/pull/717)
- Fixed splitting Neighbours packet [#710](https://github.com/paritytech/parity/pull/710)
- management of account expiration & memory [#701](https://github.com/paritytech/parity/pull/701)
- Remove EarlyMerge from user docs. [#708](https://github.com/paritytech/parity/pull/708)
- Fixes and traces for refcountdb. [#705](https://github.com/paritytech/parity/pull/705)
- Check for NULL_RLP in AccountDB [#706](https://github.com/paritytech/parity/pull/706)
- ethminer as crate [#700](https://github.com/paritytech/parity/pull/700)
- Old ref-counted DB code [#692](https://github.com/paritytech/parity/pull/692)
- next batch of rpc tests and fixes [#699](https://github.com/paritytech/parity/pull/699)
- implemented eth_geStorageAt rpc method, added more tests for rpc [#695](https://github.com/paritytech/parity/pull/695)
- Fix JournalDB era marker [#690](https://github.com/paritytech/parity/pull/690)
- More sync fixes [#685](https://github.com/paritytech/parity/pull/685)
- mark some key tests as heavy [#694](https://github.com/paritytech/parity/pull/694)
- Limit incoming connections [#693](https://github.com/paritytech/parity/pull/693)
- Updating clippy [#688](https://github.com/paritytech/parity/pull/688)
- eth_accounts, eth_getBalance rpc functions && tests [#691](https://github.com/paritytech/parity/pull/691)
- state query for archive jdb [#683](https://github.com/paritytech/parity/pull/683)
- Fix for option 1 of JournalDB [#658](https://github.com/paritytech/parity/pull/658)
- Rename into something that is a little more descriptive. [#689](https://github.com/paritytech/parity/pull/689)
- JournalDB with in-memory overlay (option2) [#634](https://github.com/paritytech/parity/pull/634)
- additional (failing) SecretStore test [#682](https://github.com/paritytech/parity/pull/682)
- Updating clippy & fixing warnings. [#670](https://github.com/paritytech/parity/pull/670)
- rpc web3 tests [#681](https://github.com/paritytech/parity/pull/681)
- Making personal json-rpc configurable via cli [#677](https://github.com/paritytech/parity/pull/677)
- RPC Pending Transactions Filter [#661](https://github.com/paritytech/parity/pull/661)
- Rearrange journaldb infrastructure to make more extensible [#678](https://github.com/paritytech/parity/pull/678)
- JournalDB -> Box<JournalDB>, and it's a trait. [#673](https://github.com/paritytech/parity/pull/673)
- fix warning for transaction_queue.add usage [#676](https://github.com/paritytech/parity/pull/676)
- Adding std::mem back (only for asm) [#680](https://github.com/paritytech/parity/pull/680)
- update readme to exclude beta step (stable is ok) [#679](https://github.com/paritytech/parity/pull/679)
- fixed U256 and transaction request deserialization [#675](https://github.com/paritytech/parity/pull/675)
- More geth compatibility. [#666](https://github.com/paritytech/parity/pull/666)
- Removing running clippy by default on nightly. [#671](https://github.com/paritytech/parity/pull/671)
- rpc net submodule tests [#667](https://github.com/paritytech/parity/pull/667)
- Client module overhaul [#665](https://github.com/paritytech/parity/pull/665)
- Rpc transaction signing [#587](https://github.com/paritytech/parity/pull/587)
- Transaction queue exposed via JSON rpc. [#652](https://github.com/paritytech/parity/pull/652)
- Remove unneeded locking [#499](https://github.com/paritytech/parity/pull/499)
- extend sync status interface to sync provider [#664](https://github.com/paritytech/parity/pull/664)
- --archive is default. --pruning is option. [#663](https://github.com/paritytech/parity/pull/663)
- jsonrpc uses client and sync interfaces [#641](https://github.com/paritytech/parity/pull/641)
- Expose transaction insertion in sync lib [#609](https://github.com/paritytech/parity/pull/609)
- Removing get prefix from poll_info [#660](https://github.com/paritytech/parity/pull/660)
- Tx queue update height bug [#657](https://github.com/paritytech/parity/pull/657)
- Tx_queue_docs -> To master [#651](https://github.com/paritytech/parity/pull/651)
- blockchain import_route [#645](https://github.com/paritytech/parity/pull/645)
- Stop workers before stopping event loop [#655](https://github.com/paritytech/parity/pull/655)
- Validate sender before importing to queue [#650](https://github.com/paritytech/parity/pull/650)
- Gas price threshold for transactions [#640](https://github.com/paritytech/parity/pull/640)
- `dev` feature enabled when compiling without `--release` [#627](https://github.com/paritytech/parity/pull/627)
- Don't call mark_as_bad needlessly [#648](https://github.com/paritytech/parity/pull/648)
- Fixed sync handling large forks [#647](https://github.com/paritytech/parity/pull/647)
- Additional documentation for transaction queue [#631](https://github.com/paritytech/parity/pull/631)
- Transaction Queue Integration [#607](https://github.com/paritytech/parity/pull/607)
- Keys cli [#639](https://github.com/paritytech/parity/pull/639)
- fix build warning [#643](https://github.com/paritytech/parity/pull/643)
- updated jsonrpc-core and http-server libs [#642](https://github.com/paritytech/parity/pull/642)
- jsonrpc panics gracefully shutdown client [#638](https://github.com/paritytech/parity/pull/638)
- Fixing CLI parameters [#633](https://github.com/paritytech/parity/pull/633)
- Normal CLI options with geth. [#628](https://github.com/paritytech/parity/pull/628)
- Do not remove the peer immediatelly on send error [#626](https://github.com/paritytech/parity/pull/626)
- Jsonrpc block behind [#622](https://github.com/paritytech/parity/pull/622)
- Remove println!s. [#624](https://github.com/paritytech/parity/pull/624)
- JournalDB option 1 fix [#613](https://github.com/paritytech/parity/pull/613)
- Network tracing cleanup [#611](https://github.com/paritytech/parity/pull/611)
- Revert "Transaction Queue integration" [#602](https://github.com/paritytech/parity/pull/602)
- fix benches compilation [#601](https://github.com/paritytech/parity/pull/601)
- Transaction Queue integration [#595](https://github.com/paritytech/parity/pull/595)
- verifier trait improvements [#597](https://github.com/paritytech/parity/pull/597)
- build on rust stable [#600](https://github.com/paritytech/parity/pull/600)
- Geth import silent if no geth [#599](https://github.com/paritytech/parity/pull/599)
- Additional journaldb logging and assert [#593](https://github.com/paritytech/parity/pull/593)
- Uncle inclusion in block authoring. [#578](https://github.com/paritytech/parity/pull/578)
- Fixed potential deadlock on startup [#592](https://github.com/paritytech/parity/pull/592)
- Fixing an overflow panic [#591](https://github.com/paritytech/parity/pull/591)
- Fixed one more case of sync stalling [#590](https://github.com/paritytech/parity/pull/590)
- JournalDB can now operate in "archive" mode [#589](https://github.com/paritytech/parity/pull/589)
- Secret store integration with client [#586](https://github.com/paritytech/parity/pull/586)
- fix build on nightly rust [#588](https://github.com/paritytech/parity/pull/588)
- deserialization for uint generic [#585](https://github.com/paritytech/parity/pull/585)
- TransactionsQueue implementation [#559](https://github.com/paritytech/parity/pull/559)
- JSON-RPC personal service (follows #582) [#583](https://github.com/paritytech/parity/pull/583)
- making key directory thread-safe [#582](https://github.com/paritytech/parity/pull/582)
- verifier trait [#581](https://github.com/paritytech/parity/pull/581)
- shrink_to_fit after removing hashes. [#580](https://github.com/paritytech/parity/pull/580)
- support for rpc polling [#504](https://github.com/paritytech/parity/pull/504)
- limit serde codegen only to rpc types submodule [#569](https://github.com/paritytech/parity/pull/569)
- fork test for Issue test/568 [#573](https://github.com/paritytech/parity/pull/573)
- Fixing clippy warnings = small refactoring of `request_blocks` [#560](https://github.com/paritytech/parity/pull/560)
- Improved journaldb logging [#571](https://github.com/paritytech/parity/pull/571)
- Additional check to ancient enactments. [#570](https://github.com/paritytech/parity/pull/570)
- chainfilter shouldnt exclude to_block from results [#564](https://github.com/paritytech/parity/pull/564)
- Fix coverage test run [#567](https://github.com/paritytech/parity/pull/567)
- Mining [#547](https://github.com/paritytech/parity/pull/547)
- fix uint warnings [#565](https://github.com/paritytech/parity/pull/565)
- Finished blockchain generator. [#562](https://github.com/paritytech/parity/pull/562)
- fixed broken master [#563](https://github.com/paritytech/parity/pull/563)
- uint to separate crate [#544](https://github.com/paritytech/parity/pull/544)
- improved test chain generator [#554](https://github.com/paritytech/parity/pull/554)
- Fixing spelling in propagade->propagate [#558](https://github.com/paritytech/parity/pull/558)
- Changing RefCell to Cell in transaction. [#557](https://github.com/paritytech/parity/pull/557)
- Fix for morden consensus. [#556](https://github.com/paritytech/parity/pull/556)
- blockchain generator [#550](https://github.com/paritytech/parity/pull/550)
- Sparse Table Implementation (Row, Col) -> Val [#545](https://github.com/paritytech/parity/pull/545)
- fixup install script [#548](https://github.com/paritytech/parity/pull/548)
- Fixing clippy warnings [#546](https://github.com/paritytech/parity/pull/546)
- ignore out directory [#543](https://github.com/paritytech/parity/pull/543)
- u256 full multiplication [#539](https://github.com/paritytech/parity/pull/539)
- Fix panic when downloading stales, update homestead transition [#537](https://github.com/paritytech/parity/pull/537)
- changing x64 asm config [#534](https://github.com/paritytech/parity/pull/534)
- uncomment state transition tests [#533](https://github.com/paritytech/parity/pull/533)
- jsonrpc uses weak pointers to client [#532](https://github.com/paritytech/parity/pull/532)
- Morden switch to Homestead rules at #494,000. [#531](https://github.com/paritytech/parity/pull/531)
- Blockchain module cleanup [#524](https://github.com/paritytech/parity/pull/524)
- Multiplication issue + very exhaustive tests for it [#528](https://github.com/paritytech/parity/pull/528)
- EIP-8 [#498](https://github.com/paritytech/parity/pull/498)
- Make "random" trie tests fully deterministic. [#527](https://github.com/paritytech/parity/pull/527)
- udpated serde to version 0.7.0 [#526](https://github.com/paritytech/parity/pull/526)
- Better memory management [#516](https://github.com/paritytech/parity/pull/516)
- Typo [#523](https://github.com/paritytech/parity/pull/523)
- U512 add/sub optimize [#521](https://github.com/paritytech/parity/pull/521)
- Account management + geth keystore import (no utility crate added) [#509](https://github.com/paritytech/parity/pull/509)
- Delayed UPnP initialization [#505](https://github.com/paritytech/parity/pull/505)
- Fixing marking blocks as bad & SyncMessage bugs + small client refactoring. [#503](https://github.com/paritytech/parity/pull/503)
- optimization of U256 [#515](https://github.com/paritytech/parity/pull/515)
- Removed rocksdb from build scripts and instructions [#520](https://github.com/paritytech/parity/pull/520)
- RocksDB abstraction layer + Hash index for state DB [#464](https://github.com/paritytech/parity/pull/464)
- bloomfilter [#418](https://github.com/paritytech/parity/pull/418)
- Fixed a race condition when connecting peer disconnects immediately [#519](https://github.com/paritytech/parity/pull/519)
- ignore intellij idea project files as well [#518](https://github.com/paritytech/parity/pull/518)
- updated version of unicase [#517](https://github.com/paritytech/parity/pull/517)
- jsonrpc security, cors headers, fixed #359 [#493](https://github.com/paritytech/parity/pull/493)
- Rust implementations to replace data tables (#161) [#482](https://github.com/paritytech/parity/pull/482)
- fix issue with starting requested block number was not included itself [#512](https://github.com/paritytech/parity/pull/512)
- fixed travis --org GH_TOKEN [#510](https://github.com/paritytech/parity/pull/510)
- Improved log format [#506](https://github.com/paritytech/parity/pull/506)
- Log address on failed connection attempt [#502](https://github.com/paritytech/parity/pull/502)
- Bumping clippy and fixing warnings. [#501](https://github.com/paritytech/parity/pull/501)
- Bumping versions. Fixes #496 [#500](https://github.com/paritytech/parity/pull/500)
- Manage final user-input errors. [#494](https://github.com/paritytech/parity/pull/494)
- Remove unneeded code, fix minor potential issue with length. [#495](https://github.com/paritytech/parity/pull/495)
- Remove "unknown" from version string. [#488](https://github.com/paritytech/parity/pull/488)
- Include git commit date & hash. [#486](https://github.com/paritytech/parity/pull/486)
- Use proper version string. [#485](https://github.com/paritytech/parity/pull/485)
- Networking fixes [#480](https://github.com/paritytech/parity/pull/480)
- Fix potential deadlock on node table update [#484](https://github.com/paritytech/parity/pull/484)
- Squash more warnings [#481](https://github.com/paritytech/parity/pull/481)
- dev/test/build tools to separate crate [#477](https://github.com/paritytech/parity/pull/477)
- Back to original slab crate [#479](https://github.com/paritytech/parity/pull/479)
- Better user errors. [#476](https://github.com/paritytech/parity/pull/476)
- UDP Discovery [#440](https://github.com/paritytech/parity/pull/440)
- update readme with rust override [#475](https://github.com/paritytech/parity/pull/475)
- fixed warnings on rust beta [#474](https://github.com/paritytech/parity/pull/474)
- Secret store (part2 - encrypted key/value svc) [#449](https://github.com/paritytech/parity/pull/449)
- Kill bad test. [#473](https://github.com/paritytech/parity/pull/473)
- Make clippy an optional dependency [#422](https://github.com/paritytech/parity/pull/422)
- parity compiling fine [#469](https://github.com/paritytech/parity/pull/469)
- compiling ethcore on beta [#468](https://github.com/paritytech/parity/pull/468)
- Utils compiling in beta [#467](https://github.com/paritytech/parity/pull/467)
- Get rid of lru_cache dependency [#466](https://github.com/paritytech/parity/pull/466)
- Add daemonization. [#459](https://github.com/paritytech/parity/pull/459)
- Master upgrade [#448](https://github.com/paritytech/parity/pull/448)
- Remove contributing stuff now that we have CLA bot. [#447](https://github.com/paritytech/parity/pull/447)
- Add Morden bootnode. [#446](https://github.com/paritytech/parity/pull/446)
- beta fixes to master [#441](https://github.com/paritytech/parity/pull/441)
- Secret store (part1 - key management) [#423](https://github.com/paritytech/parity/pull/423)
- Use 1100000 as the homestead transition, fix build instructions. [#438](https://github.com/paritytech/parity/pull/438)
- More sync and propagation fixes [#420](https://github.com/paritytech/parity/pull/420)
- back to cargo crates [#436](https://github.com/paritytech/parity/pull/436)
- Fixing clippy warnings [#435](https://github.com/paritytech/parity/pull/435)
- preserving root cargo lock [#434](https://github.com/paritytech/parity/pull/434)
- Nightly fix [#432](https://github.com/paritytech/parity/pull/432)
- nightly fixes [#431](https://github.com/paritytech/parity/pull/431)
- Delay Homestead transition from 1,000,000. [#429](https://github.com/paritytech/parity/pull/429)
- Nightly fix effort (still should fail) [#428](https://github.com/paritytech/parity/pull/428)
- clippy version update, docopt-macro moving to fork [#425](https://github.com/paritytech/parity/pull/425)
- Network/Sync fixes and optimizations [#416](https://github.com/paritytech/parity/pull/416)
- Use latest era instead of end era as journal marker [#414](https://github.com/paritytech/parity/pull/414)
- api changes [#402](https://github.com/paritytech/parity/pull/402)
- Option for no init nodes. [#408](https://github.com/paritytech/parity/pull/408)
- Fixed block_bodies not returning a list [#406](https://github.com/paritytech/parity/pull/406)
- Fix test. [#405](https://github.com/paritytech/parity/pull/405)
- Allow path to be configured. [#404](https://github.com/paritytech/parity/pull/404)
- Upnp [#400](https://github.com/paritytech/parity/pull/400)
- eth_syncing, fixed #397 [#398](https://github.com/paritytech/parity/pull/398)
- Using modified version of ctrlc that catches SIGTERM [#399](https://github.com/paritytech/parity/pull/399)
- Catching panics. [#396](https://github.com/paritytech/parity/pull/396)
- jsonrpc [#391](https://github.com/paritytech/parity/pull/391)
- Externalities tests (still clumsy) [#394](https://github.com/paritytech/parity/pull/394)
- excluding test code itself from coverage [#395](https://github.com/paritytech/parity/pull/395)
- Additional tweaks to options. [#390](https://github.com/paritytech/parity/pull/390)
- --chain option for setting which network to go on. [#388](https://github.com/paritytech/parity/pull/388)
- Ethash unit tests final [#387](https://github.com/paritytech/parity/pull/387)
- jsonrpc [#374](https://github.com/paritytech/parity/pull/374)
- Editorconfig file. [#384](https://github.com/paritytech/parity/pull/384)
- Coverage effort [in progress] [#382](https://github.com/paritytech/parity/pull/382)
- making root kcov runner simular to the one running on CI [#380](https://github.com/paritytech/parity/pull/380)
- add gcc as a dependency to dockerfiles [#381](https://github.com/paritytech/parity/pull/381)
- Check for handshake expiration before attempting connection replace [#375](https://github.com/paritytech/parity/pull/375)
- Blocks propagation [#364](https://github.com/paritytech/parity/pull/364)
- Network params. [#376](https://github.com/paritytech/parity/pull/376)
- Add parity-node-zero to bootnodes. [#373](https://github.com/paritytech/parity/pull/373)
- kcov uses travis_job_id instead of coveralls token [#370](https://github.com/paritytech/parity/pull/370)
- Add parity-node-zero.ethcore.io to boot nodes. [#371](https://github.com/paritytech/parity/pull/371)

## Parity [v1.0.0-rc1](https://github.com/paritytech/parity/releases/tag/v1.0.0-rc1) (2016-03-15)

First Parity 1.0.0 release candidate.

- Version 1.0 in beta [#712](https://github.com/paritytech/parity/pull/712)
- Fix test for beta [#617](https://github.com/paritytech/parity/pull/617)
- JournalDB fix option 1 for beta [#614](https://github.com/paritytech/parity/pull/614)
- Failing test. [#606](https://github.com/paritytech/parity/pull/606)
- Fix transition points [#604](https://github.com/paritytech/parity/pull/604)
- (BETA) Update README.md [#549](https://github.com/paritytech/parity/pull/549)
- (BETA) instructions for beta release channel [#456](https://github.com/paritytech/parity/pull/456)
- (BETA) fix nightly - remerge [#454](https://github.com/paritytech/parity/pull/454)
- (BETA) fixing nightly version for beta [#452](https://github.com/paritytech/parity/pull/452)

## Parity [beta-0.9.1](https://github.com/paritytech/parity/releases/tag/beta-0.9.1) (2016-02-16)

Homestead transition block changed to 1100000.

- Beta patch to 0.9.1 [#445](https://github.com/paritytech/parity/pull/445)
- Delay homestead transition [#430](https://github.com/paritytech/parity/pull/430)
- (BETA) https link in the installer (?) [#392](https://github.com/paritytech/parity/pull/392)
- beta: Check for handshake expiration before attempting replace [#377](https://github.com/paritytech/parity/pull/377)

## Parity [beta-0.9](https://github.com/paritytech/parity/releases/tag/beta-0.9) (2016-02-08)

First Parity Beta 0.9 released.

- Panic on missing counters; Client cleanup [#368](https://github.com/paritytech/parity/pull/368)
- Update README for new PPAs. [#369](https://github.com/paritytech/parity/pull/369)
- block_queue::clear should be more thorough [#365](https://github.com/paritytech/parity/pull/365)
- Fixed an issue with forked counters [#363](https://github.com/paritytech/parity/pull/363)
- Install parity [#362](https://github.com/paritytech/parity/pull/362)
- DB directory versioning [#358](https://github.com/paritytech/parity/pull/358)
- Raise FD limit for MacOS [#357](https://github.com/paritytech/parity/pull/357)
- Travis slack integration. [#356](https://github.com/paritytech/parity/pull/356)
- SignedTransaction structure [#350](https://github.com/paritytech/parity/pull/350)
- License [#354](https://github.com/paritytech/parity/pull/354)
- Performance optimizations [#353](https://github.com/paritytech/parity/pull/353)
- Gitter in README. [#355](https://github.com/paritytech/parity/pull/355)
- test efforts, receipt requests [#352](https://github.com/paritytech/parity/pull/352)
- sync tests setup & local module coverage [#348](https://github.com/paritytech/parity/pull/348)
- install parity script [#347](https://github.com/paritytech/parity/pull/347)
- evmjit homestead merge [#342](https://github.com/paritytech/parity/pull/342)
- Fixed sync stalling on fork [#343](https://github.com/paritytech/parity/pull/343)
- Remerge 264 [#334](https://github.com/paritytech/parity/pull/334)
- Ethsync tests bfix [#339](https://github.com/paritytech/parity/pull/339)
- Fix default options. [#335](https://github.com/paritytech/parity/pull/335)
- sync queue limit hotfix [#338](https://github.com/paritytech/parity/pull/338)
- Network tests, separate local coverage for utils [#333](https://github.com/paritytech/parity/pull/333)
- fix parity version so netstats can parse it [#332](https://github.com/paritytech/parity/pull/332)
- reveal surprise [#331](https://github.com/paritytech/parity/pull/331)
- Revert removal of `new_code`. [#330](https://github.com/paritytech/parity/pull/330)
- Network mod tests first part [#329](https://github.com/paritytech/parity/pull/329)
- Look ma no `dead_code` [#323](https://github.com/paritytech/parity/pull/323)
- Fixing JIT, Updating hook to run `ethcore` tests. [#326](https://github.com/paritytech/parity/pull/326)
- Final docs [#327](https://github.com/paritytech/parity/pull/327)
- update install-deps.sh [#316](https://github.com/paritytech/parity/pull/316)
- Finish all my docs. Fix previous test compilation. [#320](https://github.com/paritytech/parity/pull/320)
- Additional evm tests (extops, call, jumps) and some docs [#317](https://github.com/paritytech/parity/pull/317)
- More documentation. [#318](https://github.com/paritytech/parity/pull/318)
- Additional documentation. [#315](https://github.com/paritytech/parity/pull/315)
- unused functions cleanup [#310](https://github.com/paritytech/parity/pull/310)
- update ethcore.github.io documentation automatically [#311](https://github.com/paritytech/parity/pull/311)
- Another try with travis ci credentials [#314](https://github.com/paritytech/parity/pull/314)
- Document some stuff. [#309](https://github.com/paritytech/parity/pull/309)
- Check block parent on import; Peer timeouts [#303](https://github.com/paritytech/parity/pull/303)
- Increasing coverage for evm. [#306](https://github.com/paritytech/parity/pull/306)
- ethcore docs [#301](https://github.com/paritytech/parity/pull/301)
- Replacing secure token for deployment [#305](https://github.com/paritytech/parity/pull/305)
- doc.sh [#299](https://github.com/paritytech/parity/pull/299)
- Building beta-* and stable-* tags [#302](https://github.com/paritytech/parity/pull/302)
- Deploying artifacts for tags (release/beta) [#300](https://github.com/paritytech/parity/pull/300)
- cov.sh to show coverage locally [#298](https://github.com/paritytech/parity/pull/298)
- benchmark fixes [#297](https://github.com/paritytech/parity/pull/297)
- Include JSONRPC CLI options. [#296](https://github.com/paritytech/parity/pull/296)
- travis.yml fixes [#293](https://github.com/paritytech/parity/pull/293)
- Improve version string. [#295](https://github.com/paritytech/parity/pull/295)
- Fixed block queue test [#294](https://github.com/paritytech/parity/pull/294)
- Util docs [#292](https://github.com/paritytech/parity/pull/292)
- fixed building docs [#289](https://github.com/paritytech/parity/pull/289)
- update travis to build PRs only against master [#290](https://github.com/paritytech/parity/pull/290)
- Coverage effort [#272](https://github.com/paritytech/parity/pull/272)
- updated docker containers [#288](https://github.com/paritytech/parity/pull/288)
- rpc module fixes [#287](https://github.com/paritytech/parity/pull/287)
- Test for Receipt RLP. [#282](https://github.com/paritytech/parity/pull/282)
- Building from source guide [#284](https://github.com/paritytech/parity/pull/284)
- Fixed neted empty list RLP encoding [#283](https://github.com/paritytech/parity/pull/283)
- Fix CALLDATACOPY (and bonus CODECOPY, too!). [#279](https://github.com/paritytech/parity/pull/279)
- added travis && coveralls badge to README.md [#280](https://github.com/paritytech/parity/pull/280)
- coveralls coverage [#277](https://github.com/paritytech/parity/pull/277)
- Travis [in progress] [#257](https://github.com/paritytech/parity/pull/257)
- Travis on reorganized repo [#276](https://github.com/paritytech/parity/pull/276)
- umbrella project [#275](https://github.com/paritytech/parity/pull/275)
- Ethash disk cache [#273](https://github.com/paritytech/parity/pull/273)
- Parity executable name and version [#274](https://github.com/paritytech/parity/pull/274)
- Dockerfile [#195](https://github.com/paritytech/parity/pull/195)
- Garbage collection test fix [#267](https://github.com/paritytech/parity/pull/267)
- Fix stCallCreateCallCodeTest, add more tests [#271](https://github.com/paritytech/parity/pull/271)
- Moved sync out of ethcore crate; Added block validation [#265](https://github.com/paritytech/parity/pull/265)
- RLP encoder refactoring [#252](https://github.com/paritytech/parity/pull/252)
- Chain sync tests and minor refactoring [#264](https://github.com/paritytech/parity/pull/264)
- Common log init function [#263](https://github.com/paritytech/parity/pull/263)
- changed max vm depth from 128 to 64, change homestead block to 1_000_000 [#262](https://github.com/paritytech/parity/pull/262)
- fixed blockchain tests crash on log init [#261](https://github.com/paritytech/parity/pull/261)
- Blockchain tests and some helpers for guarding temp directory [#256](https://github.com/paritytech/parity/pull/256)
- Fix logging and random tests. [#260](https://github.com/paritytech/parity/pull/260)
- Fix difficulty calculation algo. [#259](https://github.com/paritytech/parity/pull/259)
- fix submodule version [#258](https://github.com/paritytech/parity/pull/258)
- temp dir spawn refactoring [#246](https://github.com/paritytech/parity/pull/246)
- fixed tests submodule branch [#254](https://github.com/paritytech/parity/pull/254)
- rpc net methods returns real peer count && protocol version [#253](https://github.com/paritytech/parity/pull/253)
- Add homestead & random tests. [#245](https://github.com/paritytech/parity/pull/245)
- Fixing suicide with self-refund to be consistent with CPP. [#247](https://github.com/paritytech/parity/pull/247)
- stubs for rpc methods [#251](https://github.com/paritytech/parity/pull/251)
- clippy, missing docs, renaming etc. [#244](https://github.com/paritytech/parity/pull/244)
- impl missing methods in tests [#243](https://github.com/paritytech/parity/pull/243)
- General tests and some helpers [#239](https://github.com/paritytech/parity/pull/239)
- Note additional tests are fixed, fix doc test. [#242](https://github.com/paritytech/parity/pull/242)
- jsonrpc http server [#193](https://github.com/paritytech/parity/pull/193)
- Ethash nonce is H64 not a u64 [#240](https://github.com/paritytech/parity/pull/240)
- Fix import for bcMultiChainTest [#236](https://github.com/paritytech/parity/pull/236)
- Client basic tests [#232](https://github.com/paritytech/parity/pull/232)
- Fix ensure_db_good() and flush_queue(), block refactoring, check block format, be strict. [#231](https://github.com/paritytech/parity/pull/231)
- Rlp [#207](https://github.com/paritytech/parity/pull/207)
- Schedule documentation [#219](https://github.com/paritytech/parity/pull/219)
- U256<->H256 Conversion [#206](https://github.com/paritytech/parity/pull/206)
- Spawning new thread when we are reaching stack limit [#217](https://github.com/paritytech/parity/pull/217)
- Blockchain tests [#211](https://github.com/paritytech/parity/pull/211)
- fixed failing sync test [#218](https://github.com/paritytech/parity/pull/218)
- Removing println [#216](https://github.com/paritytech/parity/pull/216)
- Cleaning readme [#212](https://github.com/paritytech/parity/pull/212)
- Fixing delegatecall [#196](https://github.com/paritytech/parity/pull/196)
- Autogenerate the Args from the docopt macro. [#205](https://github.com/paritytech/parity/pull/205)
- Networking fixes [#202](https://github.com/paritytech/parity/pull/202)
- Argument parsing from CLI [#204](https://github.com/paritytech/parity/pull/204)
- Removed wildcard from clippy version [#203](https://github.com/paritytech/parity/pull/203)
- Fixed tests and tweaked sync progress report [#201](https://github.com/paritytech/parity/pull/201)
- Heavy tests [#199](https://github.com/paritytech/parity/pull/199)
- Mutithreaded IO [#198](https://github.com/paritytech/parity/pull/198)
- Populating last_hashes [#197](https://github.com/paritytech/parity/pull/197)
- Fixing clippy stuff [#170](https://github.com/paritytech/parity/pull/170)
- basic .travis.yml [#194](https://github.com/paritytech/parity/pull/194)
- Generating coverage reports. [#190](https://github.com/paritytech/parity/pull/190)
- Adding doc requests comments [#192](https://github.com/paritytech/parity/pull/192)
- moved src/bin/client.rs -> src/bin/client/main.rs [#185](https://github.com/paritytech/parity/pull/185)
- removed overflowing_shr [#188](https://github.com/paritytech/parity/pull/188)
- fixed wrapping ops on latest nightly [#187](https://github.com/paritytech/parity/pull/187)
- Pruned state DB [#176](https://github.com/paritytech/parity/pull/176)
- Memory management for cache [#180](https://github.com/paritytech/parity/pull/180)
- Implement signs having low-s. [#183](https://github.com/paritytech/parity/pull/183)
- Introduce sha3 crate and use it in ethash [#178](https://github.com/paritytech/parity/pull/178)
- Multithreaded block queue [#173](https://github.com/paritytech/parity/pull/173)
- Iterator for NibbleSlice and TrieDB. [#171](https://github.com/paritytech/parity/pull/171)
- Handling all possible overflows [#145](https://github.com/paritytech/parity/pull/145)
- Global secp256k1 context [#164](https://github.com/paritytech/parity/pull/164)
- Ethash [#152](https://github.com/paritytech/parity/pull/152)
- Move util into here [#153](https://github.com/paritytech/parity/pull/153)
- EVM Interpreter [#103](https://github.com/paritytech/parity/pull/103)
- Homestead transition support, maybe. [#141](https://github.com/paritytech/parity/pull/141)
- externalities refactor [#131](https://github.com/paritytech/parity/pull/131)
- More open files. [#140](https://github.com/paritytech/parity/pull/140)
- Single array for logs output. [#133](https://github.com/paritytech/parity/pull/133)
- Client app event handler [#132](https://github.com/paritytech/parity/pull/132)
- Various consensus fixes. [#130](https://github.com/paritytech/parity/pull/130)
- callcode builtins tests pass [#127](https://github.com/paritytech/parity/pull/127)
- Client state syncing [#119](https://github.com/paritytech/parity/pull/119)
- Split externalities from executive. [#126](https://github.com/paritytech/parity/pull/126)
- executive error on not enoguh base gas [#124](https://github.com/paritytech/parity/pull/124)
- Gav [#125](https://github.com/paritytech/parity/pull/125)
- builtin sets excepted to true [#123](https://github.com/paritytech/parity/pull/123)
- More state tests. [#122](https://github.com/paritytech/parity/pull/122)
- updated to rocksdb wrapper version 0.3 [#121](https://github.com/paritytech/parity/pull/121)
- out_of_gas -> excepted [#120](https://github.com/paritytech/parity/pull/120)
- Parametrizing evm::Factory [#111](https://github.com/paritytech/parity/pull/111)
- stLogs tests passing [#118](https://github.com/paritytech/parity/pull/118)
- Fix executive. [#117](https://github.com/paritytech/parity/pull/117)
- Fixes for marek's shooting from the hip. [#116](https://github.com/paritytech/parity/pull/116)
- Executive revert fix [#115](https://github.com/paritytech/parity/pull/115)
- Fix storage/account and add butress test. [#114](https://github.com/paritytech/parity/pull/114)
- Refactored Pod & Diff types into separate files, JSON infrastructure revamp. [#113](https://github.com/paritytech/parity/pull/113)
- Fix storage stuff and introduce per-item dirty-tracking. [#112](https://github.com/paritytech/parity/pull/112)
- Check logs in state tests. [#109](https://github.com/paritytech/parity/pull/109)
- executive gas calculation fixes [#108](https://github.com/paritytech/parity/pull/108)
- proper gas calculation in executive [#107](https://github.com/paritytech/parity/pull/107)
- Fixing MaxDepth param for executive [#105](https://github.com/paritytech/parity/pull/105)
- Fix determination of state roots. [#106](https://github.com/paritytech/parity/pull/106)
- transact substracts tx_gas [#104](https://github.com/paritytech/parity/pull/104)
- Pretty-print and fix for state. [#102](https://github.com/paritytech/parity/pull/102)
- Tier step price. [#101](https://github.com/paritytech/parity/pull/101)
- Refactor Diff datastructures. [#100](https://github.com/paritytech/parity/pull/100)
- externalities use u256 instead of u64 for gas calculation [#99](https://github.com/paritytech/parity/pull/99)
- Executive tests [#97](https://github.com/paritytech/parity/pull/97)
- State conensus tests now print mismatching diff on fail. [#98](https://github.com/paritytech/parity/pull/98)
- State testing framework. First test is failing. [#96](https://github.com/paritytech/parity/pull/96)
- executive tests [#95](https://github.com/paritytech/parity/pull/95)
- Use U512s for ether cost calculation, complete transaction API [#94](https://github.com/paritytech/parity/pull/94)
- Utils for consensus test decoding and better layout. [#93](https://github.com/paritytech/parity/pull/93)
- executive fixes + tests [#89](https://github.com/paritytech/parity/pull/89)
- All transaction tests pass. Nicer testing framework. [#92](https://github.com/paritytech/parity/pull/92)
- Block verification tests; BlockProvider blockchain trait for testing [#88](https://github.com/paritytech/parity/pull/88)
- State::exists, docs and tests. [#87](https://github.com/paritytech/parity/pull/87)
- Add tests module, add two more transaction tests. [#86](https://github.com/paritytech/parity/pull/86)
- bring back removed tests, removed build warnings [#82](https://github.com/paritytech/parity/pull/82)
- Nicer transaction validation API. Nicer OutOfBounds API in general. [#85](https://github.com/paritytech/parity/pull/85)
- Transaction fixes and consensus tests (all passing) [#84](https://github.com/paritytech/parity/pull/84)
- fixed getting block info in evmjit + tests [#81](https://github.com/paritytech/parity/pull/81)
- evm tests cleanup [#80](https://github.com/paritytech/parity/pull/80)
- renamed VmFactory -> Factory [#77](https://github.com/paritytech/parity/pull/77)
- fixed rust-evmjit description of improper_ctypes usage [#76](https://github.com/paritytech/parity/pull/76)
- jit feature enabled by default [#75](https://github.com/paritytech/parity/pull/75)
- evm [#52](https://github.com/paritytech/parity/pull/52)
- state clone [#74](https://github.com/paritytech/parity/pull/74)
- Block Verification (no tests yet) [#72](https://github.com/paritytech/parity/pull/72)
- Improvements to LogEntry and Transaction [#73](https://github.com/paritytech/parity/pull/73)
- Use getter in header in preparation for a Header trait; additional testing in enact_block(). [#64](https://github.com/paritytech/parity/pull/64)
- BlockChain sync and Client app [#55](https://github.com/paritytech/parity/pull/55)
- Block enactment (including test) [#63](https://github.com/paritytech/parity/pull/63)
- Block complete. Needs tests. [#62](https://github.com/paritytech/parity/pull/62)
- More on OpenBlock::close; State::kill_account added [#61](https://github.com/paritytech/parity/pull/61)
- Remove genesis module, add more chain specs and separate out ethereum-specific stuff [#60](https://github.com/paritytech/parity/pull/60)
- State::new_contract, camelCase engine params, missing param [#59](https://github.com/paritytech/parity/pull/59)
- Use reorganisation [#58](https://github.com/paritytech/parity/pull/58)
- Initial Ethash/Block skeleton implementations. [#57](https://github.com/paritytech/parity/pull/57)
- Spec with tested Morden genesis decoder and builtins. [#54](https://github.com/paritytech/parity/pull/54)
- Move all chain parameters into `engine_params` [#50](https://github.com/paritytech/parity/pull/50)
- jit ffi improvements [please review] [#51](https://github.com/paritytech/parity/pull/51)
- blockchain [please review] [#34](https://github.com/paritytech/parity/pull/34)
- Move information from networkparams.rs into spec.rs [#48](https://github.com/paritytech/parity/pull/48)
- Move bulking out in Engine/Params. [#47](https://github.com/paritytech/parity/pull/47)
- Removed need for mutation in State. [#46](https://github.com/paritytech/parity/pull/46)
- State::code and State::storage_at + tests. [#45](https://github.com/paritytech/parity/pull/45)
- State functions for balance and nonce operations [#44](https://github.com/paritytech/parity/pull/44)
- Account::storage_at, Account::ensure_cached and tests. [#43](https://github.com/paritytech/parity/pull/43)
- Additional tests. [#42](https://github.com/paritytech/parity/pull/42)
- seal todo done [#41](https://github.com/paritytech/parity/pull/41)
- missing rustc_serialize crate && rlp `as_list` function [#40](https://github.com/paritytech/parity/pull/40)
- More methods in Account, documentation and tests. [#39](https://github.com/paritytech/parity/pull/39)
- Minor reworking of Account. [#38](https://github.com/paritytech/parity/pull/38)
- Add Account and State classes. [#37](https://github.com/paritytech/parity/pull/37)
- Revert regressions [#36](https://github.com/paritytech/parity/pull/36)
