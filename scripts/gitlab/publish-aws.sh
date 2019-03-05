#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "__________Push binaries to AWS S3____________"
case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
    (beta|stable|nightly)
      export BUCKET=releases.parity.io/ethereum;
      ;;
    (*)
      export BUCKET=builds-parity;
      ;;
  esac
aws s3 sync ./artifacts s3://${BUCKET}/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/
echo "__________Read from S3____________"
aws s3 ls s3://${BUCKET}/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/
    --recursive --human-readable --summarize
