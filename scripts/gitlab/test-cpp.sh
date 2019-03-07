#!/bin/bash
echo "________Running test-cpp.sh________"
set -e # fail on any error
set -u # treat unset variables as error
#use nproc `linux only
THREADS=$(nproc)

echo $RUST_FILES_MODIFIED
if [ "${RUST_FILES_MODIFIED}" = "0" ]
then
  echo "__________Skipping Rust tests since no Rust files modified__________";
  exit 0
fi

echo "________Running the C++ example________"
DIR=parity-clib/examples/cpp/build
mkdir -p $DIR
cd $DIR
cmake ..
make -j $THREADS
# Note: we don't try to run the example because it tries to sync Kovan, and we don't want
#       that to happen on CI
cd -
rm -rf $DIR
