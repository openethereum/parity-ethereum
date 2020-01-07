#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

export CC="sccache "$CC
export CXX="sccache "$CXX
echo "__________Show ENVIROMENT__________"
echo "CARGO_TARGET:     " $CARGO_TARGET
echo "CC:               " $CC
echo "CXX:              " $CXX
#strip ON
export RUSTFLAGS+=" -C link-arg=-s"

echo "_____ Building target: "$CARGO_TARGET" _____"
if [ "${CARGO_TARGET}" = "x86_64-unknown-linux-gnu" ] || [ "${CARGO_TARGET}" = "x86_64-apple-darwin" ]
then
  # NOTE: Enables the aes-ni instructions for RustCrypto dependency.
  # If you change this please remember to also update .cargo/config
  export RUSTFLAGS+=" -C target-feature=+aes,+sse2,+ssse3"
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
  if [[ $CARGO_TARGET == *"x86_64"* ]];
  then
      ./parity tools hash $binary > $binary.sha3
  else
      echo ">[WARN] ${binary} cannot be hashed with cross-compiled binary (keccak256)"
  fi
done
#show sccache statistics
sccache --show-stats
