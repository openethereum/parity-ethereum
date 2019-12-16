#!/bin/bash
echo "________Running test-cpp.sh________"
set -e # fail on any error
set -u # treat unset variables as error
set -B # enable brace expansion
#use nproc `linux only
THREADS=$(nproc)

echo "________Running the C++ example________"
DIR=parity-clib/examples/cpp/build
mkdir -p $DIR
cd $DIR
cmake -DCMAKE_C{,XX}_COMPILER_LAUNCHER=sccache
make VERBOSE=1 -j $THREADS
# Note: we don't try to run the example because it tries to sync Kovan, and we don't want
#       that to happen on CI
cd -
rm -rf $DIR
#show sccache statistics
sccache --show-stats
