# ethstore

[![Build Status][travis-image]][travis-url]

[travis-image]: https://travis-ci.org/ethcore/ethstore.svg?branch=master
[travis-url]: https://travis-ci.org/ethcore/ethstore

Ethereum key management.

[Documentation](http://ethcore.github.io/ethstore/ethstore/index.html)

### Usage

```
Ethereum key management.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    ethstore insert <secret> <password> [--dir DIR]
    ethstore change-pwd <address> <old-pwd> <new-pwd> [--dir DIR]
    ethstore list [--dir DIR]
    ethstore import [--src DIR] [--dir DIR]
    ethstore import-wallet <path> <password> [--dir DIR]
    ethstore remove <address> <password> [--dir DIR]
    ethstore sign <address> <password> <message> [--dir DIR]
    ethstore [-h | --help]

Options:
    -h, --help         Display this message and exit.
    --dir DIR          Specify the secret store directory. It may be either
                       parity, parity-test, geth, geth-test
                       or a path [default: parity].
    --src DIR          Specify import source. It may be either
                       parity, parity-test, get, geth-test
                       or a path [default: geth].

Commands:
    insert             Save account with password.
    change-pwd         Change account password.
    list               List accounts.
    import             Import accounts from src.
    import-wallet      Import presale wallet.
    remove             Remove account.
    sign               Sign message.
```

### Examples

#### `insert <secret> <password> [--dir DIR]`
*Encrypt secret with a password and save it in secret store.*

- `<secret>` - ethereum secret, 32 bytes long
- `<password>` - account password, file path
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore insert 7d29fab185a33e2cd955812397354c472d2b84615b645aa135ff539f6b0d70d5 password.txt
```

```
a8fa5dd30a87bb9e3288d604eb74949c515ab66e
```

--

```
ethstore insert `ethkey generate random -s` "this is sparta"
```

```
24edfff680d536a5f6fe862d36df6f8f6f40f115
```

--

#### `change-pwd <address> <old-pwd> <new-pwd> [--dir DIR]`
*Change account password.*

- `<address>` - ethereum address, 20 bytes long
- `<old-pwd>` - old account password, file path
- `<new-pwd>` - new account password, file path
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore change-pwd a8fa5dd30a87bb9e3288d604eb74949c515ab66e old_pwd.txt new_pwd.txt
```

```
true
```

--

#### `list [--dir DIR]`
*List secret store accounts.*

- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore list
```

```
 0: 24edfff680d536a5f6fe862d36df6f8f6f40f115
 1: 6edddfc6349aff20bc6467ccf276c5b52487f7a8
 2: e6a3d25a7cb7cd21cb720df5b5e8afd154af1bbb
```

--

#### `import [--src DIR] [--dir DIR]`
*Import accounts from src.*

- `[--src DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: geth
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore import
```

```
 0: e6a3d25a7cb7cd21cb720df5b5e8afd154af1bbb
 1: 6edddfc6349aff20bc6467ccf276c5b52487f7a8
```

--

#### `import-wallet <path> <password> [--dir DIR]`
*Import account from presale wallet.*

- `<path>` - presale wallet path
- `<password>` - account password, file path
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore import-wallet ethwallet.json password.txt
```

```
e6a3d25a7cb7cd21cb720df5b5e8afd154af1bbb
```

--

#### `remove <address> <password> [--dir DIR]`
*Remove account from secret store.*

- `<address>` - ethereum address, 20 bytes long
- `<password>` - account password, file path
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore remove a8fa5dd30a87bb9e3288d604eb74949c515ab66e password.txt
```

```
true
```

--

#### `sign <address> <password> <message> [--dir DIR]`
*Sign message with account's secret.*

- `<address>` - ethereum address, 20 bytes long
- `<password>` - account password, file path
- `<message>` - message to sign, 32 bytes long
- `[--dir DIR]` - secret store directory, It may be either parity, parity-test, geth, geth-test or a path. default: parity

```
ethstore sign 24edfff680d536a5f6fe862d36df6f8f6f40f115 password.txt 7d29fab185a33e2cd955812397354c472d2b84615b645aa135ff539f6b0d70d5
```

```
c6649f9555232d90ff716d7e552a744c5af771574425a74860e12f763479eb1b708c1f3a7dc0a0a7f7a81e0a0ca88c6deacf469222bb3d9c5bf0847f98bae54901
```

--

# Ethcore toolchain
*this project is a part of the ethcore toolchain*

- [**ethkey**](https://github.com/ethcore/ethkey) - Ethereum keys generator and signer.
- [**ethstore**](https://github.com/ethcore/ethstore) - Ethereum key management.
- [**ethabi**](https://github.com/ethcore/ethabi) - Ethereum function calls encoding.
