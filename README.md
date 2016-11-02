# [Parity](https://ethcore.io/parity.html)
### Fast, light, and robust Ethereum implementation

[![Join the chat at https://gitter.im/ethcore/parity.js](https://badges.gitter.im/ethcore/parity.js.svg)](https://gitter.im/ethcore/parity.js?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[![Build Status][travis-image]][travis-url] [![Coverage Status][coveralls-image]][coveralls-url] [![Join the chat at https://gitter.im/ethcore/parity][gitter-image]][gitter-url] [![GPLv3][license-image]][license-url]

[Internal Documentation][doc-url]


Be sure to check out [our wiki][wiki-url] for more information.

[travis-image]: https://travis-ci.org/ethcore/parity.svg?branch=master
[travis-url]: https://travis-ci.org/ethcore/parity
[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/ethcore/parity?branch=master
[gitter-image]: https://badges.gitter.im/Join%20Chat.svg
[gitter-url]: https://gitter.im/ethcore/parity?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
[license-image]: https://img.shields.io/badge/license-GPL%20v3-green.svg
[license-url]: https://www.gnu.org/licenses/gpl-3.0.en.html
[doc-url]: https://ethcore.github.io/parity/ethcore/index.html
[wiki-url]: https://github.com/ethcore/parity/wiki

**Requires Rust version 1.12.0 to build**

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
or hop on our [gitter chat room][gitter-url] to ask a question. We are glad to help!

Parity's current release is 1.3. You can download it at https://ethcore.io/parity.html or follow the instructions
below to build from source.

----

## Build dependencies

Parity is fully compatible with Stable Rust.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have rustup, you can install it like this:

- Linux and OSX:
	```bash
	$ curl https://sh.rustup.rs -sSf | sh
	```

- Windows

    Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the rustup installer from
	https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the msvc toolchain:
    ```
	$ rustup default stable-x86_64-pc-windows-msvc
    ```

Once you have rustup, install parity or download and build from source

----

## Quick install

```bash
cargo install --git https://github.com/ethcore/parity.git parity
```

----

## Build from source

```bash
# download Parity code
$ git clone https://github.com/ethcore/parity
$ cd parity

# build in release mode
$ cargo build --release
```

This will produce an executable in the `./target/release` subdirectory.

## Start Parity
### Manually
To start Parity manually, just run
```bash
$ ./target/release/parity
```

and Parity will begin syncing the Ethereum blockchain.

### Using systemd service file
To start Parity as a regular user using systemd init:

1. Copy `parity/scripts/parity.service` to your
systemd user directory (usually `~/.config/systemd/user`).
2. To pass any argument to Parity, write a `~/.parity/parity.conf` file this way:
`ARGS="ARG1 ARG2 ARG3"`.

	Example: `ARGS="ui --geth --identity MyMachine"`.
