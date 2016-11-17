#!/bin/bash

# change into the js directory (one down from scripts)
pushd `dirname $0`
cd ..

# run build (production) and store the exit code
EXITCODE=0
BUILDDIR=./.dist
rm -rf $BUILDDIR
mkdir -p $BUILDDIR/src
BUILD_DEST=$BUILDDIR/build npm run ci:build || EXITCODE=1

# Copy rust files
cp Cargo.precompiled.toml $BUILDDIR/Cargo.toml
cp build.rs $BUILDDIR
cp src/lib.rs* $BUILDDIR/src

# back to root
popd

# exit with exit code
exit $EXITCODE
