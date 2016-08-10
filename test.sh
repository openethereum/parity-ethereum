#!/bin/sh
# Running Parity Full Test Sute

FEATURES="json-tests ipc"

case $1 in
    --no-json)
    FEATURES="ipc"
    shift # past argument=value
    ;;
    *)
            # unknown option
    ;;
esac

. ./scripts/targets.sh
cargo test --release --features "$FEATURES" $TARGETS $1 \

