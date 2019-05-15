## ethdb

Parity Ethereum interaction with the DB on-demand.

### Usage

```
Parity Ethereum interaction with the DB on-demand.
  Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
    ethdb trace [--chain CHAIN --from=<block> --to=<block> --diff]
    ethdb extract [--chain CHAIN --from=<block> --to=<block> --block --receipts]
    ethdb state [--chain CHAIN --from=<block> --to=<block> --json --std-json  --std-err-only --std-out-only --std-dump-json]
    ethdb [-h | --help]
    ethdb --version

Commands:
    trace              Build TraceDB on-demand and add to node.
    extract            Extract data and output in JSON.
    state              State dump.

Trace options:
    --diff             Re-run block and produce state difference.

Extract options:
    --block            Block data.
    --receipts         Receipts.

General options:
    --chain CHAIN      Build only from specific chain.
    --from BLOCK       Build only from a specific block.
    --to BLOCK         Build only to a specific block.
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
