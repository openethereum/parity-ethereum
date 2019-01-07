#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo CI_COMMIT_REF NAME: $CI_COMMIT_REF_NAME
echo VERSION: $VERSION
echo SCHEDULE_TAG-CI_COMMIT_REF_NAME: ${SCHEDULE_TAG-${CI_COMMIT_REF_NAME}}" = "nightly" && VERSION="${VERSION}-${ID_SHORT}-${DATE_STR}"
test "$SCHEDULE_TAG-${CI_COMMIT_REF_NAME}}" = "nightly" 
echo VERSION-ID_SHORT-DATE_STR: "${VERSION}-${ID_SHORT}-${DATE_STR}"
test VERSION="${VERSION}-${ID_SHORT}-${DATE_STR}"
test "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" = "nightly" && VERSION="${VERSION}-${ID_SHORT}-${DATE_STR}"

case $ {track} in
  nightly) export VERSION_NIGHTLY=vn;;
  beta) export VERSION_BETA=vb;;
  stable) export VERSION_STABLE=vs;;
  *) echo "Non releasable track" exit 0;;
esac

case ${CI_COMMIT_REF_NAME} in
  nightly|*v2.3*) export GRADE="devel" CHANNEL="edge";;
  beta|*v2.2*) export GRADE="stable" CHANNEL="beta";;
  stable|*v2.1*) export GRADE="stable" CHANNEL="stable";;
  pr|*10142*) export GRADE="debug_grade" CHANNEL="debug_channel";; # delme
  *) echo "No release" exit 0;;
esac

SNAP_PACKAGE="parity_"$VERSION"_"$BUILD_ARCH".snap"

echo "__________Create snap package__________"
echo "Release channel :" $GRADE " Branch/tag: " $CI_COMMIT_REF_NAME
echo $VERSION:$GRADE:$BUILD_ARCH
cat scripts/snap/snapcraft.template.yaml | envsubst '$VERSION:$GRADE:$BUILD_ARCH:$CARGO_TARGET' > snapcraft.yaml
cat snapcraft.yaml
snapcraft --target-arch=$BUILD_ARCH
ls *.snap

echo "__________Calculating checksums__________"
rhash --sha256 $SNAP_PACKAGE -o $SNAP_PACKAGE".sha256"
cat $SNAP_PACKAGE".sha256"

echo "__________Releasing snap package__________"
echo "Release channel :" $CHANNEL " Branch/tag: " $CI_COMMIT_REF_NAME

# echo $SNAPCRAFT_LOGIN_PARITY_BASE64 | base64 --decode > snapcraft.login
# snapcraft login --with snapcraft.login
# snapcraft push --release $CHANNEL $SNAP_PACKAGE
# snapcraft status parity
# snapcraft logout
