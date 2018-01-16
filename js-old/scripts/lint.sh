#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# run lint & tests and store the exit code
EXITCODE=0
npm run lint || EXITCODE=1

# back to root
popd

# exit with exit code
exit $EXITCODE
