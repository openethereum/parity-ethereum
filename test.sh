#!/bin/sh
# Running Parity Full Test Sute

FEATURES="--features ethcore/json-tests"

case $1 in
    --no-json)
    FEATURES=""
    shift # past argument=value
    ;;
    *)
            # unknown option
    ;;
esac

. ./scripts/targets.sh
cargo test --no-default-features $FEATURES $TARGETS $1 \

