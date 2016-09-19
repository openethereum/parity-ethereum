#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# install deps and store the exit code
npm install
EXICCODE=$?

# back to root
popd

# exit with exit code
exit $EXITCODE
