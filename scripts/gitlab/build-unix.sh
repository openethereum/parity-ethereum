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
time cargo build --target $CARGO_TARGET --release -p whisper-cli

echo "_____ Post-processing binaries _____"
rm -rf artifacts
mkdir -p artifacts
cd artifacts
mkdir -p $CARGO_TARGET
cd $CARGO_TARGET
cp ../../target/$CARGO_TARGET/release/parity ./parity
cp ../../target/$CARGO_TARGET/release/parity-evm ./parity-evm
cp ../../target/$CARGO_TARGET/release/ethstore ./ethstore
cp ../../target/$CARGO_TARGET/release/ethkey ./ethkey
cp ../../target/$CARGO_TARGET/release/whisper ./whisper
strip -v ./*
echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256
  ./parity tools hash $binary > $binary.sha3
done
