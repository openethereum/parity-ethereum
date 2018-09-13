#!/bin/bash
# ARGUMENT $1 Rust flavor to test with (stable/beta/nightly)

set -e # fail on any error
set -u # treat unset variables as error


rustup default $1

git submodule update --init --recursive
rustup show

./test.sh || exit $?

# if [[ "$CI_COMMIT_REF_NAME" == "nightly" ]];
#   ### @TODO re-enable fail after https://github.com/paritytech/parity-import-tests/issues/3
#   then sh scripts/aura-test.sh; # || exit $?;
# fi
