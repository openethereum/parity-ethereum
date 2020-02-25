#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "__________Show ENVIROMENT__________"
echo "CC:               " $CC
echo "CXX:              " $CXX
#strip ON
export RUSTFLAGS+=" -C link-arg=-s -C target-feature=+aes,+sse2,+ssse3"
fi
time cargo build --target $CARGO_TARGET --verbose --color=always --release --features final
time cargo build --target $CARGO_TARGET --verbose --color=always --release -p evmbin
time cargo build --target $CARGO_TARGET --verbose --color=always --release -p ethstore-cli
time cargo build --target $CARGO_TARGET --verbose --color=always --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf artifacts/*
mkdir -p artifacts/$CARGO_TARGET
cd artifacts/$CARGO_TARGET

cp -v ../../target/$CARGO_TARGET/release/parity ./parity
cp -v ../../target/$CARGO_TARGET/release/parity-evm ./parity-evm
cp -v ../../target/$CARGO_TARGET/release/ethstore ./ethstore
cp -v ../../target/$CARGO_TARGET/release/ethkey ./ethkey

echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256 #do we still need this hash (SHA2)?
done
cd ..
zip -r artifacts.zip artifacts/
