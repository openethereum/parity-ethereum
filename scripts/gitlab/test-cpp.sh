#!/usr/bin/env bash

echo "________Running test-cpp.sh________"
set -e # fail on any error
set -u # treat unset variables as error
#use nproc `linux only
THREADS=$(nproc)

echo "________Running the C++ example________"
DIR=parity-clib/examples/cpp/build
mkdir -p $DIR
cd $DIR
cmake --build . -- -j $THREADS
./parity-example
cd -
rm -rf $DIR
