#!/bin/bash
set -e

# variables
PACKAGES=( "parity" )

# change into the build directory
BASEDIR=`dirname $0`
cd $BASEDIR/..

# build all packages
echo "*** Building packages for npmjs"
echo "$NPM_TOKEN" >> ~/.npmrc

for PACKAGE in ${PACKAGES[@]}
do
  echo "*** Building $PACKAGE"
  LIBRARY=$PACKAGE npm run ci:build:npm
  DIRECTORY=.npmjs/$PACKAGE

  cd $DIRECTORY

  echo "*** Publishing $PACKAGE from $DIRECTORY"
  echo "npm publish --access public || true"
  cd ../..

done
cd ..

# exit with exit code
exit 0
