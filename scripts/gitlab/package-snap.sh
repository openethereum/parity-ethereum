#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
case ${CI_COMMIT_REF_NAME} in
  nightly|*v2.2*) export GRADE="devel";;
  beta|*v2.1*) export GRADE="stable";;
  stable|*v2.0*) export GRADE="stable";;
  *) echo "No release" exit 0;;
esac
SNAP_PACKAGE="parity_"$VERSION"_"$BUILD_ARCH".snap"
echo "__________Create snap package__________"
echo "Release channel :" $GRADE " Branch/tag: " $CI_COMMIT_REF_NAME
snapcraft clean
echo $VERSION:$GRADE:$BUILD_ARCH
cat scripts/gitlab/templates/snapcraft.template.yaml | envsubst '$VERSION:$GRADE:$BUILD_ARCH:$CARGO_TARGET' > snapcraft.yaml
cat snapcraft.yaml
snapcraft --target-arch=$BUILD_ARCH
ls *.snap
echo "__________Post-processing snap package__________"
mkdir -p artifacts
mv -v $SNAP_PACKAGE "artifacts/"$SNAP_PACKAGE
echo "_____ Calculating checksums _____"
cd artifacts
rhash --sha256 $SNAP_PACKAGE -o $SNAP_PACKAGE".sha256"
