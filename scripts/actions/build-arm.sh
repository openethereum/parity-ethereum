#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
#strip ON
export RUSTFLAGS=" -Clink-arg=-s -Ctarget-feature=+aes"

echo "_____ Setup OpenEthereum cross-compiling tools _____"
time sudo apt-get install -qq gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf clang
time rustup target add armv7-unknown-linux-gnueabihf

echo "_____ Build OpenEthereum and tools _____"

time cargo build --verbose --color=always --release --features final --target=armv7-unknown-linux-gnueabihf
time cargo build --verbose --color=always --release -p evmbin --target=armv7-unknown-linux-gnueabihf
time cargo build --verbose --color=always --release -p ethstore-cli --target=armv7-unknown-linux-gnueabihf
time cargo build --verbose --color=always --release -p ethkey-cli --target=armv7-unknown-linux-gnueabihf

echo "_____ Post-processing binaries _____"
rm -rf artifacts/*
mkdir -p artifacts/

cp -v target/release/openethereum artifacts/openethereum
cp -v target/release/openethereum-evm artifacts/openethereum-evm
cp -v target/release/ethstore artifacts/ethstore
cp -v target/release/ethkey artifacts/ethkey
