#!/bin/bash
echo "________Running rust_changes.sh________"
set -e # fail on any error
set -u # treat unset variables as error

echo "__________Checking if Rust files were changed__________"
git log --graph --oneline --decorate=short -n 10

case ${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}} in
  (stable)
    export GIT_COMPARE=origin/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}~
    ;;
  (master|nightly)
    export GIT_COMPARE=master~
    ;;
  (*)
    export GIT_COMPARE=master
    ;;
esac

export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^CHANGELOG.md -e ^test.sh -e ^scripts/ -e ^docs/ -e ^docker/ -e ^snap/ | wc -l | tr -d ' ')"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"

if [ "${RUST_FILES_MODIFIED}" = "0" ]
then
  echo "__________Skipping Rust tests since no Rust files modified__________";
  exit 0
fi

rustup show
