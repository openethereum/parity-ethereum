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

set -e

./scripts/validate_chainspecs.sh

cargo test -j 8 $OPTIONS --features "$FEATURES" --all --exclude parity-ipfs-api --exclude evmjit $1
