#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

set INCLUDE="C:\Program Files (x86)\Microsoft SDKs\Windows\v7.1A\Include;C:\vs2015\VC\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.10240.0\ucrt"
set LIB="C:\vs2015\VC\lib;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.10240.0\ucrt\x64"

rustup default stable-x86_64-pc-windows-msvc

echo "_____ Building _____"
time cargo build --locked --target $CARGO_TARGET --release --features final
time cargo build --locked --target $CARGO_TARGET --release -p evmbin
time cargo build --locked --target $CARGO_TARGET --release -p ethstore-cli
time cargo build --locked --target $CARGO_TARGET --release -p ethkey-cli

echo "_____ Post-processing binaries _____"
mkdir -p artifacts
cd artifacts
cp --verbose ../target/$CARGO_TARGET/release/{parity,parity-evm,ethstore,ethkey}.exe .

echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --md5 $binary -o $binary.unsigned.md5
  rhash --sha256 $binary -o $binary.unsigned.sha256
done
