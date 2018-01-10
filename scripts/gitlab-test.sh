#!/bin/bash
#ARGUMENT test for RUST, JS, COVERAGE or JS_RELEASE
set -e # fail on any error
set -u # treat unset variables as error
export JS_FILES_MODIFIED="$(git --no-pager diff --name-only master...$CI_BUILD_REF | grep ^js/ | wc -l)"
export JS_OLD_FILES_MODIFIED="$(git --no-pager diff --name-only master...$CI_BUILD_REF | grep ^js-old/ | wc -l)"
export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only master...$CI_BUILD_REF | grep -v -e ^js -e ^\\. -e ^LICENSE -e ^README.md -e ^test.sh -e ^windows/ -e ^scripts/ -e^mac/ -e ^nsis/ | wc -l)"
export RUST_BACKTRACE=1
TEST_SWITCH=$1
rust_test () {
  git submodule update --init --recursive
  rustup show
  if [ "${RUST_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping Rust tests since no Rust files modified.";
    else ./test.sh;
  fi
  if [ "$CI_BUILD_REF_NAME" == "nightly" ];
    then sh scripts/aura-test.sh;
  fi
}
js_test () {
  git submodule update --init --recursive
  if [ "${JS_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping JS deps install since no JS files modified.";
    else ./js/scripts/install-deps.sh;
  fi
  if [ "${JS_OLD_FILES_MODIFIED}" -eq 0  ];
    then echo "Skipping JS (old) deps install since no JS files modified.";
    else ./js-old/scripts/install-deps.sh;
  fi
  if [ "${JS_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping JS lint since no JS files modified.";
    else ./js/scripts/lint.sh && ./js/scripts/test.sh && ./js/scripts/build.sh;
  fi
  if [ "${JS_OLD_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping JS (old) lint since no JS files modified.";
    else ./js-old/scripts/lint.sh && ./js-old/scripts/test.sh && ./js-old/scripts/build.sh;
  fi
}
js_release () {
  rustup default stable
  echo "${JS_FILES_MODIFIED}"
  if [ "${JS_FILES_MODIFIED}" -eq 0  ];
    then echo "Skipping JS deps install since no JS files modified.";
    else ./js/scripts/install-deps.sh;
  fi
  if [ "${JS_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping JS rebuild since no JS files modified.";
    else ./js/scripts/build.sh && ./js/scripts/push-precompiled.sh;
  fi
  echo "${JS_OLD_FILES_MODIFIED}"
  if [ "${JS_OLD_FILES_MODIFIED}" -eq 0  ];
    then echo "Skipping JS (old) deps install since no JS files modified.";
    else ./js-old/scripts/install-deps.sh;
  fi
  if [ "${JS_OLD_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping JS (old) rebuild since no JS files modified.";
    else ./js-old/scripts/build.sh && ./js-old/scripts/push-precompiled.sh;
  fi
  if [ "${JS_FILES_MODIFIED}" -eq 0 ] && [ "${JS_OLD_FILES_MODIFIED}" -eq 0 ];
    then echo "Skipping Cargo update since no JS files modified.";
    else ./js/scripts/push-cargo.sh;
  fi
}
coverage_test () {
  git submodule update --init --recursive
  rm -rf target/*
  rm -rf js/.coverage
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
  js-test)
    js_test
    ;;
  js-release)
    js_release
    ;;
  test-coverage)
    coverage_test
    ;;
esac
