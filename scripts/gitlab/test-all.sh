#!/bin/bash
# ARGUMENT $1 Rust flavor to test with (stable/beta/nightly)

set -e # fail on any error
set -u # treat unset variables as error

git log --graph --oneline --decorate=short -n 10

THREADS=8

# temporarily here
cpp_test () {
  case $CARGO_TARGET in
    (x86_64-unknown-linux-gnu)
      # Running the C++ example
      echo "________Running the C++ example________"
      cd parity-clib-examples/cpp && \
        mkdir -p build && \
        cd build && \
        cmake .. && \
        make -j $THREADS && \
        ./parity-example && \
        cd .. && \
        rm -rf build && \
        cd ../..
      ;;
    (*)
      echo "________Skipping the C++ example________"
      ;;
  esac
}

# 

# case ${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}} in
#   (beta|stable)
#     export GIT_COMPARE=origin/${SCHEDULE_TAG:-${CI_COMMIT_REF_NAME}}~
#     ;;
#   (master|nightly)
#     export GIT_COMPARE=master~
#     ;;
#   (*)
#     export GIT_COMPARE=master
#     ;;
# esac

# export RUST_FILES_MODIFIED="$(git --no-pager diff --name-only $GIT_COMPARE...$CI_COMMIT_SHA | grep -v -e ^\\. -e ^LICENSE -e ^README.md -e ^CHANGELOG.md -e ^test.sh -e ^scripts/ -e ^docs/ -e ^docker/ -e ^snap/ | wc -l | tr -d ' ')"
# echo "RUST_FILES_MODIFIED: $RUST_FILES_MODIFIED"

# if [ "${RUST_FILES_MODIFIED}" = "0" ]
# then
#   echo "__________Skipping Rust tests since no Rust files modified__________";
#   exit 0
# fi

# rustup default $1

git submodule update --init --recursive
rustup show

# exec ./test.sh
echo "________Validate chainspecs________"
time ./scripts/validate_chainspecs.sh

echo "________Running Parity Full Test Suite________"
time cargo test --release --features json-tests ci-skip-issue --all --target $CARGO_TARGET -v -- --test-threads $THREADS
cpp_test