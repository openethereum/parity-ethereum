#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error
echo "________Running validate_chainspecs.sh________"

PARAMS="--target $CARGO_TARGET --locked"
ERR=0

echo "________Validate build________"
time cargo check $PARAMS --no-default-features
time cargo check $PARAMS --manifest-path util/io/Cargo.toml --no-default-features
time cargo check $PARAMS --manifest-path util/io/Cargo.toml --features "mio"

echo "________Validate chainspecs________"
time cargo build --release -p chainspec

for spec in ethcore/res/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

for spec in ethcore/res/ethereum/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

exit $ERR
