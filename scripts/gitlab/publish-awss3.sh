#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "__________Register Release__________"
DATA="secret=$RELEASES_SECRET"

echo "Pushing release to Mainnet"
./scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1337/push-release/$CI_COMMIT_REF_NAME/$CI_COMMIT_SHA"

echo "Pushing release to Kovan"
./scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1338/push-release/$CI_COMMIT_REF_NAME/$CI_COMMIT_SHA"

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
      ../../scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1337/push-build/$CI_COMMIT_REF_NAME/$DIR"
      # Kovan
      ../../scripts/gitlab/safe-curl.sh $DATA "http://update.parity.io:1338/push-build/$CI_COMMIT_REF_NAME/$DIR"
      ;;
  esac
  cd ..
done

echo "__________Push binaries to AWS S3____________"
aws configure set aws_access_key_id $s3_key
aws configure set aws_secret_access_key $s3_secret
if [[ "$CI_COMMIT_REF_NAME" = "beta" || "$CI_COMMIT_REF_NAME" = "stable" || "$CI_COMMIT_REF_NAME" = "nightly" ]];
  then
    export S3_BUCKET=builds-parity-published;
  else
    export S3_BUCKET=builds-parity;
fi
aws s3 sync ./ s3://$S3_BUCKET/$CI_COMMIT_REF_NAME/
