## evmbin

EVM implementation for OpenEthereum.

### Usage

```
EVM implementation for OpenEthereum.
  Copyright 2015-2020 Parity Technologies (UK) Ltd.

Usage:
    openethereum-evm state-test <file> [--json --std-json --std-dump-json --only NAME --chain CHAIN --std-out-only --std-err-only]
    openethereum-evm stats [options]
    openethereum-evm stats-jsontests-vm <file>
    openethereum-evm [options]
    openethereum-evm [-h | --help]

Commands:
    state-test         Run a state test from a json file.
    stats              Execute EVM runtime code and return the statistics.
    stats-jsontests-vm Execute standard json-tests format VMTests and return
                       timing statistics in tsv format.

Transaction options:
    --code CODE        Contract code as hex (without 0x).
    --to ADDRESS       Recipient address (without 0x).
    --from ADDRESS     Sender address (without 0x).
    --input DATA       Input data as hex (without 0x).
    --gas GAS          Supplied gas as hex (without 0x).
    --gas-price WEI    Supplied gas price as hex (without 0x).

State test options:
    --only NAME        Runs only a single test matching the name.
    --chain CHAIN      Run only tests from specific chain.

General options:
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --std-err-only     With --std-json redirect to err output only.
    --std-out-only     With --std-json redirect to out output only.
    --std-dump-json    Display results in standardized JSON format
                       with additional state dump.
Display result state dump in standardized JSON format.
    --chain CHAIN      Chain spec file path.
    -h, --help         Display this message and exit.
```

## OpenEthereum toolchain
_This project is a part of the OpenEthereum toolchain._

- [evmbin](https://github.com/openethereum/openethereum/blob/master/evmbin/) - EVM implementation for OpenEthereum
- [ethabi](https://github.com/openethereum/ethabi) - OpenEthereum function calls encoding.
- [ethstore](https://github.com/openethereum/openethereum/blob/master/accounts/ethstore) - OpenEthereum key management.
- [ethkey](https://github.com/openethereum/openethereum/blob/master/accounts/ethkey) - OpenEthereum keys generator.
