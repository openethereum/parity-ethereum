#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
updater_push_release () {
  echo "push release"
  # Mainnet

}
echo "__________Set ENVIROMENT__________"
DESCRIPTION="$(cat CHANGELOG.md)"
RELEASE_TABLE="$(cat scripts/gitlab/templates/release-table.md)"
RELEASE_TABLE="$(echo "${RELEASE_TABLE//\$VERSION/${VERSION}}")"
#The text in the file CANGELOG.md before which the table with links is inserted. Must be present in this file necessarily
REPLACE_TEXT="The full list of included changes:"
case ${CI_COMMIT_REF_NAME} in
  nightly|*v2.1*) NAME="Parity "$VERSION" nightly";;
  beta|*v2.0*) NAME="Parity "$VERSION" beta";;
  stable|*v1.11*) NAME="Parity "$VERSION" stable";;
  *) echo "No release" exit 0;;
esac
cd artifacts
ls -l | sort -k9
filetest=( * )
echo ${filetest[*]}
for DIR in "${filetest[@]}";
do
  cd $DIR
  if [[ $DIR == "*windows*" ]];
    then
      WIN=".exe";
    else
      WIN="";
  fi
  for binary in $(ls parity.sha256)
  do
    sha256=$(cat $binary | awk '{ print $1}' )
    case $DIR in
      x86_64* )
        DATA="commit=$CI_BUILD_REF&sha3=$sha256&filename=parity$WIN&secret=$RELEASES_SECRET"
        ../../scripts/gitlab/safe_curl.sh $DATA "http://update.parity.io:1337/push-build/$CI_BUILD_REF_NAME/$DIR"
        # Kovan
        ../../scripts/gitlab/safe_curl.sh $DATA "http://update.parity.io:1338/push-build/$CI_BUILD_REF_NAME/$DIR"
        ;;
    esac
    RELEASE_TABLE="$(echo "${RELEASE_TABLE/sha$DIR/${sha256}}")"
  done
  cd ..
done
#do not touch the following 3 lines. Features of output in Markdown
DESCRIPTION="$(echo "${DESCRIPTION/${REPLACE_TEXT}/${RELEASE_TABLE}

${REPLACE_TEXT}}")"
echo "$DESCRIPTION"
if [[ "$CI_COMMIT_REF_NAME" == "nightly" ]]; then DESCRIPTION=""; fi #TODO in the future, we need to prepare a script that will do changelog
echo "__________Create release to Github____________"
github-release release --user devops-parity --repo parity-ethereum --tag "$CI_COMMIT_REF_NAME" --draft --name "$NAME" --description "$DESCRIPTION"
echo "__________Push binaries to AWS S3____________"
aws configure set aws_access_key_id $s3_key
aws configure set aws_secret_access_key $s3_secret
if [[ "$CI_BUILD_REF_NAME" = "beta" || "$CI_BUILD_REF_NAME" = "stable" || "$CI_BUILD_REF_NAME" = "nightly" ]];
  then
    export S3_BUCKET=builds-parity-published;
  else
    export S3_BUCKET=builds-parity;
fi
aws s3 sync ./ s3://$S3_BUCKET/$CI_BUILD_REF_NAME/
