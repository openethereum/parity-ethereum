#!/bin/sh

set -e

git submodule update --init --recursive
$TRAVIS_BUILD_DIR/scripts/validate_chainspecs.sh
cargo test --features json-tests,ci-skip-issue --all $@ -- --test-threads 8
