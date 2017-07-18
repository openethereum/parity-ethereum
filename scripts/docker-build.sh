#!/bin/bash
cd docker/hub
DOCKER_BUILD_TAG=$1
echo "Docker build tag: " $DOCKER_BUILD_TAG
docker build --build-arg BUILD_TAG=$DOCKER_BUILD_TAG --no-cache=true --tag parity/parity:$DOCKER_BUILD_TAG .
docker run -it parity/parity:$DOCKER_BUILD_TAG -v
docker push parity/parity:$DOCKER_BUILD_TAG
