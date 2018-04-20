# Parity - fast, light, and robust Ethereum client

## [» Download the latest release «](https://github.com/paritytech/parity/releases/latest)

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

Official website: https://parity.io

Be sure to check out [our wiki](https://wiki.parity.io) for more information.

----

## About Parity

Parity's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Parity using the sophisticated and cutting-edge Rust programming language. Parity is licensed under the GPLv3, and can be used for all your Ethereum needs.

Parity comes with a built-in wallet, to install it please follow [these instructions](https://wiki.parity.io/Parity-Wallet). It includes various functionality allowing you to:

- create and manage your Ethereum accounts;
- manage your Ether and any Ethereum tokens;
- create and register your own tokens;
- and much more.

From Parity Ethereum client version >=1.10, the User Interface (UI) is accessible in a separate application called Parity UI. To keep using the UI in the browser (deprecated), [follow these steps](https://wiki.parity.io/FAQ-Basic-Operations,-Configuration,-and-Synchronization.md#the-parity-ui-application-isnt-working-the-way-i-want).

By default, Parity will also run a JSONRPC server on `127.0.0.1:8545` and a websockets server on `127.0.0.1:8546`. This is fully configurable and supports a number of APIs.

If you run into an issue while using Parity, feel free to file one in this repository or hop on our [Gitter](https://gitter.im/paritytech/parity) or [Riot](https://riot.im/app/#/group/+parity:matrix.parity.io) chat room to ask a question. We are glad to help!

**For security-critical issues**, please refer to the security policy outlined in [SECURITY.MD](SECURITY.md).

Parity's current release is 1.9. You can download it at https://github.com/paritytech/parity/releases or follow the instructions below to build from source.

----

## Build dependencies

**Parity requires Rust version 1.23.0 to build**

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
  ```bash
	$ rustup default stable-x86_64-pc-windows-msvc
  ```

Once you have rustup installed, then you need to install:
* [Perl](https://www.perl.org)
* [Yasm](http://yasm.tortall.net)

Make sure that these binaries are in your `PATH`. After that you should be able to build parity from source.

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
# download Parity code
$ git clone https://github.com/paritytech/parity
$ cd parity

# build in release mode
$ cargo build --release
```

This will produce an executable in the `./target/release` subdirectory.

Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```

Note: When compiling a crate and you receive the following error:

```
error: the crate is compiled with the panic strategy `abort` which is incompatible with this crate's strategy of `unwind`
```

Cleaning the repository will most likely solve the issue, try:

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
bash <(curl https://get.parity.io -Lk)
```

The one-line installer always defaults to the latest beta release. To install a stable release, run:

```bash
bash <(curl https://get.parity.io -Lk) -r stable
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

1. Copy `./scripts/parity.service` to your
systemd user directory (usually `~/.config/systemd/user`).
2. To configure Parity, write a `/etc/parity/config.toml` config file, see [Configuring Parity](https://paritytech.github.io/wiki/Configuring-Parity) for details.
