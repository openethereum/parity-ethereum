# [Parity](https://ethcore.io/parity.html)
### Fast, light, and robust Ethereum implementation

[![build status](https://gitlab.ethcore.io/parity/parity/badges/master/build.svg)](https://gitlab.ethcore.io/parity/parity/commits/master) [![Coverage Status][coveralls-image]][coveralls-url]  [![GPLv3][license-image]][license-url]

### Join the chat!

Parity [![Join the chat at https://gitter.im/ethcore/parity][gitter-image]][gitter-url] and
parity.js [![Join the chat at https://gitter.im/ethcore/parity.js](https://badges.gitter.im/ethcore/parity.js.svg)](https://gitter.im/ethcore/parity.js?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[Internal Documentation][doc-url]


Be sure to check out [our wiki][wiki-url] for more information.

[coveralls-image]: https://coveralls.io/repos/github/ethcore/parity/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/ethcore/parity?branch=master
[gitter-image]: https://badges.gitter.im/Join%20Chat.svg
[gitter-url]: https://gitter.im/ethcore/parity?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
[license-image]: https://img.shields.io/badge/license-GPL%20v3-green.svg
[license-url]: https://www.gnu.org/licenses/gpl-3.0.en.html
[doc-url]: https://ethcore.github.io/parity/ethcore/index.html
[wiki-url]: https://github.com/ethcore/parity/wiki

**Parity requires Rust version 1.15.0 to build**

----


## About Parity

Parity's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Parity using the sophisticated and
cutting-edge Rust programming language. Parity is licensed under the GPLv3, and can be used for all your Ethereum needs.

Parity comes with a built-in wallet. To access [Parity Wallet](http://127.0.0.1:8080/) simply go to http://127.0.0.1:8080/. It
includes various functionality allowing you to:
- create and manage your Ethereum accounts;
- manage your Ether and any Ethereum tokens;
- create and register your own tokens;
- and much more.    

By default, Parity will also run a JSONRPC server on `127.0.0.1:8545`. This is fully configurable and supports a number
of RPC APIs.

If you run into an issue while using parity, feel free to file one in this repository
or hop on our [gitter chat room][gitter-url] to ask a question. We are glad to help!

Parity's current release is 1.5. You can download it at https://ethcore.io/parity.html or follow the instructions
below to build from source.

----

## Build dependencies

Parity is fully compatible with Stable Rust.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have rustup, you can install it like this:

- Linux:
	```bash
	$ curl https://sh.rustup.rs -sSf | sh
	```
	
	Parity also requires `gcc`, `g++`, `libssl-dev`/`openssl`, `libudev-dev` and `pkg-config` packages to be installed.
- OSX:
	```bash
	$ curl https://sh.rustup.rs -sSf | sh
	```

	`clang` is required. It comes with Xcode command line tools or can be installed with homebrew.
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
Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```
----

## Simple one-line installer for Mac and Ubuntu

```bash
bash <(curl https://get.parity.io -Lk)
```

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

	Example: `ARGS="ui --identity MyMachine"`.
