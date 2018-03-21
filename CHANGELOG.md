## Parity [v1.10.0](https://github.com/paritytech/parity/releases/tag/v1.10.0) (2018-03-22)

This is the Parity 1.10.0-beta release! Cool!

### Disabling the Parity Wallet

The **Parity Wallet (a.k.a. "UI") is now disabled by default**. We are preparing to split the wallet from the core client.

To reactivate the parity wallet, you have to run Parity either with `parity --force-ui` (not recommended) or `parity ui` (deprecated) from the command line. Or, if you feel super fancy and want to test our pre-releases of the stand-alone electron wallet, head over to the [Parity-JS repositories and check the releases](https://github.com/Parity-JS/shell/releases).

### Introducing the Wasm VM

We are excited to announce support for **Wasm Smart Contracts on Kovan network**. The hard-fork to activate the Wasm-VM will take place on block `6600000`.

To enable Wasm contracts on your custom network, just schedule a `wasmActivationTransition` at your favorite block number (e.g., `42`, `666`, or `0xbada55`). To hack your first Wasm smart contracts in Rust, have a look at the [Parity Wasm Tutorials](https://github.com/paritytech/pwasm-tutorial).

### Empty step messages in PoA

To **reduce blockchain bloat, proof-of-authority networks can now enable _empty step messages_ which replace empty blocks**. Each step message will be signed and broadcasted by the issuing authorities, and included and rewarded in the next non-empty block.

To enable empty step messages, set the `emptyStepsTransition` to your favorite block number. You can also specify a maximum number of empty steps with `maximumEmptySteps` in your chain spec.

### Other noteworthy changes

We removed the old database migrations from 2016. In case you upgrade Parity from a really, really old version, you will have to reset your database manually first with `parity <options> db kill`.

We fixed  DELEGATECALL `from` and `to` fields, see [#7166](https://github.com/paritytech/parity/issues/7166).

We reduced the default USD per transaction value to 0.0001. Thanks, @MysticRyuujin!

The full list of included changes:

- @TODO

## Previous releases

- [CHANGELOG-1.8](docs/CHANGELOG-1.9.md) (_stable_)
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
