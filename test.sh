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

# Validate --no-default-features build
echo "________Validate build________"
cargo check --no-default-features

# Validate chainspecs
echo "________Validate chainspecs________"
./scripts/validate_chainspecs.sh

# Running test's
echo "________Running Parity Full Test Suite________"

cargo test -j 8 $OPTIONS --features "$FEATURES" --all --exclude evmjit $1

