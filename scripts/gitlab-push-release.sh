#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

DATA="secret=$RELEASES_SECRET"

echo "Pushing release to Mainnet"
source scripts/safe_curl.sh $DATA "http://localhost:1337/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF"

echo "Pushing release to Kovan"
source scripts/safe_curl.sh $DATA "http://localhost:1338/push-release/$CI_BUILD_REF_NAME/$CI_BUILD_REF"
