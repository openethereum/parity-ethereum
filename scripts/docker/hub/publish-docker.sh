#!/bin/sh

set -e # fail on any error

VERSION=$(cat ./tools/VERSION)
TRACK=$(cat ./tools/TRACK)
echo "OpenEthereum version = ${VERSION}"
echo "OpenEthereum track = ${TRACK}"

test "$Docker_Hub_User_OpenEthereum" -a "$Docker_Hub_Pass_OpenEthereum" \
    || ( echo "no docker credentials provided"; exit 1 )
docker login -u "$Docker_Hub_User_OpenEthereum" -p "$Docker_Hub_Pass_OpenEthereum"
echo "__________Docker info__________"
docker info

# we stopped pushing nightlies to dockerhub, will push to own registry prb.
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    "$SCHEDULE_TAG")
        echo "Docker TAG - 'openethereum/openethereum:${SCHEDULE_TAG}'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "openethereum/openethereum:${SCHEDULE_TAG}" \
            --file tools/Dockerfile .;
        docker push "openethereum/openethereum:${SCHEDULE_TAG}";;
    "stable")
        echo "Docker TAGs - 'openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}', 'openethereum/openethereum:stable'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}" \
            --tag "openethereum/openethereum:latest" \
            --tag "openethereum/openethereum:stable" \
            --file tools/Dockerfile .;
        docker push "openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}";
        docker push "openethereum/openethereum:stable";
        docker push "openethereum/openethereum:latest";;
    v[0-9]*.[0-9]*)
        echo "Docker TAG - 'openethereum/openethereum:${VERSION}-${TRACK}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "openethereum/openethereum:${VERSION}-${TRACK}" \
            --file tools/Dockerfile .;
        docker push "openethereum/openethereum:${VERSION}-${TRACK}";;
    *)
        echo "Docker TAG - 'openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}" \
            --file tools/Dockerfile .;
        docker push "openethereum/openethereum:${VERSION}-${CI_COMMIT_REF_NAME}";;
esac

docker logout
