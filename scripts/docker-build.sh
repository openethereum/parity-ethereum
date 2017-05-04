#!/bin/bash
cd docker/hub
if [ "$1" == "latest" ]; then DOCKER_BUILD_TAG="beta-release"; fi
docker build --build-arg BUILD_TAG=$DOCKER_BUILD_TAG --no-cache=true --tag $2/parity:$1 .
docker push $2/parity:$1
