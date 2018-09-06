#!/bin/bash

set -x
git submodule update --init --recursive
rm -rf target/*
cargo test --all --exclude evmjit --no-run -- --test-threads 8|| exit $?
KCOV_TARGET="target/cov"
KCOV_FLAGS="--verify"
mkdir -p $KCOV_TARGET
echo "__________Cover RUST___________"
for FILE in `find target/debug/deps ! -name "*.*" -type f`
do
  timeout --signal=SIGKILL 5m kcov --include-path=$(pwd) --exclude-path=$(pwd)/target $KCOV_FLAGS $KCOV_TARGET $FILE
done
timeout --signal=SIGKILL 5m kcov --include-path=$(pwd) --exclude-path=$(pwd)/target $KCOV_FLAGS $KCOV_TARGET target/debug/parity-*

bash <(curl -s https://codecov.io/bash) &&
  echo "Uploaded code coverage"

exit 0
