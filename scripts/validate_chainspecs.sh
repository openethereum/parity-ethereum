#!/usr/bin/env sh

ERR=0
cargo build --release -p chainspec

for spec in ethcore/res/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

for spec in ethcore/res/ethereum/*.json; do
    if ! ./target/release/chainspec "$spec"; then ERR=1; fi
done

exit $ERR
