#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "__________Register Release__________"
DATA="secret=$RELEASES_SECRET"

echo "Pushing release to Mainnet"
./scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1337/push-release/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/$CI_COMMIT_SHA"

echo "Pushing release to Kovan"
./scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1338/push-release/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/$CI_COMMIT_SHA"

cd artifacts
ls -l | sort -k9
filetest=( * )
echo ${filetest[*]}
for DIR in "${filetest[@]}";
do
  cd $DIR
  if [[ $DIR =~ "windows" ]];
    then
      WIN=".exe";
    else
      WIN="";
  fi
  sha3=$(cat parity.sha3 | awk '{print $1}')
  case $DIR in
    x86_64* )
      DATA="commit=$CI_COMMIT_SHA&sha3=$sha3&filename=parity$WIN&secret=$RELEASES_SECRET"
      ../../scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1337/push-build/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/$DIR"
      # Kovan
      ../../scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1338/push-build/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/$DIR"
      ;;
  esac
  cd ..
done

echo "__________Push binaries to AWS S3____________"
aws configure set aws_access_key_id $s3_key
aws configure set aws_secret_access_key $s3_secret

case "${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}" in
  (beta|stable|nightly)
    export S3_BUCKET=builds-parity-published;
    ;;
  (*)
    export S3_BUCKET=builds-parity;
    ;;
esac

aws s3 sync ./ s3://$S3_BUCKET/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}/

