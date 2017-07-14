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
cd ..
ls target/debug
git clone https://github.com/paritytech/parity-import-tests
cp target/debug/parity-* parity-import-tests/aura.parity
cd parity-import-tests/aura
echo "start Aura test"
parity import blocks.rlp --chain chain.json
parity restore snap --chain chain.json
ehco "Aura test complete"



