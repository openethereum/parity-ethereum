#!/usr/bin/env sh

# The image name
PARITY_IMAGE_REPO=${PARITY_IMAGE_REPO:-parity/parity}
# The tag to be used for builder image
PARITY_BUILDER_IMAGE_TAG=${PARITY_BUILDER_IMAGE_TAG:-build}
# The tag to be used for runner image
PARITY_RUNNER_IMAGE_TAG=${PARITY_RUNNER_IMAGE_TAG:-latest}

echo Building $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H")
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") . -f docker/centos/Dockerfile.build

echo Creating $PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H"), extracting binary
docker create --name extract $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") 
mkdir docker/centos/parity
docker cp extract:/build/parity-ethereum/target/release/parity docker/centos/parity

echo Building $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG docker/centos/ -f docker/centos/Dockerfile

echo Cleaning up ...
rm -rf docker/centos/parity
docker rm -f extract
docker rmi -f $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H")

echo Echoing Parity version:
docker run $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG --version

echo Done.
