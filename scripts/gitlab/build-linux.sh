#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

case "$BUILD_TARGET:$BUILD_ARCH"
    "ubuntu:x86_64")
        export BINARIES_TARGET="x86_64-unknown-linux-gnu";
        export CARGO_TARGET="x86_64-unknown-linux-gnu";
        ;;
esac

rustup default stable
