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
