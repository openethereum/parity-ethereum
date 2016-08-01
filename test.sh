#!/bin/sh
# Running Parity Full Test Sute

FEATURES="--features json-tests"

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
cargo test --release $FEATURES $TARGETS $1 \

