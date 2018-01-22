Note: Parity 1.3 reached End-of-Life on 2017-01-19 (EOL).

## Parity [v1.3.15](https://github.com/paritytech/parity/releases/tag/v1.3.15) (2016-12-10)

This patch release fixes an issue with syncing on the Ropsten test network.

- Backporting to stable [#3793](https://github.com/paritytech/parity/pull/3793)

## Parity [v1.3.14](https://github.com/paritytech/parity/releases/tag/v1.3.14) (2016-11-25)

Parity 1.3.14 fixes a few stability issues and adds support for the Ropsten testnet.

- Backporting to stable [#3616](https://github.com/paritytech/parity/pull/3616)

## Parity [v1.3.13](https://github.com/paritytech/parity/releases/tag/v1.3.13) (2016-11-18)

This release fixes an issue with EIP-155 transactions being allowed into the transaction pool.

- [stable] Check tx signatures before adding to the queue. [#3521](https://github.com/paritytech/parity/pull/3521)
- Fix Stable Docker Build [#3479](https://github.com/paritytech/parity/pull/3479)

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
