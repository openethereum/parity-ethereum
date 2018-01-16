#!/bin/sh
# Running Parity Full Test Suite

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

# Validate chainspecs
./scripts/validate_chainspecs.sh

cargo test -j 8 $OPTIONS --features "$FEATURES" --all --exclude evmjit $1

# Validate --no-default-features build
cargo check --no-default-features
