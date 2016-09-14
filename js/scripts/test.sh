#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# run lint & tests and store the exit code
npm run lint && npm run test
EXICCODE=$?

# back to root
popd

# exit with exit code
exit $EXITCODE
