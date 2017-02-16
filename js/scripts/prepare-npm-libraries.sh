#!/bin/bash
set -e

# change into the build directory
BASEDIR=`dirname $0`
cd $BASEDIR/..

# build all packages
echo "$NPM_TOKEN" >> ~/.npmrc

printf "\n\n"
printf "\n***************************************"
printf "\n***** Building jsonrpc for NPM ********"
printf "\n***************************************\n\n"
npm run ci:build:jsonrpc
cp LICENSE npm/jsonrpc/LICENSE
mkdir -p npm/jsonrpc/src
cp -R src/jsonrpc/* npm/jsonrpc/src
env LIBRARY=jsonrpc npm run ci:build:npm
node test/e2e/jsonrpc.smoke.js

printf "\n\n"
printf "\n***************************************"
printf "\n***** Building parity.js for NPM ******"
printf "\n***************************************\n\n"
cp LICENSE npm/parity/LICENSE
mkdir -p npm/parity/src
cp src/parity.npm.js npm/parity/src/index.js
cp -R src/abi npm/parity/src
cp -R src/api npm/parity/src
env LIBRARY=parity npm run ci:build:npm
node test/e2e/parity.smoke.js

printf "\n\n"
printf "\n***************************************"
printf "\n***** Building etherscan for NPM ******"
printf "\n***************************************\n\n"
cp LICENSE npm/etherscan/LICENSE
mkdir -p npm/etherscan/src
cp -LR src/3rdparty/etherscan/* npm/etherscan/src
env LIBRARY=etherscan npm run ci:build:npm
node test/e2e/etherscan.smoke.js

printf "\n\n"
printf "\n***************************************"
printf "\n***** Building shapeshift for NPM *****"
printf "\n***************************************\n\n"
cp LICENSE npm/shapeshift/LICENSE
mkdir -p npm/shapeshift/src
cp -R src/3rdparty/shapeshift/* npm/shapeshift/src
env LIBRARY=shapeshift npm run ci:build:npm
node test/e2e/shapeshift.smoke.js

# exit with exit code
exit 0
