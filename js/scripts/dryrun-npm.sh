#!/bin/bash
set -e

# variables
PACKAGES=( "Parity" "Etherscan" "ShapeShift" )

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
  DIRECTORY=.npmjs/$(echo $PACKAGE | tr '[:upper:]' '[:lower:]')

  echo "*** Publishing $PACKAGE from $DIRECTORY"
  cd $DIRECTORY
  echo "npm publish --access public || true"
  cd ../..

done
cd ..

# exit with exit code
exit 0
