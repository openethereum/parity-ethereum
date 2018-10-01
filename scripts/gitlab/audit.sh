#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

rustup default stable

git submodule update --init --recursive

rustup show
cargo install cargo-audit
cargo audit
