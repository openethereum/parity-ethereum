![Parity Ethereum](docs/logo-parity-ethereum.svg)

<h2 align="center">The Fastest and most Advanced Ethereum Client.</h2>

<p align="center"><strong><a href="https://github.com/paritytech/parity-ethereum/releases/latest">» Download the latest release «</a></strong></p>

<p align="center"><a href="https://gitlab.parity.io/parity/parity-ethereum/commits/master" target="_blank"><img src="https://gitlab.parity.io/parity/parity-ethereum/badges/master/build.svg" /></a>
<a href="https://www.gnu.org/licenses/gpl-3.0.en.html" target="_blank"><img src="https://img.shields.io/badge/license-GPL%20v3-green.svg" /></a></p>

## Table of Contents

1. [Description](#chapter-001)
2. [Technical Overview](#chapter-002)
3. [Building](#chapter-003)<br>
  3.1 [Building Dependencies](#chapter-0031)<br>
  3.2 [Building from Source Code](#chapter-0032)<br>
  3.3 [Simple One-Line Installer for Mac and Linux](#chapter-0033)<br>
  3.4 [Starting Parity Ethereum](#chapter-0034)
4. [Testing](#chapter-004)
5. [Documentation](#chapter-005)
6. [Toolchain](#chapter-006)
7. [Community](#chapter-007)
8. [Contributing](#chapter-008)
9. [License](#chapter-009)


## 1. Description <a id="chapter-001"></a>

**Built for mission-critical use**: Miners, service providers, and exchanges need fast synchronisation and maximum uptime. Parity Ethereum provides the core infrastructure essential for speedy and reliable services.

- Clean, modular codebase for easy customisation
- Advanced CLI-based client
- Minimal memory and storage footprint
- Synchronise in hours, not days with Warp Sync
- Modular for light integration into your service or product

## 2. Technical Overview <a id="chapter-002"></a>

Parity Ethereum's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Parity Ethereum using the sophisticated and cutting-edge **Rust programming language**. Parity Ethereum is licensed under the GPLv3 and can be used for all your Ethereum needs.

By default, Parity Ethereum runs a JSON-RPC HTTP server on port `:8545` and a Web-Sockets server on port `:8546`. This is fully configurable and supports a number of APIs.

If you run into problems while using Parity Ethereum, check out the [wiki for documentation](https://wiki.parity.io/), feel free to [file an issue in this repository](https://github.com/paritytech/parity-ethereum/issues/new), or hop on our [Gitter](https://gitter.im/paritytech/parity) or [Riot](https://riot.im/app/#/group/+parity:matrix.parity.io) chat room to ask a question. We are glad to help! **For security-critical issues**, please refer to the security policy outlined in [SECURITY.md](SECURITY.md).

Parity Ethereum's current beta-release is 2.1. You can download it at [the releases page](https://github.com/paritytech/parity-ethereum/releases) or follow the instructions below to build from source. Please, mind the [CHANGELOG.md](CHANGELOG.md) for a list of all changes between different versions.

## 3. Building <a id="chapter-003"></a>

### 3.1 Build Dependencies <a id="chapter-0031"></a>

Parity Ethereum requires **latest stable Rust version** to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  Parity Ethereum also requires `gcc`, `g++`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

- OSX:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  `clang` is required. It comes with Xcode command line tools or can be installed with homebrew.

- Windows:
  Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the `rustup` installer from
  https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the `msvc` toolchain:
  ```bash
  $ rustup default stable-x86_64-pc-windows-msvc
  ```

Once you have `rustup` installed, then you need to install:
* [Perl](https://www.perl.org)
* [Yasm](https://yasm.tortall.net)

Make sure that these binaries are in your `PATH`. After that, you should be able to build Parity Ethereum from source.

### 3.2 Build from Source Code <a id="chapter-0032"></a>

```bash
# download Parity Ethereum code
$ git clone https://github.com/paritytech/parity-ethereum
$ cd parity-ethereum

# build in release mode
$ cargo build --release --features final
```

This produces an executable in the `./target/release` subdirectory.

Note: if cargo fails to parse manifest try:

```bash
$ ~/.cargo/bin/cargo build --release
```

Note, when compiling a crate and you receive errors, it's in most cases your outdated version of Rust, or some of your crates have to be recompiled. Cleaning the repository will most likely solve the issue if you are on the latest stable version of Rust, try:

```bash
$ cargo clean
```

This always compiles the latest nightly builds. If you want to build stable or beta, do a

```bash
$ git checkout stable
```

or

```bash
$ git checkout beta
```

### 3.3 Simple One-Line Installer for Mac and Linux <a id="chapter-0033"></a>

```bash
bash <(curl https://get.parity.io -L)
```

The one-line installer always defaults to the latest beta release. To install a stable release, run:

```bash
bash <(curl https://get.parity.io -L) -r stable
```

### 3.4 Starting Parity Ethereum <a id="chapter-0034"></a>

#### Manually

To start Parity Ethereum manually, just run

```bash
$ ./target/release/parity
```

so Parity Ethereum begins syncing the Ethereum blockchain.

#### Using `systemd` service file

To start Parity Ethereum as a regular user using `systemd` init:

1. Copy `./scripts/parity.service` to your
`systemd` user directory (usually `~/.config/systemd/user`).
2. Copy release to bin folder, write `sudo install ./target/release/parity /usr/bin/parity`
3. To configure Parity Ethereum, write a `/etc/parity/config.toml` config file, see [Configuring Parity Ethereum](https://paritytech.github.io/wiki/Configuring-Parity) for details.

## 4. Testing <a id="chapter-004"></a>

You can run tests with the following commands:

* **All** packages
  ```
  cargo test --all
  ```

* Specific package
  ```
  cargo test --package <spec>
  ```

Replace `<spec>` with one of the packages from the [package list](#package-list) (e.g. `cargo test --package evmbin`).

You can show your logs in the test output by passing `--nocapture` (i.e. `cargo test --package evmbin -- --nocapture`)

## 5. Documentation <a id="chapter-005"></a>

Official website: https://parity.io

Be sure to [check out our wiki](https://wiki.parity.io) for more information.

### Viewing documentation for Parity Ethereum packages

You can generate documentation for Parity Ethereum Rust packages that automatically opens in your web browser using [rustdoc with Cargo](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html#using-rustdoc-with-cargo) (of the The Rustdoc Book), by running the the following commands:

* **All** packages
  ```
  cargo doc --document-private-items --open
  ```

* Specific package
  ```
  cargo doc --package <spec> -- --document-private-items --open
  ```

Use`--document-private-items` to also view private documentation and `--no-deps` to exclude building documentation for dependencies.

Replacing `<spec>` with one of the following from the details section below (i.e. `cargo doc --package parity-ethereum --open`):

<a id="package-list"></a>
**Package List**
<details><p>

* Parity Ethereum (EthCore) Client Application
  ```bash
  parity-ethereum
  ```
* Parity Ethereum Account Management, Key Management Tool, and Keys Generator
  ```bash
  ethcore-accounts, ethkey-cli, ethstore, ethstore-cli
  ```
* Parity Chain Specification
  ```bash
  chainspec
  ```
* Parity CLI Signer Tool & RPC Client
  ```bash
  cli-signer parity-rpc-client
  ```
* Parity Ethereum Ethash & ProgPoW Implementations
  ```bash
  ethash
  ```
* Parity (EthCore) Library
  ```bash
  ethcore
  ```
  * Parity Ethereum Blockchain Database, Test Generator, Configuration,
Caching, Importing Blocks, and Block Information
    ```bash
    ethcore-blockchain
    ```
  * Parity Ethereum (EthCore) Contract Calls and Blockchain Service & Registry Information
    ```bash
    ethcore-call-contract
    ```
  * Parity Ethereum (EthCore) Database Access & Utilities, Database Cache Manager
    ```bash
    ethcore-db
    ```
  * Parity Ethereum Virtual Machine (EVM) Rust Implementation
    ```bash
    evm
    ```
  * Parity Ethereum (EthCore) Light Client Implementation
    ```bash
    ethcore-light
    ```
  * Parity Smart Contract based Node Filter, Manage Permissions of Network Connections
    ```bash
    node-filter
    ```
  * Parity Private Transactions
    ```bash
    ethcore-private-tx
    ```
  * Parity Ethereum (EthCore) Client & Network Service Creation & Registration with the I/O Subsystem
    ```bash
    ethcore-service
    ```
  * Parity Ethereum (EthCore) Blockchain Synchronization
    ```bash
    ethcore-sync
    ```
  * Parity Ethereum Common Types
    ```bash
    common-types
    ```
  * Parity Ethereum Virtual Machines (VM) Support Library
    ```bash
    vm
    ```
  * Parity Ethereum WASM Interpreter
    ```bash
    wasm
    ```
  * Parity Ethereum WASM Test Runner
    ```bash
    pwasm-run-test
    ```
  * Parity EVM Implementation
    ```bash
    evmbin
    ```
  * Parity Ethereum IPFS-compatible API
    ```bash
    parity-ipfs-api
    ```
  * Parity Ethereum JSON Deserialization
    ```bash
    ethjson
    ```
  * Parity Ethereum State Machine Generalization for Consensus Engines
    ```bash
    parity-machine
    ```
* Parity Ethereum (EthCore) Miner Interface
  ```bash
  ethcore-miner parity-local-store price-info ethcore-stratum using_queue
  ```
* Parity Ethereum (EthCore) Logger Implementation
  ```bash
  ethcore-logger
  ```
* C bindings library for the Parity Ethereum client
  ```bash
  parity-clib
  ```
* Parity Ethereum JSON-RPC Servers
  ```bash
  parity-rpc
  ```
* Parity Ethereum (EthCore) Secret Store
  ```bash
  ethcore-secretstore
  ```
* Parity Updater Service
  ```bash
  parity-updater parity-hash-fetch
  ```
* Parity Core Libraries (Parity Util)
  ```bash
  ethcore-bloom-journal blooms-db dir eip-712 fake-fetch fastmap fetch ethcore-io
  journaldb keccak-hasher len-caching-lock macros memory-cache memzero
  migration-rocksdb ethcore-network ethcore-network-devp2p panic_hook
  patricia-trie-ethereum registrar rlp_compress rlp_derive parity-runtime stats
  time-utils triehash-ethereum unexpected parity-version
  ```

</p></details>

### Contributing to documentation for Parity Ethereum packages

[Document source code](https://doc.rust-lang.org/1.9.0/book/documentation.html) for Parity Ethereum packages by annotating the source code with documentation comments.

Example (generic documentation comment):
```markdown
/// Summary
///
/// Description
///
/// # Panics
///
/// # Errors
///
/// # Safety
///
/// # Examples
///
/// Summary of Example 1
///
/// ```rust
/// // insert example 1 code here for use with documentation as tests
/// ```
///
```

## 6. Toolchain <a id="chapter-006"></a>

In addition to the Parity Ethereum client, there are additional tools in this repository available:

- [evmbin](./evmbin) - Parity Ethereum EVM Implementation.
- [ethstore](./accounts/ethstore) - Parity Ethereum Key Management.
- [ethkey](./accounts/ethkey) - Parity Ethereum Keys Generator.

The following tool is available in a separate repository:
- [ethabi](https://github.com/paritytech/ethabi) - Parity Ethereum Encoding of Function Calls. [Docs here](https://crates.io/crates/ethabi)
- [whisper](https://github.com/paritytech/whisper) - Parity Ethereum Whisper-v2 PoC Implementation.

## 7. Community <a id="chapter-007"></a>

### Join the chat!

Questions? Get in touch with us on Gitter:
[![Gitter: Parity](https://img.shields.io/badge/gitter-parity-4AB495.svg)](https://gitter.im/paritytech/parity)
[![Gitter: Parity.js](https://img.shields.io/badge/gitter-parity.js-4AB495.svg)](https://gitter.im/paritytech/parity.js)
[![Gitter: Parity/Miners](https://img.shields.io/badge/gitter-parity/miners-4AB495.svg)](https://gitter.im/paritytech/parity/miners)
[![Gitter: Parity-PoA](https://img.shields.io/badge/gitter-parity--poa-4AB495.svg)](https://gitter.im/paritytech/parity-poa)

Alternatively, join our community on Matrix:
[![Riot: +Parity](https://img.shields.io/badge/riot-%2Bparity%3Amatrix.parity.io-orange.svg)](https://riot.im/app/#/group/+parity:matrix.parity.io)

## 8. Contributing <a id="chapter-008"></a>

An introduction has been provided in the ["So You Want to be a Core Developer" presentation slides by Hernando Castano](http://tiny.cc/contrib-to-parity-eth). Additional guidelines are provided in [CONTRIBUTING](./.github/CONTRIBUTING.md).

### Contributor Code of Conduct

[CODE_OF_CONDUCT](./.github/CODE_OF_CONDUCT.md)

## 9. License <a id="chapter-009"></a>

[LICENSE](./LICENSE)
