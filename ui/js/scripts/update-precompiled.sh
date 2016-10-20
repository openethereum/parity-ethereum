#!/bin/bash


# change into the submodule build directory
pushd `dirname $0`
cd ../build

if [ -z "$1" ]; then
  popd
  echo "Usage: $0 <sha-commit>"
  exit 1
fi

git fetch
git fetch origin $1
git merge $1 -X theirs

popd
exit 0
