# [Parity](https://ethcore.io/parity.html)
### Fast, light, and robust Ethereum implementation

[![Build Status][travis-image]][travis-url] [![Coverage Status][coveralls-image]][coveralls-url] [![Join the chat at https://gitter.im/trogdoro/xiki][gitter-image]][gitter-url] [![GPLv3][license-image]][license-url]

[travis-image]: https://travis-ci.org/ethcore/parity.svg?branch=master
[travis-url]: https://travis-ci.org/ethcore/parity
[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/ethcore/parity?branch=master
[gitter-image]: https://badges.gitter.im/Join%20Chat.svg
[gitter-url]: https://gitter.im/ethcore/parity?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
[license-image]: https://img.shields.io/badge/license-GPL%20v3-green.svg
[license-url]: http://www.gnu.org/licenses/gpl-3.0.en.html

[Internal Documentation](http://ethcore.github.io/parity/ethcore/index.html)

----

## About Parity

Parity's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Parity using the sophisticated and
cutting-edge Rust programming language. Parity is licensed under the GPLv3, and can be used for all your Ethereum needs.

By default, Parity will run a JSONRPC server on `127.0.0.1:8545`. This is fully configurable and supports a number
of RPC APIs.

Parity also runs a server for running decentralized apps, or "Dapps", on `http://127.0.0.1:8080`.
This includes a few useful Dapps, including Ethereum Wallet, Maker OTC, and a node status page.
In a near-future release, it will be easy to install Dapps and use them through this web interface.

If you run into an issue while using parity, feel free to file one in this repository
or hop on our [gitter chat room]([gitter-url]) to ask a question. We are glad to help!

Parity's current release is 1.1. You can download it at https://ethcore.io/parity.html or follow the instructions
below to build from source.

----

## Building from source

Parity is fully compatible with Stable Rust.

We recommend installing Rust through [multirust](https://github.com/brson/multirust). If you don't already have multirust, you can install it like this:

- Linux:
```bash
$ curl -sf https://raw.githubusercontent.com/brson/multirust/master/quick-install.sh | sh
```

- OSX with Homebrew:
```bash
$ brew update && brew install multirust
$ multirust default stable
```

Then, download and build Parity:

```bash
# download Parity code
$ git clone https://github.com/ethcore/parity
$ cd parity

# build in release mode
$ cargo build --release
```

This will produce an executable in the `target/release` subdirectory.
Either run `cd target/release`, or copy `target/release/parity` to another location.

To get started, just run
```bash
$ parity
```

and parity will begin syncing the Ethereum blockchain.