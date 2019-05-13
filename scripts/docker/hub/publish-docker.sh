#!/bin/bash

set -e # fail on any error

# we stopped pushing nightlies to dockerhub, will push to own registry prb.
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    $SCHEDULE_TAG)
        echo "Docker TAG - ${CONTAINER_IMAGE}:${SCHEDULE_TAG}";;
        docker build --no-cache
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}"
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
            --tag ${CONTAINER_IMAGE}:$SCHEDULE_TAG
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:$SCHEDULE_TAG;;
    beta)
        echo "Docker TAGs - ${CONTAINER_IMAGE}:beta, ${CONTAINER_IMAGE}:latest";
        docker build --no-cache
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}"
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
            --tag ${CONTAINER_IMAGE}:beta
            --tag ${CONTAINER_IMAGE}:latest
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:beta;
        docker push ${CONTAINER_IMAGE}:latest ;;
    stable|*)
        echo "Docker TAGs - ${CONTAINER_IMAGE}:${CI_COMMIT_REF_NAME}";
        docker build --no-cache
            --build-arg VCS_REF="${CI_COMMIT_SHORT_SHA}"
            --build-arg BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
            --tag ${CONTAINER_IMAGE}:$CI_COMMIT_REF_NAME
            --file artifacts/Dockerfile .;
        docker push ${CONTAINER_IMAGE}:$CI_COMMIT_REF_NAME ;;
esac
