#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

#strip ON
export RUSTFLAGS=" -Clink-arg=-s -Ctarget-feature=+aes,+sse2,+ssse3"

echo "_____ Build OpenEthereum and tools _____"

time cargo build --verbose --color=always --release --features final
time cargo build --verbose --color=always --release -p evmbin
time cargo build --verbose --color=always --release -p ethstore-cli
time cargo build --verbose --color=always --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf artifacts/*
mkdir -p artifacts/

cp -v target/release/parity artifacts/parity
cp -v target/release/parity-evm artifacts/parity-evm
cp -v target/release/ethstore artifacts/ethstore
cp -v target/release/ethkey artifacts/ethkey

#echo "_____ Calculating checksums _____"
#for binary in $(ls)
#do
#  rhash --sha256 $binary -o $binary.sha256 #do we still need this hash (SHA2)?
#done

echo "_____ Zip artifacts _____"

zip -r ${{matrix.platform}}.artifacts.zip artifacts/
