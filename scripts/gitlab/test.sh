#!/bin/bash
# ARGUMENT $1 Rust flavor to test with (stable/beta/nightly)

set -e # fail on any error
set -u # treat unset variables as error


set -x # full command output for development
git log --graph --oneline --all --decorate=short -n 10


case $CI_COMMIT_REF_NAME in
  (master|beta|stable)
    export GIT_COMPARE=$CI_COMMIT_REF_NAME~
    ;;
  (*)
    export GIT_COMPARE=master
  ;;
esac


export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^scripts/ | wc -l | tr -d ' ')"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"


if [ "${RUST_FILES_MODIFIED}" = "0" ]
then
  echo "__________Skipping Rust tests since no Rust files modified__________";
  exit 0
fi


rustup default $1

git submodule update --init --recursive
rustup show

exec ./test.sh

# if [[ "$CI_COMMIT_REF_NAME" == "nightly" ]];
#   ### @TODO re-enable fail after https://github.com/paritytech/parity-import-tests/issues/3
#   then sh scripts/aura-test.sh; # || exit $?;
# fi

