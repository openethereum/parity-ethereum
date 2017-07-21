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

cargo test -j 8 $OPTIONS --features "$FEATURES" $TARGETS $1

echo "Starting import tests"
cargo run $OPTIONS -- \
-d .test-parity \
import ethcore/res/parity-import-tests/aura/blocks.rlp \
--chain ethcore/res/parity-import-tests/aura/chain.json

cargo run $OPTIONS -- \
-d .test-parity \
restore ethcore/res/parity-import-tests/aura/snap \
--chain ethcore/res/parity-import-tests/aura/chain.json

rm -rf .test-parity
echo "Import tests finished"
