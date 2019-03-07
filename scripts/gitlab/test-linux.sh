#!/bin/bash
echo "________Running test.sh________"
set -e # fail on any error
set -u # treat unset variables as error

FEATURES="json-tests,ci-skip-tests"
OPTIONS="--release"
#use nproc `linux only
THREADS=$(nproc)

echo $RUST_FILES_MODIFIED
if [ "${RUST_FILES_MODIFIED}" = "0" ]
then
  echo "__________Skipping Rust tests since no Rust files modified__________";
  exit 0
fi

echo "________Running Parity Full Test Suite________"
time cargo test $OPTIONS --features "$FEATURES" --locked --all --target $CARGO_TARGET -- --test-threads $THREADS
