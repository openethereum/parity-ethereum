#!/usr/bin/env sh

# The image name
OPENETHEREUM_IMAGE_REPO=${OPENETHEREUM_IMAGE_REPO:-openethereum/openethereum}
# The tag to be used for builder image
OPENETHEREUM_BUILDER_IMAGE_TAG=${OPENETHEREUM_BUILDER_IMAGE_TAG:-build}
# The tag to be used for runner image
OPENETHEREUM_RUNNER_IMAGE_TAG=${OPENETHEREUM_RUNNER_IMAGE_TAG:-latest}

echo Building $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_BUILDER_IMAGE_TAG-$(git log -1 --format="%H")
docker build --no-cache -t $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") . -f scripts/docker/centos/Dockerfile.build

echo Creating $OPENETHEREUM_BUILDER_IMAGE_TAG-$(git log -1 --format="%H"), extracting binary
docker create --name extract $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") 
mkdir scripts/docker/centos/openethereum
docker cp extract:/build/openethereum/target/release/openethereum scripts/docker/centos/openethereum

echo Building $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_RUNNER_IMAGE_TAG
docker build --no-cache -t $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_RUNNER_IMAGE_TAG scripts/docker/centos/ -f scripts/docker/centos/Dockerfile

echo Cleaning up ...
rm -rf scripts/docker/centos/openethereum
docker rm -f extract
docker rmi -f $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_BUILDER_IMAGE_TAG-$(git log -1 --format="%H")

echo Echoing OpenEthereum version:
docker run $OPENETHEREUM_IMAGE_REPO:$OPENETHEREUM_RUNNER_IMAGE_TAG --version

echo Done.
