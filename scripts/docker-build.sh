#!/bin/bash
cd docker/hub
docker build --build-arg BUILD_TAG=$1 --no-cache=true --tag ethcore/parity:$1 .
docker push ethcore/parity:$1
