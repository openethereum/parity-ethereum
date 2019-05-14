#!/bin/sh

set -e # fail on any error

VERSION="$(cat ./artifacts/VERSION)"
echo "Parity Ethereum version = ${VERSION}"

# we stopped pushing nightlies to dockerhub, will push to own registry prb.
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    $SCHEDULE_TAG)
        echo "Docker TAG - ${CONTAINER_IMAGE}:${SCHEDULE_TAG}";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag ${CONTAINER_IMAGE}:${SCHEDULE_TAG} \
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:${SCHEDULE_TAG};;
    beta)
        echo "Docker TAGs - ${CONTAINER_IMAGE}:beta, ${CONTAINER_IMAGE}:latest, \
            ${CONTAINER_IMAGE}:${VERSION}";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag ${CONTAINER_IMAGE}:beta \
            --tag ${CONTAINER_IMAGE}:latest \
            --tag ${CONTAINER_IMAGE}:${VERSION} \
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:beta;
        docker push ${CONTAINER_IMAGE}:latest;
        docker push ${CONTAINER_IMAGE}:${VERSION};;
    stable)
        echo "Docker TAGs - ${CONTAINER_IMAGE}:${VERSION}, ${CONTAINER_IMAGE}:stable";
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag ${CONTAINER_IMAGE}:${VERSION} \
            --tag ${CONTAINER_IMAGE}:stable \
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:${VERSION};
        docker push ${CONTAINER_IMAGE}:stable;;
    *)
        echo "Docker TAG - ${CONTAINER_IMAGE}:'${VERSION}-${CI_COMMIT_SHORT_SHA}'"
        docker build --no-cache \
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}" \
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')" \
            --tag ${CONTAINER_IMAGE}:'${VERSION}-${CI_COMMIT_SHORT_SHA}' \
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:'${VERSION}-${CI_COMMIT_SHORT_SHA}';;
esac
