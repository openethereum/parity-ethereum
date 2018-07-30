#!/bin/sh

# The tag used when doing git checkout
PARITY_BUILD_TAG=${PARITY_BUILD_TAG:-master}
# The repo to pull
PARITY_BUILD_REPO=${PARITY_BUILD_REPO:-https://github.com/paritytech/parity-ethereum}
# The image name
PARITY_IMAGE_REPO=${PARITY_IMAGE_REPO:-parity/parity}
# The tag to be used for builder image
PARITY_BUILDER_IMAGE_TAG=${PARITY_BUILDER_IMAGE_TAG:-build-latest}
# The tag to be used for runner image
PARITY_RUNNER_IMAGE_TAG=${PARITY_RUNNER_IMAGE_TAG:-latest}

echo Building $PARITY_BUILDER_IMAGE_TAG from $PARITY_BUILD_REPO:$PARITY_BUILD_TAG
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG . -f build.Dockerfile

echo Creating $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG, extracting binary
docker create --name extract $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG 
mkdir parity
docker cp extract:/build/parity-ethereum/target/release/parity ./parity

echo Building $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG .

echo Cleaning up ...
rm -rf ./parity
docker rm -f extract

echo Done.
