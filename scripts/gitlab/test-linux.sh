#!/bin/bash
# ARGUMENT $1 Rust flavor to run test with (stable/beta/nightly)

echo "________Running test-linux.sh________"
set -e # fail on any error
set -u # treat unset variables as error

FEATURES="json-tests,ci-skip-tests"
OPTIONS=""
#use nproc `linux only
THREADS=$(nproc)

rustup default $1
rustup show

echo "________Running Parity Full Test Suite________"
time cargo test $OPTIONS --features "$FEATURES" --locked --all --target $CARGO_TARGET --verbose --color=always -- --test-threads $THREADS
