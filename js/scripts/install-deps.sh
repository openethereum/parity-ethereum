#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# install deps and store the exit code
EXITCODE=0
node --version
npm --version
npm install --progress=false || EXITCODE=1

# back to root
popd

# exit with exit code
exit $EXITCODE
