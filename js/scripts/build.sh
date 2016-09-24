#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# run build (production) and store the exit code
NODE_ENV=production npm run build
EXICCODE=$?

# back to root
popd

# exit with exit code
exit $EXITCODE
