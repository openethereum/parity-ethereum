#!/usr/bin/env sh

# The image name
OPENETHEREUM_IMAGE_REPO=${OPENETHEREUM_IMAGE_REPO}
# The tag to be used for the image
OPENETHEREUM_BUILDER_IMAGE_TAG=${OPENETHEREUM_IMAGE_TAG}

echo Building $OPENETHEREUM_IMAGE_REPO:
docker build -t $OPENETHEREUM_IMAGE_REPO . -f scripts/docker/alpine/Dockerfile

echo Running OpenEthereum:
docker run -d --name $OPENETHEREUM_IMAGE_TAG $OPENETHEREUM_IMAGE_REPO

echo Echoing OpenEthereum version:
docker exec -it $OPENETHEREUM_IMAGE_TAG ./openethereum --version

echo Done.
