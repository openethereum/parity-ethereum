#!/bin/bash
# ARGUMENT $1 Rust flavor to test with (stable/beta/nightly)

set -e # fail on any error
set -u # treat unset variables as error

rustup default $1

if [[ "$CI_COMMIT_REF_NAME" = "master" || "$CI_COMMIT_REF_NAME" = "beta" || "$CI_COMMIT_REF_NAME" = "stable" ]]; then
  export GIT_COMPARE=$CI_COMMIT_REF_NAME~;
else
  export GIT_COMPARE=master;
fi

export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^windows/ -e ^scripts/ -e ^mac/ -e ^nsis/ | wc -l)"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"

git submodule update --init --recursive
rustup show
if [[ "${RUST_FILES_MODIFIED}" == "0" ]];
then echo "__________Skipping Rust tests since no Rust files modified__________";
else ./test.sh || exit $?;
fi

# if [[ "$CI_COMMIT_REF_NAME" == "nightly" ]];
#   ### @TODO re-enable fail after https://github.com/paritytech/parity-import-tests/issues/3
#   then sh scripts/aura-test.sh; # || exit $?;
# fi
