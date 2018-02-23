#!/bin/bash
set -e # fail on any error
if [[ -z "$protect" ]]; then echo "__________Skipping Docker build and push__________"&&exit 0; fi
if [ "$CI_BUILD_REF_NAME" == "beta" ]; then DOCKER_BUILD_TAG="latest"; else DOCKER_BUILD_TAG=$CI_BUILD_REF_NAME; fi
echo "__________Docker TAG__________"
echo $DOCKER_BUILD_TAG
docker login -u $Docker_Hub_User_Parity -p $Docker_Hub_Pass_Parity
cd docker/hub
docker build --build-arg BUILD_TAG=$DOCKER_BUILD_TAG --no-cache=true --tag parity/parity:$DOCKER_BUILD_TAG .
docker run -it parity/parity:$DOCKER_BUILD_TAG -v
docker push parity/parity:$DOCKER_BUILD_TAG
docker logout
echo "__________Build comlete__________"
