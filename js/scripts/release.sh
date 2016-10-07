#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# run release
EXICCODE=0

# back to root
popd

# exit with exit code
exit $EXITCODE
