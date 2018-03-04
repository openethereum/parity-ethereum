#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

case $RELEASE_TRACK in
    stable) export CHANNEL="stable" ;;
    beta)   export CHANNEL="beta" ;;
    *)      export CHANNEL="edge" ;;
esac

echo $SNAPCRAFT_LOGIN_PARITY_BASE64 | base64 --decode > snapcraft.login
snapcraft login --with snapcraft.login
snapcraft push "artifacts/parity_"$VERSION"_"$BUILD_ARCH".snap" --release $CHANNEL
snapcraft status parity
snapcraft logout