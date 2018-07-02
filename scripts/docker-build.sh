#!/usr/bin/env bash
cd docker/hub
DOCKER_BUILD_TAG=$1
echo "Docker build tag: " $DOCKER_BUILD_TAG
if [[ "$DOCKER_BUILD_TAG" = "latest" ]]; then
	docker build --build-arg BUILD_TAG="master" --no-cache=true --tag parity/parity:$DOCKER_BUILD_TAG .
else
	docker build --build-arg BUILD_TAG=$DOCKER_BUILD_TAG --no-cache=true --tag parity/parity:$DOCKER_BUILD_TAG .
fi
docker run -it parity/parity:$DOCKER_BUILD_TAG -v
docker push parity/parity:$DOCKER_BUILD_TAG
