#!/bin/bash
set -e

# change into the build directory
BASEDIR=`dirname $0`
cd $BASEDIR/..

# build all packages
echo "$NPM_TOKEN" >> ~/.npmrc

echo "*** Building jsonrpc for NPM"
npm run ci:build:jsonrpc
mkdir -p npm/jsonrpc/src
cp -R src/jsonrpc/* npm/jsonrpc/src
env LIBRARY=jsonrpc npm run ci:build:npm

pushd .; cd npm/jsonrpc
npm test
popd

echo "*** Building parity.js for NPM"
mkdir -p npm/parity/src
cp src/parity.js npm/parity/src/index.js
cp -R src/abi npm/parity/src
cp -R src/api npm/parity/src
env LIBRARY=parity npm run ci:build:npm

pushd .; cd npm/parity
npm test
popd

echo "*** Building etherscan for NPM"
mkdir -p npm/etherscan/src
cp -R src/3rdparty/etherscan/* npm/etherscan/src
env LIBRARY=etherscan npm run ci:build:npm

pushd .; cd npm/etherscan
npm test
popd

echo "*** Building shapeshift for NPM"
mkdir -p npm/shapeshift/src
cp -R src/3rdparty/shapeshift/* npm/shapeshift/src
env LIBRARY=shapeshift npm run ci:build:npm

pushd .; cd npm/shapeshift
npm test
popd

# exit with exit code
exit 0
