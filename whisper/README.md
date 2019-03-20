## Whisper

Implementation of Whisper based on the Whisper-v2 PoC.

### Usage

```
Parity Whisper-v2 CLI.
	Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
	whisper [options]
	whisper [-h | --help]

Options:
	--whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
	-p, --port PORT                Specify which P2P port to use [default: random].
	-a, --address ADDRESS          Specify which P2P address to use [default: 127.0.0.1].
	-s, --secret KEYFILE           Specify which file contains the key to generate the enode.
    -P, --rpc-port PORT            Specify which RPC port to use [default: 8545].
    -A, --rpc-address ADDRESS      Specify which RPC address to use [default: 127.0.0.1].
	-l, --log LEVEL                Specify the logging level. Must conform to the same format as RUST_LOG [default: Error].
	-h, --help                     Display this message and exit.
```

## Parity Ethereum toolchain
_This project is a part of the Parity Ethereum toolchain._

- [evmbin](https://github.com/paritytech/parity-ethereum/blob/master/evmbin/) - EVM implementation for Parity Ethereum.
- [ethabi](https://github.com/paritytech/ethabi) - Parity Ethereum function calls encoding.
- [ethstore](https://github.com/paritytech/parity-ethereum/blob/master/accounts/ethstore) - Parity Ethereum key management.
- [ethkey](https://github.com/paritytech/parity-ethereum/blob/master/accounts/ethkey) - Parity Ethereum keys generator.
- [whisper](https://github.com/paritytech/parity-ethereum/blob/master/whisper/) - Implementation of Whisper-v2 PoC.
