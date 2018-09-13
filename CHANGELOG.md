## Parity-Ethereum [v2.1.0](https://github.com/paritytech/parity-ethereum/releases/tag/v2.1.0) (2018-09-17)

Parity-Ethereum 2.1.0-beta is released! Look at this!

Important notices:

- This release moves the 2.1 track of Parity-Ethereum to beta.
- This release contains a low-severity issue with the web-sockets ports. [#9545](https://github.com/paritytech/parity-ethereum/pull/9545)
- This release resolves a potential network fragmentation issue. [#9526](https://github.com/paritytech/parity-ethereum/pull/9526)
- Users are urged to upgrade to 2.0.5-stable or 2.1.0-beta.

Further changes worth highlighting:

- Generalized `blockReward` and `difficultyBombDelays` config ([#9480](https://github.com/paritytech/parity-ethereum/pull/9480)): This removes `eip649*` parameters from chain specs and allows `blockReward` to accept `multi`. Please review your chain-specs!
- Implement EIP234 `block_hash` for `eth_getLogs` ([#9256](https://github.com/paritytech/parity-ethereum/pull/9256)): If `block_hash` and `from_block`/`to_block` present, return error. This also changes `eth_getLogs` to return error if any of `block_hash`/`from_block`/`to_block` cannot be found.
- Comply EIP-86 with the new definition ([#9140](https://github.com/paritytech/parity-ethereum/pull/9140)). Note that this is backward incompatible, with the assumption that none (or not many) working chains have attempted to set the `eip86Transition` flag, because it's not really useful by itself.
- Light clients provide the actual account for `eth_coinbase` RPC ([#9383](https://github.com/paritytech/parity-ethereum/pull/9383)). Note, this behavior is different from the full client where we return the `0x0` address if no accounts are found!
- Light servers give free credits for reserved peers ([#9448](https://github.com/paritytech/parity-ethereum/pull/9448)): When connecting to a reserved peers in light client, assign free credits to them. This fixes the scenario where a home server is running a full node, and another light client connects to the full node as a reserved peer.
- Multi-threaded snapshot creation ([#9239](https://github.com/paritytech/parity-ethereum/pull/9239)): to speed up the generation of snapshots on disk, this can be parallelized now. By default N/2 CPU cores are being used. Use `--snapshot-threads` flag to customize the number of threads.
- Remove all dapp permissions related settings ([#9120](https://github.com/paritytech/parity-ethereum/pull/9120)). This completely removes dapp permission settings from `AccountProvider` and JSON-RPC: In JSON-RPC, all available accounts are returned, regardless of the origin; previously we return accounts based on dapps policy. It's not possible to set a "default account" (as for dapps) any more; this is now always the first account in the available account list.
- Remove unused `--tx_queue_gas` parameter ([#9153](https://github.com/paritytech/parity-ethereum/pull/9153)). Please use `parity --help` to learn about available transaction queue strategies.

The full list of included changes:

- @TBA

## Previous releases

- [CHANGELOG-2.0](docs/CHANGELOG-2.0.md) (_stable_)
- [CHANGELOG-1.11](docs/CHANGELOG-1.11.md) (EOL: 2018-09-17)
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
