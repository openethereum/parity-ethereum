#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

echo "_____ Building _____"
time cargo build --locked --target $CARGO_TARGET --release --features final
time cargo build --locked --target $CARGO_TARGET --release -p evmbin
time cargo build --locked --target $CARGO_TARGET --release -p ethstore-cli
time cargo build --locked --target $CARGO_TARGET --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
mkdir -p artifacts
cd artifacts
cp --verbose ../target/$CARGO_TARGET/release/{parity,parity-evm,ethstore,ethkey} .
strip --verbose --enable-deterministic-archives ./*

echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --md5 $binary -o $binary.md5
  rhash --sha256 $binary -o $binary.sha256
done
