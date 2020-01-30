#!/bin/sh

set -e # fail on any error

VERSION=$(cat ./tools/VERSION)
TRACK=$(cat ./tools/TRACK)
echo "Parity Ethereum version = ${VERSION}"
echo "Parity Ethereum track = ${TRACK}"

test "$Docker_Hub_User_Parity" -a "$Docker_Hub_Pass_Parity" \
    || ( echo "no docker credentials provided"; exit 1 )
docker login -u "$Docker_Hub_User_Parity" -p "$Docker_Hub_Pass_Parity"
echo "__________Docker info__________"
docker info

# we stopped pushing nightlies to dockerhub, will push to own registry prb.
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    "$SCHEDULE_TAG")
        echo "Docker TAG - 'parity/parity:${SCHEDULE_TAG}'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "parity/parity:${SCHEDULE_TAG}" \
            --file tools/Dockerfile .;
        docker push "parity/parity:${SCHEDULE_TAG}";;
    "stable")
        echo "Docker TAGs - 'parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}', 'parity/parity:stable'";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}" \
            --tag "parity/parity:latest" \
            --tag "parity/parity:stable" \
            --file tools/Dockerfile .;
        docker push "parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}";
        docker push "parity/parity:stable";
        docker push "parity/parity:latest";;
    v[0-9]*.[0-9]*)
        echo "Docker TAG - 'parity/parity:${VERSION}-${TRACK}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "parity/parity:${VERSION}-${TRACK}" \
            --file tools/Dockerfile .;
        docker push "parity/parity:${VERSION}-${TRACK}";;
    *)
        echo "Docker TAG - 'parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag "parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}" \
            --file tools/Dockerfile .;
        docker push "parity/parity:${VERSION}-${CI_COMMIT_REF_NAME}";;
esac

docker logout
