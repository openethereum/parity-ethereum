#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

set INCLUDE="C:\Program Files (x86)\Microsoft SDKs\Windows\v7.1A\Include;C:\vs2015\VC\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.10240.0\ucrt"
set LIB="C:\vs2015\VC\lib;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.10240.0\ucrt\x64"
sccache -s

echo "__________Show ENVIROMENT__________"
echo "CI_SERVER_NAME:   " $CI_SERVER_NAME
echo "CARGO_HOME:       " $CARGO_HOME
echo "CARGO_TARGET:     " $CARGO_TARGET
echo "RUSTC_WRAPPER:    " $RUSTC_WRAPPER
echo "SCCACHE_DIR:      " $SCCACHE_DIR

echo "_____ Building target: "$CARGO_TARGET" _____"
  # NOTE: Enables the aes-ni instructions for RustCrypto dependency.
  # If you change this please remember to also update .cargo/config
export RUSTFLAGS=" -Ctarget-feature=+aes,+sse2,+ssse3 -Ctarget-feature=+crt-static"

time cargo build --target $CARGO_TARGET --verbose --release --features final
time cargo build --target $CARGO_TARGET --verbose --release -p evmbin
time cargo build --target $CARGO_TARGET --verbose --release -p ethstore-cli
time cargo build --target $CARGO_TARGET --verbose --release -p ethkey-cli
time cargo build --target $CARGO_TARGET --verbose --release -p whisper-cli

echo "__________Sign binaries__________"
scripts/gitlab/sign-win.cmd $keyfile $certpass target/$CARGO_TARGET/release/parity.exe
scripts/gitlab/sign-win.cmd $keyfile $certpass target/$CARGO_TARGET/release/parity-evm.exe
scripts/gitlab/sign-win.cmd $keyfile $certpass target/$CARGO_TARGET/release/ethstore.exe
scripts/gitlab/sign-win.cmd $keyfile $certpass target/$CARGO_TARGET/release/ethkey.exe
scripts/gitlab/sign-win.cmd $keyfile $certpass target/$CARGO_TARGET/release/whisper.exe

echo "_____ Post-processing binaries _____"
rm -rf artifacts
mkdir -p artifacts
cd artifacts
mkdir -p $CARGO_TARGET
cd $CARGO_TARGET
cp --verbose ../../target/$CARGO_TARGET/release/parity.exe ./parity.exe
cp --verbose ../../target/$CARGO_TARGET/release/parity-evm.exe ./parity-evm.exe
cp --verbose ../../target/$CARGO_TARGET/release/ethstore.exe ./ethstore.exe
cp --verbose ../../target/$CARGO_TARGET/release/ethkey.exe ./ethkey.exe
cp --verbose ../../target/$CARGO_TARGET/release/whisper.exe ./whisper.exe

echo "_____ Calculating checksums _____"
for binary in $(ls)
do
  rhash --sha256 $binary -o $binary.sha256
  ./parity.exe tools hash $binary > $binary.sha3
done
cp parity.exe.sha256 parity.sha256
cp parity.exe.sha3 parity.sha3

sccache -s
