#!/bin/bash
set -e # fail on any error
set -u # treat unset variables as error

cargo build -j $(nproc) --release --features final $CARGOFLAGS
git clone https://github.com/paritytech/parity-import-tests
cp target/release/purity parity-import-tests/aura/purity
cd parity-import-tests/aura
echo "Start Aura test"
./purity import blocks.rlp --chain chain.json
./purity restore snap --chain chain.json
echo "Aura test complete"
