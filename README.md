## Parity-Ethereum - a fast, light, and robust EVM and WASM blockchain client

### [» Download the latest release «](https://github.com/paritytech/parity-ethereum/releases/latest)

[![build status](https://gitlab.parity.io/parity/parity/badges/master/build.svg)](https://gitlab.parity.io/parity/parity/commits/master)
[![codecov](https://codecov.io/gh/paritytech/parity/branch/master/graph/badge.svg)](https://codecov.io/gh/paritytech/parity)
[![Snap Status](https://build.snapcraft.io/badge/paritytech/parity.svg)](https://build.snapcraft.io/user/paritytech/parity)
[![GPLv3](https://img.shields.io/badge/license-GPL%20v3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)


### Join the chat!

Get in touch with us on Gitter:
[![Gitter: Parity](https://img.shields.io/badge/gitter-parity-4AB495.svg)](https://gitter.im/paritytech/parity)
[![Gitter: Parity.js](https://img.shields.io/badge/gitter-parity.js-4AB495.svg)](https://gitter.im/paritytech/parity.js)
[![Gitter: Parity/Miners](https://img.shields.io/badge/gitter-parity/miners-4AB495.svg)](https://gitter.im/paritytech/parity/miners)
[![Gitter: Parity-PoA](https://img.shields.io/badge/gitter-parity--poa-4AB495.svg)](https://gitter.im/paritytech/parity-poa)

Or join our community on Matrix:
[![Riot: +Parity](https://img.shields.io/badge/riot-%2Bparity%3Amatrix.parity.io-orange.svg)](https://riot.im/app/#/group/+parity:matrix.parity.io)

Official website: https://parity.io | Be sure to check out [our wiki](https://wiki.parity.io) for more information.

----

## About Parity-Ethereum

Parity-Ethereum's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Parity-Ethereum using the sophisticated and cutting-edge Rust programming language. Parity-Ethereum is licensed under the GPLv3, and can be used for all your Ethereum needs.

By default, Parity-Ethereum will run a JSON-RPC HTTP server on `127.0.0.1:8545` and a Web-Sockets server on `127.0.0.1:8546`. This is fully configurable and supports a number of APIs.

If you run into problems while using Parity-Ethereum, feel free to file an issue in this repository or hop on our [Gitter](https://gitter.im/paritytech/parity) or [Riot](https://riot.im/app/#/group/+parity:matrix.parity.io) chat room to ask a question. We are glad to help! **For security-critical issues**, please refer to the security policy outlined in [SECURITY.md](SECURITY.md).

Parity-Ethereum's current beta-release is 2.0. You can download it at [the releases page](https://github.com/paritytech/parity-ethereum/releases) or follow the instructions below to build from source. Please, mind the [CHANGELOG.md](CHANGELOG.md) for a list of all changes between different versions.

----

## Build dependencies

**Parity-Ethereum requires Rust version 1.27.0 to build**

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have rustup, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  Parity-Ethereum also requires `gcc`, `g++`, `libudev-dev`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

- OSX:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  `clang` is required. It comes with Xcode command line tools or can be installed with homebrew.

- Windows
  Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the rustup installer from
  https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the msvc toolchain:
  ```bash
  $ rustup default stable-x86_64-pc-windows-msvc
  ```

Once you have rustup installed, then you need to install:
* [Perl](https://www.perl.org)
* [Yasm](http://yasm.tortall.net)

Make sure that these binaries are in your `PATH`. After that you should be able to build Parity-Ethereum from source.

----

## Install from the snap store

In any of the [supported Linux distros](https://snapcraft.io/docs/core/install):

```bash
sudo snap install parity
```

Or, if you want to contribute testing the upcoming release:

```bash
sudo snap install parity --beta
```

And to test the latest code landed into the master branch:

```bash
sudo snap install parity --edge
```

----

## Build from source

```bash
# download Parity-Ethereum code
$ git clone https://github.com/paritytech/parity-ethereum
$ cd parity

# build in release mode
$ cargo build --release --features final
```

This will produce an executable in the `./target/release` subdirectory.

Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```

Note, when compiling a crate and you receive errors, it's in most cases your outdated version of Rust, or some of your crates have to be recompiled. Cleaning the repository will most likely solve the issue if you are on the latest stable version of Rust, try:

```bash
$ cargo clean
```

This will always compile the latest nightly builds. If you want to build stable or beta, do a

```bash
$ git checkout stable
```

or

```bash
$ git checkout beta
```

first.

----

## Simple one-line installer for Mac and Ubuntu

```bash
bash <(curl https://get.parity.io -L)
```

The one-line installer always defaults to the latest beta release. To install a stable release, run:

```bash
bash <(curl https://get.parity.io -L) -r stable
```

## Start Parity-Ethereum

### Manually

To start Parity-Ethereum manually, just run

```bash
$ ./target/release/parity
```

and Parity-Ethereum will begin syncing the Ethereum blockchain.

### Using systemd service file

To start Parity-Ethereum as a regular user using systemd init:

1. Copy `./scripts/parity.service` to your
systemd user directory (usually `~/.config/systemd/user`).
2. To configure Parity-Ethereum, write a `/etc/parity/config.toml` config file, see [Configuring Parity-Ethereum](https://paritytech.github.io/wiki/Configuring-Parity) for details.
