#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

git submodule update --init --recursive
rm -rf target/*
scripts/cov.sh
