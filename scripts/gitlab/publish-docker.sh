#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

if [ "$CI_BUILD_REF_NAME" == "beta" ]; then DOCKER_BUILD_TAG="latest"; else DOCKER_BUILD_TAG=$CI_BUILD_REF_NAME; fi
echo "__________Docker TAG__________"
echo $DOCKER_BUILD_TAG

docker build --no-cache=true --tag parity/parity:$DOCKER_BUILD_TAG --file ./docker/hub/Dockerfile

docker login -u $Docker_Hub_User_Parity -p $Docker_Hub_Pass_Parity
docker push parity/parity:$DOCKER_BUILD_TAG
docker logout
