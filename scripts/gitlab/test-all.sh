#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

git log --graph --oneline --decorate=short -n 10

# FIXME:
git submodule update --init --recursive
rustup show

exec ./test.sh
