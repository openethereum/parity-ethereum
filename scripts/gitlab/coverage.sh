#!/bin/bash

set -x
git submodule update --init --recursive
rm -rf target/*
cargo test --all --exclude evmjit --no-run -- --test-threads 8|| exit $?
KCOV_TARGET="target/cov"
KCOV_FLAGS="--verify"
EXCLUDE="/usr/lib,/usr/include,$HOME/.cargo,$HOME/.multirust,rocksdb,secp256k1"
mkdir -p $KCOV_TARGET
echo "__________Cover RUST___________"
for FILE in `find target/debug/deps ! -name "*.*"`
  do
   timeout --signal=SIGKILL 5m kcov --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET $FILE
  done
timeout --signal=SIGKILL 5m kcov --exclude-pattern $EXCLUDE $KCOV_FLAGS $KCOV_TARGET target/debug/parity-*
echo "Cover JS"
bash <(curl -s https://codecov.io/bash)&&
echo "Uploaded code coverage"
exit 0
