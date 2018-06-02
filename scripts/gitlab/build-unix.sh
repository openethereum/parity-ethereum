#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
echo "__________Show ENVIROMENT__________"
echo "CC:       " $CC
echo "CXX:      " $CXX

echo "__________CARGO CONFIG__________"
rm -rf .cargo
mkdir -p .cargo
echo "[target.$CARGO_TARGET]" >> .cargo/config
echo "linker= \"$CC\"" >> .cargo/config
cat .cargo/config

echo "_____ Building target: "$CARGO_TARGET" _____"
time cargo build --target $CARGO_TARGET --release --features final
time cargo build --target $CARGO_TARGET --release -p evmbin
time cargo build --target $CARGO_TARGET --release -p ethstore-cli
time cargo build --target $CARGO_TARGET --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf artifacts
mkdir -p artifacts
cd artifacts
cp ../target/$CARGO_TARGET/release/{parity,parity-evm,ethstore,ethkey} .
strip -v ./*
echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256
done
