#!/bin/bash

cd $TRAVIS_BUILD_DIR

echo "________________________________________________________________________________"
echo "BUILD PARITY: cargo build --features final"
cargo build --release --features final
TIMEOUT=$(echo 50*60-1800-$SECONDS|bc)

echo "________________________________________________________________________________"
echo "RUN PARITY FOR $TIMEOUT SECONDS: parity --chain goerli"
$TRAVIS_BUILD_DIR/target/debug/parity --chain goerli --log-file $TRAVIS_BUILD_DIR/parity.log & sleep $TIMEOUT && killall parity

echo "________________________________________________________________________________"
echo "FINISHED SYNC TASK: tail -n32 parity.log"
tail -n32 $TRAVIS_BUILD_DIR/parity.log
