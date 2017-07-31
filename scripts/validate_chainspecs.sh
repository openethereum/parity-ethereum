#!/bin/sh

ERR=0
cargo build --release -p chainspec

for spec in `ls ethcore/res/*.json`; do
    ./target/release/chainspec $spec
    if [ $? -ne "0" ]; then ERR=1; fi
done

for spec in `ls ethcore/res/ethereum/*.json`; do
    ./target/release/chainspec $spec
    if [ $? -ne "0" ]; then ERR=1; fi
done

exit $ERR
