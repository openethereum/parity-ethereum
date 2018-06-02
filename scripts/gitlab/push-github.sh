#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
echo "__________Set ENVIROMENT__________"
DOWNLOAD_PREFIX="https://github.com/paritytech/parity/releases/download/"$CI_COMMIT_REF_NAME"/"
DESCRIPTION="$(cat CHANGELOG.md)"
RELEASE_TABLE="$(cat scripts/gitlab/templates/release-table.md)"
RELEASE_TABLE="$(echo "${RELEASE_TABLE//\$VERSION/${VERSION}}")"
RELEASE_TABLE="$(echo "${RELEASE_TABLE//\$DOWNLOAD_PREFIX/${DOWNLOAD_PREFIX}}")"
#The text in the file CANGELOG.md before which the table with links is inserted. Must be present in this file necessarily
REPLACE_TEXT="The full list of included changes:"
case ${CI_COMMIT_REF_NAME} in
  master|*v1.12*|nightly) NAME="Parity "$VERSION" nightly";;
  beta|*v1.11*) NAME="Parity "$VERSION" beta";;
  stable|*v1.10*) NAME="Parity "$VERSION" stable";;
  *) echo "No release" exit 0;;
esac
cd packages
i=1
for binary in $(ls *.sha256)
do
  sha256=$(cat $binary | awk '{ print $1}' )
  RELEASE_TABLE="$(echo "${RELEASE_TABLE/sha${i}/${sha256}}")"
  let ++i
done
#do not touch the following 3 lines. Features of output in Markdown
DESCRIPTION="$(echo "${DESCRIPTION/${REPLACE_TEXT}/${RELEASE_TABLE}

${REPLACE_TEXT}}")"
echo "__________Create release to Github____________"
github-release release --user "$GITHUB_USER" --repo parity --tag "$CI_COMMIT_REF_NAME" --draft --name "$NAME" --description "$DESCRIPTION"
echo "__________Upload files to Github____________"

for binary in $(ls -I "*.sha256")
do
    github-release upload --user "$GITHUB_USER" --repo parity --tag "$CI_COMMIT_REF_NAME" --replace --name "$binary" --file $binary
done
