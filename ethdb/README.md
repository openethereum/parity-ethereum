## ethdb

Parity Ethereum interaction with the DB on-demand.

### Usage

```
Parity Ethereum interaction with the DB on-demand.
  Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
    ethdb build-trace-db
    ethdb [options]
    ethdb [-h | --help]

Commands:
    build-trace-db     Build TraceDB on-demand and output to json file.

Tracing options:
    --chain CHAIN      Build only from specific chain.
    --from BLOCK       Build only from a specific block.
    --to BLOCK         Build only to a specific block.

General options:
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --std-err-only     With --std-json redirect to err output only.
    --std-out-only     With --std-json redirect to out output only.
    --std-dump-json    Display results in standardized JSON format
                       with additional state dump.
    -h, --help         Display this message and exit.
```

## Parity Ethereum toolchain
_This project is a part of the Parity Ethereum toolchain._

- [evmbin](https://github.com/paritytech/parity-ethereum/blob/master/evmbin/) - EVM implementation for Parity Ethereum.
- [ethabi](https://github.com/paritytech/ethabi) - Parity Ethereum function calls encoding.
- [ethdb](https://github.com/paritytech/ethdb) - Parity Ethereum interaction with the DB on-demand.
- [ethstore](https://github.com/paritytech/parity-ethereum/blob/master/accounts/ethstore) - Parity Ethereum key management.
- [ethkey](https://github.com/paritytech/parity-ethereum/blob/master/accounts/ethkey) - Parity Ethereum keys generator.
- [whisper](https://github.com/paritytech/parity-ethereum/blob/master/whisper/) - Implementation of Whisper-v2 PoC.
