#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

CARGO_TARGET_DIR=./target cargo +stable install cargo-audit --force
cargo +stable audit
