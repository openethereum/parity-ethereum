#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

case ${CI_COMMIT_REF_NAME} in
  master|*v1.12*|gitlab-next) export CHANNEL="edge";;
  beta|*v1.11*) export CHANNEL="beta";;
  stable|*v1.10*) export CHANNEL="stable";;
  *) echo "No release" exit 0;;
esac
echo "Release channel :" $CHANNEL " Branch/tag: " $CI_COMMIT_REF_NAME

echo $SNAPCRAFT_LOGIN_PARITY_BASE64 | base64 --decode > snapcraft.login
snapcraft login --with snapcraft.login
snapcraft push --release $CHANNEL "packages/parity_"$VERSION"_"$BUILD_ARCH".snap"
snapcraft status parity
snapcraft logout
