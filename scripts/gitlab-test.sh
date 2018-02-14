#!/bin/bash
#ARGUMENT test for RUST and COVERAGE
set -e # fail on any error
set -u # treat unset variables as error
if [[ "$CI_COMMIT_REF_NAME" = "beta" || "$CI_COMMIT_REF_NAME" = "stable" ]]; then
  export GIT_COMPARE=$CI_COMMIT_REF_NAME;
else
  export GIT_COMPARE=master;
fi
export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^windows/ -e ^scripts/ -e ^mac/ -e ^nsis/ | wc -l)"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"
echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"
TEST_SWITCH=$1
rust_test () {
  git submodule update --init --recursive
  rustup show
  if [[ "${RUST_FILES_MODIFIED}" == "0" ]];
    then echo "Skipping Rust tests since no Rust files modified.";
    else ./test.sh || exit $?;
  fi
  if [[ "$CI_COMMIT_REF_NAME" == "nightly" ]];
    then sh scripts/aura-test.sh || exit $?;
  fi
}
coverage_test () {
  git submodule update --init --recursive
  rm -rf target/*
  scripts/cov.sh
}
case $TEST_SWITCH in
  stable )
    rustup default stable
    rust_test
    ;;
  beta)
    rustup default beta
    rust_test
    ;;
  nightly)
    rustup default nightly
    rust_test
    ;;
  test-coverage)
    coverage_test
    ;;
esac
