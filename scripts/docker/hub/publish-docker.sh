#!/bin/sh

set -e # fail on any error

# VERSION=v"$(sed -r -n '1,/^version/s/^version = "([^"]+)".*$/\1/p' Cargo.toml)"
echo "Parity Ethereum version = ${VERSION}"

test "$Docker_Hub_User_Parity" -a "$Docker_Hub_Pass_Parity" \
    || ( echo "no docker credentials provided"; exit 1 )
docker login -u "$Docker_Hub_User_Parity" -p "$Docker_Hub_Pass_Parity"
echo "__________Docker info__________"
docker info

# we stopped pushing nightlies to dockerhub, will push to own registry prb.
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    "$SCHEDULE_TAG")
        echo "Docker TAG - '${CONTAINER_IMAGE}:${SCHEDULE_TAG}'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "${CONTAINER_IMAGE}:${SCHEDULE_TAG}" \
            --file tools/Dockerfile .;
        docker push "${CONTAINER_IMAGE}:${SCHEDULE_TAG}";;
    "beta")
        echo "Docker TAGs - '${CONTAINER_IMAGE}:beta', '${CONTAINER_IMAGE}:latest', \
            '${CONTAINER_IMAGE}:${VERSION}'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "${CONTAINER_IMAGE}:beta" \
            --tag "${CONTAINER_IMAGE}:latest" \
            --tag "${CONTAINER_IMAGE}:${VERSION}" \
            --file tools/Dockerfile .;
        docker push "${CONTAINER_IMAGE}:beta";
        docker push "${CONTAINER_IMAGE}:latest";
        docker push "${CONTAINER_IMAGE}:${VERSION}";;
    "stable")
        echo "Docker TAGs - '${CONTAINER_IMAGE}:${VERSION}', '${CONTAINER_IMAGE}:stable'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "${CONTAINER_IMAGE}:${VERSION}" \
            --tag "${CONTAINER_IMAGE}:stable" \
            --file tools/Dockerfile .;
        docker push "${CONTAINER_IMAGE}:${VERSION}";
        docker push "${CONTAINER_IMAGE}:stable";;
    *)
        echo "Docker TAG - '${CONTAINER_IMAGE}:${VERSION}-${CI_COMMIT_SHORT_SHA}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "${CONTAINER_IMAGE}:${VERSION}-${CI_COMMIT_SHORT_SHA}" \
            --file tools/Dockerfile .;
        docker push "${CONTAINER_IMAGE}:${VERSION}-${CI_COMMIT_SHORT_SHA}";;
esac

docker logout
