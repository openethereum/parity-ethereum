#!/bin/bash

cd $TRAVIS_BUILD_DIR
cargo build --features final
TIMEOUT=$(echo 50*60-60-$SECONDS|bc)
$TRAVIS_BUILD_DIR/target/debug/parity --chain goerli --log-file $TRAVIS_BUILD_DIR/parity.log & sleep $TIMEOUT && killall parity
tail $TRAVIS_BUILD_DIR/parity.log
