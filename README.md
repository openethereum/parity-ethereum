![Open Ethereum](docs/logo-parity-ethereum.svg)

<h2 align="center">The Fastest and most Advanced Ethereum Client.</h2>

<p align="center"><strong><a href="https://github.com/OpenEthereum/open-ethereum/releases/latest">» Download the latest release «</a></strong></p>

<p align="center"><a href="https://github.com/OpenEthereum/open-ethereum/actions" target="_blank"><img src="https://github.com/OpenEthereum/open-ethereum/workflows/Build%20and%20Test%20Suite/badge.svg" /></a>
<a href="https://www.gnu.org/licenses/gpl-3.0.en.html" target="_blank"><img src="https://img.shields.io/badge/license-GPL%20v3-green.svg" /></a></p>

## Table of Contents

1. [Description](#chapter-001)
2. [Technical Overview](#chapter-002)
3. [Building](#chapter-003)<br>
  3.1 [Building Dependencies](#chapter-0031)<br>
  3.2 [Building from Source Code](#chapter-0032)<br>
  3.3 [Starting Open Ethereum](#chapter-0034)
4. [Testing](#chapter-004)
5. [Documentation](#chapter-005)
6. [Toolchain](#chapter-006)
7. [Community](#chapter-007)
8. [Contributing](#chapter-008)
9. [License](#chapter-009)


## 1. Description <a id="chapter-001"></a>

**Built for mission-critical use**: Miners, service providers, and exchanges need fast synchronisation and maximum uptime. Open Ethereum provides the core infrastructure essential for speedy and reliable services.

- Clean, modular codebase for easy customisation
- Advanced CLI-based client
- Minimal memory and storage footprint
- Synchronise in hours, not days with Warp Sync
- Modular for light integration into your service or product

## 2. Technical Overview <a id="chapter-002"></a>

Open Ethereum's goal is to be the fastest, lightest, and most secure Ethereum client. We are developing Open Ethereum using the sophisticated and cutting-edge **Rust programming language**. Open Ethereum is licensed under the GPLv3 and can be used for all your Ethereum needs.

By default, Open Ethereum runs a JSON-RPC HTTP server on port `:8545` and a Web-Sockets server on port `:8546`. This is fully configurable and supports a number of APIs.

If you run into problems while using Open Ethereum, check out the [old wiki for documentation](https://wiki.parity.io/), feel free to [file an issue in this repository](https://github.com/OpenEthereum/open-ethereum/issues/new), or hop on our [Discord](https://discord.gg/FJUgGe) chat room to ask a question. We are glad to help! **For security-critical issues**, please refer to the security policy outlined in [SECURITY.md](SECURITY.md).

You can download Open Ethereum's latest release at [the releases page](https://github.com/OpenEthereum/open-ethereum/releases) or follow the instructions below to build from source. Please, mind the [CHANGELOG.md](CHANGELOG.md) for a list of all changes between different versions.

## 3. Building <a id="chapter-003"></a>

### 3.1 Build Dependencies <a id="chapter-0031"></a>

Open Ethereum requires **latest stable Rust version** to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

  Open Ethereum also requires `clang` (>= 9.0), `clang++`, `pkg-config`, `file`, `make`, and `cmake` packages to be installed.

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

Make sure that these binaries are in your `PATH`. After that, you should be able to build Open Ethereum from source.

### 3.2 Build from Source Code <a id="chapter-0032"></a>

```bash
# download Open Ethereum code
$ git clone https://github.com/OpenEthereum/open-ethereum
$ cd open-ethereum

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

This always compiles the latest nightly builds. If you want to build stable, do a

```bash
$ git checkout stable
```

### 3.3 Starting Open Ethereum <a id="chapter-0034"></a>

#### Manually

To start Open Ethereum manually, just run

```bash
$ ./target/release/parity
```

so Open Ethereum begins syncing the Ethereum blockchain.

#### Using `systemd` service file

To start Open Ethereum as a regular user using `systemd` init:

1. Copy `./scripts/parity.service` to your
`systemd` user directory (usually `~/.config/systemd/user`).
2. Copy release to bin folder, write `sudo install ./target/release/parity /usr/bin/parity`
3. To configure Open Ethereum, write a `/etc/parity/config.toml` config file, see [our old wiki](https://paritytech.github.io/wiki/Configuring-Parity) for details.

## 4. Testing <a id="chapter-004"></a>

Download the required test files: `git submodule update --init --recursive`. You can run tests with the following commands:

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

Be sure to [check out our old wiki](https://wiki.parity.io) for more information.

### Viewing documentation for Open Ethereum packages

You can generate documentation for Open Ethereum Rust packages that automatically opens in your web browser using [rustdoc with Cargo](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html#using-rustdoc-with-cargo) (of the The Rustdoc Book), by running the the following commands:

* **All** packages
  ```
  cargo doc --document-private-items --open
  ```

* Specific package
  ```
  cargo doc --package <spec> -- --document-private-items --open
  ```

Use`--document-private-items` to also view private documentation and `--no-deps` to exclude building documentation for dependencies.

Replacing `<spec>` with one of the following from the details section below (i.e. `cargo doc --package open-ethereum --open`):

<a id="package-list"></a>
**Package List**
<details><p>

* Open Ethereum (EthCore) Client Application
  ```bash
  open-ethereum
  ```
* Open Ethereum Account Management, Key Management Tool, and Keys Generator
  ```bash
  ethcore-accounts, ethkey-cli, ethstore, ethstore-cli
  ```
* Open Ethereum Chain Specification
  ```bash
  chainspec
  ```
* Open Ethereum CLI Signer Tool & RPC Client
  ```bash
  cli-signer parity-rpc-client
  ```
* Open Ethereum Ethash & ProgPoW Implementations
  ```bash
  ethash
  ```
* EthCore Library
  ```bash
  ethcore
  ```
  * Open Ethereum Blockchain Database, Test Generator, Configuration,
Caching, Importing Blocks, and Block Information
    ```bash
    ethcore-blockchain
    ```
  * Open Ethereum (EthCore) Contract Calls and Blockchain Service & Registry Information
    ```bash
    ethcore-call-contract
    ```
  * Open Ethereum (EthCore) Database Access & Utilities, Database Cache Manager
    ```bash
    ethcore-db
    ```
  * Open Ethereum Virtual Machine (EVM) Rust Implementation
    ```bash
    evm
    ```
  * Open Ethereum (EthCore) Light Client Implementation
    ```bash
    ethcore-light
    ```
  * Smart Contract based Node Filter, Manage Permissions of Network Connections
    ```bash
    node-filter
    ```
  * Private Transactions
    ```bash
    ethcore-private-tx
    ```
  * Open Ethereum (EthCore) Client & Network Service Creation & Registration with the I/O Subsystem
    ```bash
    ethcore-service
    ```
  * Open Ethereum (EthCore) Blockchain Synchronization
    ```bash
    ethcore-sync
    ```
  * Open Ethereum Common Types
    ```bash
    common-types
    ```
  * Open Ethereum Virtual Machines (VM) Support Library
    ```bash
    vm
    ```
  * Open Ethereum WASM Interpreter
    ```bash
    wasm
    ```
  * Open Ethereum WASM Test Runner
    ```bash
    pwasm-run-test
    ```
  * Open Ethereum EVM Implementation
    ```bash
    evmbin
    ```
  * Open Ethereum JSON Deserialization
    ```bash
    ethjson
    ```
  * Open Ethereum State Machine Generalization for Consensus Engines
    ```bash
    parity-machine
    ```
* Open Ethereum (EthCore) Miner Interface
  ```bash
  ethcore-miner parity-local-store price-info ethcore-stratum using_queue
  ```
* Open Ethereum (EthCore) Logger Implementation
  ```bash
  ethcore-logger
  ```
* Open Ethereum JSON-RPC Servers
  ```bash
  parity-rpc
  ```
* Open Ethereum Updater Service
  ```bash
  parity-updater parity-hash-fetch
  ```
* Open Ethereum Core Libraries (`util`)
  ```bash
  ethcore-bloom-journal blooms-db dir eip-712 fake-fetch fastmap fetch ethcore-io
  journaldb keccak-hasher len-caching-lock memory-cache memzero
  migration-rocksdb ethcore-network ethcore-network-devp2p panic_hook
  patricia-trie-ethereum registrar rlp_compress stats
  time-utils triehash-ethereum unexpected parity-version
  ```

</p></details>

### Contributing to documentation for Open Ethereum packages

[Document source code](https://doc.rust-lang.org/1.9.0/book/documentation.html) for Open Ethereum packages by annotating the source code with documentation comments.

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

In addition to the Open Ethereum client, there are additional tools in this repository available:

- [evmbin](./evmbin) - Open Ethereum EVM Implementation.
- [ethstore](./accounts/ethstore) - Open Ethereum Key Management.
- [ethkey](./accounts/ethkey) - Open Ethereum Keys Generator.

The following tool is available in a separate repository:
- [ethabi](https://github.com/OpenEthereum/ethabi) - Open Ethereum Encoding of Function Calls. [Docs here](https://crates.io/crates/ethabi)
- [whisper](https://github.com/OpenEthereum/whisper) - Open Ethereum Whisper-v2 PoC Implementation.

## 7. Community <a id="chapter-007"></a>

### Join the chat!

Questions? Get in touch with us on Discord:
[![Discord: Open Ethereum](https://img.shields.io/discord/669192218728202270.svg?logo=discord)](https://discord.gg/FJUgGe)

## 8. Contributing <a id="chapter-008"></a>

An introduction has been provided in the ["So You Want to be a Core Developer" presentation slides by Hernando Castano](http://tiny.cc/contrib-to-parity-eth). Additional guidelines are provided in [CONTRIBUTING](./.github/CONTRIBUTING.md).

### Contributor Code of Conduct

[CODE_OF_CONDUCT](./.github/CODE_OF_CONDUCT.md)

## 9. License <a id="chapter-009"></a>

[LICENSE](./LICENSE)
