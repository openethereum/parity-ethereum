#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

case $RELEASE_TRACK in
    stable) export CHANNEL="stable" ;;
    beta)   export CHANNEL="beta" ;;
    *)      export CHANNEL="edge" ;;
esac

SNAP_PACKAGE="parity_"$VERSION"_"$BUILD_ARCH".snap"

snapcraft clean
cat scripts/gitlab/templates/snapcraft.template.yaml | envsubst '$VERSION,$CHANNEL,$BUILD_ARCH' > snapcraft.yaml
snapcraft build -d

rm -rf artifacts
mkdir -p artifacts
mv $SNAP_PACKAGE "artifacts/"

cd artifacts
rhash --md5 $SNAP_PACKAGE -o $SNAP_PACKAGE".md5"
rhash --sha256 $SNAP_PACKAGE -o $SNAP_PACKAGE".sha256"
