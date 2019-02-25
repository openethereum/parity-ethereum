#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "__________Show ENVIROMENT__________"
echo "CI_SERVER_NAME:   " $CI_SERVER_NAME
echo "CARGO_HOME:       " $CARGO_HOME
echo "CARGO_TARGET:     " $CARGO_TARGET
echo "CC:               " $CC
echo "CXX:              " $CXX
#strip ON
export RUSTFLAGS=" -C link-arg=-s"
# Linker for crosscomile
echo "_____ Linker _____"
cat .cargo/config

echo "_____ Building target: "$CARGO_TARGET" _____"
if [ "${CARGO_TARGET}" = "armv7-linux-androideabi" ]
then
  time cargo build --target $CARGO_TARGET --release -p parity-clib --features final
else
  time cargo build --target $CARGO_TARGET --release --features final
  time cargo build --target $CARGO_TARGET --release -p evmbin
  time cargo build --target $CARGO_TARGET --release -p ethstore-cli
  time cargo build --target $CARGO_TARGET --release -p ethkey-cli
  time cargo build --target $CARGO_TARGET --release -p whisper-cli
fi

echo "_____ Post-processing binaries _____"
rm -rf artifacts/*
mkdir -p artifacts/$CARGO_TARGET
cd artifacts/$CARGO_TARGET

if [ "${CARGO_TARGET}" = "armv7-linux-androideabi" ]
then
 cp -v ../../target/$CARGO_TARGET/release/libparity.so ./libparity.so
else
 cp -v ../../target/$CARGO_TARGET/release/parity ./parity
 cp -v ../../target/$CARGO_TARGET/release/parity-evm ./parity-evm
 cp -v ../../target/$CARGO_TARGET/release/ethstore ./ethstore
 cp -v ../../target/$CARGO_TARGET/release/ethkey ./ethkey
 cp -v ../../target/$CARGO_TARGET/release/whisper ./whisper
fi

echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256 #do we still need this hash (SHA2)?
  rhash --sha3-256 $binary -o $binary.sha3
done
