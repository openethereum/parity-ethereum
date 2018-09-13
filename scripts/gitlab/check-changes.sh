#!/bin/sh
#
# script returns 0 if there are no changes to the rust codebase
# 1 otherwise

set -e # fail on any error
set -u # treat unset variables as error

set -x # full command output for development

git log --graph --oneline --all -n 20

if [ "$CI_COMMIT_REF_NAME" = "master" -o "$CI_COMMIT_REF_NAME" = "beta" -o "$CI_COMMIT_REF_NAME" = "stable" ]
then
  export GIT_COMPARE=$CI_COMMIT_REF_NAME~
else
  export GIT_COMPARE=master
fi

export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^windows/ -e ^scripts/ -e ^mac/ -e ^nsis/ | wc -l)"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"


if [ "${RUST_FILES_MODIFIED}" = "0" ]
then
  echo "__________Skipping Rust tests since no Rust files modified__________";
  exit 1
else
  echo "__________Rust files modified__________";
fi

