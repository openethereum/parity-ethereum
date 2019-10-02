#!/bin/bash
# ARGUMENT $1 Rust flavor to run test with (stable/beta/nightly)

echo "________Running test-linux.sh________"
set -e # fail on any error
set -u # treat unset variables as error

export CC="sccache gcc"
export CXX="sccache g++"
FEATURES="json-tests"

OPTIONS="--release"
#use nproc `linux only
THREADS=$(nproc)

rustup default $1
rustup show

echo "________Running Parity Full Test Suite________"
# Why are we using RUSTFLAGS? See https://github.com/paritytech/parity-ethereum/pull/10719
CARGO_INCREMENTAL=0 RUSTFLAGS="-C opt-level=3 -C overflow-checks=on -C debuginfo=2 -Ctarget-feature=+aes,+sse2,+ssse3" time cargo test $OPTIONS --features "$FEATURES" --locked --all --target $CARGO_TARGET --verbose --color=never -- --test-threads $THREADS

#show sccache statistics
sccache --stop-server
