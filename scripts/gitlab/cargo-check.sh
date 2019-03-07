#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error
echo "________Running cargo_check.sh________"

PARAMS="--target $CARGO_TARGET --locked"

echo "________Validate build________"
time cargo check $PARAMS --no-default-features
time cargo check $PARAMS --manifest-path util/io/Cargo.toml --no-default-features
time cargo check $PARAMS --manifest-path util/io/Cargo.toml --features "mio"
