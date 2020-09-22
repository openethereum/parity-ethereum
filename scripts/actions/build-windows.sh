#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error
# NOTE: Enables the aes-ni instructions for RustCrypto dependency.
# If you change this please remember to also update .cargo/config
export RUSTFLAGS=" -Ctarget-feature=+aes,+sse2,+ssse3 -Ctarget-feature=+crt-static  -Clink-arg=-s"

echo "_____ Build Parity and tools _____"
time cargo build --verbose --release --features final
time cargo build --verbose --release -p evmbin
time cargo build --verbose --release -p ethstore-cli
time cargo build --verbose --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf artifacts
mkdir -p artifacts

cp --verbose target/release/openethereum.exe artifacts/openethereum.exe
cp --verbose target/release/openethereum-evm.exe artifacts/openethereum-evm.exe
cp --verbose target/release/ethstore.exe artifacts/ethstore.exe
cp --verbose target/release/ethkey.exe artifacts/ethkey.exe
