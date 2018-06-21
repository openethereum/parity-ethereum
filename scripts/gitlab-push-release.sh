#!/usr/bin/env bash

set -e # fail on any error
set -u # treat unset variables as error

DATA="secret=$RELEASES_SECRET"

echo "Pushing release to Mainnet"
./scripts/safe_curl.sh $DATA "http://update.parity.io:1337/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF"

echo "Pushing release to Kovan"
./scripts/safe_curl.sh $DATA "http://update.parity.io:1338/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF"
