#!/bin/bash
set -e

# change into the build directory
BASEDIR=`dirname $0`
cd $BASEDIR/..

# build all packages
echo "$NPM_TOKEN" >> ~/.npmrc

echo "*** Building jsonrpc for NPM"
npm run ci:build:jsonrpc
cp -r src/jsonrpc npm/jsonrpc/src
env LIBRARY=jsonrpc npm run ci:build:npm

echo "*** Building parity.js for NPM"
cp -r src/abi npm/parity/src/abi
cp -r src/api npm/parity/src/api
env LIBRARY=parity npm run ci:build:npm

echo "*** Building etherscan for NPM"
cp -r src/3rdparty/etherscan npm/etherscan/src
env LIBRARY=etherscan npm run ci:build:npm

echo "*** Building shapeshift for NPM"
cp -r src/3rdparty/shapeshift npm/shapeshift/src
env LIBRARY=shapeshift npm run ci:build:npm

# exit with exit code
exit 0
