#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

apt install -y rhash
MD5_BIN="rhash --md5"
SHA256_BIN="rhash --sha256"

snapcraft clean

case $RELEASE_TRACK in
    stable) export CHANNEL="stable" ;;
    beta)   export CHANNEL="beta" ;;
    *)      export CHANNEL="edge" ;;
esac

SNAP_PACKAGE="parity_"$VERSION"_"$BUILD_ARCH".snap"

cat scripts/gitlab/templates/snapcraft.template.yaml | envsubst '$VERSION,$CHANNEL,$BUILD_ARCH' > snapcraft.yaml
snapcraft build -d

rm -rf artifacts
mkdir -p artifacts
mv $SNAP_PACKAGE "artifacts/"

cd artifacts
$MD5_BIN $SNAP_PACKAGE -o $SNAP_PACKAGE".md5"
$SHA256_BIN $SNAP_PACKAGE > $SNAP_PACKAGE".sha256"