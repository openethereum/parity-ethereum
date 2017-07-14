#!/bin/sh
# Running Parity Full Test Sute

FEATURES="json-tests"
OPTIONS="--release"

case $1 in
    --no-json)
    FEATURES="ipc"
    shift # past argument=value
    ;;
	--no-release)
	OPTIONS=""
	shift
	;;
	--no-run)
	OPTIONS="--no-run"
	shift
	;;
    *)
            # unknown option
    ;;
esac

. ./scripts/targets.sh
cargo test -j 8 $OPTIONS --features "$FEATURES" $TARGETS $1 \
ls target/debug
cp target/debug/parity-* target/debug/parity
git clone https://github.com/paritytech/parity-import-tests
cd /parity-import-tests/aura
target/debug/parity import blocks.rlp --chain chain.json
target/debug/parity restore snap --chain chain.json


