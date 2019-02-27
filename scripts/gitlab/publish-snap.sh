#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

# some necromancy:
# gsub(/"/, "", $2) deletes "qoutes"
# gsub(/ /, "", $2) deletes whitespaces
TRACK=`awk -F '=' '/^track/ {gsub(/"/, "", $2); gsub(/ /, "", $2); print $2}' ./util/version/Cargo.toml`
echo Track is: $TRACK
# prepare variables
VERSION=v"$(sed -r -n '1,/^version/s/^version = "([^"]+)".*$/\1/p' Cargo.toml)"
SNAP_PACKAGE="parity_"$VERSION"_"$BUILD_ARCH".snap"
CARGO_TARGET="$(ls artifacts)"
# Choose snap release channel based on parity ethereum version track
case ${TRACK} in
  nightly) export GRADE="devel" CHANNEL="edge";;
  beta) export GRADE="stable" CHANNEL="beta";;
  stable) export GRADE="stable" CHANNEL="stable";;
  *) echo "No release" && exit 0;;
esac

# Release untagged versions from branches to the candidate snap channel
case ${CI_COMMIT_REF_NAME} in
  beta|stable) export GRADE="stable" CHANNEL="candidate";;
esac
echo "__________Create snap package__________"
echo "Release channel :" $GRADE " Branch/tag: " $CI_COMMIT_REF_NAME
echo $VERSION:$GRADE:$BUILD_ARCH:$CARGO_TARGET

sed -e 's/$VERSION/'"$VERSION"'/g' \
    -e 's/$GRADE/'"$GRADE"'/g' \
    -e 's/$BUILD_ARCH/'"$BUILD_ARCH"'/g' \
    -e 's/$CARGO_TARGET/'"$CARGO_TARGET"'/g' \
    scripts/snap/snapcraft.template.yaml > snapcraft.yaml

apt update
apt install -y --no-install-recommends rhash
cat snapcraft.yaml
snapcraft --target-arch=$BUILD_ARCH
ls *.snap

echo "__________Calculating checksums__________"
rhash --sha256 $SNAP_PACKAGE -o $SNAP_PACKAGE".sha256"
cat $SNAP_PACKAGE".sha256"

echo "__________Releasing snap package__________"
echo "Release channel :" $CHANNEL " Branch/tag: " $CI_COMMIT_REF_NAME

echo $SNAPCRAFT_LOGIN_PARITY_BASE64 | base64 --decode > snapcraft.login
snapcraft login --with snapcraft.login
snapcraft push --release $CHANNEL $SNAP_PACKAGE
snapcraft status parity
snapcraft logout
