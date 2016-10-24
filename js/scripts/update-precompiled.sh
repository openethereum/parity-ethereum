#!/bin/bash
set -e

# change into main dir
pushd `dirname $0`
cd ../../

cargo update -p parity-ui-precompiled

popd
exit 0
